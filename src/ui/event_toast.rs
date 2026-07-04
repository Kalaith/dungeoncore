use macroquad::prelude::*;

use crate::game_state::{GameState, LogEntry};

use super::theme::*;

/// Draw the always-visible event log panel. Shows the most recent events,
/// colour-coded by type, so the player can follow what is happening in the
/// dungeon without expanding anything.
pub fn draw_event_log(state: &GameState, rect: Rect) {
    draw_panel(rect, Some("Event Log"), MANA);

    let inner = Rect::new(rect.x + 12.0, rect.y + 36.0, rect.w - 24.0, rect.h - 44.0);
    let line_h = 17.0;
    let max_lines = (inner.h / line_h).floor().max(1.0) as usize;

    if state.log.is_empty() {
        draw_text_fit(
            "No events yet. Open the dungeon to draw adventurers in.",
            inner.x,
            inner.y + 14.0,
            inner.w,
            12.0,
            TEXT_DIM,
        );
        return;
    }

    // Oldest of the visible window first, newest at the bottom.
    let entries: Vec<&LogEntry> = state.log.iter().rev().take(max_lines).collect();
    let mut y = inner.y + 13.0;
    for entry in entries.into_iter().rev() {
        let color = event_color(entry);
        draw_text_fit(event_label(entry), inner.x, y, 34.0, 10.0, color);
        draw_text_fit(
            &entry.message,
            inner.x + 40.0,
            y,
            inner.w - 40.0,
            12.0,
            TEXT_MUTED,
        );
        y += line_h;
    }
}

fn event_color(entry: &LogEntry) -> Color {
    match entry.log_type.as_str() {
        "combat" => DANGER,
        "adventure" => WARNING,
        "building" => EMERALD,
        _ => MANA,
    }
}

fn event_label(entry: &LogEntry) -> &'static str {
    match entry.log_type.as_str() {
        "combat" => "COM",
        "adventure" => "ADV",
        "building" => "BLD",
        _ => "SYS",
    }
}
