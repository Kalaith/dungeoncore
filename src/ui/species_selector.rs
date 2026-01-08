use macroquad::prelude::*;
use macroquad_toolkit::{colors::dark, ui::{ButtonStyle, button_styled, panel}};
use crate::data::monsters::{get_all_species, get_species_unlock_cost};
use crate::game_state::GameState;

pub fn draw_species_selector(
    state: &mut GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Option<String> {
    
    // Draw background panel
    panel(x, y, w, h, Some("Select Starter Species"));

    // We'll reimplement the list without using the heavy widgets module for now
    // to match the style of other panels and fix the import issues
    let mut selected = None;
    let mut current_y = y + 40.0;
    
    let species_list = get_all_species();

    for species in species_list {
        if current_y + 30.0 > y + h {
            break; 
        }

        let cost = crate::data::monsters::get_species_unlock_cost(&species.name).unwrap_or(0);
        let can_afford = cost == 0 || state.gold >= cost;
        let label = if cost == 0 {
            format!("{} (Free)", species.name)
        } else {
            format!("{} ({} gold)", species.name, cost)
        };

        let button_color = if can_afford {
            &ButtonStyle::default_dark()
        } else {
            // Grayed out style for unaffordable species
            &ButtonStyle {
                normal: Color::new(0.3, 0.3, 0.3, 1.0),
                hovered: Color::new(0.3, 0.3, 0.3, 1.0),
                pressed: Color::new(0.3, 0.3, 0.3, 1.0),
                border: Color::new(0.4, 0.4, 0.4, 1.0),
                text_color: Color::new(0.5, 0.5, 0.5, 1.0),
                disabled: Color::new(0.1, 0.1, 0.1, 1.0),
            }
        };

        if can_afford && button_styled(x + 10.0, current_y, w - 20.0, 30.0, &label, button_color) {
             selected = Some(species.name.clone());
        } else if !can_afford {
            // Draw disabled button
            button_styled(x + 10.0, current_y, w - 20.0, 30.0, &label, button_color);
        }
        
        current_y += 35.0;
    }
    
    selected
}
