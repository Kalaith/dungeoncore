use macroquad::prelude::*;
use macroquad_toolkit::{colors::dark, ui::*};

use crate::data::monsters::get_monster_templates;
use crate::game_state::GameState;

/// Draw the monster selector panel
/// Returns the name of a newly selected monster, if any
pub fn draw_monster_selector(
    state: &GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Option<String> {
    let mut selected = None;

    panel(x, y, w, h, Some("Monsters"));

    let templates = get_monster_templates();
    let mut btn_y = y + 35.0;
    let btn_h = 32.0;

    for template in &templates {
        let unlocked = state.unlocked_species.contains(&template.species);

        if btn_y + btn_h > y + h - 10.0 {
            break; // Don't overflow panel
        }

        if unlocked {
            let is_selected = state.selected_monster.as_ref() == Some(&template.name);
            let label = format!("{} {} ({}💎)", template.emoji, template.name, template.base_cost);

            let style = if is_selected {
                ButtonStyle {
                    normal: dark::ACCENT,
                    ..ButtonStyle::default_dark()
                }
            } else {
                ButtonStyle::default_dark()
            };

            if button_styled(x + 5.0, btn_y, w - 10.0, btn_h, &label, &style) {
                selected = Some(template.name.clone());
            }
        } else {
            // Locked monster
            draw_rectangle(x + 5.0, btn_y, w - 10.0, btn_h, dark::PANEL);
            draw_rectangle_lines(x + 5.0, btn_y, w - 10.0, btn_h, 1.0, Color::from_hex(0x444444));
            draw_text(
                &format!("🔒 {} ({})", template.name, template.species),
                x + 10.0,
                btn_y + 20.0,
                13.0,
                dark::TEXT_DIM,
            );
        }

        btn_y += btn_h + 5.0;
    }

    // Instructions/Info at bottom
    if let Some(ref monster_name) = state.selected_monster {
        let description = templates
            .iter()
            .find(|t| t.name == *monster_name)
            .map(|t| t.description.clone())
            .unwrap_or_default();

        let info_y = y + h - 55.0;
        
        // Separator
        draw_rectangle(x, info_y, w, 1.0, dark::TEXT_DIM);
        
        // Description
        draw_text(
            &description,
            x + 10.0,
            info_y + 20.0,
            12.0,
            dark::TEXT,
        );

        // Traits
        if let Some(t) = templates.iter().find(|t| t.name == *monster_name) {
             if !t.traits.is_empty() {
                 let traits_str = format!("Traits: {}", t.traits.join(", "));
                 draw_text(
                    &traits_str,
                    x + 10.0,
                    info_y + 35.0,
                    11.0,
                    dark::TEXT_DIM,
                 );
            }
        }

        // Instruction
        draw_text(
            "Click a room to place",
            x + 10.0,
            info_y + 55.0,
            12.0,
            dark::ACCENT,
        );
    }

    selected
}
