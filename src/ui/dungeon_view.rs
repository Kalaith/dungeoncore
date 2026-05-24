use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::data::constants::{get_room_cost, MAX_ROOMS_PER_FLOOR};
use crate::game_state::{GameState, Room, RoomType};

use super::theme::*;

const BASE_ROOM_W: f32 = 156.0;
const BASE_ROOM_H: f32 = 122.0;
const BASE_CONNECTOR_W: f32 = 36.0;
const FLOOR_RAIL_W: f32 = 58.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonAction {
    None,
    RoomSelected(i32, usize),
    BuildRoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlacementState {
    Idle,
    Valid,
    Invalid,
}

#[derive(Debug, Clone)]
struct BuildPreview {
    floor: i32,
    position: usize,
    room_type: RoomType,
    cost: i32,
    new_floor: bool,
}

/// Draw the production-style dungeon board and return the selected board action.
pub fn draw_dungeon_board(state: &GameState, rect: Rect) -> DungeonAction {
    let mut action = DungeonAction::None;
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.18),
        Color::new(BORDER.r, BORDER.g, BORDER.b, 0.18),
    );

    draw_text_fit("Dungeon", rect.x + 22.0, rect.y + 30.0, 160.0, 24.0, TEXT);
    draw_text_fit(
        &current_objective(state),
        rect.x + 22.0,
        rect.y + 54.0,
        (rect.w * 0.48).max(240.0),
        13.0,
        TEXT_MUTED,
    );

    if let Some(monster) = &state.selected_monster {
        draw_pill(
            Rect::new(rect.x + rect.w - 360.0, rect.y + 28.0, 184.0, 38.0),
            &format!("PLACING {}", monster.to_uppercase()),
            SOUL,
        );
    }

    let content = Rect::new(rect.x + 10.0, rect.y + 76.0, rect.w - 20.0, rect.h - 86.0);
    draw_board_surface(content);

    let sorted_floors = sorted_floors(state);
    if sorted_floors.is_empty() {
        draw_centered_text("No dungeon mapped", content, 18.0, TEXT_MUTED);
        return action;
    }

    let gap = 14.0;
    let floor_count = sorted_floors.len() as f32;
    let row_h = ((content.h - gap * (sorted_floors.len().saturating_sub(1) as f32)) / floor_count)
        .clamp(150.0, 224.0);
    let used_h = row_h * floor_count + gap * (sorted_floors.len().saturating_sub(1) as f32);
    let mut row_y = content.y + ((content.h - used_h) * 0.5).max(14.0);
    let preview = next_build_preview(state);

    for floor in sorted_floors {
        if row_y + row_h > content.y + content.h - 2.0 {
            break;
        }

        let row_rect = Rect::new(content.x + 18.0, row_y, content.w - 36.0, row_h);
        let selected_floor = state
            .selected_room
            .map(|(floor_num, _)| floor_num == floor.number)
            .unwrap_or(floor.is_deepest);
        let row_border = if selected_floor {
            Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.48)
        } else {
            BORDER_MUTED
        };
        draw_room_route_backplate(row_rect, selected_floor, row_border);

        let rail = Rect::new(
            row_rect.x + 8.0,
            row_rect.y + 8.0,
            FLOOR_RAIL_W,
            row_rect.h - 16.0,
        );
        draw_floor_rail(rail, floor.number, floor.rooms.len(), floor.is_deepest);

        let rooms_area = Rect::new(
            rail.x + rail.w + 10.0,
            row_rect.y + 8.0,
            row_rect.w - rail.w - 28.0,
            row_rect.h - 16.0,
        );
        if let Some(row_action) = draw_floor_rooms(state, floor, rooms_area, preview.as_ref()) {
            action = row_action;
        }

        row_y += row_h + gap;
    }

    action
}

/// Compatibility wrapper for older callers.
pub fn draw_dungeon(state: &GameState, x: f32, y: f32, w: f32, h: f32) -> Option<(i32, usize)> {
    match draw_dungeon_board(state, Rect::new(x, y, w, h)) {
        DungeonAction::RoomSelected(floor, pos) => Some((floor, pos)),
        DungeonAction::None | DungeonAction::BuildRoom => None,
    }
}

fn draw_floor_rooms(
    state: &GameState,
    floor: &crate::game_state::Floor,
    area: Rect,
    preview: Option<&BuildPreview>,
) -> Option<DungeonAction> {
    let mut action = None;
    let rooms = sorted_rooms(&floor.rooms);
    let future_in_floor = preview.filter(|plan| plan.floor == floor.number);
    let has_future = future_in_floor.is_some();
    let future_before_core = future_in_floor.map(|plan| !plan.new_floor).unwrap_or(false);
    let future_after_core = future_in_floor.map(|plan| plan.new_floor).unwrap_or(false);
    let node_count = rooms.len() + usize::from(has_future);
    let total_w =
        node_count as f32 * BASE_ROOM_W + node_count.saturating_sub(1) as f32 * BASE_CONNECTOR_W;
    let scale = (area.w / total_w.max(1.0)).clamp(0.58, 1.0);
    let tile_w = BASE_ROOM_W * scale;
    let tile_h = BASE_ROOM_H * scale;
    let connector_w = BASE_CONNECTOR_W * scale;
    let label_h = 32.0 * scale;
    let tile_y = area.y + ((area.h - tile_h - label_h) * 0.5).max(0.0);
    let mut x = area.x + 6.0;
    let mut drawn_nodes = 0usize;
    let total_nodes = node_count;

    for room in rooms {
        if future_before_core && room.room_type == RoomType::Core {
            if let Some(plan) = future_in_floor {
                let future_rect = Rect::new(x, tile_y, tile_w, tile_h);
                if draw_future_room_tile(state, future_rect, plan) {
                    action = Some(DungeonAction::BuildRoom);
                }
                drawn_nodes += 1;
                x += tile_w;
                if drawn_nodes < total_nodes {
                    draw_connector(
                        Rect::new(x, tile_y + tile_h * 0.36, connector_w, tile_h * 0.28),
                        true,
                    );
                    x += connector_w;
                }
            }
        }

        let rect = Rect::new(x, tile_y, tile_w, tile_h);
        let placement = placement_state(state, room);
        if draw_room_tile(state, room, rect, placement) {
            action = Some(DungeonAction::RoomSelected(
                room.floor_number,
                room.position,
            ));
        }
        drawn_nodes += 1;
        x += tile_w;

        if drawn_nodes < total_nodes {
            draw_connector(
                Rect::new(x, tile_y + tile_h * 0.36, connector_w, tile_h * 0.28),
                false,
            );
            x += connector_w;
        }
    }

    if future_after_core {
        if let Some(plan) = future_in_floor {
            let future_rect = Rect::new(x, tile_y, tile_w, tile_h);
            if draw_future_room_tile(state, future_rect, plan) {
                action = Some(DungeonAction::BuildRoom);
            }
        }
    }

    action
}

fn draw_board_surface(rect: Rect) {
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

fn draw_room_route_backplate(rect: Rect, selected: bool, border: Color) {
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

fn draw_floor_rail(rect: Rect, floor_num: i32, room_count: usize, deepest: bool) {
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

fn draw_room_tile(state: &GameState, room: &Room, rect: Rect, placement: PlacementState) -> bool {
    let hovered = is_hovered_rect(rect);
    let selected = state.selected_room == Some((room.floor_number, room.position));
    let adventurers = adventurer_count_in_room(state, room);
    let alive = room.monsters.iter().filter(|monster| monster.alive).count();
    let dead = room.monsters.len().saturating_sub(alive);
    let fighting = adventurers > 0 && alive > 0;
    let (fill, border, icon_color, title) = room_tone(room);
    let mut draw_rect = rect;
    if hovered {
        draw_rect.y -= 2.0;
    }

    draw_room_chamber_art(draw_rect, room, fill, border, icon_color);
    if fighting {
        let pulse = (get_time() as f32 * 5.5).sin().abs();
        draw_rectangle_lines(
            draw_rect.x - 2.0,
            draw_rect.y - 2.0,
            draw_rect.w + 4.0,
            draw_rect.h + 4.0,
            2.0,
            Color::new(WARNING.r, WARNING.g, WARNING.b, 0.45 + pulse * 0.35),
        );
    }
    if selected {
        draw_rectangle_lines(
            draw_rect.x - 2.0,
            draw_rect.y - 2.0,
            draw_rect.w + 4.0,
            draw_rect.h + 4.0,
            3.0,
            MANA,
        );
    }

    match placement {
        PlacementState::Valid => {
            draw_rectangle_lines(
                draw_rect.x + 2.0,
                draw_rect.y + 2.0,
                draw_rect.w - 4.0,
                draw_rect.h - 4.0,
                2.0,
                EMERALD,
            );
            draw_pill(
                Rect::new(
                    draw_rect.x + draw_rect.w - 44.0,
                    draw_rect.y + draw_rect.h * 0.46,
                    39.0,
                    16.0,
                ),
                "Place",
                EMERALD,
            );
        }
        PlacementState::Invalid => {
            draw_rectangle(
                draw_rect.x,
                draw_rect.y,
                draw_rect.w,
                draw_rect.h,
                Color::new(0.0, 0.0, 0.0, 0.36),
            );
        }
        PlacementState::Idle => {}
    }

    let label_rect = Rect::new(
        draw_rect.x + draw_rect.w * 0.12,
        draw_rect.y + draw_rect.h + 9.0,
        draw_rect.w * 0.76,
        27.0,
    );
    draw_room_label_plate(label_rect, title, room, icon_color);

    if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
        draw_badge(
            Rect::new(draw_rect.x + 6.0, draw_rect.y + 6.0, 38.0, 15.0),
            &format!("M {alive}"),
            if alive > 0 { EMERALD } else { TEXT_DIM },
        );
        if dead > 0 {
            draw_badge(
                Rect::new(draw_rect.x + 48.0, draw_rect.y + 6.0, 34.0, 15.0),
                &format!("D {dead}"),
                DANGER,
            );
        }
    }
    if adventurers > 0 {
        draw_badge(
            Rect::new(
                draw_rect.x + draw_rect.w - 42.0,
                draw_rect.y + 6.0,
                36.0,
                15.0,
            ),
            &format!("A {adventurers}"),
            WARNING,
        );
    }

    was_clicked_rect(rect)
}

fn draw_room_chamber_art(rect: Rect, room: &Room, fill: Color, border: Color, icon_color: Color) {
    draw_card(rect, fill, border);

    let wall = Rect::new(rect.x + 8.0, rect.y + 8.0, rect.w - 16.0, rect.h - 18.0);
    draw_rectangle(
        wall.x,
        wall.y,
        wall.w,
        wall.h,
        Color::new(0.0, 0.0, 0.0, 0.18),
    );

    let brick = Color::new(0.20, 0.22, 0.25, 0.13);
    let mut by = wall.y + 8.0;
    while by < wall.y + wall.h - 8.0 {
        draw_line(wall.x + 6.0, by, wall.x + wall.w - 6.0, by, 1.0, brick);
        by += 16.0;
    }
    let mut bx = wall.x + 12.0;
    while bx < wall.x + wall.w - 10.0 {
        draw_line(
            bx,
            wall.y + 10.0,
            bx,
            wall.y + wall.h - 10.0,
            1.0,
            Color::new(0.18, 0.20, 0.24, 0.08),
        );
        bx += 24.0;
    }

    match room.room_type {
        RoomType::Entrance => draw_entrance_art(wall, icon_color),
        RoomType::Normal | RoomType::Boss => draw_combat_art(wall, icon_color),
        RoomType::Core => draw_core_art(wall, icon_color),
    }
}

fn draw_entrance_art(rect: Rect, color: Color) {
    let arch = Rect::new(
        rect.x + rect.w * 0.30,
        rect.y + rect.h * 0.22,
        rect.w * 0.40,
        rect.h * 0.64,
    );
    draw_rectangle(
        arch.x,
        arch.y + arch.h * 0.36,
        arch.w,
        arch.h * 0.64,
        Color::new(0.0, 0.0, 0.0, 0.38),
    );
    draw_triangle(
        vec2(arch.x, arch.y + arch.h * 0.38),
        vec2(arch.x + arch.w * 0.5, arch.y),
        vec2(arch.x + arch.w, arch.y + arch.h * 0.38),
        Color::new(0.0, 0.0, 0.0, 0.38),
    );
    let line = Color::new(color.r, color.g, color.b, 0.52);
    draw_rectangle_lines(
        arch.x,
        arch.y + arch.h * 0.36,
        arch.w,
        arch.h * 0.64,
        2.0,
        line,
    );
    draw_line(
        arch.x,
        arch.y + arch.h * 0.38,
        arch.x + arch.w * 0.5,
        arch.y,
        2.0,
        line,
    );
    draw_line(
        arch.x + arch.w,
        arch.y + arch.h * 0.38,
        arch.x + arch.w * 0.5,
        arch.y,
        2.0,
        line,
    );
    draw_circle(
        arch.x + arch.w * 0.28,
        arch.y + arch.h * 0.70,
        4.0,
        Color::new(EMERALD.r, EMERALD.g, EMERALD.b, 0.58),
    );
}

fn draw_combat_art(rect: Rect, color: Color) {
    let banner = Rect::new(
        rect.x + rect.w * 0.34,
        rect.y + rect.h * 0.18,
        rect.w * 0.32,
        rect.h * 0.38,
    );
    draw_rectangle(
        banner.x,
        banner.y,
        banner.w,
        banner.h,
        Color::new(0.08, 0.035, 0.025, 0.30),
    );
    draw_triangle(
        vec2(banner.x, banner.y + banner.h),
        vec2(banner.x + banner.w * 0.50, banner.y + banner.h * 0.72),
        vec2(banner.x + banner.w, banner.y + banner.h),
        Color::new(0.08, 0.035, 0.025, 0.30),
    );
    let c = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.52);
    draw_room_icon(&RoomType::Normal, c, rect.h * 0.22, color);
    draw_torch(vec2(rect.x + rect.w * 0.18, rect.y + rect.h * 0.62));
    draw_torch(vec2(rect.x + rect.w * 0.82, rect.y + rect.h * 0.62));
}

fn draw_core_art(rect: Rect, color: Color) {
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.50);
    draw_circle(
        center.x,
        center.y,
        rect.h * 0.36,
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.13),
    );
    draw_room_icon(&RoomType::Core, center, rect.h * 0.28, color);
    draw_line(
        center.x,
        rect.y + 10.0,
        center.x,
        rect.y + rect.h - 8.0,
        1.5,
        Color::new(color.r, color.g, color.b, 0.24),
    );
}

fn draw_torch(pos: Vec2) {
    draw_rectangle(
        pos.x - 2.0,
        pos.y - 8.0,
        4.0,
        22.0,
        Color::new(0.28, 0.18, 0.10, 0.90),
    );
    draw_circle(
        pos.x,
        pos.y - 9.0,
        6.0,
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.30),
    );
    draw_triangle(
        vec2(pos.x, pos.y - 18.0),
        vec2(pos.x - 5.0, pos.y - 5.0),
        vec2(pos.x + 5.0, pos.y - 5.0),
        TREASURE,
    );
}

fn draw_room_label_plate(rect: Rect, title: &str, room: &Room, color: Color) {
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.34),
        Color::new(color.r, color.g, color.b, 0.28),
    );
    draw_text_fit(
        title,
        rect.x + 28.0,
        rect.y + 18.0,
        rect.w - 34.0,
        12.0,
        color,
    );
    draw_room_icon(
        &room.room_type,
        vec2(rect.x + 15.0, rect.y + rect.h * 0.50),
        7.0,
        color,
    );
}

fn draw_future_room_tile(state: &GameState, rect: Rect, plan: &BuildPreview) -> bool {
    let can_afford = state.mana >= plan.cost;
    let can_build = can_afford && state.adventurer_parties.is_empty();
    let hovered = is_hovered_rect(rect);
    let fill = if can_build {
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.10)
    } else {
        Color::new(0.045, 0.052, 0.075, 0.72)
    };
    let border = if can_build { TREASURE } else { BORDER_MUTED };
    let mut draw_rect = rect;
    if hovered && can_build {
        draw_rect.y -= 2.0;
    }

    draw_card(draw_rect, fill, border);
    draw_dashed_border(draw_rect, border);
    draw_centered_text(
        "+",
        Rect::new(draw_rect.x, draw_rect.y + 8.0, draw_rect.w, 26.0),
        30.0,
        border,
    );
    let label = if plan.new_floor {
        format!("Floor {}", plan.floor)
    } else if plan.room_type == RoomType::Boss {
        "Boss".to_string()
    } else {
        "Room".to_string()
    };
    draw_centered_text(
        "Build Room",
        Rect::new(
            draw_rect.x,
            draw_rect.y + draw_rect.h * 0.57,
            draw_rect.w,
            22.0,
        ),
        13.0,
        border,
    );
    let label_rect = Rect::new(
        draw_rect.x + draw_rect.w * 0.18,
        draw_rect.y + draw_rect.h + 9.0,
        draw_rect.w * 0.64,
        27.0,
    );
    draw_card(
        label_rect,
        Color::new(0.0, 0.0, 0.0, 0.34),
        Color::new(border.r, border.g, border.b, 0.24),
    );
    draw_centered_text(&label, label_rect, 11.0, TEXT_MUTED);
    draw_centered_text(
        &format!("{}M", plan.cost),
        Rect::new(label_rect.x, label_rect.y + 24.0, label_rect.w, 14.0),
        11.0,
        if can_afford { MANA } else { DANGER },
    );

    was_clicked_rect(rect)
}

fn draw_connector(rect: Rect, ghost: bool) {
    let alpha = if ghost { 0.32 } else { 0.70 };
    let fill = Color::new(0.110, 0.145, 0.195, alpha);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(0.35, 0.40, 0.48, alpha),
    );
    draw_line(
        rect.x + 4.0,
        rect.y + rect.h * 0.5,
        rect.x + rect.w - 4.0,
        rect.y + rect.h * 0.5,
        2.0,
        Color::new(
            TREASURE.r,
            TREASURE.g,
            TREASURE.b,
            if ghost { 0.22 } else { 0.36 },
        ),
    );
}

fn draw_room_icon(room_type: &RoomType, center: Vec2, size: f32, color: Color) {
    match room_type {
        RoomType::Entrance => {
            draw_rectangle_lines(
                center.x - size * 0.42,
                center.y - size * 0.48,
                size * 0.84,
                size * 0.96,
                3.0,
                color,
            );
            draw_line(
                center.x - size * 0.18,
                center.y + size * 0.40,
                center.x + size * 0.18,
                center.y + size * 0.40,
                3.0,
                color,
            );
            draw_circle(center.x + size * 0.20, center.y, size * 0.06, color);
        }
        RoomType::Normal => {
            draw_line(
                center.x - size * 0.50,
                center.y - size * 0.48,
                center.x + size * 0.48,
                center.y + size * 0.50,
                4.0,
                color,
            );
            draw_line(
                center.x + size * 0.50,
                center.y - size * 0.48,
                center.x - size * 0.48,
                center.y + size * 0.50,
                4.0,
                color,
            );
            draw_line(
                center.x - size * 0.64,
                center.y + size * 0.32,
                center.x - size * 0.30,
                center.y + size * 0.66,
                3.0,
                color,
            );
            draw_line(
                center.x + size * 0.64,
                center.y + size * 0.32,
                center.x + size * 0.30,
                center.y + size * 0.66,
                3.0,
                color,
            );
        }
        RoomType::Boss => {
            draw_circle_lines(center.x, center.y - size * 0.05, size * 0.44, 3.0, color);
            draw_circle(
                center.x - size * 0.17,
                center.y - size * 0.10,
                size * 0.07,
                color,
            );
            draw_circle(
                center.x + size * 0.17,
                center.y - size * 0.10,
                size * 0.07,
                color,
            );
            draw_line(
                center.x - size * 0.18,
                center.y + size * 0.20,
                center.x + size * 0.18,
                center.y + size * 0.20,
                3.0,
                color,
            );
            draw_line(
                center.x - size * 0.32,
                center.y - size * 0.48,
                center.x - size * 0.48,
                center.y - size * 0.70,
                3.0,
                color,
            );
            draw_line(
                center.x + size * 0.32,
                center.y - size * 0.48,
                center.x + size * 0.48,
                center.y - size * 0.70,
                3.0,
                color,
            );
        }
        RoomType::Core => {
            draw_triangle(
                vec2(center.x, center.y - size * 0.62),
                vec2(center.x - size * 0.48, center.y),
                vec2(center.x, center.y + size * 0.62),
                Color::new(color.r, color.g, color.b, 0.28),
            );
            draw_triangle(
                vec2(center.x, center.y - size * 0.62),
                vec2(center.x + size * 0.48, center.y),
                vec2(center.x, center.y + size * 0.62),
                Color::new(color.r, color.g, color.b, 0.28),
            );
            draw_line(
                center.x,
                center.y - size * 0.62,
                center.x - size * 0.48,
                center.y,
                3.0,
                color,
            );
            draw_line(
                center.x - size * 0.48,
                center.y,
                center.x,
                center.y + size * 0.62,
                3.0,
                color,
            );
            draw_line(
                center.x,
                center.y + size * 0.62,
                center.x + size * 0.48,
                center.y,
                3.0,
                color,
            );
            draw_line(
                center.x + size * 0.48,
                center.y,
                center.x,
                center.y - size * 0.62,
                3.0,
                color,
            );
            draw_line(
                center.x,
                center.y - size * 0.62,
                center.x,
                center.y + size * 0.62,
                2.0,
                color,
            );
        }
    }
}

fn draw_badge(rect: Rect, text: &str, color: Color) {
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.36),
        Color::new(color.r, color.g, color.b, 0.52),
    );
    draw_centered_text(text, rect, 10.0, color);
}

fn draw_dashed_border(rect: Rect, color: Color) {
    let dash = 7.0;
    let gap = 5.0;
    let mut x = rect.x + 3.0;
    while x < rect.x + rect.w - 3.0 {
        draw_line(
            x,
            rect.y + 3.0,
            (x + dash).min(rect.x + rect.w - 3.0),
            rect.y + 3.0,
            1.5,
            color,
        );
        draw_line(
            x,
            rect.y + rect.h - 3.0,
            (x + dash).min(rect.x + rect.w - 3.0),
            rect.y + rect.h - 3.0,
            1.5,
            color,
        );
        x += dash + gap;
    }
    let mut y = rect.y + 3.0;
    while y < rect.y + rect.h - 3.0 {
        draw_line(
            rect.x + 3.0,
            y,
            rect.x + 3.0,
            (y + dash).min(rect.y + rect.h - 3.0),
            1.5,
            color,
        );
        draw_line(
            rect.x + rect.w - 3.0,
            y,
            rect.x + rect.w - 3.0,
            (y + dash).min(rect.y + rect.h - 3.0),
            1.5,
            color,
        );
        y += dash + gap;
    }
}

fn room_tone(room: &Room) -> (Color, Color, Color, &'static str) {
    match room.room_type {
        RoomType::Entrance => (
            Color::new(0.05, 0.34, 0.22, 0.94),
            Color::new(EMERALD.r, EMERALD.g, EMERALD.b, 0.78),
            Color::new(0.70, 1.00, 0.82, 1.0),
            "Entrance",
        ),
        RoomType::Normal => (
            Color::new(0.07, 0.19, 0.29, 0.94),
            Color::new(MANA.r, MANA.g, MANA.b, 0.58),
            Color::new(0.72, 0.91, 1.0, 1.0),
            "Room",
        ),
        RoomType::Boss => (
            Color::new(0.36, 0.12, 0.07, 0.94),
            Color::new(WARNING.r, WARNING.g, WARNING.b, 0.78),
            Color::new(1.0, 0.80, 0.58, 1.0),
            "Boss",
        ),
        RoomType::Core => (
            Color::new(0.26, 0.10, 0.42, 0.96),
            Color::new(SOUL.r, SOUL.g, SOUL.b, 0.82),
            Color::new(0.93, 0.78, 1.0, 1.0),
            "Core",
        ),
    }
}

fn placement_state(state: &GameState, room: &Room) -> PlacementState {
    if state.selected_monster.is_none() {
        return PlacementState::Idle;
    }

    if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
        PlacementState::Valid
    } else {
        PlacementState::Invalid
    }
}

fn adventurer_count_in_room(state: &GameState, room: &Room) -> usize {
    state
        .adventurer_parties
        .iter()
        .filter(|party| {
            party.current_floor == room.floor_number
                && party.current_room == room.position
                && !party.retreating
        })
        .map(|party| party.members.iter().filter(|member| member.alive).count())
        .sum()
}

fn current_objective(state: &GameState) -> String {
    if let Some(monster) = &state.selected_monster {
        return format!("Place {monster} in a combat room.");
    }

    if !state.adventurer_parties.is_empty() {
        return "Adventurers are inside. Hold the route.".to_string();
    }

    if state.mana < 20 {
        return "Gather mana before expanding.".to_string();
    }

    "Build deeper or strengthen a selected room.".to_string()
}

fn all_rooms(state: &GameState) -> Vec<&Room> {
    state
        .floors
        .iter()
        .flat_map(|floor| floor.rooms.iter())
        .collect()
}

fn sorted_floors(state: &GameState) -> Vec<&crate::game_state::Floor> {
    let mut floors: Vec<_> = state.floors.iter().collect();
    floors.sort_by_key(|floor| floor.number);
    floors
}

fn sorted_rooms(rooms: &[Room]) -> Vec<&Room> {
    let mut sorted: Vec<_> = rooms.iter().collect();
    sorted.sort_by_key(|room| room.position);
    sorted
}

fn next_build_preview(state: &GameState) -> Option<BuildPreview> {
    let deepest = state.deepest_floor()?;
    let non_core_count = deepest
        .rooms
        .iter()
        .filter(|room| room.room_type != RoomType::Core)
        .count();
    let total_rooms = state.total_room_count();

    if non_core_count >= MAX_ROOMS_PER_FLOOR + 1 {
        let floor = state.total_floors + 1;
        return Some(BuildPreview {
            floor: deepest.number,
            position: 1,
            room_type: RoomType::Normal,
            cost: get_room_cost(total_rooms, false),
            new_floor: true,
        })
        .map(|mut preview| {
            preview.floor = floor;
            preview
        });
    }

    let position = non_core_count;
    let is_boss = position == MAX_ROOMS_PER_FLOOR;
    Some(BuildPreview {
        floor: deepest.number,
        position,
        room_type: if is_boss {
            RoomType::Boss
        } else {
            RoomType::Normal
        },
        cost: get_room_cost(total_rooms, is_boss),
        new_floor: false,
    })
}
