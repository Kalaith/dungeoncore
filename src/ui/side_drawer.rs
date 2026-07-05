use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::data::constants::{get_room_cost, MAX_ROOMS_PER_FLOOR};
use crate::data::evolutions::get_evolution_for_monster;
use crate::data::monsters::{
    get_all_species, get_monster_templates, get_species_display_name, MonsterTemplate,
};
use crate::data::traits::get_trait;
use crate::game_state::{GameState, RoomType};

use super::theme::*;

pub const DRAWER_OPEN_WIDTH: f32 = 274.0;
pub const DRAWER_COLLAPSED_WIDTH: f32 = 64.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerTab {
    Build,
    Monsters,
    Traps,
    Evolution,
}

/// Sections within the Traps tab, so each upgrade family gets its own list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeSection {
    Traps,
    Loot,
    Buffs,
    Shrines,
}

impl UpgradeSection {
    fn label(self) -> &'static str {
        match self {
            UpgradeSection::Traps => "Traps",
            UpgradeSection::Loot => "Loot",
            UpgradeSection::Buffs => "Buffs",
            UpgradeSection::Shrines => "Shrines",
        }
    }

    /// Which upgrade-template types this section lists.
    fn matches(self, upgrade_type: &str) -> bool {
        match self {
            UpgradeSection::Traps => upgrade_type == "trap",
            UpgradeSection::Loot => upgrade_type == "treasure",
            UpgradeSection::Buffs => upgrade_type == "reinforcement" || upgrade_type == "evolution",
            UpgradeSection::Shrines => upgrade_type == "attunement",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawerAction {
    None,
    BuildRoom,
    SelectMonster(String),
    SelectUpgrade(String),
    ProcessEvolutions,
    UnlockSpecies(String),
    ResetGame,
}

pub fn draw_side_drawer(
    state: &GameState,
    rect: Rect,
    active_tab: &mut DrawerTab,
    open: &mut bool,
    upgrade_section: &mut UpgradeSection,
) -> DrawerAction {
    let mut action = DrawerAction::None;
    draw_panel(rect, None, ARCANE);

    let rail_w = DRAWER_COLLAPSED_WIDTH.min(rect.w);
    if draw_tab_rail(state, rect, rail_w, active_tab, open) {
        action = DrawerAction::ResetGame;
    }

    if !*open || rect.w <= rail_w + 24.0 {
        return action;
    }

    let content = Rect::new(
        rect.x + rail_w + 12.0,
        rect.y + 16.0,
        rect.w - rail_w - 24.0,
        rect.h - 32.0,
    );

    match active_tab {
        DrawerTab::Build => {
            if draw_build_tab(state, content) {
                action = DrawerAction::BuildRoom;
            }
        }
        DrawerTab::Monsters => {
            if let Some(monster) = draw_monster_tab(state, content) {
                action = DrawerAction::SelectMonster(monster);
            }
        }
        DrawerTab::Traps => {
            if let Some(upgrade) = draw_traps_tab(state, content, upgrade_section) {
                action = DrawerAction::SelectUpgrade(upgrade);
            }
        }
        DrawerTab::Evolution => {
            action = draw_evolution_tab(state, content);
        }
    }

    action
}

fn draw_tab_rail(
    state: &GameState,
    rect: Rect,
    rail_w: f32,
    active_tab: &mut DrawerTab,
    open: &mut bool,
) -> bool {
    let toggle = Rect::new(rect.x + 9.0, rect.y + 12.0, rail_w - 18.0, 34.0);
    if draw_small_tab(toggle, if *open { "<" } else { ">" }, ARCANE, true) {
        *open = !*open;
    }

    let mut y = rect.y + 58.0;
    for (tab, icon, label, color) in [
        (DrawerTab::Monsters, "M", "MONSTERS", SOUL),
        (DrawerTab::Traps, "T", "TRAPS", DANGER),
        (DrawerTab::Build, "B", "BUILD", TREASURE),
        (DrawerTab::Evolution, "E", "EVOLUTION", MANA),
    ] {
        let tab_rect = Rect::new(rect.x + 7.0, y, rail_w - 14.0, 66.0);
        if draw_rail_tab(tab_rect, icon, label, color, *active_tab == tab) {
            *active_tab = tab;
            *open = true;
        }
        y += 74.0;
    }

    let chip_rect = Rect::new(rect.x + 9.0, rect.y + rect.h - 88.0, rail_w - 18.0, 34.0);
    let color = if state.adventurer_parties.is_empty() {
        EMERALD
    } else {
        WARNING
    };
    draw_card(
        chip_rect,
        Color::new(color.r, color.g, color.b, 0.09),
        Color::new(color.r, color.g, color.b, 0.26),
    );
    draw_centered_text(
        if state.adventurer_parties.is_empty() {
            "Safe"
        } else {
            "Alert"
        },
        chip_rect,
        10.0,
        color,
    );

    // Reset control lives quietly at the bottom of the rail.
    let reset_rect = Rect::new(rect.x + 9.0, rect.y + rect.h - 46.0, rail_w - 18.0, 34.0);
    draw_small_tab(reset_rect, "RESET", DANGER, false)
}

fn draw_build_tab(state: &GameState, rect: Rect) -> bool {
    draw_section_title(rect, "BUILD", "Shape the dungeon path.");

    let (label, detail, cost) = next_build_summary(state);
    let can_build = state.adventurer_parties.is_empty() && state.mana >= cost;

    let card = Rect::new(rect.x, rect.y + 70.0, rect.w, 126.0);
    draw_card(
        card,
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.075),
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.24),
    );
    draw_text_fit(
        &label,
        card.x + 12.0,
        card.y + 27.0,
        card.w - 24.0,
        17.0,
        TEXT,
    );
    draw_text_fit(
        &detail,
        card.x + 12.0,
        card.y + 56.0,
        card.w - 24.0,
        12.0,
        TEXT_MUTED,
    );
    draw_text_fit(
        &format!("{cost} mana"),
        card.x + 12.0,
        card.y + 86.0,
        card.w - 24.0,
        14.0,
        if state.mana >= cost { MANA } else { DANGER },
    );
    draw_text_fit(
        if can_build {
            "Click the glowing room or build here."
        } else if state.adventurer_parties.is_empty() {
            "Gather more mana."
        } else {
            "Wait until the dungeon is safe."
        },
        card.x + 12.0,
        card.y + 112.0,
        card.w - 24.0,
        12.0,
        if can_build { EMERALD } else { TEXT_DIM },
    );

    draw_command_button(
        Rect::new(rect.x, card.y + card.h + 16.0, rect.w, 42.0),
        "Build",
        ButtonTone::Arcane,
        can_build,
    )
}

fn draw_monster_tab(state: &GameState, rect: Rect) -> Option<String> {
    let mut selected = None;
    draw_section_title(rect, "MONSTERS", "Choose a defender.");

    // Only summonable (unlocked) monsters appear; new forms join the list
    // as species are bought and evolutions are earned.
    let templates: Vec<MonsterTemplate> = get_monster_templates()
        .into_iter()
        .filter(|t| {
            state.unlocked_species.contains(&t.species)
                && state.unlocked_monsters.contains(&t.name)
        })
        .collect();
    let available_h = (rect.h - 138.0).max(0.0);
    let row_h = (available_h / templates.len().max(1) as f32).clamp(46.0, 72.0);
    let row_gap = 6.0;
    let mut y = rect.y + 72.0;
    for template in &templates {
        if y + row_h > rect.y + rect.h - 68.0 {
            break;
        }
        let row = Rect::new(rect.x, y, rect.w, row_h);
        if draw_monster_option(state, template, row) {
            selected = Some(template.name.clone());
        }
        y += row_h + row_gap;
    }

    if let Some(monster) = &state.selected_monster {
        let hint = Rect::new(rect.x, rect.y + rect.h - 62.0, rect.w, 52.0);
        draw_card(
            hint,
            Color::new(SOUL.r, SOUL.g, SOUL.b, 0.10),
            Color::new(SOUL.r, SOUL.g, SOUL.b, 0.30),
        );
        draw_text_fit(
            monster,
            hint.x + 10.0,
            hint.y + 20.0,
            hint.w - 20.0,
            13.0,
            TEXT,
        );
        draw_text_fit(
            "Click rooms to place; reclick entry to stop.",
            hint.x + 10.0,
            hint.y + 39.0,
            hint.w - 20.0,
            11.0,
            SOUL,
        );
    }

    selected
}

fn draw_traps_tab(
    state: &GameState,
    rect: Rect,
    section: &mut UpgradeSection,
) -> Option<String> {
    let mut selected = None;
    draw_section_title(rect, "TRAPS & LOOT", "Outfit a room.");

    // Section chips: each upgrade family gets its own list.
    let chip_h = 26.0;
    let chip_gap = 6.0;
    let chip_w = (rect.w - chip_gap) / 2.0;
    for (idx, option) in [
        UpgradeSection::Traps,
        UpgradeSection::Loot,
        UpgradeSection::Buffs,
        UpgradeSection::Shrines,
    ]
    .into_iter()
    .enumerate()
    {
        let col = (idx % 2) as f32;
        let row = (idx / 2) as f32;
        let chip = Rect::new(
            rect.x + col * (chip_w + chip_gap),
            rect.y + 56.0 + row * (chip_h + chip_gap),
            chip_w,
            chip_h,
        );
        let tone = if *section == option {
            ButtonTone::Primary
        } else {
            ButtonTone::Ghost
        };
        if draw_command_button(chip, option.label(), tone, true) {
            *section = option;
        }
    }

    let upgrades: Vec<_> = crate::data::upgrades::get_all_upgrades()
        .into_iter()
        .filter(|t| section.matches(&t.upgrade_type))
        .collect();
    let list_top = rect.y + 56.0 + (chip_h + chip_gap) * 2.0 + 8.0;
    let available_h = (rect.y + rect.h - 68.0 - list_top).max(0.0);
    let row_h = (available_h / upgrades.len().max(1) as f32 - 6.0).clamp(46.0, 64.0);
    let row_gap = 6.0;
    let mut y = list_top;
    for template in &upgrades {
        if y + row_h > rect.y + rect.h - 68.0 {
            break;
        }
        let row = Rect::new(rect.x, y, rect.w, row_h);
        if draw_upgrade_option(state, template, row) {
            selected = Some(template.name.clone());
        }
        y += row_h + row_gap;
    }

    if let Some(upgrade) = &state.selected_upgrade {
        let hint = Rect::new(rect.x, rect.y + rect.h - 62.0, rect.w, 52.0);
        draw_card(
            hint,
            Color::new(DANGER.r, DANGER.g, DANGER.b, 0.10),
            Color::new(DANGER.r, DANGER.g, DANGER.b, 0.30),
        );
        draw_text_fit(
            upgrade,
            hint.x + 10.0,
            hint.y + 20.0,
            hint.w - 20.0,
            13.0,
            TEXT,
        );
        draw_text_fit(
            "Click rooms to place; reclick entry to stop.",
            hint.x + 10.0,
            hint.y + 39.0,
            hint.w - 20.0,
            11.0,
            DANGER,
        );
    }

    selected
}

fn draw_upgrade_option(
    state: &GameState,
    template: &crate::data::upgrades::UpgradeTemplate,
    rect: Rect,
) -> bool {
    let can_afford = state.mana >= template.mana_cost && state.souls >= template.souls_cost;
    let selected = state.selected_upgrade.as_ref() == Some(&template.name);
    let hovered = can_afford && is_hovered_rect(rect);
    let fill = if selected {
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.13)
    } else if hovered {
        Color::new(DANGER.r, DANGER.g, DANGER.b, 0.10)
    } else {
        CARD
    };
    let border = if selected {
        TREASURE
    } else {
        Color::new(DANGER.r, DANGER.g, DANGER.b, 0.24)
    };
    draw_card(rect, fill, border);

    draw_text_fit(
        &template.name,
        rect.x + 12.0,
        rect.y + rect.h * 0.38,
        rect.w - 76.0,
        13.0,
        if can_afford { TEXT } else { TEXT_DIM },
    );
    // Traps show their behavior kind; other upgrades show their family.
    let kind_label = if template.upgrade_type == "trap" && !template.effect_kind.is_empty() {
        template.effect_kind.as_str()
    } else {
        template.upgrade_type.as_str()
    };
    draw_text_fit(
        &format!(
            "{}{}",
            kind_label,
            template
                .element
                .as_deref()
                .map(|e| format!(" • {}", e))
                .unwrap_or_default()
        ),
        rect.x + 12.0,
        rect.y + rect.h * 0.72,
        rect.w - 76.0,
        10.0,
        TEXT_MUTED,
    );
    let cost_label = if template.souls_cost > 0 {
        format!("{}M+{}S", template.mana_cost, template.souls_cost)
    } else {
        format!("{}M", template.mana_cost)
    };
    draw_text_fit_right(
        &cost_label,
        rect.x + rect.w - 10.0,
        rect.y + rect.h * 0.58,
        60.0,
        12.0,
        if can_afford { MANA } else { DANGER },
    );

    can_afford && was_clicked_rect(rect)
}

fn draw_evolution_tab(state: &GameState, rect: Rect) -> DrawerAction {
    draw_section_title(rect, "EVOLUTION", "Advance unlocked species.");
    let mut action = DrawerAction::None;

    let rows = collect_evolution_rows(state);
    let ready_count = rows.iter().filter(|row| row.ready).count();
    let waiting_count = rows.iter().filter(|row| !row.ready && row.has_path).count();
    let final_count = rows.iter().filter(|row| !row.has_path).count();

    let card = Rect::new(rect.x, rect.y + 70.0, rect.w, 126.0);
    draw_card(card, CARD, BORDER_MUTED);
    draw_text_fit(
        &format!("Ready: {}  Waiting: {}", ready_count, waiting_count),
        card.x + 12.0,
        card.y + 28.0,
        card.w - 24.0,
        15.0,
        TEXT,
    );
    draw_text_fit(
        &format!("Final forms: {}", final_count),
        card.x + 12.0,
        card.y + 55.0,
        card.w - 24.0,
        13.0,
        TEXT_MUTED,
    );
    draw_text_fit(
        &format!(
            "Species: {}  Souls: {}",
            state.unlocked_species.len(),
            state.souls
        ),
        card.x + 12.0,
        card.y + 82.0,
        card.w - 24.0,
        13.0,
        SOUL,
    );
    draw_text_fit(
        "Current defenders and their next forms.",
        card.x + 12.0,
        card.y + 108.0,
        card.w - 24.0,
        11.0,
        TEXT_DIM,
    );

    let mut row_y = card.y + card.h + 12.0;
    let row_h = 46.0;
    for row in rows
        .iter()
        .take(((rect.y + rect.h - row_y - 106.0) / row_h).max(0.0) as usize)
    {
        draw_evolution_row(row, Rect::new(rect.x, row_y, rect.w, row_h - 6.0));
        row_y += row_h;
    }

    if let Some(species) = next_locked_species(state) {
        let unlock_cost = species.unlock_cost;
        let can_afford = state.gold >= unlock_cost;
        let species_name = species.name.clone();
        let unlock_rect = Rect::new(rect.x, rect.y + rect.h - 94.0, rect.w, 40.0);
        draw_card(
            unlock_rect,
            Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.075),
            Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.24),
        );
        draw_text_fit(
            &format!("Next race: {}", get_species_display_name(&species_name)),
            unlock_rect.x + 10.0,
            unlock_rect.y + 16.0,
            unlock_rect.w - 96.0,
            11.0,
            TEXT,
        );
        draw_text_fit(
            &format!("{} gold", unlock_cost),
            unlock_rect.x + 10.0,
            unlock_rect.y + 32.0,
            unlock_rect.w - 96.0,
            10.0,
            if can_afford { TREASURE } else { TEXT_DIM },
        );
        if draw_command_button(
            Rect::new(
                unlock_rect.x + unlock_rect.w - 78.0,
                unlock_rect.y + 7.0,
                68.0,
                26.0,
            ),
            "Unlock",
            ButtonTone::Ghost,
            can_afford,
        ) {
            action = DrawerAction::UnlockSpecies(species_name);
        }
    }

    if draw_command_button(
        Rect::new(rect.x, rect.y + rect.h - 46.0, rect.w, 42.0),
        "Evolve",
        ButtonTone::Arcane,
        ready_count > 0,
    ) {
        action = DrawerAction::ProcessEvolutions;
    }

    action
}

fn draw_monster_option(state: &GameState, template: &MonsterTemplate, rect: Rect) -> bool {
    let unlocked = state.unlocked_species.contains(&template.species)
        && state.unlocked_monsters.contains(&template.name);
    let can_afford = state.mana >= template.base_cost && state.souls >= template.souls_cost;
    let enabled = unlocked && can_afford;
    let selected = state.selected_monster.as_ref() == Some(&template.name);
    let hovered = enabled && is_hovered_rect(rect);
    let fill = if selected {
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.13)
    } else if hovered {
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.10)
    } else {
        CARD
    };
    let border = if selected {
        TREASURE
    } else if unlocked {
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.24)
    } else {
        BORDER_MUTED
    };

    draw_card(rect, fill, border);
    let portrait = Rect::new(rect.x + 9.0, rect.y + 9.0, 54.0, rect.h - 18.0);
    draw_monster_portrait(portrait, unlocked, selected);
    let title = if unlocked {
        template.name.as_str()
    } else {
        template.species.as_str()
    };
    draw_text_fit(
        title,
        rect.x + 74.0,
        rect.y + rect.h * 0.38,
        rect.w - 126.0,
        14.0,
        if unlocked { TEXT } else { TEXT_DIM },
    );
    let traits = trait_summary(&template.traits);
    let detail = if unlocked {
        format!(
            "T{} {} {}{}  {}",
            template.tier,
            get_species_display_name(&template.species),
            template.element.as_deref().unwrap_or("Neutral"),
            if template.boss_only { " • Boss room" } else { "" },
            traits
        )
    } else {
        "Locked".to_string()
    };
    draw_text_fit(
        &detail,
        rect.x + 74.0,
        rect.y + rect.h * 0.70,
        rect.w - 126.0,
        11.0,
        TEXT_MUTED,
    );
    let cost_label = if template.souls_cost > 0 {
        format!("{}M+{}S", template.base_cost, template.souls_cost)
    } else {
        format!("{}M", template.base_cost)
    };
    draw_text_fit_right(
        &cost_label,
        rect.x + rect.w - 10.0,
        rect.y + rect.h * 0.58,
        54.0,
        13.0,
        if can_afford { MANA } else { DANGER },
    );

    enabled && was_clicked_rect(rect)
}

fn draw_section_title(rect: Rect, title: &str, subtitle: &str) {
    draw_text_fit(
        title,
        rect.x + 24.0,
        rect.y + 28.0,
        rect.w - 24.0,
        20.0,
        TEXT,
    );
    draw_poly_lines(rect.x + 10.0, rect.y + 22.0, 6, 8.0, 30.0, 1.5, SOUL);
    draw_text_fit(subtitle, rect.x, rect.y + 54.0, rect.w, 12.0, TEXT_MUTED);
}

fn draw_small_tab(rect: Rect, text: &str, color: Color, active: bool) -> bool {
    let hovered = is_hovered_rect(rect);
    draw_card(
        rect,
        if active {
            Color::new(color.r, color.g, color.b, 0.16)
        } else if hovered {
            Color::new(color.r, color.g, color.b, 0.10)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.10)
        },
        Color::new(color.r, color.g, color.b, if active { 0.42 } else { 0.18 }),
    );
    draw_centered_text(text, rect, 10.0, if active { color } else { TEXT_MUTED });
    was_clicked_rect(rect)
}

fn draw_rail_tab(rect: Rect, icon: &str, label: &str, color: Color, active: bool) -> bool {
    let hovered = is_hovered_rect(rect);
    draw_card(
        rect,
        if active {
            Color::new(color.r, color.g, color.b, 0.14)
        } else if hovered {
            Color::new(color.r, color.g, color.b, 0.08)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.10)
        },
        Color::new(color.r, color.g, color.b, if active { 0.48 } else { 0.16 }),
    );
    draw_centered_text(
        icon,
        Rect::new(rect.x, rect.y + 10.0, rect.w, 24.0),
        22.0,
        if active { color } else { TEXT_DIM },
    );
    draw_centered_text(
        label,
        Rect::new(rect.x + 2.0, rect.y + 43.0, rect.w - 4.0, 20.0),
        8.0,
        if active { TEXT } else { TEXT_MUTED },
    );
    was_clicked_rect(rect)
}

fn draw_monster_portrait(rect: Rect, unlocked: bool, selected: bool) {
    let color = if unlocked { EMERALD } else { TEXT_DIM };
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.30),
        Color::new(
            color.r,
            color.g,
            color.b,
            if selected { 0.55 } else { 0.20 },
        ),
    );
    if unlocked {
        draw_circle(
            rect.x + rect.w * 0.50,
            rect.y + rect.h * 0.42,
            rect.w * 0.22,
            Color::new(color.r, color.g, color.b, 0.42),
        );
        draw_triangle(
            vec2(rect.x + rect.w * 0.21, rect.y + rect.h * 0.30),
            vec2(rect.x + rect.w * 0.38, rect.y + rect.h * 0.38),
            vec2(rect.x + rect.w * 0.30, rect.y + rect.h * 0.55),
            color,
        );
        draw_triangle(
            vec2(rect.x + rect.w * 0.79, rect.y + rect.h * 0.30),
            vec2(rect.x + rect.w * 0.62, rect.y + rect.h * 0.38),
            vec2(rect.x + rect.w * 0.70, rect.y + rect.h * 0.55),
            color,
        );
        draw_circle(rect.x + rect.w * 0.42, rect.y + rect.h * 0.40, 2.0, BG_DEEP);
        draw_circle(rect.x + rect.w * 0.58, rect.y + rect.h * 0.40, 2.0, BG_DEEP);
    } else {
        draw_rectangle(
            rect.x + rect.w * 0.30,
            rect.y + rect.h * 0.38,
            rect.w * 0.40,
            rect.h * 0.34,
            Color::new(0.0, 0.0, 0.0, 0.34),
        );
        draw_rectangle_lines(
            rect.x + rect.w * 0.34,
            rect.y + rect.h * 0.45,
            rect.w * 0.32,
            rect.h * 0.22,
            2.0,
            TEXT_DIM,
        );
        draw_circle_lines(
            rect.x + rect.w * 0.50,
            rect.y + rect.h * 0.43,
            rect.w * 0.16,
            2.0,
            TEXT_DIM,
        );
    }
}

#[derive(Debug)]
struct EvolutionUiRow {
    monster: String,
    location: String,
    xp_label: String,
    status: String,
    color: Color,
    ready: bool,
    has_path: bool,
}

fn collect_evolution_rows(state: &GameState) -> Vec<EvolutionUiRow> {
    let mut rows = Vec::new();

    for floor in &state.floors {
        for room in &floor.rooms {
            for monster in &room.monsters {
                let location = format!("F{} R{}", room.floor_number, room.position);
                if let Some(path) = get_evolution_for_monster(&monster.type_name) {
                    let ready_xp = monster.experience >= path.experience_required;
                    let ready_floor = room.floor_number >= path.conditions.min_floor;
                    let ready_gold = state.gold >= path.conditions.gold_cost;
                    let ready = ready_xp && ready_floor && ready_gold;
                    let (status, color) = if ready {
                        (format!("Ready -> {}", path.to_monster), EMERALD)
                    } else if !ready_xp {
                        (
                            format!("Needs {} XP", path.experience_required - monster.experience),
                            MANA,
                        )
                    } else if !ready_floor {
                        (
                            format!("Needs floor {}", path.conditions.min_floor),
                            WARNING,
                        )
                    } else {
                        (
                            format!("Needs {} gold", path.conditions.gold_cost),
                            TREASURE,
                        )
                    };

                    rows.push(EvolutionUiRow {
                        monster: monster.type_name.clone(),
                        location,
                        xp_label: format!("{}/{} XP", monster.experience, path.experience_required),
                        status,
                        color,
                        ready,
                        has_path: true,
                    });
                } else {
                    rows.push(EvolutionUiRow {
                        monster: monster.type_name.clone(),
                        location,
                        xp_label: format!("{} XP", monster.experience),
                        status: "Final form".to_string(),
                        color: TEXT_DIM,
                        ready: false,
                        has_path: false,
                    });
                }
            }
        }
    }

    rows.sort_by(|a, b| {
        b.ready
            .cmp(&a.ready)
            .then_with(|| a.monster.cmp(&b.monster))
    });
    rows
}

fn draw_evolution_row(row: &EvolutionUiRow, rect: Rect) {
    draw_card(
        rect,
        Color::new(row.color.r, row.color.g, row.color.b, 0.075),
        Color::new(row.color.r, row.color.g, row.color.b, 0.26),
    );
    draw_text_fit(
        &row.monster,
        rect.x + 9.0,
        rect.y + 16.0,
        rect.w - 82.0,
        12.0,
        TEXT,
    );
    draw_text_fit(
        &format!("{}  {}", row.location, row.xp_label),
        rect.x + 9.0,
        rect.y + 32.0,
        rect.w - 82.0,
        10.0,
        TEXT_MUTED,
    );
    draw_text_fit_right(
        &row.status,
        rect.x + rect.w - 8.0,
        rect.y + 25.0,
        92.0,
        10.0,
        row.color,
    );
}

fn trait_summary(trait_ids: &[String]) -> String {
    if trait_ids.is_empty() {
        return "No traits".to_string();
    }

    trait_ids
        .iter()
        .take(2)
        .map(|trait_id| {
            get_trait(trait_id)
                .map(|trait_def| trait_def.name)
                .unwrap_or_else(|| trait_id.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn next_locked_species(state: &GameState) -> Option<crate::data::monsters::SpeciesData> {
    let mut locked = get_all_species()
        .into_iter()
        .filter(|species| !state.unlocked_species.contains(&species.name))
        .collect::<Vec<_>>();
    locked.sort_by_key(|species| species.unlock_cost);
    locked.into_iter().next()
}

fn next_build_summary(state: &GameState) -> (String, String, i32) {
    let Some(deepest) = state.deepest_floor() else {
        return (
            "Entrance".to_string(),
            "No floor mapped yet.".to_string(),
            0,
        );
    };

    let non_core_count = deepest
        .rooms
        .iter()
        .filter(|room| room.room_type != RoomType::Core)
        .count();
    let total_rooms = state.total_room_count();

    if non_core_count > MAX_ROOMS_PER_FLOOR {
        return (
            format!("Open floor {}", state.total_floors + 1),
            "Move the core deeper.".to_string(),
            get_room_cost(total_rooms, false),
        );
    }

    let is_boss = non_core_count == MAX_ROOMS_PER_FLOOR;
    (
        if is_boss {
            "Boss chamber".to_string()
        } else {
            "Combat room".to_string()
        },
        format!("Floor {}", deepest.number),
        get_room_cost(total_rooms, is_boss),
    )
}
