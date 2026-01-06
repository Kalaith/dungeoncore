use macroquad::prelude::*;
use macroquad_toolkit::{colors::dark, input::*, ui::panel};

use crate::game_state::{GameState, RoomType};

const ROOM_SIZE: f32 = 90.0;
const ROOM_GAP: f32 = 30.0;

/// Draw the dungeon view and return clicked room if any
pub fn draw_dungeon(
    state: &GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Option<(i32, usize)> {
    let mut clicked_room = None;

    panel(x, y, w, h, Some("Dungeon Layout"));

    // Header
    draw_text(
        &format!("Total Floors: {}", state.floors.len()),
        x + w - 130.0,
        y + 20.0,
        14.0,
        dark::TEXT_DIM,
    );

    let content_y = y + 35.0;
    let mut floor_y = content_y + 5.0;

    for floor in state.floors.iter() {
        // Floor header
        let floor_label = if floor.is_deepest {
            format!("Floor {} (Deepest)", floor.number)
        } else {
            format!("Floor {}", floor.number)
        };
        draw_text(&floor_label, x + 15.0, floor_y + 15.0, 16.0, dark::TEXT_BRIGHT);

        floor_y += 25.0;
        let mut room_x = x + 20.0;

        // Sort rooms by position
        let mut sorted_rooms: Vec<_> = floor.rooms.iter().collect();
        sorted_rooms.sort_by_key(|r| r.position);

        for (idx, room) in sorted_rooms.iter().enumerate() {
            // Room color based on type
            let color = match room.room_type {
                RoomType::Entrance => Color::from_hex(0x4CAF50),
                RoomType::Normal => Color::from_hex(0x607D8B),
                RoomType::Boss => Color::from_hex(0xE53935),
                RoomType::Core => Color::from_hex(0x9C27B0),
            };

            // Draw room background
            draw_rectangle(room_x, floor_y, ROOM_SIZE, ROOM_SIZE, color);

            // Selection highlight
            if state.selected_room == Some((floor.number, room.position)) {
                draw_rectangle_lines(room_x, floor_y, ROOM_SIZE, ROOM_SIZE, 3.0, YELLOW);
            }

            // Hover highlight
            if is_hovered(room_x, floor_y, ROOM_SIZE, ROOM_SIZE) {
                draw_rectangle(
                    room_x,
                    floor_y,
                    ROOM_SIZE,
                    ROOM_SIZE,
                    Color::new(1.0, 1.0, 1.0, 0.2),
                );
            }

            // Room icon
            let icon = match room.room_type {
                RoomType::Entrance => "🚪",
                RoomType::Normal => "⚔️",
                RoomType::Boss => "👑",
                RoomType::Core => "💎",
            };
            draw_text(icon, room_x + 30.0, floor_y + 35.0, 28.0, WHITE);

            // Room label
            let label = match room.room_type {
                RoomType::Entrance => "Entry".to_string(),
                RoomType::Core => "Core".to_string(),
                _ => format!("Room {}", room.position),
            };
            draw_text(&label, room_x + 10.0, floor_y + 55.0, 12.0, WHITE);

            // Monster count
            let alive = room.monsters.iter().filter(|m| m.alive).count();
            let dead = room.monsters.iter().filter(|m| !m.alive).count();
            if alive > 0 || dead > 0 {
                let monster_text = format!("🐲{} 💀{}", alive, dead);
                draw_text(&monster_text, room_x + 5.0, floor_y + ROOM_SIZE - 8.0, 11.0, WHITE);
            }

            // Check for adventurer party in this room
            let has_party = state
                .adventurer_parties
                .iter()
                .any(|p| p.current_floor == floor.number && p.current_room == room.position);
            if has_party {
                draw_text("🧑‍🎤", room_x + ROOM_SIZE - 25.0, floor_y + 20.0, 18.0, WHITE);
            }

            // Click detection
            if was_clicked(room_x, floor_y, ROOM_SIZE, ROOM_SIZE) {
                clicked_room = Some((floor.number, room.position));
            }

            // Arrow connector between rooms
            room_x += ROOM_SIZE;
            if idx < sorted_rooms.len() - 1 {
                draw_text("→", room_x + 5.0, floor_y + 45.0, 20.0, dark::TEXT);
                room_x += ROOM_GAP;
            }
        }

        floor_y += ROOM_SIZE + 25.0;
    }

    clicked_room
}
