//! EVOLVE tab: current defenders, their next forms, and species unlocks.

use macroquad::prelude::*;

use crate::data::evolutions::get_evolution_for_monster;
use crate::data::monsters::{get_all_species, get_species_display_name};
use crate::game_state::GameState;
use crate::ui::theme::*;

use super::{draw_section_title, DrawerAction};

pub(super) fn draw_evolution_tab(state: &GameState, rect: Rect) -> DrawerAction {
    draw_section_title(rect, "EVOLUTION", "Advance unlocked species.");
    let mut action = DrawerAction::None;

    let rows = collect_evolution_rows(state);
    let ready_count = rows.iter().filter(|row| row.ready).count();
    let waiting_count = rows.iter().filter(|row| !row.ready && row.has_path).count();
    let final_count = rows.iter().filter(|row| !row.has_path).count();

    let card = Rect::new(rect.x, rect.y + 70.0, rect.w, 126.0);
    draw_card(card, CARD, BORDER_MUTED);
    draw_text_fit(
        &format!("Ready: {}  Waiting: {}", ready_count, waiting_count),
        card.x + 12.0,
        card.y + 28.0,
        card.w - 24.0,
        15.0,
        TEXT,
    );
    draw_text_fit(
        &format!("Final forms: {}", final_count),
        card.x + 12.0,
        card.y + 55.0,
        card.w - 24.0,
        13.0,
        TEXT_MUTED,
    );
    draw_text_fit(
        &format!(
            "Species: {}  Souls: {}",
            state.unlocked_species.len(),
            state.souls
        ),
        card.x + 12.0,
        card.y + 82.0,
        card.w - 24.0,
        13.0,
        SOUL,
    );
    draw_text_fit(
        "Current defenders and their next forms.",
        card.x + 12.0,
        card.y + 108.0,
        card.w - 24.0,
        11.0,
        TEXT_DIM,
    );

    let mut row_y = card.y + card.h + 12.0;
    let row_h = 46.0;
    for row in rows
        .iter()
        .take(((rect.y + rect.h - row_y - 106.0) / row_h).max(0.0) as usize)
    {
        draw_evolution_row(row, Rect::new(rect.x, row_y, rect.w, row_h - 6.0));
        row_y += row_h;
    }

    if let Some(species) = next_locked_species(state) {
        let unlock_cost = species.unlock_cost;
        let can_afford = state.gold >= unlock_cost;
        let species_name = species.name.clone();
        let unlock_rect = Rect::new(rect.x, rect.y + rect.h - 94.0, rect.w, 40.0);
        draw_card(
            unlock_rect,
            Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.075),
            Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.24),
        );
        draw_text_fit(
            &format!("Next race: {}", get_species_display_name(&species_name)),
            unlock_rect.x + 10.0,
            unlock_rect.y + 16.0,
            unlock_rect.w - 96.0,
            11.0,
            TEXT,
        );
        draw_text_fit(
            &format!("{} gold", unlock_cost),
            unlock_rect.x + 10.0,
            unlock_rect.y + 32.0,
            unlock_rect.w - 96.0,
            10.0,
            if can_afford { TREASURE } else { TEXT_DIM },
        );
        if draw_command_button(
            Rect::new(
                unlock_rect.x + unlock_rect.w - 78.0,
                unlock_rect.y + 7.0,
                68.0,
                26.0,
            ),
            "Unlock",
            ButtonTone::Ghost,
            can_afford,
        ) {
            action = DrawerAction::UnlockSpecies(species_name);
        }
    }

    if draw_command_button(
        Rect::new(rect.x, rect.y + rect.h - 46.0, rect.w, 42.0),
        "Evolve",
        ButtonTone::Arcane,
        ready_count > 0,
    ) {
        action = DrawerAction::ProcessEvolutions;
    }

    action
}

#[derive(Debug)]
struct EvolutionUiRow {
    monster: String,
    location: String,
    xp_label: String,
    status: String,
    color: Color,
    ready: bool,
    has_path: bool,
}

fn collect_evolution_rows(state: &GameState) -> Vec<EvolutionUiRow> {
    let mut rows = Vec::new();

    for floor in &state.floors {
        for room in &floor.rooms {
            for monster in &room.monsters {
                let location = format!("F{} R{}", room.floor_number, room.position);
                if let Some(path) = get_evolution_for_monster(&monster.type_name) {
                    let ready_xp = monster.experience >= path.experience_required;
                    let ready_floor = room.floor_number >= path.conditions.min_floor;
                    let ready_gold = state.gold >= path.conditions.gold_cost;
                    let ready = ready_xp && ready_floor && ready_gold;
                    let (status, color) = if ready {
                        (format!("Ready -> {}", path.to_monster), EMERALD)
                    } else if !ready_xp {
                        (
                            format!("Needs {} XP", path.experience_required - monster.experience),
                            MANA,
                        )
                    } else if !ready_floor {
                        (
                            format!("Needs floor {}", path.conditions.min_floor),
                            WARNING,
                        )
                    } else {
                        (
                            format!("Needs {} gold", path.conditions.gold_cost),
                            TREASURE,
                        )
                    };

                    rows.push(EvolutionUiRow {
                        monster: monster.type_name.clone(),
                        location,
                        xp_label: format!("{}/{} XP", monster.experience, path.experience_required),
                        status,
                        color,
                        ready,
                        has_path: true,
                    });
                } else {
                    rows.push(EvolutionUiRow {
                        monster: monster.type_name.clone(),
                        location,
                        xp_label: format!("{} XP", monster.experience),
                        status: "Final form".to_string(),
                        color: TEXT_DIM,
                        ready: false,
                        has_path: false,
                    });
                }
            }
        }
    }

    rows.sort_by(|a, b| {
        b.ready
            .cmp(&a.ready)
            .then_with(|| a.monster.cmp(&b.monster))
    });
    rows
}

fn draw_evolution_row(row: &EvolutionUiRow, rect: Rect) {
    draw_card(
        rect,
        Color::new(row.color.r, row.color.g, row.color.b, 0.075),
        Color::new(row.color.r, row.color.g, row.color.b, 0.26),
    );
    draw_text_fit(
        &row.monster,
        rect.x + 9.0,
        rect.y + 16.0,
        rect.w - 82.0,
        12.0,
        TEXT,
    );
    draw_text_fit(
        &format!("{}  {}", row.location, row.xp_label),
        rect.x + 9.0,
        rect.y + 32.0,
        rect.w - 82.0,
        10.0,
        TEXT_MUTED,
    );
    draw_text_fit_right(
        &row.status,
        rect.x + rect.w - 8.0,
        rect.y + 25.0,
        92.0,
        10.0,
        row.color,
    );
}

fn next_locked_species(state: &GameState) -> Option<crate::data::monsters::SpeciesData> {
    let mut locked = get_all_species()
        .into_iter()
        .filter(|species| !state.unlocked_species.contains(&species.name))
        .collect::<Vec<_>>();
    locked.sort_by_key(|species| species.unlock_cost);
    locked.into_iter().next()
}
