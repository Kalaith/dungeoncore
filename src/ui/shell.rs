use macroquad::prelude::*;

use crate::game_state::{DungeonStatus, GameState, RoomType};

use super::controls::ControlAction;
use super::theme::*;

pub const HUD_HEIGHT: f32 = 74.0;
pub const COMMAND_BAR_HEIGHT: f32 = 76.0;
pub const OUTER_MARGIN: f32 = 8.0;
pub const PANEL_GAP: f32 = 12.0;
pub const SIDE_PANEL_WIDTH: f32 = 274.0;

pub fn draw_top_hud(state: &GameState, rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.0, 0.0, 0.0, 0.34),
    );
    draw_line(
        rect.x,
        rect.y + rect.h - 1.0,
        rect.x + rect.w,
        rect.y + rect.h - 1.0,
        1.0,
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.22),
    );

    let title_w = (rect.w * 0.25).clamp(260.0, 340.0);
    let title_rect = Rect::new(rect.x + 14.0, rect.y + 10.0, title_w, rect.h - 20.0);
    draw_brand_mark(
        vec2(title_rect.x + 30.0, title_rect.y + title_rect.h * 0.5),
        26.0,
    );
    draw_text_fit(
        "DUNGEON CORE",
        title_rect.x + 66.0,
        title_rect.y + 36.0,
        title_rect.w - 70.0,
        26.0,
        TEXT,
    );

    let status_w = (rect.w * 0.15).clamp(150.0, 206.0);
    let stats_x = title_rect.x + title_rect.w + 18.0;
    let stats_w = rect.x + rect.w - stats_x - status_w - 24.0;
    let stat_w = (stats_w / 4.0).clamp(112.0, 170.0);
    let y = rect.y + 14.0;

    draw_top_stat(
        Rect::new(stats_x, y, stat_w, rect.h - 28.0),
        "Mana",
        &format!("{}/{}", state.mana, state.max_mana),
        MANA,
        StatIcon::Mana,
        Some((state.mana as f32, state.max_mana as f32)),
    );
    draw_top_stat(
        Rect::new(stats_x + stat_w, y, stat_w, rect.h - 28.0),
        "Gold",
        &state.gold.to_string(),
        TREASURE,
        StatIcon::Gold,
        None,
    );
    draw_top_stat(
        Rect::new(stats_x + stat_w * 2.0, y, stat_w, rect.h - 28.0),
        "Souls",
        &state.souls.to_string(),
        SOUL,
        StatIcon::Soul,
        None,
    );
    draw_top_stat(
        Rect::new(stats_x + stat_w * 3.0, y, stat_w, rect.h - 28.0),
        "",
        &format!("Day {} {:02}:00", state.day, state.hour),
        TEXT,
        StatIcon::Time,
        None,
    );

    let (status_color, status_label) = match state.status {
        DungeonStatus::Open => (EMERALD, "Open"),
        DungeonStatus::Closing => (WARNING, "Closing"),
        DungeonStatus::Closed => (DANGER, "Closed"),
        DungeonStatus::Maintenance => (TEXT_DIM, "Maint."),
    };
    draw_status_block(
        Rect::new(
            rect.x + rect.w - status_w - 14.0,
            rect.y + 12.0,
            status_w,
            rect.h - 24.0,
        ),
        status_label,
        status_color,
    );
}

pub fn draw_command_bar(state: &GameState, rect: Rect) -> ControlAction {
    let mut action = ControlAction::None;
    draw_panel(rect, None, BORDER);

    let inner = Rect::new(rect.x + 22.0, rect.y + 16.0, rect.w - 44.0, rect.h - 32.0);
    let speed = Rect::new(inner.x, inner.y, 230.0, inner.h);
    let primary = Rect::new(speed.x + speed.w + 34.0, inner.y, 210.0, inner.h);
    let respawn = Rect::new(
        primary.x + primary.w + 36.0,
        inner.y,
        220.0_f32.min(inner.w * 0.18),
        inner.h,
    );
    let evolve = Rect::new(
        respawn.x + respawn.w + 36.0,
        inner.y,
        220.0_f32.min(inner.w * 0.18),
        inner.h,
    );
    let reset = Rect::new(inner.x + inner.w - 164.0, inner.y, 164.0, inner.h);

    if draw_speed_segments(speed, state.speed) {
        action = ControlAction::ToggleSpeed;
    }

    let (status_text, status_tone, enabled) = match state.status {
        DungeonStatus::Open => ("Close Dungeon", ButtonTone::Danger, true),
        DungeonStatus::Closed => ("Open Dungeon", ButtonTone::Primary, true),
        DungeonStatus::Closing => ("Closing...", ButtonTone::Ghost, false),
        DungeonStatus::Maintenance => ("Maintenance", ButtonTone::Ghost, false),
    };
    if draw_command_button(primary, status_text, status_tone, enabled) {
        action = ControlAction::ToggleDungeon;
    }

    let can_respawn = state.adventurer_parties.is_empty();
    if draw_command_button(respawn, "Respawn", ButtonTone::Arcane, can_respawn) {
        action = ControlAction::RespawnMonsters;
    }

    if draw_command_button(evolve, "Evolve", ButtonTone::Ghost, true) {
        action = ControlAction::ProcessEvolutions;
    }

    if draw_reset_button(reset) {
        action = ControlAction::ResetGame;
    }

    action
}

pub fn draw_adventurer_status_chip(state: &GameState, rect: Rect) {
    let (label, color, icon) = adventurer_status(state);
    draw_card(
        rect,
        Color::new(color.r, color.g, color.b, 0.10),
        Color::new(color.r, color.g, color.b, 0.42),
    );
    draw_text_fit(
        icon,
        rect.x + 12.0,
        rect.y + rect.h * 0.62,
        24.0,
        18.0,
        color,
    );
    draw_centered_text(
        label,
        Rect::new(rect.x + 28.0, rect.y, rect.w - 34.0, rect.h),
        13.0,
        color,
    );
}

#[derive(Clone, Copy)]
enum StatIcon {
    Mana,
    Gold,
    Soul,
    Time,
}

fn draw_top_stat(
    rect: Rect,
    label: &str,
    value: &str,
    color: Color,
    icon: StatIcon,
    bar: Option<(f32, f32)>,
) {
    draw_line(
        rect.x,
        rect.y,
        rect.x,
        rect.y + rect.h,
        1.0,
        Color::new(BORDER.r, BORDER.g, BORDER.b, 0.20),
    );
    draw_stat_icon(
        vec2(rect.x + 28.0, rect.y + rect.h * 0.54),
        13.0,
        icon,
        color,
    );
    if !label.is_empty() {
        draw_text_fit(
            label,
            rect.x + 50.0,
            rect.y + 16.0,
            rect.w - 56.0,
            11.0,
            TEXT_MUTED,
        );
    }
    draw_text_fit(
        value,
        rect.x + 50.0,
        if label.is_empty() {
            rect.y + 29.0
        } else {
            rect.y + 38.0
        },
        rect.w - 56.0,
        if label.is_empty() { 17.0 } else { 18.0 },
        color,
    );
    if let Some((current, max)) = bar {
        draw_bar(
            Rect::new(rect.x + 50.0, rect.y + rect.h - 4.0, rect.w - 70.0, 3.0),
            current,
            max,
            color,
            None,
        );
    }
}

fn draw_status_block(rect: Rect, value: &str, color: Color) {
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.20),
        Color::new(BORDER.r, BORDER.g, BORDER.b, 0.34),
    );
    draw_circle(
        rect.x - 18.0,
        rect.y + rect.h * 0.5,
        15.0,
        Color::new(color.r, color.g, color.b, 0.10),
    );
    draw_stat_icon(
        vec2(rect.x - 18.0, rect.y + rect.h * 0.5),
        11.0,
        StatIcon::Soul,
        color,
    );
    draw_text_fit(
        "DUNGEON STATUS",
        rect.x + 12.0,
        rect.y + 18.0,
        rect.w - 24.0,
        10.0,
        TEXT_MUTED,
    );
    draw_text_fit(
        value,
        rect.x + 12.0,
        rect.y + 41.0,
        rect.w - 24.0,
        18.0,
        color,
    );
}

fn draw_speed_segments(rect: Rect, speed: i32) -> bool {
    let clicked = rect.contains(vec2(mouse_position().0, mouse_position().1))
        && is_mouse_button_released(MouseButton::Left);
    draw_card(rect, Color::new(0.018, 0.028, 0.045, 0.94), BORDER_MUTED);
    let labels = ["||", "1x", "2x", "4x"];
    let seg_w = rect.w / labels.len() as f32;
    for (idx, label) in labels.iter().enumerate() {
        let seg = Rect::new(rect.x + idx as f32 * seg_w, rect.y, seg_w, rect.h);
        let active =
            (idx == 1 && speed == 1) || (idx == 2 && speed == 2) || (idx == 3 && speed >= 4);
        if active {
            draw_rectangle(
                seg.x,
                seg.y,
                seg.w,
                seg.h,
                Color::new(MANA.r, MANA.g, MANA.b, 0.12),
            );
        }
        if idx > 0 {
            draw_line(
                seg.x,
                seg.y + 8.0,
                seg.x,
                seg.y + seg.h - 8.0,
                1.0,
                BORDER_MUTED,
            );
        }
        draw_centered_text(label, seg, 15.0, if active { TEXT } else { TEXT_DIM });
    }
    clicked
}

fn draw_reset_button(rect: Rect) -> bool {
    let mouse = mouse_position();
    let hovered = rect.contains(vec2(mouse.0, mouse.1));
    let clicked = hovered && is_mouse_button_released(MouseButton::Left);
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.18),
        Color::new(
            DANGER.r,
            DANGER.g,
            DANGER.b,
            if hovered { 0.34 } else { 0.14 },
        ),
    );
    draw_centered_text("Reset", rect, 14.0, if hovered { DANGER } else { TEXT_DIM });
    clicked
}

fn draw_brand_mark(center: Vec2, radius: f32) {
    draw_poly(
        center.x,
        center.y,
        4,
        radius,
        45.0,
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.22),
    );
    draw_poly_lines(center.x, center.y, 4, radius, 45.0, 2.0, SOUL);
    draw_poly_lines(
        center.x,
        center.y,
        4,
        radius * 0.66,
        45.0,
        2.0,
        Color::new(1.0, 0.85, 1.0, 0.80),
    );
    draw_line(
        center.x,
        center.y - radius,
        center.x,
        center.y + radius,
        2.0,
        SOUL,
    );
    draw_line(
        center.x - radius,
        center.y,
        center.x + radius,
        center.y,
        2.0,
        SOUL,
    );
}

fn draw_stat_icon(center: Vec2, radius: f32, icon: StatIcon, color: Color) {
    match icon {
        StatIcon::Mana => {
            draw_circle(
                center.x,
                center.y + 3.0,
                radius * 0.55,
                Color::new(color.r, color.g, color.b, 0.22),
            );
            draw_triangle(
                vec2(center.x, center.y - radius),
                vec2(center.x - radius * 0.55, center.y + radius * 0.35),
                vec2(center.x + radius * 0.55, center.y + radius * 0.35),
                color,
            );
        }
        StatIcon::Gold => {
            draw_circle(
                center.x,
                center.y,
                radius,
                Color::new(color.r, color.g, color.b, 0.18),
            );
            draw_circle_lines(center.x, center.y, radius, 2.0, color);
            draw_text_fit(
                "G",
                center.x - radius * 0.42,
                center.y + radius * 0.46,
                radius * 0.9,
                radius,
                color,
            );
        }
        StatIcon::Soul => {
            draw_poly(
                center.x,
                center.y,
                4,
                radius,
                45.0,
                Color::new(color.r, color.g, color.b, 0.20),
            );
            draw_poly_lines(center.x, center.y, 4, radius, 45.0, 2.0, color);
        }
        StatIcon::Time => {
            draw_circle_lines(center.x, center.y, radius, 1.6, color);
            draw_line(
                center.x,
                center.y,
                center.x,
                center.y - radius * 0.62,
                1.5,
                color,
            );
            draw_line(
                center.x,
                center.y,
                center.x + radius * 0.55,
                center.y,
                1.5,
                color,
            );
        }
    }
}

fn adventurer_status(state: &GameState) -> (&'static str, Color, &'static str) {
    if state.adventurer_parties.is_empty() {
        return ("SAFE TO REBUILD", EMERALD, "+");
    }

    let core_threat = state.adventurer_parties.iter().any(|party| {
        state.floors.iter().any(|floor| {
            floor.number == party.current_floor
                && floor.rooms.iter().any(|room| {
                    room.position == party.current_room && room.room_type == RoomType::Core
                })
        })
    });
    if core_threat {
        ("CORE UNDER THREAT", DANGER, "!")
    } else if state
        .adventurer_parties
        .iter()
        .any(|party| party.current_room == 0)
    {
        ("ADVENTURERS APPROACHING", WARNING, "!")
    } else {
        ("PARTY INSIDE", WARNING, "!")
    }
}
