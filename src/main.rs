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
use macroquad_toolkit::capture;

use game_state::GameState;
use ui::*;

/// Env-var prefix for the screenshot capture harness (DUNGEON_CORE_CAPTURE_*).
const CAPTURE_PREFIX: &str = "DUNGEON_CORE";

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
    capture::capture_window_conf(CAPTURE_PREFIX, "Dungeon Core", 1280, 720)
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

    // Screenshot capture harness: when DUNGEON_CORE_CAPTURE_PATH is set, seed a
    // scene, render a fixed number of frames, write a PNG, and exit. No input,
    // no simulation drift, and the player's save file is left untouched.
    if let Some(config) = capture::CaptureConfig::from_env(CAPTURE_PREFIX) {
        let mut cap_state = create_new_game();
        seed_capture_scene(&mut cap_state, &config.scene);
        let mut drawer_tab = DrawerTab::Monsters;
        let mut upgrade_section = UpgradeSection::Traps;
        let mut drawer_open = true;
        let mut upgrade_scroll = 0.0;
        let mut species_scroll = 0.0;
        let mut defender_scroll = 0.0;
        let mut heroes_scroll = 0.0;
        let mut show_codex = false;
        let mut codex_scroll = 0.0;
        let mut t0 = get_time();
        let mut t1 = t0;
        let mut t2 = t0;
        capture::run_capture(&config, |_dt| {
            render_playing_frame(
                &mut cap_state,
                &mut drawer_tab,
                &mut upgrade_section,
                &mut drawer_open,
                &mut upgrade_scroll,
                &mut species_scroll,
                &mut defender_scroll,
                &mut heroes_scroll,
                &mut show_codex,
                &mut codex_scroll,
                &mut t0,
                &mut t1,
                &mut t2,
                false,
            );
        })
        .await;
        return;
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
    let mut upgrade_section = UpgradeSection::Traps;
    let mut drawer_open = true;
    let mut upgrade_scroll = 0.0;
    let mut species_scroll = 0.0;
    let mut defender_scroll = 0.0;
    let mut heroes_scroll = 0.0;
    let mut show_codex = false;
    let mut codex_scroll = 0.0;

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

        render_playing_frame(
            &mut state,
            &mut drawer_tab,
            &mut upgrade_section,
            &mut drawer_open,
            &mut upgrade_scroll,
            &mut species_scroll,
            &mut defender_scroll,
            &mut heroes_scroll,
            &mut show_codex,
            &mut codex_scroll,
            &mut last_time_advance,
            &mut last_adventure_tick,
            &mut last_save,
            true,
        );

        next_frame().await;
    }
}

/// Render (and, when `simulate` is true, step) one frame of the Playing screen.
/// Shared by the interactive loop and the screenshot capture harness; the
/// capture path passes `simulate = false` so the seeded scene stays frozen and
/// the save file is never touched.
#[allow(clippy::too_many_arguments)]
fn render_playing_frame(
    state: &mut GameState,
    drawer_tab: &mut DrawerTab,
    upgrade_section: &mut UpgradeSection,
    drawer_open: &mut bool,
    upgrade_scroll: &mut f32,
    species_scroll: &mut f32,
    defender_scroll: &mut f32,
    heroes_scroll: &mut f32,
    show_codex: &mut bool,
    codex_scroll: &mut f32,
    last_time_advance: &mut f64,
    last_adventure_tick: &mut f64,
    last_save: &mut f64,
    simulate: bool,
) {
    let now = get_time();
    let sw = screen_width();
    let sh = screen_height();
    draw_game_background(sw, sh);

    if simulate {
        // Age transient combat effects each frame.
        state.decay_effects(get_frame_time());

        // === Time-based Updates ===

        // Advance game time based on speed
        let time_interval = 5.0 / state.speed as f64;
        if now - *last_time_advance > time_interval {
            simulation::advance_time(state);
            *last_time_advance = now;
        }

        // Process adventurer system
        if now - *last_adventure_tick > 2.0 {
            simulation::spawn_party(state);
            simulation::process_parties(state);
            *last_adventure_tick = now;
        }

        // Auto-save every 30 seconds
        if now - *last_save > 30.0 {
            if let Err(e) = persistence::save_game(state) {
                eprintln!("Failed to save: {}", e);
            }
            *last_save = now;
        }
    }

    // Game over: the core has fallen. Offer a fresh dungeon.
    if state.game_over {
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.82));
        if draw_game_over_overlay(state, sw, sh) {
            *state = create_new_game();
            let _ = persistence::save_game(state);
            reset_timers(last_time_advance, last_adventure_tick, last_save);
        }
        return;
    }

    // Modal overlay: Species Selection (Prioritize over everything else)
    if state.unlocked_species.is_empty() {
        let modal_w = 460.0;
        let modal_h = 540.0;
        let modal_x = (sw - modal_w) / 2.0;
        let modal_y = (sh - modal_h) / 2.0;

        // Draw a semi-transparent background to dim the game
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.7));

        if let Some(selected_species_id) =
            draw_species_selector(state, modal_x, modal_y, modal_w, modal_h, species_scroll)
        {
            // Unlock the selected species
            if let Err(e) = simulation::unlock_species(state, &selected_species_id) {
                eprintln!("Error unlocking species: {}", e);
            } else {
                // Species unlocked successfully - player can now place monsters manually
                state.add_log(crate::game_state::LogEntry::system(format!(
                     "Chosen starter race: {}. Build rooms and place its units to defend your dungeon.",
                     crate::data::monsters::get_species_display_name(&selected_species_id)
                 )));
            }
        }

        return;
    }

    let hud_rect = Rect::new(
        OUTER_MARGIN,
        OUTER_MARGIN,
        sw - OUTER_MARGIN * 2.0,
        HUD_HEIGHT,
    );
    match draw_top_hud(state, hud_rect) {
        ControlAction::ToggleSpeed => simulation::toggle_speed(state),
        ControlAction::ToggleDungeon => simulation::toggle_dungeon_status(state),
        _ => {}
    }

    let log_rect = Rect::new(
        OUTER_MARGIN,
        sh - OUTER_MARGIN - LOG_BAR_HEIGHT,
        sw - OUTER_MARGIN * 2.0,
        LOG_BAR_HEIGHT,
    );

    let body_top = hud_rect.y + hud_rect.h + PANEL_GAP;
    let body_bottom = log_rect.y - PANEL_GAP;
    let body_h = (body_bottom - body_top).max(220.0);

    let drawer_w = if *drawer_open {
        SIDE_PANEL_WIDTH.min((sw * 0.22).clamp(250.0, DRAWER_OPEN_WIDTH))
    } else {
        DRAWER_COLLAPSED_WIDTH
    };
    let drawer_rect = Rect::new(OUTER_MARGIN, body_top, drawer_w, body_h);
    match draw_side_drawer(
        state,
        drawer_rect,
        drawer_tab,
        drawer_open,
        upgrade_section,
        heroes_scroll,
    ) {
        DrawerAction::SelectMonster(monster) => {
            if state.selected_monster.as_ref() == Some(&monster) {
                state.selected_monster = None;
            } else {
                state.selected_room = None;
                state.selected_upgrade = None;
                state.selected_monster = Some(monster);
            }
        }
        DrawerAction::SelectUpgrade(upgrade) => {
            if state.selected_upgrade.as_ref() == Some(&upgrade) {
                state.selected_upgrade = None;
            } else {
                state.selected_room = None;
                state.selected_monster = None;
                state.selected_upgrade = Some(upgrade);
            }
        }
        DrawerAction::BuildRoom => {
            if let Err(e) = simulation::add_room(state, None) {
                state.add_log(game_state::LogEntry::system(e));
            }
        }
        DrawerAction::ProcessEvolutions => simulation::process_evolutions(state),
        DrawerAction::UnlockSpecies(species) => {
            if let Err(e) = simulation::unlock_species(state, &species) {
                state.add_log(game_state::LogEntry::system(e));
            }
        }
        DrawerAction::BuyCorePower(id) => {
            if let Err(e) = simulation::endgame::buy_core_power(state, &id) {
                state.add_log(game_state::LogEntry::system(e));
            }
        }
        DrawerAction::ResetGame => {
            *state = create_new_game();
            let _ = persistence::save_game(state);
            reset_timers(last_time_advance, last_adventure_tick, last_save);
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

    match draw_dungeon_board(state, dungeon_rect) {
        DungeonAction::RoomSelected(floor_num, room_pos) => {
            if let Some(ref monster_name) = state.selected_monster.clone() {
                // Selection stays armed on success so more can be placed with
                // further clicks; it clears on failure (no mana, bad room) or
                // by re-clicking the drawer entry.
                if let Err(e) = simulation::place_monster(state, floor_num, room_pos, monster_name)
                {
                    state.add_log(game_state::LogEntry::system(e));
                    state.selected_monster = None;
                }
            } else if let Some(ref upgrade_name) = state.selected_upgrade.clone() {
                if let Err(e) = simulation::apply_upgrade(state, floor_num, room_pos, upgrade_name)
                {
                    state.add_log(game_state::LogEntry::system(e));
                    state.selected_upgrade = None;
                }
            } else if state.selected_room == Some((floor_num, room_pos)) {
                state.selected_room = None;
                *upgrade_scroll = 0.0;
                *defender_scroll = 0.0;
            } else {
                state.selected_room = Some((floor_num, room_pos));
                *upgrade_scroll = 0.0;
                *defender_scroll = 0.0;
            }
        }
        DungeonAction::BuildRoom => {
            if let Err(e) = simulation::add_room(state, None) {
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
            state,
            upgrade_panel_x,
            upgrade_panel_y,
            upgrade_panel_w,
            upgrade_panel_h,
            upgrade_scroll,
            defender_scroll,
        );
        match upgrade_action {
            UpgradeAction::Apply(name) => {
                if let Some((floor, pos)) = state.selected_room {
                    if let Err(e) = simulation::apply_upgrade(state, floor, pos, &name) {
                        state.add_log(game_state::LogEntry::system(e));
                    }
                }
            }
            UpgradeAction::Remove(upgrade_type) => {
                if let Some((floor, pos)) = state.selected_room {
                    if let Err(e) = simulation::remove_upgrade(state, floor, pos, upgrade_type) {
                        state.add_log(game_state::LogEntry::system(e));
                    }
                }
            }
            UpgradeAction::DismissMonster(monster_id) => {
                if let Some((floor, pos)) = state.selected_room {
                    if let Err(e) = simulation::remove_monster(state, floor, pos, monster_id) {
                        state.add_log(game_state::LogEntry::system(e));
                    }
                }
            }
            UpgradeAction::Close => {
                state.selected_room = None;
                state.selected_monster = None;
                *upgrade_scroll = 0.0;
                *defender_scroll = 0.0;
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
        state,
        Rect::new(
            dungeon_rect.x + dungeon_rect.w - chip_w - 24.0,
            dungeon_rect.y + 24.0,
            chip_w,
            36.0,
        ),
    );

    // Post-raid summary card: shows what the last raid cost and earned until
    // the player dismisses it (or the next raid replaces it).
    if let Some(summary) = state.last_raid_summary.clone() {
        if draw_raid_summary(&summary, dungeon_rect) {
            state.last_raid_summary = None;
        }
    }

    draw_event_log(state, log_rect);

    // A siege turns the whole screen into an alarm state.
    if state.siege_active {
        draw_siege_overlay(sw, sh);
    }

    // Onboarding tutorial: highlight the relevant panel and advance as the
    // player completes each step.
    if tutorial::is_active(state) {
        let anchor_rect = match tutorial::current_anchor(state) {
            Some(tutorial::TutorialAnchor::Drawer) => drawer_rect,
            Some(tutorial::TutorialAnchor::Hud) => hud_rect,
            _ => dungeon_rect,
        };
        if tutorial::draw(state, dungeon_rect, anchor_rect) {
            tutorial::skip(state);
        }
    }
    if simulate {
        tutorial::advance(state);
    }

    // Codex overlay: opened with 'C', drawn last so it sits over everything.
    if !*show_codex && is_key_pressed(KeyCode::C) {
        *show_codex = true;
        *codex_scroll = 0.0;
    }
    if *show_codex && draw_codex(state, sw, sh, codex_scroll) {
        *show_codex = false;
    }
}

/// First species flagged as a starter, used to seed capture scenes.
fn first_starter_species() -> Option<String> {
    crate::data::monsters::get_all_species()
        .into_iter()
        .find(|species| species.starter)
        .map(|species| species.name)
}

/// First combat-capable room (Normal or Boss) in the dungeon.
fn find_combat_room(state: &GameState) -> Option<(i32, usize)> {
    for floor in &state.floors {
        for room in &floor.rooms {
            if room.room_type == game_state::RoomType::Normal
                || room.room_type == game_state::RoomType::Boss
            {
                return Some((room.floor_number, room.position));
            }
        }
    }
    None
}

/// Seed `state` into a representative scene for a screenshot. Scenes:
/// `species` (starter-race modal), `tutorial` (onboarding overlay), and
/// `gameplay` (default: a mid-raid dungeon showing icons, effects, threat, log).
fn seed_capture_scene(state: &mut GameState, scene: &str) {
    use game_state::{
        Adventurer, AdventurerParty, DungeonStatus, EffectKind, Equipment, LogEntry, Stats,
    };

    state.mana = 999;
    state.max_mana = 999;
    state.gold = 500;

    match scene {
        "species" => {
            state.unlocked_species.clear();
            state.unlocked_monsters.clear();
        }
        "tutorial" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = true;
            state.tutorial_step = 0;
            state.status = DungeonStatus::Closed;
        }
        "placement" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            // Attune the first combat room to Fire so the synergy hint shows.
            if let Some((floor, pos)) = find_combat_room(state) {
                if let Some(f) = state.floors.iter_mut().find(|f| f.number == floor) {
                    if let Some(r) = f.rooms.iter_mut().find(|r| r.position == pos) {
                        r.upgrades.push(game_state::RoomUpgrade {
                            upgrade_type: game_state::RoomUpgradeType::Attunement,
                            name: "Fire Shrine".to_string(),
                            effect: "Fire attunement".to_string(),
                            multiplier: 1.3,
                            element: Some("Fire".to_string()),
                            effect_kind: String::new(),
                            disarmed: false,
                        });
                    }
                }
            }
            state.status = DungeonStatus::Closed;
            // The player is mid-placement with a Fire monster selected.
            state.selected_monster = Some("Ember Wisp".to_string());
        }
        "siege" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                let _ = simulation::place_monster(state, floor, pos, &monster);
            }
            // Peak threat with the dungeon clear musters a real siege party.
            state.total_deaths = 100;
            simulation::endgame::maybe_launch_siege(state);
            state.core_hp = 380;
            state.core_max_hp = 500;
        }
        "summary" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                let _ = simulation::place_monster(state, floor, pos, &monster);
            }
            state.status = DungeonStatus::Open;
            state.total_deaths = 14;
            // A concluded raid, so the post-raid summary card is on screen.
            state.last_raid_summary = Some(game_state::RaidSummary {
                outcome: game_state::RaidOutcome::Wiped,
                party_size: 4,
                slain: 4,
                survivors: 0,
                mana_gained: 60,
                souls_gained: 1,
                gold_gained: 0,
                defenders_lost: 1,
            });
        }
        _ => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;

            // Build a couple of combat rooms.
            let _ = simulation::add_room(state, None);
            let _ = simulation::add_room(state, None);

            // Place defenders in the first combat room.
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                for _ in 0..3 {
                    let _ = simulation::place_monster(state, floor, pos, &monster);
                }
            }

            state.status = DungeonStatus::Open;
            state.total_deaths = 14; // -> "Wary" threat tier

            // Drop an adventuring party into the defended room for a live fight.
            if let Some((floor, pos)) = find_combat_room(state) {
                let members = (0..3u64)
                    .map(|i| Adventurer {
                        id: 100 + i,
                        name: ["Aldric", "Bryn", "Cael"][i as usize].to_string(),
                        class_name: "Warrior".to_string(),
                        race: "Human".to_string(),
                        level: 2,
                        hp: 30,
                        max_hp: 40,
                        alive: true,
                        experience: 0,
                        gold: 0,
                        equipment: Equipment::default(),
                        conditions: Vec::new(),
                        scaled_stats: Stats {
                            hp: 40,
                            attack: 8,
                            defense: 3,
                        },
                    })
                    .collect();
                state.adventurer_parties.push(AdventurerParty {
                    id: 1,
                    members,
                    current_floor: floor,
                    current_room: pos,
                    retreating: false,
                    casualties: 1,
                    loot: 40,
                    entry_time: 8,
                    target_floor: 1,
                    snared_ticks: 0,
                    alarmed: false,
                    sieging: false,
                });

                state.push_effect(floor, pos, "-12", EffectKind::Damage);
                state.push_effect(floor, pos, "Slain!", EffectKind::AdventurerDown);

                // Show the room inspector (defender list + upgrade catalog).
                state.selected_room = Some((floor, pos));
            }

            // Seed the hero ledger so the HEROES tab has content to show.
            use game_state::{HeroRecord, HeroStatus};
            let seed_hero = |id,
                             name: &str,
                             class: &str,
                             race: &str,
                             level,
                             delves,
                             kills,
                             gold,
                             status,
                             df,
                             dd| HeroRecord {
                id,
                name: name.to_string(),
                class_name: class.to_string(),
                race: race.to_string(),
                level,
                experience: 0,
                delves,
                kills,
                gold_stolen: gold,
                status,
                death_floor: df,
                death_day: dd,
            };
            state.known_adventurers = vec![
                seed_hero(
                    100,
                    "Aldric",
                    "Warrior",
                    "Human",
                    2,
                    1,
                    0,
                    0,
                    HeroStatus::Inside,
                    0,
                    0,
                ),
                seed_hero(
                    101,
                    "Bryn",
                    "Warrior",
                    "Dwarf",
                    2,
                    1,
                    0,
                    0,
                    HeroStatus::Inside,
                    0,
                    0,
                ),
                seed_hero(
                    200,
                    "Sable",
                    "Rogue",
                    "Halfling",
                    4,
                    5,
                    12,
                    180,
                    HeroStatus::Alive,
                    0,
                    0,
                ),
                seed_hero(
                    201,
                    "Wren",
                    "Ranger",
                    "Elf",
                    3,
                    3,
                    6,
                    90,
                    HeroStatus::Alive,
                    0,
                    0,
                ),
                seed_hero(
                    300,
                    "Mordred",
                    "Mage",
                    "Human",
                    2,
                    2,
                    3,
                    40,
                    HeroStatus::Dead,
                    2,
                    3,
                ),
            ];

            state.add_log(LogEntry::adventure(
                "New adventurer party enters! (3 members)",
            ));
            state.add_log(LogEntry::combat(
                "Goblin uses Ambush! Dealt 12 damage to 3 adventurers.",
            ));
            state.add_log(LogEntry::combat(
                "Bryn has fallen on floor 1! +20 mana, +10 XP to monsters",
            ));
            state.add_log(LogEntry::building("Spawned defender on floor 1, room 1."));
        }
    }
}
