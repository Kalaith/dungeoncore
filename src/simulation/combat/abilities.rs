//! Active monster abilities (OnCombatStart) and lingering party afflictions.

use crate::data::traits::get_trait;
use crate::game_state::{EffectKind, GameState, LogEntry};

use super::rewards::reward_adventurer_kills;

/// Fire monsters' active abilities (currently OnCombatStart party damage).
pub(super) fn resolve_abilities(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
    floor_num: i32,
    room_pos: usize,
) {
    let mut combat_logs: Vec<LogEntry> = Vec::new();
    let mut ability_deaths = 0;
    let mut ability_used: Option<(String, i32)> = None;

    {
        let floor = &mut state.floors[floor_idx];
        let room = &mut floor.rooms[room_idx];
        let party = &mut state.adventurer_parties[party_idx];

        for monster in room.monsters.iter_mut().filter(|m| m.alive) {
            for trait_data in &mut monster.active_traits {
                if trait_data.cooldown_timer > 0 {
                    trait_data.cooldown_timer -= 1;
                    continue;
                }
                let Some(trait_def) = get_trait(&trait_data.id) else {
                    continue;
                };
                if trait_def.trait_type == "Active"
                    && trait_def.applies_to == "OnCombatStart"
                    && trait_def.effect_type == "DamageFlat"
                {
                    let damage = trait_def.value as i32;
                    trait_data.cooldown_timer = trait_def.cooldown;

                    // "EnemyParty" hits everyone; "Enemy" hits the front adventurer.
                    let single_target = trait_def.target_type == "Enemy";
                    let mut total_hits = 0;
                    for adv in party.members.iter_mut().filter(|a| a.alive) {
                        adv.hp -= damage;
                        total_hits += 1;
                        if adv.hp <= 0 {
                            adv.hp = 0;
                            adv.alive = false;
                            party.casualties += 1;
                            ability_deaths += 1;
                        }
                        if single_target {
                            break;
                        }
                    }

                    if total_hits > 0 {
                        combat_logs.push(LogEntry::combat(format!(
                            "{} uses {}! Dealt {} damage to {} adventurer{}.",
                            monster.type_name,
                            trait_def.name,
                            damage,
                            total_hits,
                            if total_hits == 1 { "" } else { "s" }
                        )));
                        ability_used = Some((trait_def.name.clone(), damage));
                    }
                }
            }
        }
    }

    for log in combat_logs {
        state.add_log(log);
    }
    if let Some((ability_name, damage)) = ability_used {
        state.push_effect(
            floor_num,
            room_pos,
            format!("{} -{}", ability_name, damage),
            EffectKind::Ability,
        );
    }
    if ability_deaths > 0 {
        state.total_deaths += ability_deaths;
        state.push_effect(floor_num, room_pos, "Slain!", EffectKind::AdventurerDown);
    }
}

/// Lingering conditions (poison, burn) damage the party each combat tick.
pub(super) fn tick_conditions(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
) {
    let floor_num = state.floors[floor_idx].number;
    let room_pos = state.floors[floor_idx].rooms[room_idx].position;

    let mut kills: Vec<(String, i32)> = Vec::new();
    let mut total_damage = 0;
    {
        let party = &mut state.adventurer_parties[party_idx];
        for adv in party.members.iter_mut() {
            if !adv.alive || adv.conditions.is_empty() {
                continue;
            }
            let mut damage = 0;
            for condition in &mut adv.conditions {
                damage += condition.power;
                condition.ticks -= 1;
            }
            adv.conditions.retain(|c| c.ticks > 0);
            if damage > 0 {
                adv.hp -= damage;
                total_damage += damage;
                if adv.hp <= 0 {
                    adv.hp = 0;
                    adv.alive = false;
                    party.casualties += 1;
                    kills.push((adv.name.clone(), adv.level));
                }
            }
        }
    }

    if total_damage > 0 && kills.is_empty() {
        state.push_effect(
            floor_num,
            room_pos,
            format!("-{}", total_damage),
            EffectKind::Damage,
        );
    }
    reward_adventurer_kills(state, party_idx, floor_idx, room_idx, &kills);
}
