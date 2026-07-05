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
    let template = get_monster_template(monster_name)
        .ok_or_else(|| format!("Unknown monster: {}", monster_name))?;

    // Check if species is unlocked
    if !state.unlocked_species.contains(&template.species) {
        return Err(format!("Species '{}' is not unlocked!", template.species));
    }

    // Check if this specific monster type is unlocked
    if !state.unlocked_monsters.contains(&template.name) {
        return Err(format!(
            "Monster '{}' is not unlocked! Evolve to unlock higher tiers.",
            template.name
        ));
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
    if template.boss_only && !is_boss {
        return Err(format!(
            "{} can only be summoned in a Boss room!",
            template.name
        ));
    }

    // Boss uniques already price in their throne room — no 2x boss surcharge.
    let boss_surcharge = is_boss && !template.boss_only;
    let cost = get_monster_mana_cost(template.base_cost, floor_num, boss_surcharge);

    if state.mana < cost {
        return Err(format!("Not enough mana! Need {} mana.", cost));
    }
    if state.souls < template.souls_cost {
        return Err(format!(
            "Not enough souls! Need {} souls.",
            template.souls_cost
        ));
    }

    state.mana -= cost;
    state.souls -= template.souls_cost;

    // Scale stats based on floor and boss status
    let base_stats = Stats {
        hp: template.hp,
        attack: template.attack,
        defense: template.defense,
    };
    let scaled = crate::data::get_scaled_stats(base_stats, floor_num, is_boss);

    // Initialize traits
    let active_traits = template
        .traits
        .iter()
        .map(|trait_id| {
            // Look up trait name (optional, but good for display without full lookup)
            // For now we just store ID and initial cooldown 0
            crate::game_state::ActiveTrait {
                id: trait_id.clone(),
                name: crate::data::traits::get_trait(trait_id)
                    .map(|t| t.name)
                    .unwrap_or_else(|| trait_id.clone()),
                cooldown_timer: 0,
            }
        })
        .collect();

    let monster = Monster {
        id: macroquad_toolkit::rng::random_u64(),
        type_name: monster_name.into(),
        hp: scaled.hp,
        max_hp: scaled.hp,
        alive: true,
        is_boss,
        scaled_stats: scaled,
        active_traits,
        experience: 0,
    };

    room.monsters.push(monster);

    let boss_suffix = if is_boss { " (Boss)" } else { "" };
    state.add_log(LogEntry::building(format!(
        "Spawned {}{} on floor {}, room {} for {} mana.",
        monster_name, boss_suffix, floor_num, room_pos, cost
    )));

    Ok(())
}

/// Dismiss a placed monster, refunding half its summon mana.
pub fn remove_monster(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    monster_id: u64,
) -> Result<(), String> {
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot dismiss monsters while adventurers are in the dungeon!".into());
    }

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

    let idx = room
        .monsters
        .iter()
        .position(|m| m.id == monster_id)
        .ok_or("Monster not found in this room")?;
    let monster = room.monsters.remove(idx);

    // Refund half of what the summon cost at this floor/room (souls are spent
    // essence and stay spent).
    let refund = get_monster_template(&monster.type_name)
        .map(|template| {
            let boss_surcharge = room.room_type == RoomType::Boss && !template.boss_only;
            get_monster_mana_cost(template.base_cost, floor_num, boss_surcharge) / 2
        })
        .unwrap_or(0);
    state.mana = (state.mana + refund).min(state.max_mana);

    state.add_log(LogEntry::building(format!(
        "Dismissed {} from floor {}, room {}. Refunded {} mana.",
        monster.type_name, floor_num, room_pos, refund
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

    // Get unlock cost from JSON data. Starter races are free only for the first pick.
    let species_data = crate::data::monsters::get_species(species_name)
        .ok_or_else(|| format!("Unknown species: {}", species_name))?;
    let is_first_species = state.unlocked_species.is_empty();
    let unlock_cost = if is_first_species && species_data.starter {
        0
    } else {
        species_data.unlock_cost
    };

    if unlock_cost == 0 {
        // Free unlock - still unlock the starting monster
    } else {
        if state.gold < unlock_cost {
            return Err(format!("Not enough gold! Need {} gold.", unlock_cost));
        }
        state.gold -= unlock_cost;
    }

    state.unlocked_species.push(species_name.to_string());

    let mut unlocked_now = Vec::new();
    for template in crate::data::monsters::get_starter_monsters_for_species(species_name) {
        if !state.unlocked_monsters.contains(&template.name) {
            state.unlocked_monsters.push(template.name.clone());
            unlocked_now.push(template.name);
        }
    }

    if unlocked_now.is_empty() {
        if let Some(starting_monster) =
            crate::data::evolutions::get_starting_monsters().get(species_name)
        {
            if !state.unlocked_monsters.contains(starting_monster) {
                state.unlocked_monsters.push(starting_monster.clone());
                unlocked_now.push(starting_monster.clone());
            }
        }
    }

    state.add_log(LogEntry::system(format!(
        "Unlocked {} for {} gold. Available summons: {}.",
        crate::data::monsters::get_species_display_name(species_name),
        unlock_cost,
        if unlocked_now.is_empty() {
            "none".to_string()
        } else {
            unlocked_now.join(", ")
        }
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
                        if def.applies_to == "Hourly"
                            && def.trait_type == "Passive"
                            && def.effect_type == "HealPercent"
                        {
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

/// Unlock evolved forms as defenders gain experience, WITHOUT transforming the
/// placed monster. The player can then choose to summon the new tier and retire
/// the old one. Runs hourly; each form is only announced once.
pub fn process_evolution_unlocks(state: &mut GameState) {
    // Collect evolved forms whose conditions are met. A monster with
    // branching paths unlocks every branch it qualifies for — the player
    // chooses which to summon.
    let mut candidates: Vec<String> = Vec::new();
    for floor in &state.floors {
        for room in &floor.rooms {
            for monster in &room.monsters {
                for path in
                    crate::data::evolutions::get_evolutions_for_monster(&monster.type_name)
                {
                    let earned = monster.experience >= path.experience_required
                        && room.floor_number >= path.conditions.min_floor;
                    if earned
                        && !state.unlocked_monsters.contains(&path.to_monster)
                        && !candidates.contains(&path.to_monster)
                    {
                        candidates.push(path.to_monster);
                    }
                }
            }
        }
    }

    for new_monster in candidates {
        state.unlocked_monsters.push(new_monster.clone());
        state.add_log(LogEntry::system(format!(
            "New defender unlocked: {}! Summon it from the Monsters panel to upgrade your dungeon.",
            new_monster
        )));
    }
}

/// Check and perform monster evolutions
pub fn process_evolutions(state: &mut GameState) {
    let mut evolutions_performed = Vec::new();

    // First pass: collect all evolutions that can happen
    for floor_idx in 0..state.floors.len() {
        for room_idx in 0..state.floors[floor_idx].rooms.len() {
            let room = &state.floors[floor_idx].rooms[room_idx];
            let floor_num = room.floor_number;

            for monster_idx in 0..room.monsters.len() {
                let monster = &room.monsters[monster_idx];
                let monster_name = &monster.type_name;
                let experience = monster.experience;

                // Check if this monster can evolve
                if let Some(evolution_path) = crate::data::evolutions::can_evolve(
                    monster_name,
                    experience,
                    floor_num,
                    state.gold,
                ) {
                    evolutions_performed.push((floor_idx, room_idx, monster_idx, evolution_path));
                }
            }
        }
    }

    // Second pass: perform evolutions
    let mut log_messages = Vec::new();
    for (floor_idx, room_idx, monster_idx, evolution_path) in evolutions_performed {
        let floor = &mut state.floors[floor_idx];
        let room = &mut floor.rooms[room_idx];
        let monster = &mut room.monsters[monster_idx];

        let old_name = monster.type_name.clone();
        let new_name = evolution_path.to_monster.clone();

        // Deduct gold cost
        state.gold -= evolution_path.conditions.gold_cost;

        // Get new monster template
        if let Some(new_template) = crate::data::monsters::get_monster_template(&new_name) {
            // Update monster type and stats
            monster.type_name = new_name.clone();

            // Rescale stats based on current floor and boss status
            let base_stats = crate::game_state::Stats {
                hp: new_template.hp,
                attack: new_template.attack,
                defense: new_template.defense,
            };
            let scaled =
                crate::data::get_scaled_stats(base_stats, room.floor_number, monster.is_boss);

            monster.hp = scaled.hp;
            monster.max_hp = scaled.hp;
            monster.scaled_stats = scaled;

            // Update traits
            monster.active_traits = new_template
                .traits
                .iter()
                .map(|trait_id| crate::game_state::ActiveTrait {
                    id: trait_id.clone(),
                    name: crate::data::traits::get_trait(trait_id)
                        .map(|t| t.name)
                        .unwrap_or_else(|| trait_id.clone()),
                    cooldown_timer: 0,
                })
                .collect();

            // Reset experience for new form
            monster.experience = 0;

            // Unlock the new monster type if not already unlocked
            if !state.unlocked_monsters.contains(&new_name) {
                state.unlocked_monsters.push(new_name.clone());
            }

            log_messages.push(format!(
                "{} evolved into {} on floor {}!",
                old_name, new_name, room.floor_number
            ));
        }
    }

    // Add log messages after all mutations are done
    for message in log_messages {
        state.add_log(crate::game_state::LogEntry::system(message));
    }
}
