//! Dungeon Core - A dungeon management game
#![allow(dead_code)]
//!
//! Migrated from React/TypeScript + PHP to Rust using macroquad.

mod data;
mod game_state;
mod persistence;
mod simulation;
mod ui;

use macroquad::miniquad::window::quit;
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;

use game_state::GameState;
use ui::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppScreen {
    Title,
    Settings,
    Playing,
}

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

fn create_new_game() -> GameState {
    GameState::new()
}

fn reset_timers(last_time_advance: &mut f64, last_adventure_tick: &mut f64, last_save: &mut f64) {
    let now = get_time();
    *last_time_advance = now;
    *last_adventure_tick = now;
    *last_save = now;
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut assets = AssetManager::new();
    let _ = assets.load_asset_pack("assets.zip").await;
    if let Err(e) = assets
        .load_texture_with_filter(
            TITLE_BACKGROUND_KEY,
            TITLE_BACKGROUND_PATH,
            FilterMode::Linear,
        )
        .await
    {
        eprintln!("Failed to load title background: {}", e);
    }

    let mut state = persistence::load_game().unwrap_or_else(|_| create_new_game());
    let mut screen = AppScreen::Title;
    let mut title_notice: Option<String> = None;
    let mut fullscreen_enabled = false;

    // Timing variables
    let mut last_time_advance = get_time();
    let mut last_adventure_tick = get_time();
    let mut last_save = get_time();
    let mut drawer_tab = DrawerTab::Monsters;
    let mut drawer_open = true;
    let mut log_expanded = false;
    let mut upgrade_scroll = 0.0;

    loop {
        match screen {
            AppScreen::Title => {
                match draw_title_screen(
                    &assets,
                    persistence::save_exists(),
                    title_notice.as_deref(),
                ) {
                    TitleAction::NewGame => {
                        state = create_new_game();
                        if let Err(e) = persistence::save_game(&state) {
                            eprintln!("Failed to save new game: {}", e);
                        }
                        reset_timers(
                            &mut last_time_advance,
                            &mut last_adventure_tick,
                            &mut last_save,
                        );
                        title_notice = None;
                        screen = AppScreen::Playing;
                    }
                    TitleAction::LoadGame => match persistence::load_game() {
                        Ok(loaded_state) => {
                            state = loaded_state;
                            reset_timers(
                                &mut last_time_advance,
                                &mut last_adventure_tick,
                                &mut last_save,
                            );
                            title_notice = None;
                            screen = AppScreen::Playing;
                        }
                        Err(e) => {
                            title_notice = Some(format!("Load failed: {}", e));
                        }
                    },
                    TitleAction::Settings => {
                        title_notice = None;
                        screen = AppScreen::Settings;
                    }
                    TitleAction::Exit => {
                        quit();
                        return;
                    }
                    TitleAction::None => {}
                }
                next_frame().await;
                continue;
            }
            AppScreen::Settings => {
                match draw_title_settings_screen(
                    &assets,
                    fullscreen_enabled,
                    title_notice.as_deref(),
                ) {
                    TitleSettingsAction::ToggleFullscreen => {
                        fullscreen_enabled = !fullscreen_enabled;
                        set_fullscreen(fullscreen_enabled);
                        title_notice = Some(if fullscreen_enabled {
                            "Fullscreen enabled.".to_string()
                        } else {
                            "Fullscreen disabled.".to_string()
                        });
                    }
                    TitleSettingsAction::Back => {
                        title_notice = None;
                        screen = AppScreen::Title;
                    }
                    TitleSettingsAction::None => {}
                }
                next_frame().await;
                continue;
            }
            AppScreen::Playing => {}
        }

        let now = get_time();
        let sw = screen_width();
        let sh = screen_height();
        draw_game_background(sw, sh);

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

            if let Some(selected_species_id) =
                draw_species_selector(&mut state, modal_x, modal_y, modal_w, modal_h)
            {
                // Unlock the selected species
                if let Err(e) = simulation::unlock_species(&mut state, &selected_species_id) {
                    eprintln!("Error unlocking species: {}", e);
                } else {
                    // Species unlocked successfully - player can now place monsters manually
                    state.add_log(crate::game_state::LogEntry::system(format!(
                         "Chosen starter race: {}. Build rooms and place its units to defend your dungeon.",
                         crate::data::monsters::get_species_display_name(&selected_species_id)
                     )));
                }
            }

            // Skip drawing other UI if modal is open (optional, but good for focus)
            next_frame().await;
            continue;
        }

        let hud_rect = Rect::new(
            OUTER_MARGIN,
            OUTER_MARGIN,
            sw - OUTER_MARGIN * 2.0,
            HUD_HEIGHT,
        );
        draw_top_hud(&state, hud_rect);

        let command_rect = Rect::new(
            OUTER_MARGIN,
            sh - OUTER_MARGIN - COMMAND_BAR_HEIGHT,
            sw - OUTER_MARGIN * 2.0,
            COMMAND_BAR_HEIGHT,
        );

        let body_top = hud_rect.y + hud_rect.h + PANEL_GAP;
        let body_bottom = command_rect.y - PANEL_GAP;
        let body_h = (body_bottom - body_top).max(220.0);

        let drawer_w = if drawer_open {
            SIDE_PANEL_WIDTH.min((sw * 0.22).clamp(250.0, DRAWER_OPEN_WIDTH))
        } else {
            DRAWER_COLLAPSED_WIDTH
        };
        let drawer_rect = Rect::new(OUTER_MARGIN, body_top, drawer_w, body_h);
        match draw_side_drawer(&state, drawer_rect, &mut drawer_tab, &mut drawer_open) {
            DrawerAction::SelectMonster(monster) => {
                if state.selected_monster.as_ref() == Some(&monster) {
                    state.selected_monster = None;
                } else {
                    state.selected_room = None;
                    state.selected_monster = Some(monster);
                }
            }
            DrawerAction::BuildRoom => {
                if let Err(e) = simulation::add_room(&mut state, None) {
                    state.add_log(game_state::LogEntry::system(e));
                }
            }
            DrawerAction::ProcessEvolutions => simulation::process_evolutions(&mut state),
            DrawerAction::UnlockSpecies(species) => {
                if let Err(e) = simulation::unlock_species(&mut state, &species) {
                    state.add_log(game_state::LogEntry::system(e));
                }
            }
            DrawerAction::None => {}
        }

        let has_inspector = state.selected_room.is_some() || state.selected_monster.is_some();
        let right_panel_w = if has_inspector {
            (sw * 0.21).clamp(270.0, 330.0)
        } else {
            0.0
        };
        let right_gap = if right_panel_w > 0.0 { PANEL_GAP } else { 0.0 };
        let dungeon_x = drawer_rect.x + drawer_rect.w + PANEL_GAP;
        let dungeon_w = sw - dungeon_x - right_panel_w - right_gap - OUTER_MARGIN;
        let dungeon_h = body_h;
        let dungeon_rect = Rect::new(
            dungeon_x,
            body_top,
            dungeon_w.max(320.0),
            dungeon_h.max(220.0),
        );

        match draw_dungeon_board(&state, dungeon_rect) {
            DungeonAction::RoomSelected(floor_num, room_pos) => {
                if let Some(ref monster_name) = state.selected_monster.clone() {
                    if let Err(e) =
                        simulation::place_monster(&mut state, floor_num, room_pos, monster_name)
                    {
                        state.add_log(game_state::LogEntry::system(e));
                    }
                    state.selected_monster = None;
                } else if state.selected_room == Some((floor_num, room_pos)) {
                    state.selected_room = None;
                    upgrade_scroll = 0.0;
                } else {
                    state.selected_room = Some((floor_num, room_pos));
                    upgrade_scroll = 0.0;
                }
            }
            DungeonAction::BuildRoom => {
                if let Err(e) = simulation::add_room(&mut state, None) {
                    state.add_log(game_state::LogEntry::system(e));
                }
            }
            DungeonAction::None => {}
        }

        // Inspector panel (room, monster, and upgrade context)
        if has_inspector {
            let upgrade_panel_w = right_panel_w;
            let upgrade_panel_h = dungeon_h;
            let upgrade_panel_x = sw - upgrade_panel_w - OUTER_MARGIN;
            let upgrade_panel_y = body_top;

            let upgrade_action = draw_upgrade_panel(
                &state,
                upgrade_panel_x,
                upgrade_panel_y,
                upgrade_panel_w,
                upgrade_panel_h,
                &mut upgrade_scroll,
            );
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
                    state.selected_monster = None;
                    upgrade_scroll = 0.0;
                }
                UpgradeAction::None => {}
            }
        }

        let chip_w = if state.adventurer_parties.is_empty() {
            132.0
        } else {
            184.0
        };
        draw_adventurer_status_chip(
            &state,
            Rect::new(
                dungeon_rect.x + dungeon_rect.w - chip_w - 24.0,
                dungeon_rect.y + 24.0,
                chip_w,
                36.0,
            ),
        );

        let toast_w = (dungeon_rect.w * 0.66).clamp(420.0, 680.0);
        let toast_rect = Rect::new(
            dungeon_rect.x + (dungeon_rect.w - toast_w) * 0.5 - 28.0,
            dungeon_rect.y + dungeon_rect.h - 56.0,
            toast_w.min(dungeon_rect.w - 92.0),
            44.0,
        );
        if draw_event_toast(&state, toast_rect, log_expanded) == EventToastAction::ToggleLog {
            log_expanded = !log_expanded;
        }

        let control_action = draw_command_bar(&state, command_rect);
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
                state = create_new_game();
                let _ = persistence::save_game(&state);
                reset_timers(
                    &mut last_time_advance,
                    &mut last_adventure_tick,
                    &mut last_save,
                );
            }
            ControlAction::ProcessEvolutions => simulation::process_evolutions(&mut state),
            ControlAction::None => {}
        }

        next_frame().await;
    }
}
