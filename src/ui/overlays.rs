use macroquad::prelude::*;

use crate::game_state::GameState;

use super::theme::*;

/// Full-screen "the core has fallen" overlay. Returns true when the player
/// clicks to begin a new dungeon.
pub fn draw_game_over_overlay(state: &GameState, sw: f32, sh: f32) -> bool {
    let w = 520.0_f32.min(sw - 40.0);
    let h = 300.0_f32.min(sh - 40.0);
    let x = (sw - w) / 2.0;
    let y = (sh - h) / 2.0;
    let panel = Rect::new(x, y, w, h);
    draw_panel(panel, None, DANGER);

    draw_centered_text(
        "THE CORE HAS FALLEN",
        Rect::new(x, y + 40.0, w, 30.0),
        30.0,
        DANGER,
    );
    draw_centered_text(
        "The realm's army has shattered your dungeon heart.",
        Rect::new(x, y + 96.0, w, 20.0),
        14.0,
        TEXT,
    );
    draw_centered_text(
        &format!(
            "You survived {} days and repelled {} sieges.",
            state.day, state.prestige
        ),
        Rect::new(x, y + 130.0, w, 20.0),
        13.0,
        TEXT_MUTED,
    );
    draw_centered_text(
        &format!("Adventurers slain across the run: {}", state.total_deaths),
        Rect::new(x, y + 156.0, w, 20.0),
        12.0,
        TEXT_MUTED,
    );

    let btn = Rect::new(x + w / 2.0 - 110.0, y + h - 70.0, 220.0, 44.0);
    draw_command_button(btn, "Raise a New Dungeon", ButtonTone::Primary, true)
}
