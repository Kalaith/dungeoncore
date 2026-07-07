//! BUILD tab: next-room summary, the build button, and soul-bought core powers.

use macroquad::prelude::*;

use crate::data::constants::{get_room_cost, MAX_ROOMS_PER_FLOOR};
use crate::game_state::{GameState, RoomType};
use crate::ui::theme::*;

use super::{draw_section_title, BuildTabAction};

pub(super) fn draw_build_tab(state: &GameState, rect: Rect) -> BuildTabAction {
    draw_section_title(rect, "BUILD", "Shape the dungeon path.");

    let (label, detail, cost) = next_build_summary(state);
    let can_build = state.adventurer_parties.is_empty() && state.mana >= cost;

    let card = Rect::new(rect.x, rect.y + 70.0, rect.w, 126.0);
    draw_card(
        card,
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.075),
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.24),
    );
    draw_text_fit(
        &label,
        card.x + 12.0,
        card.y + 27.0,
        card.w - 24.0,
        17.0,
        TEXT,
    );
    draw_text_fit(
        &detail,
        card.x + 12.0,
        card.y + 56.0,
        card.w - 24.0,
        12.0,
        TEXT_MUTED,
    );
    draw_text_fit(
        &format!("{cost} mana"),
        card.x + 12.0,
        card.y + 86.0,
        card.w - 24.0,
        14.0,
        if state.mana >= cost { MANA } else { DANGER },
    );
    draw_text_fit(
        if can_build {
            "Click the glowing room or build here."
        } else if state.adventurer_parties.is_empty() {
            "Gather more mana."
        } else {
            "Wait until the dungeon is safe."
        },
        card.x + 12.0,
        card.y + 112.0,
        card.w - 24.0,
        12.0,
        if can_build { EMERALD } else { TEXT_DIM },
    );

    let build_clicked = draw_command_button(
        Rect::new(rect.x, card.y + card.h + 16.0, rect.w, 42.0),
        "Build",
        ButtonTone::Arcane,
        can_build,
    );
    if build_clicked {
        return BuildTabAction::Build;
    }

    // Permanent, soul-bought core powers below the build controls.
    let mut y = card.y + card.h + 70.0;
    draw_text_fit(
        &format!("CORE POWERS · {} souls", state.souls),
        rect.x,
        y,
        rect.w,
        10.0,
        SOUL,
    );
    y += 14.0;
    for power in crate::simulation::endgame::CORE_POWERS.iter() {
        if y + 46.0 > rect.y + rect.h {
            break;
        }
        let owned = state.has_core_power(power.id);
        let affordable = state.souls >= power.cost;
        let row = Rect::new(rect.x, y, rect.w, 44.0);
        let accent = if owned { EMERALD } else { SOUL };
        draw_card(
            row,
            Color::new(accent.r, accent.g, accent.b, 0.06),
            Color::new(accent.r, accent.g, accent.b, 0.24),
        );
        draw_text_fit(
            power.name,
            row.x + 10.0,
            row.y + 16.0,
            row.w - 60.0,
            12.0,
            TEXT,
        );
        draw_text_fit(
            power.description,
            row.x + 10.0,
            row.y + 33.0,
            row.w - 60.0,
            9.0,
            TEXT_MUTED,
        );
        if owned {
            draw_pill(
                Rect::new(row.x + row.w - 52.0, row.y + 6.0, 44.0, 16.0),
                "OWNED",
                EMERALD,
            );
        } else {
            let btn = Rect::new(row.x + row.w - 54.0, row.y + 8.0, 48.0, 28.0);
            if draw_command_button(
                btn,
                &format!("{}s", power.cost),
                ButtonTone::Arcane,
                affordable,
            ) {
                return BuildTabAction::BuyPower(power.id.to_string());
            }
        }
        y += 50.0;
    }

    BuildTabAction::None
}

fn next_build_summary(state: &GameState) -> (String, String, i32) {
    let Some(deepest) = state.deepest_floor() else {
        return (
            "Entrance".to_string(),
            "No floor mapped yet.".to_string(),
            0,
        );
    };

    let non_core_count = deepest
        .rooms
        .iter()
        .filter(|room| room.room_type != RoomType::Core)
        .count();
    let total_rooms = state.total_room_count();

    if non_core_count > MAX_ROOMS_PER_FLOOR {
        return (
            format!("Open floor {}", state.total_floors + 1),
            "Move the core deeper.".to_string(),
            get_room_cost(total_rooms, false),
        );
    }

    let is_boss = non_core_count == MAX_ROOMS_PER_FLOOR;
    (
        if is_boss {
            "Boss chamber".to_string()
        } else {
            "Combat room".to_string()
        },
        format!("Floor {}", deepest.number),
        get_room_cost(total_rooms, is_boss),
    )
}
