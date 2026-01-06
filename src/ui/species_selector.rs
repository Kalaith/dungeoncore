use macroquad::prelude::*;
use macroquad_toolkit::ui::{ButtonStyle, button_styled, panel};
use crate::data::monsters::get_all_species;
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

        let label = format!("{}", species.name);
        if button_styled(x + 10.0, current_y, w - 20.0, 30.0, &label, &ButtonStyle::default_dark()) {
             selected = Some(species.name.clone());
        }
        
        current_y += 35.0;
    }
    
    selected
}
