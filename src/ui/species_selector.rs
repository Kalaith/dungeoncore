use crate::data::monsters::{
    get_all_species, get_species_display_name, get_species_unlock_cost,
    get_starter_monsters_for_species,
};
use crate::game_state::GameState;
use macroquad::prelude::*;

use super::theme::*;

pub fn draw_species_selector(
    state: &mut GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Option<String> {
    let panel = Rect::new(x, y, w, h);
    draw_panel(panel, Some("Choose Your Starter Race"), SOUL);
    draw_text_fit(
        "Pick the species that awakens as your first defenders.",
        x + 16.0,
        y + 48.0,
        w - 32.0,
        12.0,
        TEXT_MUTED,
    );

    let choosing_starter = state.unlocked_species.is_empty();

    // Order the list so selectable races surface first.
    let mut species_list = get_all_species();
    species_list.sort_by(|a, b| {
        let a_key = (!a.starter, a.unlock_cost);
        let b_key = (!b.starter, b.unlock_cost);
        a_key.cmp(&b_key)
    });

    let mut selected = None;
    let card_h = 96.0;
    let gap = 10.0;
    let mut cy = y + 64.0;

    for species in species_list {
        if cy + card_h > y + h - 14.0 {
            break;
        }

        let cost = get_species_unlock_cost(&species.name).unwrap_or(0);
        let selectable = if choosing_starter {
            species.starter
        } else {
            state.gold >= cost
        };
        let display_name = get_species_display_name(&species.name);
        let roster = get_starter_monsters_for_species(&species.name)
            .into_iter()
            .map(|template| template.name)
            .collect::<Vec<_>>();
        let roster_text = if roster.is_empty() {
            "Roster unlocks later".to_string()
        } else {
            roster.join(", ")
        };

        let card = Rect::new(x + 14.0, cy, w - 28.0, card_h);
        let accent = if selectable { SOUL } else { BORDER_MUTED };
        draw_card(
            card,
            Color::new(accent.r, accent.g, accent.b, if selectable { 0.06 } else { 0.02 }),
            Color::new(accent.r, accent.g, accent.b, if selectable { 0.44 } else { 0.16 }),
        );

        draw_text_fit(
            &display_name,
            card.x + 14.0,
            card.y + 26.0,
            card.w - 150.0,
            18.0,
            if selectable { TEXT } else { TEXT_DIM },
        );

        // Status pill in the top-right corner.
        let (pill_text, pill_color) = if species.starter {
            ("STARTER", EMERALD)
        } else if selectable {
            ("READY", TREASURE)
        } else {
            ("LOCKED", TEXT_DIM)
        };
        draw_pill(
            Rect::new(card.x + card.w - 92.0, card.y + 12.0, 78.0, 18.0),
            pill_text,
            pill_color,
        );

        draw_text_fit(
            &species.description,
            card.x + 14.0,
            card.y + 48.0,
            card.w - 28.0,
            12.0,
            TEXT_MUTED,
        );
        draw_text_fit(
            &format!("Units: {}", roster_text),
            card.x + 14.0,
            card.y + 70.0,
            card.w - 130.0,
            11.0,
            if selectable { EMERALD } else { TEXT_DIM },
        );

        let label = if choosing_starter {
            if species.starter {
                "Choose".to_string()
            } else {
                "Locked".to_string()
            }
        } else if cost == 0 {
            "Choose".to_string()
        } else {
            format!("Unlock {}g", cost)
        };
        let tone = if species.starter {
            ButtonTone::Primary
        } else {
            ButtonTone::Arcane
        };
        let btn = Rect::new(card.x + card.w - 116.0, card.y + card.h - 38.0, 102.0, 28.0);
        if draw_command_button(btn, &label, tone, selectable) {
            selected = Some(species.name.clone());
        }

        cy += card_h + gap;
    }

    selected
}
