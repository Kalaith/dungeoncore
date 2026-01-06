use crate::data::adventurers::{get_adventurer_classes, get_adventurer_names, get_entry_quotes, get_exit_quotes};
use crate::data::constants::{ADVENTURER_SPAWN_CHANCE, MAX_PARTY_SIZE, MIN_PARTY_SIZE};
use crate::game_state::{
    Adventurer, AdventurerParty, DungeonStatus, Equipment, GameState, LogEntry, Stats,
};
use rand::Rng;

/// Try to spawn a new adventurer party
pub fn spawn_party(state: &mut GameState) {
    // Only spawn when open and no parties present
    if state.status != DungeonStatus::Open {
        return;
    }
    if !state.adventurer_parties.is_empty() {
        return;
    }
    if state.hour < state.next_party_spawn {
        return;
    }

    let mut rng = rand::thread_rng();
    if rng.gen::<f32>() >= ADVENTURER_SPAWN_CHANCE {
        return;
    }

    let classes = get_adventurer_classes();
    let names = get_adventurer_names();
    let entry_quotes = get_entry_quotes();

    let party_size = rng.gen_range(MIN_PARTY_SIZE..=MAX_PARTY_SIZE);
    let mut members = Vec::with_capacity(party_size);

    for _ in 0..party_size {
        let class = &classes[rng.gen_range(0..classes.len())];
        let name = &names[rng.gen_range(0..names.len())];
        let level = rng.gen_range(1..=3);
        let hp = class.hp + (level - 1) * 10;

        members.push(Adventurer {
            id: rng.gen(),
            name: name.clone(),
            class_name: class.name.clone(),
            level,
            hp,
            max_hp: hp,
            alive: true,
            experience: 0,
            gold: 0,
            equipment: Equipment::default(),
            conditions: Vec::new(),
            scaled_stats: Stats {
                hp,
                attack: class.attack + (level - 1) * 2,
                defense: class.defense + (level - 1),
            },
        });
    }

    let target_floor = state.floors.len().min(2) as i32;

    let party = AdventurerParty {
        id: rng.gen(),
        members,
        current_floor: 1,
        current_room: 0,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: state.hour,
        target_floor,
    };

    state.add_log(LogEntry::adventure(format!(
        "New adventurer party enters! ({} members)",
        party.members.len()
    )));

    // Random entry quote
    if rng.gen::<f32>() < 0.3 && !entry_quotes.is_empty() {
        let quote = &entry_quotes[rng.gen_range(0..entry_quotes.len())];
        let name = &party.members[0].name;
        state.add_log(LogEntry::adventure(format!("{} says: \"{}\"", name, quote)));
    }

    state.adventurer_parties.push(party);
    state.next_party_spawn = state.hour + 1;
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
            let mut rng = rand::thread_rng();
            if rng.gen::<f32>() < 0.4 && !exit_quotes.is_empty() {
                let quote = &exit_quotes[rng.gen_range(0..exit_quotes.len())];
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
    state.adventurer_parties.retain(|party| {
        if party.retreating {
            // Party has left
            false
        } else {
            true
        }
    });

    // Respawn monsters if no parties remain
    if state.adventurer_parties.is_empty() {
        super::monsters::respawn_monsters(state);
    }
}
