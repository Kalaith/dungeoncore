//! Mid-raid player agency: the dungeon heart's active "Core Smite" spell.
//!
//! Raids otherwise resolve on their own, tick by tick. Core Smite is the one
//! lever the player pulls mid-fight — a mana-fuelled blast that sears the
//! invaders pushed deepest into the dungeon, on a short cooldown so it *steers*
//! a raid rather than trivializing it (the Legend of Keepers model: watch, and
//! slightly steer).

use crate::game_state::{EffectAnchor, EffectKind, GameState, LogEntry};

/// Real-time seconds Core Smite takes to recharge after a cast.
pub const CORE_SMITE_COOLDOWN: f32 = 16.0;
/// Mana spent per Core Smite.
pub const CORE_SMITE_MANA_COST: i32 = 40;
/// Damage of a Core Smite before dungeon scaling.
const SMITE_BASE_DAMAGE: i32 = 28;

/// Damage a single smite deals to each struck invader, scaling with how deep
/// and prestigious the dungeon has grown (mirrors the core's own siege wrath)
/// plus any offense core powers (Searing Smite, Cataclysm).
pub fn smite_damage(state: &GameState) -> i32 {
    SMITE_BASE_DAMAGE
        + state.total_floors * 6
        + state.prestige * 10
        + crate::simulation::endgame::core_smite_damage_bonus(state)
}

/// The Core Smite cooldown after core-power reductions (Quickening,
/// Worldbreaker), floored so it can never trivialize the lever.
pub fn smite_cooldown(state: &GameState) -> f32 {
    (CORE_SMITE_COOLDOWN - crate::simulation::endgame::core_smite_cooldown_reduction(state))
        .max(4.0)
}

/// Is Core Smite recharged and ready to fire?
pub fn is_ready(state: &GameState) -> bool {
    state.core_smite_cooldown <= 0.0
}

/// Index of the party a smite would strike: the living, non-retreating party
/// that has pushed deepest into the dungeon (furthest floor, then room). Siege
/// parties count — the core can turn its wrath on the army at its gates.
pub fn smite_target(state: &GameState) -> Option<usize> {
    state
        .adventurer_parties
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.retreating && p.members.iter().any(|m| m.alive))
        .max_by_key(|(_, p)| (p.current_floor, p.current_room as i32))
        .map(|(idx, _)| idx)
}

/// Cast Core Smite at the deepest invading party. Returns `Ok` on a successful
/// cast, or a short reason it could not fire (recharging, no mana, no target).
pub fn cast_core_smite(state: &mut GameState) -> Result<(), String> {
    if !is_ready(state) {
        return Err(format!(
            "Core Smite is recharging ({:.0}s).",
            state.core_smite_cooldown.ceil()
        ));
    }
    if state.mana < CORE_SMITE_MANA_COST {
        return Err(format!(
            "Not enough mana to smite! Need {}.",
            CORE_SMITE_MANA_COST
        ));
    }
    let Some(party_idx) = smite_target(state) else {
        return Err("No invaders in the dungeon to smite.".into());
    };

    state.mana -= CORE_SMITE_MANA_COST;
    state.core_smite_cooldown = smite_cooldown(state);

    let damage = smite_damage(state);
    let floor = state.adventurer_parties[party_idx].current_floor;
    let room = state.adventurer_parties[party_idx].current_room;

    // Strike every living member; note who falls for the ledger and income.
    let mut kills: Vec<(String, i32)> = Vec::new();
    {
        let party = &mut state.adventurer_parties[party_idx];
        for member in party.members.iter_mut().filter(|m| m.alive) {
            member.hp -= damage;
            if member.hp <= 0 {
                member.hp = 0;
                member.alive = false;
                party.casualties += 1;
                kills.push((member.name.clone(), member.level));
            }
        }
    }

    state.push_effect_at(
        floor,
        room,
        format!("SMITE! -{}", damage),
        EffectKind::Ability,
        EffectAnchor::Invaders,
    );

    // Death income: mana per fallen invader, exactly as combat pays, tallied
    // for the raid card and folded into threat. Hero-death recording happens
    // when the party settles — matching how combat kills are booked.
    let income_mult = state.income_mult();
    for (name, level) in &kills {
        let mana_gain = ((level * 10) as f32 * income_mult).round() as i32;
        state.mana = (state.mana + mana_gain).min(state.max_mana);
        state.total_deaths += 1;
        state.raid_tally().mana_gained += mana_gain;
        state.push_effect_at(
            floor,
            room,
            "Slain!",
            EffectKind::AdventurerDown,
            EffectAnchor::Invaders,
        );
        state.add_log(LogEntry::combat(format!(
            "Core Smite strikes down {}! +{} mana",
            name, mana_gain
        )));
    }

    if kills.is_empty() {
        state.add_log(LogEntry::combat(format!(
            "The core unleashes a Smite — {} damage sears the invaders!",
            damage
        )));
    }

    // A smite that wipes the party ends the raid; the normal settle path
    // finalizes the summary card on the next process tick.
    let wiped = state.adventurer_parties[party_idx]
        .members
        .iter()
        .all(|m| !m.alive);
    if wiped {
        state.adventurer_parties[party_idx].retreating = true;
        state.add_log(LogEntry::adventure(
            "The party is annihilated by the core's wrath!",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::{Adventurer, AdventurerParty, Stats};

    fn party_with(hp: i32, count: usize, floor: i32, room: usize) -> AdventurerParty {
        let members = (0..count as u64)
            .map(|i| Adventurer {
                id: 10 + i,
                name: format!("Hero{i}"),
                class_name: "Warrior".to_string(),
                race: "Human".to_string(),
                level: 2,
                hp,
                max_hp: hp,
                alive: true,
                experience: 0,
                gold: 0,
                equipment: Default::default(),
                conditions: Vec::new(),
                scaled_stats: Stats {
                    hp,
                    attack: 5,
                    defense: 2,
                },
            })
            .collect();
        AdventurerParty {
            id: 1,
            members,
            current_floor: floor,
            current_room: room,
            retreating: false,
            casualties: 0,
            loot: 0,
            entry_time: 0,
            target_floor: floor,
            snared_ticks: 0,
            alarmed: false,
            sieging: false,
            prev_room: 0,
            move_anim: 0.0,
        }
    }

    #[test]
    fn smite_damages_and_costs_mana_and_sets_cooldown() {
        let mut s = GameState::new();
        s.mana = 100;
        s.adventurer_parties.push(party_with(999, 2, 1, 0));
        cast_core_smite(&mut s).unwrap();
        assert!(s.core_smite_cooldown > 0.0);
        assert_eq!(s.mana, 100 - CORE_SMITE_MANA_COST);
        // Both members took the hit but survived their large HP pool.
        assert!(s.adventurer_parties[0].members.iter().all(|m| m.alive));
        assert!(s.adventurer_parties[0].members[0].hp < 999);
    }

    #[test]
    fn smite_wipe_retreats_party_and_pays_mana() {
        let mut s = GameState::new();
        s.mana = 100;
        s.max_mana = 999;
        s.adventurer_parties.push(party_with(1, 3, 1, 0));
        let deaths_before = s.total_deaths;
        cast_core_smite(&mut s).unwrap();
        assert!(s.adventurer_parties[0].retreating);
        assert!(s.adventurer_parties[0].members.iter().all(|m| !m.alive));
        assert_eq!(s.total_deaths, deaths_before + 3);
    }

    #[test]
    fn smite_blocked_while_recharging_and_without_target() {
        let mut s = GameState::new();
        s.mana = 100;
        // No party present → no target.
        assert!(cast_core_smite(&mut s).is_err());
        // On cooldown → blocked even with a target.
        s.adventurer_parties.push(party_with(999, 1, 1, 0));
        s.core_smite_cooldown = 5.0;
        assert!(cast_core_smite(&mut s).is_err());
    }

    #[test]
    fn smite_targets_deepest_party() {
        let mut s = GameState::new();
        s.adventurer_parties.push(party_with(999, 1, 1, 0));
        s.adventurer_parties.push(party_with(999, 1, 2, 1));
        assert_eq!(smite_target(&s), Some(1));
    }
}
