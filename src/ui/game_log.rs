use macroquad::prelude::*;
use macroquad_toolkit::{colors::dark, ui::panel};

use crate::game_state::GameState;
use macroquad_toolkit::ui::draw_ui_text;

/// Draw the game log panel
pub fn draw_game_log(state: &GameState, x: f32, y: f32, w: f32, h: f32) {
    panel(x, y, w, h, Some("Game Log"));

    let max_entries = ((h - 40.0) / 16.0) as usize;
    let mut log_y = y + 35.0;

    for entry in state.log.iter().rev().take(max_entries) {
        let color = match entry.log_type.as_str() {
            "combat" => Color::from_hex(0xE74C3C),
            "adventure" => Color::from_hex(0x3498DB),
            "building" => Color::from_hex(0x2ECC71),
            _ => dark::TEXT,
        };

        // Truncate message if too long
        let max_chars = ((w - 20.0) / 7.0) as usize;
        let msg = if entry.message.len() > max_chars {
            format!("{}...", &entry.message[..max_chars - 3])
        } else {
            entry.message.clone()
        };

        draw_ui_text(&msg, x + 10.0, log_y, 13.0, color);
        log_y += 16.0;
    }
}
