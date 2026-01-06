//! Dungeon Core - A dungeon management game
//!
//! Migrated from React/TypeScript + PHP to Rust using macroquad.

mod data;
mod game_state;
mod persistence;
mod simulation;
mod ui;

use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use game_state::GameState;
use ui::*;

// Helper to check if any modal is open
fn is_modal_open(state: &GameState) -> bool {
    // Add other modals here if needed
    state.unlocked_species.is_empty()
}


fn window_conf() -> Conf {
    Conf {
        window_title: "Dungeon Core".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Load or create new game
    let mut state = persistence::load_game().unwrap_or_else(|_| {
        println!("Starting new game...");
        let mut new_state = GameState::new();
        
        // Automatically unlock the first species (Goblinoid) for new games
        new_state.unlocked_species.push("Goblinoid".to_string());
        new_state.unlocked_monsters.push("Goblin".to_string());
        
        // Place starting Goblin in core room of floor 1
        if let Err(e) = simulation::place_monster(&mut new_state, 1, 1, "Goblin") {
            eprintln!("Error placing starting Goblin: {}", e);
        } else {
            new_state.add_log(crate::game_state::LogEntry::system("A Goblin has been placed in your core room."));
        }
        
        new_state
    });

    // Timing variables
    let mut last_time_advance = get_time();
    let mut last_adventure_tick = get_time();
    let mut last_save = get_time();

    loop {
        clear_background(dark::BACKGROUND);

        let now = get_time();
        let sw = screen_width();
        let sh = screen_height();

        // === Time-based Updates ===

        // Advance game time based on speed
        let time_interval = 5.0 / state.speed as f64;
        if now - last_time_advance > time_interval {
            simulation::advance_time(&mut state);
            last_time_advance = now;
        }

        // Process adventurer system
        if now - last_adventure_tick > 2.0 {
            simulation::spawn_party(&mut state);
            simulation::process_parties(&mut state);
            last_adventure_tick = now;
        }

        // Auto-save every 30 seconds
        if now - last_save > 30.0 {
            if let Err(e) = persistence::save_game(&state) {
                eprintln!("Failed to save: {}", e);
            }
            last_save = now;
        }


        
        // Modal overlay: Species Selection (Prioritize over everything else)
        if state.unlocked_species.is_empty() {
             let modal_w = 400.0;
             let modal_h = 500.0;
             let modal_x = (sw - modal_w) / 2.0;
             let modal_y = (sh - modal_h) / 2.0;

             // Draw a semi-transparent background to dim the game
             draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.7));

             if let Some(selected_species_id) = draw_species_selector(&mut state, modal_x, modal_y, modal_w, modal_h) {
                 // Unlock the selected species
                 if let Err(e) = simulation::unlock_species(&mut state, &selected_species_id) {
                     eprintln!("Error unlocking species: {}", e);
                 } else {
                     // Species unlocked successfully - player can now place monsters manually
                     state.add_log(crate::game_state::LogEntry::system(format!(
                         "Unlocked {} species! Build rooms and place monsters to defend your dungeon.",
                         selected_species_id
                     )));
                 }
             }

             // Skip drawing other UI if modal is open (optional, but good for focus)
             next_frame().await;
             continue;
        }

        // Top bar: Time display
        draw_time_display(&state, 15.0, 35.0);

        // Left sidebar
        let sidebar_x = 10.0;
        draw_resource_panel(&state, sidebar_x, TOP_BAR_HEIGHT, SIDEBAR_WIDTH);

        // Monster selector
        let monster_panel_y = TOP_BAR_HEIGHT + 140.0;
        let monster_panel_h = 220.0;
        if let Some(monster) = draw_monster_selector(&mut state, sidebar_x, monster_panel_y, SIDEBAR_WIDTH, monster_panel_h) {
            if state.selected_monster.as_ref() == Some(&monster) {
                state.selected_monster = None; // Deselect if clicking same
            } else {
                state.selected_monster = Some(monster);
            }
        }

        // Controls panel
        let controls_y = monster_panel_y + monster_panel_h + 10.0;
        let control_action = draw_controls(&state, sidebar_x, controls_y, SIDEBAR_WIDTH);
        match control_action {
            ControlAction::ToggleSpeed => simulation::toggle_speed(&mut state),
            ControlAction::ToggleDungeon => simulation::toggle_dungeon_status(&mut state),
            ControlAction::RespawnMonsters => simulation::respawn_monsters(&mut state),
            ControlAction::AddRoom => {
                if let Err(e) = simulation::add_room(&mut state, None) {
                    state.add_log(game_state::LogEntry::system(e));
                }
            }
            ControlAction::ResetGame => {
                state = GameState::new();
                let _ = persistence::save_game(&state);
            }
            ControlAction::ProcessEvolutions => simulation::process_evolutions(&mut state),
            ControlAction::None => {}
        }

        // Main dungeon view
        let dungeon_x = SIDEBAR_WIDTH + 20.0;
        let dungeon_w = sw - dungeon_x - 10.0;
        let dungeon_h = sh - TOP_BAR_HEIGHT - LOG_HEIGHT - 20.0;

        if let Some((floor_num, room_pos)) =
            draw_dungeon(&state, dungeon_x, TOP_BAR_HEIGHT, dungeon_w, dungeon_h)
        {
            // Handle room click
            if let Some(ref monster_name) = state.selected_monster.clone() {
                // Place selected monster
                if let Err(e) = simulation::place_monster(&mut state, floor_num, room_pos, monster_name) {
                    state.add_log(game_state::LogEntry::system(e));
                }
                state.selected_monster = None;
            } else {
                // Toggle room selection
                if state.selected_room == Some((floor_num, room_pos)) {
                    state.selected_room = None;
                } else {
                    state.selected_room = Some((floor_num, room_pos));
                }
            }
        }

        // Upgrade panel (when room is selected)
        if state.selected_room.is_some() {
            let upgrade_panel_w = 220.0;
            let upgrade_panel_h = 350.0;
            let upgrade_panel_x = sw - upgrade_panel_w - 15.0;
            let upgrade_panel_y = TOP_BAR_HEIGHT + 10.0;
            
            let upgrade_action = draw_upgrade_panel(&state, upgrade_panel_x, upgrade_panel_y, upgrade_panel_w, upgrade_panel_h);
            match upgrade_action {
                UpgradeAction::Apply(name) => {
                    if let Some((floor, pos)) = state.selected_room {
                        if let Err(e) = simulation::apply_upgrade(&mut state, floor, pos, &name) {
                            state.add_log(game_state::LogEntry::system(e));
                        }
                    }
                }
                UpgradeAction::Remove => {
                    if let Some((floor, pos)) = state.selected_room {
                        if let Err(e) = simulation::remove_upgrade(&mut state, floor, pos) {
                            state.add_log(game_state::LogEntry::system(e));
                        }
                    }
                }
                UpgradeAction::Close => {
                    state.selected_room = None;
                }
                UpgradeAction::None => {}
            }
        }

        // Bottom log panel
        draw_game_log(&state, dungeon_x, sh - LOG_HEIGHT - 10.0, dungeon_w, LOG_HEIGHT);


        next_frame().await;
    }
}
