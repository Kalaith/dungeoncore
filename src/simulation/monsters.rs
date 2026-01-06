use crate::data::constants::get_monster_mana_cost;
use crate::data::monsters::get_monster_template;
use crate::game_state::{GameState, LogEntry, Monster, RoomType, Stats};

/// Place a monster in a room
pub fn place_monster(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    monster_name: &str,
) -> Result<(), String> {
    // Find monster template
    let template =
        get_monster_template(monster_name).ok_or_else(|| format!("Unknown monster: {}", monster_name))?;

    // Check if species is unlocked
    if !state.unlocked_species.contains(&template.species) {
        return Err(format!("Species '{}' is not unlocked!", template.species));
    }

    // Find floor and room
    let floor = state
        .floors
        .iter_mut()
        .find(|f| f.number == floor_num)
        .ok_or("Floor not found")?;

    let room = floor
        .rooms
        .iter_mut()
        .find(|r| r.position == room_pos)
        .ok_or("Room not found")?;

    // Cannot place in entrance or core
    if room.room_type == RoomType::Entrance || room.room_type == RoomType::Core {
        return Err("Cannot place monsters in entrance or core rooms!".into());
    }

    let is_boss = room.room_type == RoomType::Boss;
    let cost = get_monster_mana_cost(template.base_cost, floor_num, is_boss);

    if state.mana < cost {
        return Err(format!("Not enough mana! Need {} mana.", cost));
    }

    state.mana -= cost;

    // Scale stats based on floor and boss status
    let base_stats = Stats {
        hp: template.hp,
        attack: template.attack,
        defense: template.defense,
    };
    let scaled = crate::data::get_scaled_stats(base_stats, floor_num, is_boss);

    // Initialize traits
    let active_traits = template.traits.iter().map(|trait_id| {
        // Look up trait name (optional, but good for display without full lookup)
        // For now we just store ID and initial cooldown 0
        crate::game_state::ActiveTrait {
            id: trait_id.clone(),
            name: crate::data::traits::get_trait(trait_id)
                .map(|t| t.name)
                .unwrap_or_else(|| trait_id.clone()),
            cooldown_timer: 0,
        }
    }).collect();

    let monster = Monster {
        id: macroquad::rand::rand() as u64,
        type_name: monster_name.into(),
        hp: scaled.hp,
        max_hp: scaled.hp,
        alive: true,
        is_boss,
        scaled_stats: scaled,
        active_traits,
    };

    room.monsters.push(monster);

    let boss_suffix = if is_boss { " (Boss)" } else { "" };
    state.add_log(LogEntry::building(format!(
        "Spawned {}{} on floor {}, room {} for {} mana.",
        monster_name, boss_suffix, floor_num, room_pos, cost
    )));

    Ok(())
}

/// Respawn all dead monsters (only when no adventurers present)
pub fn respawn_monsters(state: &mut GameState) {
    if !state.adventurer_parties.is_empty() {
        return;
    }

    let mut respawned = 0;
    for floor in &mut state.floors {
        for room in &mut floor.rooms {
            for monster in &mut room.monsters {
                if !monster.alive {
                    monster.hp = monster.max_hp;
                    monster.alive = true;
                    respawned += 1;
                }
            }
        }
    }

    if respawned > 0 {
        state.add_log(LogEntry::system(format!(
            "All monsters respawned! ({} monsters)",
            respawned
        )));
    }
}

/// Unlock a monster species
pub fn unlock_species(state: &mut GameState, species_name: &str) -> Result<(), String> {
    if state.unlocked_species.contains(&species_name.to_string()) {
        return Err(format!("Species '{}' is already unlocked!", species_name));
    }

    // Get unlock cost from JSON data
    let unlock_cost = crate::data::monsters::get_species_unlock_cost(species_name)
        .ok_or_else(|| format!("Unknown species: {}", species_name))?;

    if unlock_cost == 0 {
        // Warning: This branch might not be hit if valid cost is always returned, 
        // but nice to have distinct error or just success.
        // For now, consistent with previous code.
    }

    if state.gold < unlock_cost {
        return Err(format!("Not enough gold! Need {} gold.", unlock_cost));
    }

    state.gold -= unlock_cost;
    state.unlocked_species.push(species_name.to_string());
    state.add_log(LogEntry::system(format!(
        "Unlocked new species: {} for {} gold!",
        species_name, unlock_cost
    )));

    Ok(())
}

/// Process hourly trait effects (e.g. regeneration)
// now data driven!
pub fn process_hourly_traits(state: &mut GameState) {
    for floor in &mut state.floors {
        for room in &mut floor.rooms {
            for monster in &mut room.monsters {
                if !monster.alive {
                    continue;
                }

                for trait_data in &monster.active_traits {
                    // Look up definition
                    if let Some(def) = crate::data::traits::get_trait(&trait_data.id) {
                        if def.applies_to == "Hourly" && def.trait_type == "Passive" {
                            // Logic dispatcher based on ID still needed for *behavior*, but values come from JSON
                            // "regenerate_minor" behavior: heal % of max hp
                            if def.id == "regenerate_minor" {
                                let heal_amount = (monster.max_hp as f32 * def.value).ceil() as i32;
                                if monster.hp < monster.max_hp {
                                    monster.hp = (monster.hp + heal_amount).min(monster.max_hp);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
