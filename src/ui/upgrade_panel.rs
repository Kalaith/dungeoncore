use macroquad::prelude::*;
use macroquad_toolkit::input::was_clicked_rect;

use crate::data::monsters::get_monster_template;
use crate::data::upgrades::{get_all_upgrades, UpgradeTemplate};
use crate::game_state::{GameState, Room, RoomType};

use super::theme::*;

#[derive(Debug, Clone)]
pub enum UpgradeAction {
    None,
    Apply(String),
    Remove,
    Close,
}

pub fn draw_upgrade_panel(state: &GameState, x: f32, y: f32, w: f32, h: f32) -> UpgradeAction {
    let mut action = UpgradeAction::None;
    let rect = Rect::new(x, y, w, h);
    draw_panel(rect, None, SOUL);

    let inner = Rect::new(rect.x + 14.0, rect.y + 14.0, rect.w - 28.0, rect.h - 28.0);
    draw_text_fit(
        "INSPECTOR",
        inner.x,
        inner.y + 21.0,
        inner.w - 40.0,
        18.0,
        TEXT,
    );
    if draw_close_button(Rect::new(inner.x + inner.w - 30.0, inner.y, 30.0, 26.0)) {
        return UpgradeAction::Close;
    }

    let mut y_cursor = inner.y + 46.0;

    if let Some(monster_name) = &state.selected_monster {
        y_cursor = draw_selected_monster(state, monster_name, inner, y_cursor);
    }

    if let Some(room) = selected_room(state) {
        y_cursor = draw_selected_room(state, room, inner, y_cursor + 10.0);
        if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
            draw_upgrade_choices(state, room, inner, y_cursor + 12.0, &mut action);
        } else {
            draw_hint(
                Rect::new(inner.x, y_cursor + 12.0, inner.w, 54.0),
                match room.room_type {
                    RoomType::Entrance => "Adventurers enter here. Keep the defense deeper in.",
                    RoomType::Core => {
                        "The core must survive. Select combat rooms to build defenses."
                    }
                    RoomType::Normal | RoomType::Boss => "",
                },
                TEXT_MUTED,
            );
        }
    } else if state.selected_monster.is_none() {
        draw_hint(
            Rect::new(inner.x, y_cursor, inner.w, 72.0),
            "Select a room to inspect it, or choose a monster from the drawer.",
            TEXT_MUTED,
        );
    }

    action
}

fn selected_room(state: &GameState) -> Option<&Room> {
    let (floor_num, room_pos) = state.selected_room?;
    state
        .floors
        .iter()
        .find(|floor| floor.number == floor_num)
        .and_then(|floor| floor.rooms.iter().find(|room| room.position == room_pos))
}

fn draw_selected_monster(state: &GameState, monster_name: &str, bounds: Rect, y: f32) -> f32 {
    let rect = Rect::new(bounds.x, y, bounds.w, 98.0);
    draw_card(
        rect,
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.085),
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.25),
    );
    draw_text_fit(
        monster_name,
        rect.x + 12.0,
        rect.y + 25.0,
        rect.w - 24.0,
        18.0,
        TEXT,
    );

    if let Some(template) = get_monster_template(monster_name) {
        draw_text_fit(
            &format!("Tier {} {} defender", template.tier, template.species),
            rect.x + 12.0,
            rect.y + 50.0,
            rect.w - 24.0,
            12.0,
            TEXT_MUTED,
        );
        draw_text_fit(
            &format!("Cost starts at {} mana", template.base_cost),
            rect.x + 12.0,
            rect.y + 75.0,
            rect.w - 24.0,
            13.0,
            if state.mana >= template.base_cost {
                MANA
            } else {
                DANGER
            },
        );
    } else {
        draw_text_fit(
            "Monster data unavailable",
            rect.x + 12.0,
            rect.y + 52.0,
            rect.w - 24.0,
            12.0,
            TEXT_MUTED,
        );
    }

    y + rect.h
}

fn draw_selected_room(state: &GameState, room: &Room, bounds: Rect, y: f32) -> f32 {
    let rect = Rect::new(bounds.x, y, bounds.w, 248.0);
    let tone = room_color(room);
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.18),
        Color::new(tone.r, tone.g, tone.b, 0.26),
    );

    draw_room_badge(
        Rect::new(rect.x + 12.0, rect.y + 16.0, 34.0, 34.0),
        &room.room_type,
        tone,
    );
    draw_text_fit(
        room_name(room),
        rect.x + 56.0,
        rect.y + 27.0,
        rect.w - 68.0,
        18.0,
        TEXT,
    );
    draw_text_fit(
        &format!("Tier {}", room.floor_number),
        rect.x + 56.0,
        rect.y + 48.0,
        rect.w - 68.0,
        13.0,
        TEXT_MUTED,
    );
    draw_wrapped(
        room_role(room),
        Rect::new(rect.x + 12.0, rect.y + 74.0, rect.w - 24.0, 64.0),
        12.0,
        TEXT_MUTED,
    );

    let alive = room.monsters.iter().filter(|monster| monster.alive).count();
    let adventurers = adventurers_in_room(state, room);
    draw_section_rule(rect.x + 12.0, rect.y + 152.0, rect.w - 24.0, "ROOM STATS");
    draw_stat_line(
        rect.x + 12.0,
        rect.y + 182.0,
        rect.w - 24.0,
        "Defenders",
        &format!("{alive}/{}", room.monsters.len()),
        if alive > 0 { EMERALD } else { TEXT_DIM },
    );
    draw_stat_line(
        rect.x + 12.0,
        rect.y + 207.0,
        rect.w - 24.0,
        "Capacity",
        "3",
        TEXT,
    );
    draw_stat_line(
        rect.x + 12.0,
        rect.y + 232.0,
        rect.w - 24.0,
        "Threat",
        &adventurers.to_string(),
        if adventurers > 0 { WARNING } else { EMERALD },
    );

    y + rect.h
}

fn draw_upgrade_choices(
    state: &GameState,
    room: &Room,
    bounds: Rect,
    y: f32,
    action: &mut UpgradeAction,
) {
    let max_h = bounds.y + bounds.h - y;
    if max_h < 56.0 {
        return;
    }

    draw_section_rule(bounds.x, y + 18.0, bounds.w, "ACTIONS");
    let mut row_y = y + 36.0;

    if let Some(upgrade) = &room.upgrade {
        draw_hint(
            Rect::new(bounds.x, row_y, bounds.w, 58.0),
            &format!("{}: {}", upgrade.name, upgrade.effect),
            TREASURE,
        );
        row_y += 70.0;
        if draw_command_button(
            Rect::new(bounds.x, row_y, bounds.w, 34.0),
            "Remove Upgrade",
            ButtonTone::Danger,
            state.adventurer_parties.is_empty(),
        ) {
            *action = UpgradeAction::Remove;
        }
        return;
    }

    if draw_command_button(
        Rect::new(bounds.x, row_y, bounds.w, 44.0),
        "Manage Defenders",
        ButtonTone::Primary,
        true,
    ) {
        // Existing monster placement flow is handled by the left drawer.
    }

    row_y += 58.0;
    let upgrades = get_all_upgrades();
    if let Some(upgrade) = upgrades.first() {
        if draw_upgrade_button(state, upgrade, Rect::new(bounds.x, row_y, bounds.w, 44.0)) {
            *action = UpgradeAction::Apply(upgrade.name.clone());
        }
    }
}

fn draw_upgrade_button(state: &GameState, upgrade: &UpgradeTemplate, rect: Rect) -> bool {
    let can_afford = state.gold >= upgrade.gold_cost && state.souls >= upgrade.souls_cost;
    let enabled = can_afford && state.adventurer_parties.is_empty();
    let label = format!(
        "Upgrade Room        {}g {}s",
        upgrade.gold_cost, upgrade.souls_cost
    );
    draw_command_button(rect, &label, ButtonTone::Ghost, enabled)
}

fn draw_upgrade_row(state: &GameState, upgrade: &UpgradeTemplate, rect: Rect) -> bool {
    let can_afford = state.gold >= upgrade.gold_cost && state.souls >= upgrade.souls_cost;
    let enabled = can_afford && state.adventurer_parties.is_empty();
    let color = upgrade_color(&upgrade.upgrade_type);
    let hovered = enabled && rect.contains(vec2(mouse_position().0, mouse_position().1));
    draw_card(
        rect,
        if hovered {
            Color::new(color.r, color.g, color.b, 0.13)
        } else {
            Color::new(color.r, color.g, color.b, 0.075)
        },
        Color::new(color.r, color.g, color.b, if enabled { 0.30 } else { 0.12 }),
    );
    draw_text_fit(
        &upgrade.name,
        rect.x + 10.0,
        rect.y + 17.0,
        rect.w - 82.0,
        13.0,
        if enabled { TEXT } else { TEXT_DIM },
    );
    draw_text_fit(
        &format!("{}g {}s", upgrade.gold_cost, upgrade.souls_cost),
        rect.x + 10.0,
        rect.y + 32.0,
        rect.w - 82.0,
        10.0,
        if can_afford { TREASURE } else { TEXT_DIM },
    );
    draw_text_fit_right(
        if enabled { "Apply" } else { "Locked" },
        rect.x + rect.w - 10.0,
        rect.y + 24.0,
        64.0,
        11.0,
        if enabled { EMERALD } else { TEXT_DIM },
    );

    enabled && was_clicked_rect(rect)
}

fn draw_hint(rect: Rect, text: &str, color: Color) {
    draw_card(
        rect,
        Color::new(color.r, color.g, color.b, 0.055),
        Color::new(color.r, color.g, color.b, 0.18),
    );
    let lines = macroquad_toolkit::ui::wrap_text(text, rect.w - 20.0, 11.0);
    let mut y = rect.y + 18.0;
    for line in lines.iter().take(3) {
        draw_text_fit(line, rect.x + 10.0, y, rect.w - 20.0, 11.0, color);
        y += 14.0;
    }
}

fn draw_wrapped(text: &str, rect: Rect, size: f32, color: Color) {
    let mut y = rect.y + 14.0;
    for line in macroquad_toolkit::ui::wrap_text(text, rect.w, size)
        .iter()
        .take(4)
    {
        draw_text_fit(line, rect.x, y, rect.w, size, color);
        y += size + 5.0;
    }
}

fn draw_section_rule(x: f32, y: f32, w: f32, label: &str) {
    draw_text_fit(label, x, y, w * 0.36, 11.0, TEXT_DIM);
    draw_line(x + w * 0.36, y - 4.0, x + w, y - 4.0, 1.0, BORDER_MUTED);
}

fn draw_room_badge(rect: Rect, room_type: &RoomType, color: Color) {
    draw_card(
        rect,
        Color::new(color.r, color.g, color.b, 0.14),
        Color::new(color.r, color.g, color.b, 0.42),
    );
    draw_centered_text(room_icon_letter(room_type), rect, 17.0, color);
}

fn room_icon_letter(room_type: &RoomType) -> &'static str {
    match room_type {
        RoomType::Entrance => "E",
        RoomType::Normal => "X",
        RoomType::Boss => "B",
        RoomType::Core => "C",
    }
}

fn draw_close_button(rect: Rect) -> bool {
    let hovered = rect.contains(vec2(mouse_position().0, mouse_position().1));
    draw_card(
        rect,
        if hovered {
            Color::new(SOUL.r, SOUL.g, SOUL.b, 0.12)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.05)
        },
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.18),
    );
    draw_centered_text("X", rect, 13.0, if hovered { SOUL } else { TEXT_DIM });
    was_clicked_rect(rect)
}

fn draw_stat_line(x: f32, baseline_y: f32, w: f32, label: &str, value: &str, color: Color) {
    draw_text_fit(label, x, baseline_y, w * 0.42, 12.0, TEXT_MUTED);
    draw_text_fit_right(value, x + w, baseline_y, w * 0.56, 13.0, color);
}

fn adventurers_in_room(state: &GameState, room: &Room) -> usize {
    state
        .adventurer_parties
        .iter()
        .filter(|party| {
            party.current_floor == room.floor_number && party.current_room == room.position
        })
        .map(|party| party.members.iter().filter(|member| member.alive).count())
        .sum()
}

fn room_name(room: &Room) -> &'static str {
    match room.room_type {
        RoomType::Entrance => "Entrance",
        RoomType::Normal => "Combat Room",
        RoomType::Boss => "Boss Chamber",
        RoomType::Core => "Core",
    }
}

fn room_role(room: &Room) -> &'static str {
    match room.room_type {
        RoomType::Entrance => "Adventurers cross this threshold first.",
        RoomType::Normal => "Primary defense room.",
        RoomType::Boss => "Heavy defense and high risk.",
        RoomType::Core => "The heart of the dungeon.",
    }
}

fn room_color(room: &Room) -> Color {
    match room.room_type {
        RoomType::Entrance => EMERALD,
        RoomType::Normal => MANA,
        RoomType::Boss => WARNING,
        RoomType::Core => SOUL,
    }
}

fn upgrade_color(upgrade_type: &str) -> Color {
    match upgrade_type {
        "trap" => DANGER,
        "treasure" => TREASURE,
        "reinforcement" => EMERALD,
        "evolution" => SOUL,
        _ => TEXT_MUTED,
    }
}
