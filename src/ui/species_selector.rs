use crate::data::monsters::{
    get_all_species, get_species_display_name, get_starter_monsters_for_species,
};
use crate::game_state::GameState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{button_styled, panel, truncate_text_to_width, ButtonStyle};

pub fn draw_species_selector(
    state: &mut GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Option<String> {
    // Draw background panel
    panel(x, y, w, h, Some("Select Starter Species"));

    let mut selected = None;
    let mut current_y = y + 46.0;

    let species_list = get_all_species();
    let choosing_starter = state.unlocked_species.is_empty();

    for species in species_list {
        let row_h = 92.0;
        if current_y + row_h > y + h - 12.0 {
            break;
        }

        let cost = crate::data::monsters::get_species_unlock_cost(&species.name).unwrap_or(0);
        let can_afford = if choosing_starter {
            species.starter
        } else {
            state.gold >= cost
        };
        let display_name = get_species_display_name(&species.name);
        let starters = get_starter_monsters_for_species(&species.name)
            .into_iter()
            .map(|template| match template.element {
                Some(element) => format!("{} ({})", template.name, element),
                None => template.name,
            })
            .collect::<Vec<_>>();
        let roster = if starters.is_empty() {
            "Future unlock roster".to_string()
        } else {
            starters.join(", ")
        };
        let label = if cost == 0 {
            format!("Choose {}", display_name)
        } else if choosing_starter && species.starter {
            format!("Choose {}", display_name)
        } else if choosing_starter {
            format!("Future unlock: {}", display_name)
        } else {
            format!("Unlock {} ({} gold)", display_name, cost)
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

        draw_rectangle(
            x + 10.0,
            current_y,
            w - 20.0,
            row_h - 8.0,
            Color::new(0.04, 0.05, 0.07, 0.82),
        );
        let description = truncate_text_to_width(&species.description, w - 40.0, 13.0);
        let roster_text =
            truncate_text_to_width(&format!("Starter units: {}", roster), w - 40.0, 12.0);

        draw_text(
            &description,
            x + 20.0,
            current_y + 47.0,
            13.0,
            Color::new(0.68, 0.70, 0.76, 1.0),
        );
        draw_text(
            &roster_text,
            x + 20.0,
            current_y + 67.0,
            12.0,
            if can_afford {
                Color::new(0.44, 0.86, 0.64, 1.0)
            } else {
                Color::new(0.45, 0.45, 0.48, 1.0)
            },
        );

        if can_afford
            && button_styled(
                x + 16.0,
                current_y + 8.0,
                w - 32.0,
                28.0,
                &label,
                button_color,
            )
        {
            selected = Some(species.name.clone());
        } else if !can_afford {
            button_styled(
                x + 16.0,
                current_y + 8.0,
                w - 32.0,
                28.0,
                &label,
                button_color,
            );
        }

        current_y += row_h;
    }

    selected
}
