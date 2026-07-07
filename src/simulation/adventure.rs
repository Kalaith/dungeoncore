use crate::data::adventurers::{
    get_adventurer_class, get_adventurer_classes, get_adventurer_names, get_entry_quotes,
    get_exit_quotes,
};
use crate::data::constants::{ADVENTURER_SPAWN_CHANCE, MAX_PARTY_SIZE, MIN_PARTY_SIZE};
use crate::game_state::{
    Adventurer, AdventurerParty, DungeonStatus, GameState, HeroRecord, HeroStatus, LogEntry, Stats,
};

/// Build a combat-ready adventurer from a class, level, and identity.
fn build_adventurer(id: u64, name: String, class_name: &str, race: &str, level: i32) -> Adventurer {
    let class = get_adventurer_class(class_name)
        .unwrap_or_else(|| get_adventurer_classes().into_iter().next().unwrap());
    let race_mod = crate::data::adventurers::get_race(race).unwrap_or_default();

    let base_hp = class.hp + (level - 1) * 10 + race_mod.hp;
    let equipment = crate::data::equipment::recommended_loadout(&class.name, level);
    let equipment_bonus = crate::data::equipment::equipment_stat_bonus(&equipment, &class.name);
    let hp = (base_hp + equipment_bonus.hp).max(1);

    Adventurer {
        id,
        name,
        class_name: class.name.clone(),
        race: race.to_string(),
        level,
        hp,
        max_hp: hp,
        alive: true,
        experience: 0,
        gold: 0,
        equipment,
        conditions: Vec::new(),
        scaled_stats: Stats {
            hp,
            attack: (class.attack + (level - 1) * 2 + equipment_bonus.attack + race_mod.attack)
                .max(1),
            defense: (class.defense + (level - 1) + equipment_bonus.defense + race_mod.defense)
                .max(0),
        },
    }
}

/// Party size and adventurer level range for the current threat/floor state.
/// Low threat sends larger bands of weaker heroes; high threat sends smaller,
/// far more dangerous elites.
fn threat_party_shape(state: &GameState) -> (usize, i32, i32) {
    let tier = state.threat_tier();
    let deepest = state.total_floors.max(1);
    let level_min = 1 + tier;
    let level_max = (3 + tier + deepest / 2).max(level_min);
    let size = match tier {
        0 => macroquad_toolkit::rng::gen_range(MIN_PARTY_SIZE, MAX_PARTY_SIZE + 1),
        1 | 2 => macroquad_toolkit::rng::gen_range(MIN_PARTY_SIZE, MAX_PARTY_SIZE),
        _ => MIN_PARTY_SIZE,
    };
    (size.max(1), level_min, level_max)
}

/// Try to spawn a new adventurer party
pub fn spawn_party(state: &mut GameState) {
    // Only spawn when open and no parties present
    if state.status != DungeonStatus::Open {
        return;
    }
    if !state.adventurer_parties.is_empty() {
        return;
    }
    // Compare in absolute hours — hour-of-day comparisons broke at the day
    // wrap (a party spawned at hour 23 set next_party_spawn to 24, which
    // hour-of-day never reaches, so spawns stopped after day 1).
    let now_abs = state.day * 24 + state.hour;
    if now_abs < state.next_party_spawn {
        return;
    }

    if !macroquad_toolkit::rng::chance(ADVENTURER_SPAWN_CHANCE) {
        return;
    }

    let names = get_adventurer_names();
    let entry_quotes = get_entry_quotes();
    let races = crate::data::adventurers::get_race_names();

    // Higher threat means fewer but stronger parties (see threat_party_shape).
    let (party_size, level_min, level_max) = threat_party_shape(state);
    let mut members = Vec::with_capacity(party_size);

    // Some slots are filled by veterans returning for another delve.
    let mut returning: Vec<u64> = state
        .known_adventurers
        .iter()
        .filter(|h| h.status == HeroStatus::Alive)
        .map(|h| h.id)
        .collect();
    macroquad_toolkit::rng::shuffle(&mut returning);

    for slot in 0..party_size {
        // Roughly half the slots prefer a returning veteran, if any remain.
        let use_veteran = slot % 2 == 0 && !returning.is_empty();
        if use_veteran {
            let hero_id = returning.pop().unwrap();
            if let Some(record) = state.hero_mut(hero_id) {
                record.status = HeroStatus::Inside;
                record.delves += 1;
                let (name, class, race, level) = (
                    record.name.clone(),
                    record.class_name.clone(),
                    record.race.clone(),
                    record.level,
                );
                members.push(build_adventurer(hero_id, name, &class, &race, level));
                continue;
            }
        }

        // Fresh recruit: roll identity and register a new ledger entry.
        let classes = get_adventurer_classes();
        let class = macroquad_toolkit::rng::choose(&classes).unwrap();
        let name = macroquad_toolkit::rng::choose(&names).unwrap().clone();
        let race = macroquad_toolkit::rng::choose(&races)
            .cloned()
            .unwrap_or_else(|| "Human".to_string());
        let level = macroquad_toolkit::rng::gen_range(level_min, level_max + 1);
        let id = macroquad_toolkit::rng::random_u64();
        state.known_adventurers.push(HeroRecord {
            id,
            name: name.clone(),
            class_name: class.name.clone(),
            race: race.clone(),
            level,
            experience: 0,
            delves: 1,
            kills: 0,
            gold_stolen: 0,
            status: HeroStatus::Inside,
            death_floor: 0,
            death_day: 0,
        });
        members.push(build_adventurer(id, name, &class.name, &race, level));
    }

    let target_floor = state.floors.len().min(2) as i32;

    let party = AdventurerParty {
        id: macroquad_toolkit::rng::random_u64(),
        members,
        current_floor: 1,
        current_room: 0,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: state.hour,
        target_floor,
        snared_ticks: 0,
        alarmed: false,
        sieging: false,
    };

    state.add_log(LogEntry::adventure(format!(
        "New adventurer party enters! ({} members)",
        party.members.len()
    )));

    // Random entry quote
    if macroquad_toolkit::rng::chance(0.3) && !entry_quotes.is_empty() {
        let quote = macroquad_toolkit::rng::choose(&entry_quotes).unwrap();
        let name = &party.members[0].name;
        state.add_log(LogEntry::adventure(format!("{} says: \"{}\"", name, quote)));
    }

    state.adventurer_parties.push(party);
    state.next_party_spawn = now_abs + 1;
}

/// Process all adventurer parties
pub fn process_parties(state: &mut GameState) {
    if state.adventurer_parties.is_empty() {
        return;
    }

    // Collect party IDs to process
    let party_ids: Vec<u64> = state.adventurer_parties.iter().map(|p| p.id).collect();

    for party_id in party_ids {
        process_single_party(state, party_id);
    }

    // Handle retreating parties
    handle_retreating_parties(state);
}

fn process_single_party(state: &mut GameState, party_id: u64) {
    let party_idx = match state
        .adventurer_parties
        .iter()
        .position(|p| p.id == party_id)
    {
        Some(idx) => idx,
        None => return,
    };

    // Skip retreating parties
    if state.adventurer_parties[party_idx].retreating {
        return;
    }

    let current_floor = state.adventurer_parties[party_idx].current_floor;
    let current_room = state.adventurer_parties[party_idx].current_room;

    // Find floor and room
    let floor_idx = match state.floors.iter().position(|f| f.number == current_floor) {
        Some(idx) => idx,
        None => return,
    };

    let room_idx = match state.floors[floor_idx]
        .rooms
        .iter()
        .position(|r| r.position == current_room)
    {
        Some(idx) => idx,
        None => return,
    };

    mark_room_explored(state, floor_idx, room_idx);

    // Check for combat
    let has_alive_monsters = state.floors[floor_idx].rooms[room_idx]
        .monsters
        .iter()
        .any(|m| m.alive);

    if has_alive_monsters {
        // Combat happens in combat module
        super::combat::resolve_combat(state, party_idx, floor_idx, room_idx);
    } else {
        // Room cleared, advance
        advance_party(state, party_idx);
    }
}

fn mark_room_explored(state: &mut GameState, floor_idx: usize, room_idx: usize) {
    if let Some(room) = state
        .floors
        .get_mut(floor_idx)
        .and_then(|floor| floor.rooms.get_mut(room_idx))
    {
        room.explored = true;
    }
}

fn advance_party(state: &mut GameState, party_idx: usize) {
    let party = &state.adventurer_parties[party_idx];
    let current_floor = party.current_floor;
    let current_room = party.current_room;
    let target_floor = party.target_floor;

    // Find current floor
    let floor = match state.floors.iter().find(|f| f.number == current_floor) {
        Some(f) => f,
        None => return,
    };

    let next_room_pos = current_room + 1;
    let max_room_pos = floor.rooms.iter().map(|r| r.position).max().unwrap_or(0);

    if next_room_pos > max_room_pos {
        // A siege party at the bottom assaults the core itself.
        if state.adventurer_parties[party_idx].sieging && current_floor >= target_floor {
            let party_spent = super::endgame::assault_core(state, party_idx);
            // Repel only if the core survived; if it fell, the run is over.
            if party_spent && !state.game_over {
                super::endgame::repel_siege(state);
            }
            return;
        }
        // End of floor
        if current_floor < target_floor && current_floor < state.floors.len() as i32 {
            // Descend to next floor
            state.adventurer_parties[party_idx].current_floor += 1;
            state.adventurer_parties[party_idx].current_room = 0;
            state.add_log(LogEntry::adventure(format!(
                "Party descends to floor {}",
                current_floor + 1
            )));
        } else {
            // Completed exploration, retreat with loot
            let loot = state.adventurer_parties[party_idx].loot;
            state.gold += loot;
            state.adventurer_parties[party_idx].retreating = true;
            state.add_log(LogEntry::adventure(format!(
                "Party completed exploration! +{} gold",
                loot
            )));

            // Exit quote
            let exit_quotes = get_exit_quotes();
            if macroquad_toolkit::rng::chance(0.4) && !exit_quotes.is_empty() {
                let quote = macroquad_toolkit::rng::choose(&exit_quotes).unwrap();
                if let Some(adv) = state.adventurer_parties[party_idx]
                    .members
                    .iter()
                    .find(|a| a.alive)
                {
                    state.add_log(LogEntry::adventure(format!(
                        "{} says: \"{}\"",
                        adv.name, quote
                    )));
                }
            }
        }
    } else {
        // Advance to next room
        state.adventurer_parties[party_idx].current_room = next_room_pos;
        state.add_log(LogEntry::adventure(format!(
            "Party advances to room {} on floor {}",
            next_room_pos, current_floor
        )));
    }
}

fn handle_retreating_parties(state: &mut GameState) {
    // Settle the ledger for every departing party before it is removed:
    // survivors bank XP and gold and level up; the fallen are entombed.
    let departing: Vec<usize> = state
        .adventurer_parties
        .iter()
        .enumerate()
        .filter(|(_, p)| p.retreating)
        .map(|(i, _)| i)
        .collect();
    for idx in departing {
        settle_departing_party(state, idx);
    }

    let before = state.adventurer_parties.len();
    state.adventurer_parties.retain(|party| !party.retreating);
    let departed = before - state.adventurer_parties.len();
    if departed > 0 {
        // Every party that leaves (looted out or wiped/retreated) is a raid the
        // dungeon has weathered.
        state.raids_completed += departed as i32;
    }

    // Respawn monsters and re-arm sprung traps once the dungeon is clear
    if state.adventurer_parties.is_empty() {
        super::monsters::respawn_monsters(state);
        super::combat::rearm_traps(state);
    }
}

/// Update the hero ledger for a party that is leaving the dungeon.
fn settle_departing_party(state: &mut GameState, party_idx: usize) {
    let party_floor = state.adventurer_parties[party_idx].current_floor;
    let survivors: Vec<u64> = state.adventurer_parties[party_idx]
        .members
        .iter()
        .filter(|m| m.alive)
        .map(|m| m.id)
        .collect();
    let survivor_count = survivors.len().max(1) as i32;
    let loot_share = state.adventurer_parties[party_idx].loot / survivor_count;

    let member_ids: Vec<(u64, bool)> = state.adventurer_parties[party_idx]
        .members
        .iter()
        .map(|m| (m.id, m.alive))
        .collect();

    for (id, alive) in member_ids {
        if alive {
            // Escaped: bank XP, gold, and possibly a level.
            if let Some(record) = state.hero_mut(id) {
                record.status = HeroStatus::Alive;
                record.experience += 20 + record.delves * 5;
                record.gold_stolen += loot_share;
                while record.level < 10
                    && record.experience >= GameState::xp_for_level(record.level)
                {
                    record.experience -= GameState::xp_for_level(record.level);
                    record.level += 1;
                }
            }
        } else {
            state.record_hero_death(id, party_floor);
        }
    }
}
