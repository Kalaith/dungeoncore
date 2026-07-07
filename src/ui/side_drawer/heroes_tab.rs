//! HEROES tab: the scrollable ledger of every adventurer who has delved.

use macroquad::prelude::*;

use crate::game_state::{GameState, HeroStatus};
use crate::ui::theme::*;

use super::draw_section_title;

pub(super) fn draw_heroes_tab(state: &GameState, rect: Rect, scroll: &mut f32) {
    draw_section_title(rect, "HEROES", "Everyone who has delved.");

    if state.known_adventurers.is_empty() {
        draw_text_fit(
            "No adventurers have entered yet. Open the dungeon to draw them in.",
            rect.x,
            rect.y + 80.0,
            rect.w,
            12.0,
            TEXT_DIM,
        );
        return;
    }

    // Summary line: living / inside / fallen.
    let inside = state
        .known_adventurers
        .iter()
        .filter(|h| h.status == HeroStatus::Inside)
        .count();
    let alive = state
        .known_adventurers
        .iter()
        .filter(|h| h.status == HeroStatus::Alive)
        .count();
    let dead = state
        .known_adventurers
        .iter()
        .filter(|h| h.status == HeroStatus::Dead)
        .count();
    draw_text_fit(
        &format!("Inside {}  Free {}  Fallen {}", inside, alive, dead),
        rect.x,
        rect.y + 66.0,
        rect.w,
        11.0,
        TEXT_MUTED,
    );

    // Sort: active raiders first, then veterans by delves, graves last.
    let mut order: Vec<usize> = (0..state.known_adventurers.len()).collect();
    let rank = |s: HeroStatus| match s {
        HeroStatus::Inside => 0,
        HeroStatus::Alive => 1,
        HeroStatus::Dead => 2,
    };
    order.sort_by(|&a, &b| {
        let ha = &state.known_adventurers[a];
        let hb = &state.known_adventurers[b];
        rank(ha.status)
            .cmp(&rank(hb.status))
            .then(hb.delves.cmp(&ha.delves))
    });

    let list_top = rect.y + 86.0;
    let list_h = (rect.y + rect.h - list_top).max(0.0);
    let row_h = 44.0;
    let visible = (list_h / row_h) as usize;
    let max_scroll = order.len().saturating_sub(visible) as f32;
    if rect.contains(vec2(mouse_position().0, mouse_position().1)) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            *scroll -= wheel_y.signum();
        }
    }
    *scroll = scroll.clamp(0.0, max_scroll);
    let first = *scroll as usize;

    for (slot, &record_idx) in order.iter().skip(first).take(visible).enumerate() {
        let hero = &state.known_adventurers[record_idx];
        let row = Rect::new(rect.x, list_top + slot as f32 * row_h, rect.w, row_h - 6.0);
        let (tag, tag_color) = match hero.status {
            HeroStatus::Inside => ("IN", DANGER),
            HeroStatus::Alive => ("FREE", EMERALD),
            HeroStatus::Dead => ("DEAD", TEXT_DIM),
        };
        draw_card(
            row,
            Color::new(tag_color.r, tag_color.g, tag_color.b, 0.06),
            Color::new(tag_color.r, tag_color.g, tag_color.b, 0.22),
        );
        draw_text_fit(
            &format!("{}  L{}", hero.name, hero.level),
            row.x + 10.0,
            row.y + 15.0,
            row.w - 60.0,
            12.0,
            if hero.status == HeroStatus::Dead {
                TEXT_DIM
            } else {
                TEXT
            },
        );
        draw_pill(
            Rect::new(row.x + row.w - 50.0, row.y + 7.0, 42.0, 15.0),
            tag,
            tag_color,
        );
        let detail = match hero.status {
            HeroStatus::Dead => format!(
                "{} {} · died F{} D{}",
                hero.race, hero.class_name, hero.death_floor, hero.death_day
            ),
            _ => format!(
                "{} {} · {} delves · {} kills",
                hero.race, hero.class_name, hero.delves, hero.kills
            ),
        };
        draw_text_fit(
            &detail,
            row.x + 10.0,
            row.y + 31.0,
            row.w - 20.0,
            9.0,
            TEXT_MUTED,
        );
    }

    if order.len() > visible {
        draw_text_fit_right(
            &format!(
                "{}-{} of {}",
                first + 1,
                (first + visible).min(order.len()),
                order.len()
            ),
            rect.x + rect.w,
            rect.y + rect.h - 2.0,
            rect.w,
            9.0,
            TEXT_DIM,
        );
    }
}
