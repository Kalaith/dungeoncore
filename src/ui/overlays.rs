use macroquad::prelude::*;

use crate::data::elements::get_all_elements;
use crate::data::monsters::{get_monster_templates, get_species_display_name};
use crate::game_state::GameState;

use super::theme::*;
use super::upgrade_panel::draw_close_button;

/// Full-screen bestiary/codex: the element wheel and every monster the
/// dungeon has discovered. Returns true when the player closes it.
pub fn draw_codex(state: &GameState, sw: f32, sh: f32, scroll: &mut f32) -> bool {
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.8));
    let w = (sw - 120.0).min(940.0);
    let h = (sh - 80.0).min(620.0);
    let x = (sw - w) / 2.0;
    let y = (sh - h) / 2.0;
    let panel = Rect::new(x, y, w, h);
    draw_panel(panel, Some("Codex"), ARCANE);

    let mut close = false;
    if draw_close_button(Rect::new(x + w - 40.0, y + 14.0, 30.0, 26.0)) {
        close = true;
    }
    if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::Escape) {
        close = true;
    }

    // Left column: element effectiveness wheel.
    let col_gap = 24.0;
    let left_w = (w * 0.36).clamp(240.0, 340.0);
    let left = Rect::new(x + 20.0, y + 52.0, left_w, h - 72.0);
    draw_text_fit("ELEMENTS", left.x, left.y + 14.0, left.w, 13.0, SOUL);
    draw_text_fit(
        "Attacker is strong vs:",
        left.x,
        left.y + 34.0,
        left.w,
        10.0,
        TEXT_DIM,
    );
    let mut ey = left.y + 52.0;
    for element in get_all_elements() {
        if ey + 24.0 > left.y + left.h {
            break;
        }
        let targets = if element.strong_against.is_empty() {
            "— (neutral)".to_string()
        } else {
            element.strong_against.join(", ")
        };
        draw_text_fit(
            &format!("{}  {}", element.emoji, element.id),
            left.x,
            ey + 12.0,
            left.w * 0.42,
            11.0,
            TEXT,
        );
        draw_text_fit(
            &format!("› {}", targets),
            left.x + left.w * 0.42,
            ey + 12.0,
            left.w * 0.58,
            10.0,
            EMERALD,
        );
        ey += 22.0;
    }

    // Right column: discovered monsters (unlocked only), scrollable.
    let right = Rect::new(
        left.x + left_w + col_gap,
        y + 52.0,
        w - left_w - col_gap - 40.0,
        h - 72.0,
    );
    let discovered: Vec<_> = get_monster_templates()
        .into_iter()
        .filter(|m| state.unlocked_monsters.contains(&m.name))
        .collect();
    draw_text_fit(
        &format!("BESTIARY  ({} discovered)", discovered.len()),
        right.x,
        right.y + 14.0,
        right.w,
        13.0,
        SOUL,
    );

    let list = Rect::new(right.x, right.y + 30.0, right.w, right.h - 30.0);
    let row_h = 46.0;
    let visible = (list.h / row_h) as usize;
    let max_scroll = discovered.len().saturating_sub(visible) as f32;
    if list.contains(vec2(mouse_position().0, mouse_position().1)) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            *scroll -= wheel_y.signum();
        }
    }
    *scroll = scroll.clamp(0.0, max_scroll);
    let first = *scroll as usize;

    if discovered.is_empty() {
        draw_text_fit(
            "No monsters discovered yet. Unlock a species to begin.",
            list.x,
            list.y + 30.0,
            list.w,
            12.0,
            TEXT_DIM,
        );
    }
    for (slot, m) in discovered.iter().skip(first).take(visible).enumerate() {
        let row = Rect::new(list.x, list.y + slot as f32 * row_h, list.w, row_h - 6.0);
        draw_card(row, CARD, Color::new(BORDER.r, BORDER.g, BORDER.b, 0.2));
        draw_text_fit(
            &format!("{}  {}", m.emoji, m.name),
            row.x + 10.0,
            row.y + 16.0,
            row.w * 0.5,
            12.0,
            TEXT,
        );
        draw_text_fit(
            &format!(
                "T{} {} · {}",
                m.tier,
                get_species_display_name(&m.species),
                m.element.as_deref().unwrap_or("Neutral")
            ),
            row.x + 10.0,
            row.y + 33.0,
            row.w * 0.5,
            9.0,
            TEXT_MUTED,
        );
        draw_text_fit_right(
            &format!("HP {}  ATK {}  DEF {}", m.hp, m.attack, m.defense),
            row.x + row.w - 10.0,
            row.y + 18.0,
            row.w * 0.46,
            10.0,
            TEXT_MUTED,
        );
        if !m.traits.is_empty() {
            let names: Vec<String> = m
                .traits
                .iter()
                .filter_map(|t| crate::data::traits::get_trait(t).map(|d| d.name))
                .collect();
            draw_text_fit_right(
                &names.join(", "),
                row.x + row.w - 10.0,
                row.y + 34.0,
                row.w * 0.46,
                9.0,
                ARCANE,
            );
        }
    }

    draw_text_fit(
        "Press C or Esc to close.",
        x + 20.0,
        y + h - 14.0,
        w - 40.0,
        10.0,
        TEXT_DIM,
    );

    close
}

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
