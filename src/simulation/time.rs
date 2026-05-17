use crate::game_state::{DungeonStatus, GameState, LogEntry};

/// Advance game time by one hour
pub fn advance_time(state: &mut GameState) {
    state.hour += 1;
    if state.hour >= 24 {
        state.hour = 0;
        state.day += 1;
    }

    // Calculate adventurer bonus for mana regen
    let adventurer_count: usize = state
        .adventurer_parties
        .iter()
        .map(|p| p.members.iter().filter(|a| a.alive).count())
        .sum();
    let adventurer_bonus = adventurer_count as f32 * 0.1;

    // Mana regeneration
    let regen = 1.0 + state.deep_core_bonus + adventurer_bonus;
    state.mana_regen = regen;
    state.mana = (state.mana + regen as i32).min(state.max_mana);

    // Auto-close dungeon when closing and no parties remain
    if state.status == DungeonStatus::Closing && state.adventurer_parties.is_empty() {
        state.status = DungeonStatus::Closed;
        state.add_log(LogEntry::system("Dungeon is now closed."));
    }

    // Process hourly monster traits
    crate::simulation::monsters::process_hourly_traits(state);

    // Process monster evolutions
    crate::simulation::monsters::process_evolutions(state);
}

/// Toggle dungeon status between Open and Closed
pub fn toggle_dungeon_status(state: &mut GameState) {
    match state.status {
        DungeonStatus::Open => {
            if state.adventurer_parties.is_empty() {
                state.status = DungeonStatus::Closed;
                state.add_log(LogEntry::system("Dungeon is now closed to adventurers."));
            } else {
                state.status = DungeonStatus::Closing;
                state.add_log(LogEntry::system(
                    "Dungeon is closing... waiting for adventurers to finish.",
                ));
            }
        }
        DungeonStatus::Closed | DungeonStatus::Closing => {
            state.status = DungeonStatus::Open;
            state.add_log(LogEntry::system("Dungeon is now open to adventurers!"));
        }
        DungeonStatus::Maintenance => {
            // Can't toggle out of maintenance manually
        }
    }
}

/// Cycle game speed: 1 -> 2 -> 4 -> 1
pub fn toggle_speed(state: &mut GameState) {
    state.speed = match state.speed {
        1 => 2,
        2 => 4,
        _ => 1,
    };
}
