use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::game_state::{DungeonStatus, GameState, LogEntry, RoomUpgradeType};

use super::theme::*;

/// Which piece of UI a tutorial step points the player toward.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TutorialAnchor {
    Drawer,
    Board,
    Hud,
}

struct StepDef {
    title: &'static str,
    instruction: &'static str,
    anchor: TutorialAnchor,
}

const STEPS: [StepDef; 5] = [
    StepDef {
        title: "Build a room",
        instruction: "Open the BUILD tab on the left (or click the glowing room on the map) to add a combat room.",
        anchor: TutorialAnchor::Drawer,
    },
    StepDef {
        title: "Place a defender",
        instruction: "Open the MONSTERS tab, pick a unit, then click your new combat room to summon it.",
        anchor: TutorialAnchor::Drawer,
    },
    StepDef {
        title: "Set a trap",
        instruction: "Click a room to select it, then apply a Trap upgrade from the panel on the right.",
        anchor: TutorialAnchor::Board,
    },
    StepDef {
        title: "Open the dungeon",
        instruction: "Press 'Open Dungeon' in the top bar to invite adventurers inside.",
        anchor: TutorialAnchor::Hud,
    },
    StepDef {
        title: "Survive a raid",
        instruction: "Adventurers are coming. Hold your core and let your defenders do their work!",
        anchor: TutorialAnchor::Board,
    },
];

/// True while the tutorial is running and still has steps to show.
pub fn is_active(state: &GameState) -> bool {
    state.tutorial_active && (state.tutorial_step as usize) < STEPS.len()
}

/// The UI area the current step wants to highlight.
pub fn current_anchor(state: &GameState) -> Option<TutorialAnchor> {
    STEPS
        .get(state.tutorial_step.max(0) as usize)
        .map(|step| step.anchor)
}

/// End the tutorial early at the player's request.
pub fn skip(state: &mut GameState) {
    if state.tutorial_active {
        state.tutorial_active = false;
        state.add_log(LogEntry::system(
            "Tutorial dismissed. You can shape the dungeon however you like.",
        ));
    }
}

fn step_complete(state: &GameState, step_idx: usize) -> bool {
    match step_idx {
        // Build a room: any non-entrance/core room exists.
        0 => state.total_room_count() >= 1,
        // Place a defender: any room holds a monster.
        1 => state
            .floors
            .iter()
            .flat_map(|floor| &floor.rooms)
            .any(|room| !room.monsters.is_empty()),
        // Set a trap: any room carries a Trap upgrade.
        2 => state
            .floors
            .iter()
            .flat_map(|floor| &floor.rooms)
            .any(|room| room.has_upgrade_type(RoomUpgradeType::Trap)),
        // Open the dungeon: it is open, has visitors, or a raid already ran.
        3 => {
            state.status == DungeonStatus::Open
                || !state.adventurer_parties.is_empty()
                || state.raids_completed >= 1
        }
        // Survive a raid: at least one party has come and gone.
        4 => state.raids_completed >= 1,
        _ => true,
    }
}

/// Advance the tutorial if the current step's goal has been met. Call once per
/// frame; advances at most one step so each completion is announced.
pub fn advance(state: &mut GameState) {
    if !state.tutorial_active {
        return;
    }
    let idx = state.tutorial_step.max(0) as usize;
    if idx >= STEPS.len() {
        state.tutorial_active = false;
        return;
    }
    if step_complete(state, idx) {
        state.add_log(LogEntry::building(format!("Tutorial: {} \u{2713}", STEPS[idx].title)));
        state.tutorial_step += 1;
        if state.tutorial_step as usize >= STEPS.len() {
            state.tutorial_active = false;
            state.add_log(LogEntry::system(
                "Tutorial complete! Grow your dungeon and keep the threat in check.",
            ));
        }
    }
}

/// Draw the tutorial callout and target highlight. Returns true if the player
/// clicked Skip this frame.
pub fn draw(state: &GameState, board_rect: Rect, anchor_rect: Rect) -> bool {
    let idx = state.tutorial_step.max(0) as usize;
    let Some(step) = STEPS.get(idx) else {
        return false;
    };

    // Pulsing highlight around the step's target.
    let pulse = (get_time() as f32 * 4.0).sin().abs();
    let glow = Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.35 + pulse * 0.45);
    draw_rectangle_lines(
        anchor_rect.x - 3.0,
        anchor_rect.y - 3.0,
        anchor_rect.w + 6.0,
        anchor_rect.h + 6.0,
        3.0,
        glow,
    );

    // Callout card pinned to the top of the board area.
    let card_w = (board_rect.w - 40.0).clamp(300.0, 560.0);
    let card = Rect::new(board_rect.x + 14.0, board_rect.y + 12.0, card_w, 88.0);
    draw_card(
        card,
        Color::new(0.05, 0.04, 0.10, 0.94),
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.55),
    );

    draw_text_fit(
        &format!("TUTORIAL  {}/{}", idx + 1, STEPS.len()),
        card.x + 14.0,
        card.y + 22.0,
        card.w - 120.0,
        11.0,
        TREASURE,
    );
    draw_text_fit(
        step.title,
        card.x + 14.0,
        card.y + 44.0,
        card.w - 110.0,
        16.0,
        TEXT,
    );
    draw_text_fit(
        step.instruction,
        card.x + 14.0,
        card.y + 68.0,
        card.w - 28.0,
        12.0,
        TEXT_MUTED,
    );

    // Skip button, top-right of the card.
    let skip = Rect::new(card.x + card.w - 74.0, card.y + 12.0, 62.0, 24.0);
    let hovered = is_hovered_rect(skip);
    draw_card(
        skip,
        Color::new(0.0, 0.0, 0.0, 0.22),
        Color::new(
            TEXT_MUTED.r,
            TEXT_MUTED.g,
            TEXT_MUTED.b,
            if hovered { 0.6 } else { 0.3 },
        ),
    );
    draw_centered_text("Skip", skip, 12.0, if hovered { TEXT } else { TEXT_MUTED });

    was_clicked_rect(skip)
}
