use macroquad::prelude::*;
use macroquad_toolkit::{colors::dark, input::*, ui::*};

use crate::data::upgrades::{get_all_upgrades, UpgradeTemplate};
use crate::game_state::{GameState, RoomUpgradeType};
use crate::simulation::upgrades::upgrade_type_icon;

/// Upgrade action returned from the UI
#[derive(Debug, Clone)]
pub enum UpgradeAction {
    None,
    Apply(String),   // Upgrade name to apply
    Remove,          // Remove current upgrade
    Close,           // Close the panel
}

/// Draw the upgrade selector panel for a selected room
pub fn draw_upgrade_panel(
    state: &GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> UpgradeAction {
    let mut action = UpgradeAction::None;

    // Only show if a room is selected
    let (floor_num, room_pos) = match state.selected_room {
        Some(r) => r,
        None => return action,
    };

    // Find the room
    let room = state
        .floors
        .iter()
        .find(|f| f.number == floor_num)
        .and_then(|f| f.rooms.iter().find(|r| r.position == room_pos));

    let room = match room {
        Some(r) => r,
        None => return action,
    };

    // Don't show for entrance/core
    if room.room_type == crate::game_state::RoomType::Entrance
        || room.room_type == crate::game_state::RoomType::Core
    {
        return action;
    }

    panel(x, y, w, h, Some(&format!("Room {} Upgrades", room_pos)));

    let inner_x = x + 10.0;
    let inner_w = w - 20.0;
    let mut current_y = y + 35.0;

    // Show current upgrade if any
    if let Some(ref upgrade) = room.upgrade {
        draw_text(
            &format!("Current: {} {}", upgrade_type_icon(&upgrade.upgrade_type), upgrade.name),
            inner_x,
            current_y,
            14.0,
            dark::ACCENT,
        );
        current_y += 18.0;
        draw_text(&upgrade.effect, inner_x, current_y, 12.0, dark::TEXT_DIM);
        current_y += 20.0;

        // Remove button
        if button(inner_x, current_y, inner_w, 25.0, "Remove Upgrade") {
            action = UpgradeAction::Remove;
        }
        current_y += 35.0;
    } else {
        draw_text("No upgrade installed", inner_x, current_y, 13.0, dark::TEXT_DIM);
        current_y += 25.0;

        // List available upgrades
        let upgrades = get_all_upgrades();
        let btn_h = 28.0;

        for upgrade in upgrades.iter() {
            if current_y + btn_h > y + h - 10.0 {
                break;
            }

            let can_afford = state.gold >= upgrade.gold_cost && state.souls >= upgrade.souls_cost;
            let icon = match upgrade.upgrade_type.as_str() {
                "trap" => "⚡",
                "treasure" => "💰",
                "reinforcement" => "🛡️",
                "evolution" => "🧬",
                _ => "?",
            };

            let label = format!(
                "{} {} ({}g{})",
                icon,
                upgrade.name,
                upgrade.gold_cost,
                if upgrade.souls_cost > 0 {
                    format!(" {}s", upgrade.souls_cost)
                } else {
                    String::new()
                }
            );

            if can_afford {
                if button(inner_x, current_y, inner_w, btn_h, &label) {
                    action = UpgradeAction::Apply(upgrade.name.clone());
                }
            } else {
                draw_rectangle(inner_x, current_y, inner_w, btn_h, dark::PANEL);
                draw_rectangle_lines(inner_x, current_y, inner_w, btn_h, 1.0, Color::from_hex(0x444444));
                draw_text(&label, inner_x + 5.0, current_y + 18.0, 12.0, dark::TEXT_DIM);
            }

            current_y += btn_h + 3.0;
        }
    }

    // Close button at bottom
    if button(inner_x, y + h - 35.0, inner_w, 25.0, "Close") {
        action = UpgradeAction::Close;
    }

    action
}
