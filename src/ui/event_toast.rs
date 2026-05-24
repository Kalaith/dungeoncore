use macroquad::prelude::*;
use macroquad_toolkit::input::was_clicked_rect;

use crate::game_state::{GameState, LogEntry};

use super::theme::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventToastAction {
    None,
    ToggleLog,
}

pub fn draw_event_toast(state: &GameState, rect: Rect, expanded: bool) -> EventToastAction {
    let mut action = EventToastAction::None;
    let latest = state.log.last();

    if expanded {
        draw_expanded_log(state, Rect::new(rect.x, rect.y - 134.0, rect.w, 168.0));
    } else if let Some(entry) = latest {
        draw_toast_entry(entry, rect);
    }

    let button = Rect::new(rect.x + rect.w + 8.0, rect.y, 54.0, rect.h);
    draw_card(
        button,
        Color::new(MANA.r, MANA.g, MANA.b, 0.08),
        Color::new(MANA.r, MANA.g, MANA.b, 0.24),
    );
    draw_centered_text(if expanded { "Hide" } else { ">" }, button, 16.0, MANA);
    if was_clicked_rect(button) {
        action = EventToastAction::ToggleLog;
    }

    action
}

fn draw_toast_entry(entry: &LogEntry, rect: Rect) {
    let color = event_color(entry);
    draw_card(
        rect,
        Color::new(0.018, 0.016, 0.030, 0.88),
        Color::new(color.r, color.g, color.b, 0.30),
    );
    draw_text_fit("Event Log", rect.x + 42.0, rect.y + 26.0, 110.0, 13.0, SOUL);
    draw_poly_lines(
        rect.x + 24.0,
        rect.y + rect.h * 0.5,
        6,
        8.0,
        30.0,
        1.5,
        SOUL,
    );
    draw_text_fit(
        &entry.message,
        rect.x + 154.0,
        rect.y + 26.0,
        rect.w - 266.0,
        12.0,
        TEXT_MUTED,
    );
    draw_text_fit_right(
        "Just now",
        rect.x + rect.w - 18.0,
        rect.y + 26.0,
        82.0,
        11.0,
        TEXT_MUTED,
    );
}

fn draw_expanded_log(state: &GameState, rect: Rect) {
    draw_panel(rect, Some("Recent Events"), MANA);
    let inner = Rect::new(rect.x + 12.0, rect.y + 38.0, rect.w - 24.0, rect.h - 48.0);
    let mut y = inner.y + 13.0;
    for entry in state.log.iter().rev().take(6) {
        let color = event_color(entry);
        draw_text_fit(event_label(entry), inner.x, y, 38.0, 10.0, color);
        draw_text_fit(
            &entry.message,
            inner.x + 42.0,
            y,
            inner.w - 42.0,
            12.0,
            TEXT_MUTED,
        );
        y += 19.0;
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
