//! Board surface, citadel skyline, floor rails, and route backplates.

use macroquad::prelude::*;

use crate::ui::theme::*;

pub(super) fn draw_board_surface(rect: Rect) {
    draw_card(
        rect,
        Color::new(0.004, 0.007, 0.013, 0.82),
        Color::new(BORDER_MUTED.r, BORDER_MUTED.g, BORDER_MUTED.b, 0.20),
    );
    draw_citadel_backdrop(rect);
    let mut x = rect.x;
    while x < rect.x + rect.w {
        draw_line(
            x,
            rect.y,
            x,
            rect.y + rect.h,
            1.0,
            Color::new(0.23, 0.29, 0.39, 0.022),
        );
        x += 64.0;
    }
    let mut y = rect.y;
    while y < rect.y + rect.h {
        draw_line(
            rect.x,
            y,
            rect.x + rect.w,
            y,
            1.0,
            Color::new(0.23, 0.29, 0.39, 0.018),
        );
        y += 64.0;
    }
}

fn draw_citadel_backdrop(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h * 0.46,
        Color::new(0.020, 0.040, 0.070, 0.20),
    );

    let horizon = rect.y + rect.h * 0.42;
    let mountain = Color::new(0.055, 0.070, 0.095, 0.44);
    draw_triangle(
        vec2(rect.x + rect.w * 0.10, horizon),
        vec2(rect.x + rect.w * 0.28, rect.y + rect.h * 0.12),
        vec2(rect.x + rect.w * 0.43, horizon),
        mountain,
    );
    draw_triangle(
        vec2(rect.x + rect.w * 0.35, horizon),
        vec2(rect.x + rect.w * 0.55, rect.y + rect.h * 0.08),
        vec2(rect.x + rect.w * 0.74, horizon),
        mountain,
    );
    draw_triangle(
        vec2(rect.x + rect.w * 0.58, horizon),
        vec2(rect.x + rect.w * 0.82, rect.y + rect.h * 0.16),
        vec2(rect.x + rect.w * 0.98, horizon),
        mountain,
    );

    for i in 0..5 {
        let tx = rect.x + rect.w * (0.48 + i as f32 * 0.07);
        let th = rect.h * (0.10 + i as f32 * 0.018);
        draw_rectangle(
            tx,
            horizon - th,
            rect.w * 0.018,
            th,
            Color::new(0.020, 0.024, 0.034, 0.55),
        );
        draw_triangle(
            vec2(tx - rect.w * 0.006, horizon - th),
            vec2(tx + rect.w * 0.009, horizon - th - rect.h * 0.045),
            vec2(tx + rect.w * 0.024, horizon - th),
            Color::new(0.020, 0.024, 0.034, 0.55),
        );
    }

    draw_circle(
        rect.x + rect.w * 0.88,
        rect.y + rect.h * 0.33,
        rect.w * 0.055,
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.055),
    );
}

pub(super) fn draw_room_route_backplate(rect: Rect, selected: bool, border: Color) {
    draw_card(
        rect,
        if selected {
            Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.020)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.12)
        },
        Color::new(
            border.r,
            border.g,
            border.b,
            if selected { 0.26 } else { 0.12 },
        ),
    );
    let floor_y = rect.y + rect.h * 0.64;
    draw_rectangle(
        rect.x,
        floor_y,
        rect.w,
        rect.h * 0.18,
        Color::new(0.030, 0.030, 0.035, 0.50),
    );
    draw_line(
        rect.x,
        floor_y,
        rect.x + rect.w,
        floor_y,
        2.0,
        Color::new(
            TREASURE.r,
            TREASURE.g,
            TREASURE.b,
            if selected { 0.22 } else { 0.10 },
        ),
    );
}

pub(super) fn draw_floor_rail(rect: Rect, floor_num: i32, room_count: usize, deepest: bool) {
    draw_card(
        rect,
        Color::new(CARD.r, CARD.g, CARD.b, 0.66),
        if deepest {
            Color::new(ARCANE.r, ARCANE.g, ARCANE.b, 0.34)
        } else {
            BORDER_MUTED
        },
    );
    draw_text_fit(
        "Floor",
        rect.x + 8.0,
        rect.y + 18.0,
        rect.w - 16.0,
        10.0,
        TEXT_DIM,
    );
    draw_centered_text(
        &floor_num.to_string(),
        Rect::new(rect.x, rect.y + 19.0, rect.w, 26.0),
        24.0,
        TEXT,
    );
    draw_centered_text(
        &format!("{room_count}R"),
        Rect::new(rect.x, rect.y + rect.h - 26.0, rect.w, 18.0),
        10.0,
        if deepest { SOUL } else { TEXT_MUTED },
    );
    if deepest {
        draw_pill(
            Rect::new(rect.x + 6.0, rect.y + rect.h - 42.0, rect.w - 12.0, 14.0),
            "Deep",
            SOUL,
        );
    }
}
