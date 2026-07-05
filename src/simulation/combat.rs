use crate::data::adventurers::get_victory_quotes;
use crate::data::constants::RETREAT_THRESHOLD;
use crate::data::elements::element_multiplier;
use crate::data::traits::get_trait;
use crate::game_state::{EffectKind, GameState, LogEntry, Monster};

/// Chance per combat tick that a room's trap upgrade fires.
const TRAP_TRIGGER_CHANCE: f32 = 0.2;

/// Resolve one combat tick between a party and the monsters in a room.
///
/// Damage model: every living combatant acts once per tick.
/// Adventurers focus the front monster; monsters each strike a random
/// adventurer. Damage = max(1, attack - defense/2), further shaped by
/// room upgrades and monster traits. Deaths occur when HP reaches 0.
pub fn resolve_combat(state: &mut GameState, party_idx: usize, floor_idx: usize, room_idx: usize) {
    let has_alive_monsters = state.floors[floor_idx].rooms[room_idx]
        .monsters
        .iter()
        .any(|m| m.alive);

    if !has_alive_monsters {
        return;
    }

    let floor_num = state.floors[floor_idx].number;
    let room_pos = state.floors[floor_idx].rooms[room_idx].position;

    let trap_mult = state.floors[floor_idx].rooms[room_idx].trap_multiplier();
    let reinforcement_mult = state.floors[floor_idx].rooms[room_idx].reinforcement_multiplier();

    // Phase 1: trap damage
    if trap_mult > 1.0 {
        apply_trap_damage(state, party_idx, floor_idx, room_idx, trap_mult);
    }

    // Phase 2: active abilities (e.g. Fire Breath on combat start)
    resolve_abilities(state, party_idx, floor_idx, room_idx, floor_num, room_pos);

    // Room element attunement boosts monsters of the matching element.
    let attunement: Option<(String, f32)> = state.floors[floor_idx].rooms[room_idx]
        .attunement()
        .map(|(element, mult)| (element.to_string(), mult));

    // Phase 3: adventurers strike the front monster
    let adv_attacks: Vec<(i32, String)> = state.adventurer_parties[party_idx]
        .members
        .iter()
        .filter(|a| a.alive)
        .map(|a| (a.scaled_stats.attack, adventurer_element(&a.class_name)))
        .collect();

    let mut monster_kills: Vec<(String, bool)> = Vec::new();
    let mut split_spawns: Vec<String> = Vec::new();
    let mut party_hit_strong = false;
    let mut party_hit_weak = false;
    {
        let room = &mut state.floors[floor_idx].rooms[room_idx];
        for (attack, adv_element) in &adv_attacks {
            // Taunting monsters soak hits before the rest of the room.
            let Some(target_idx) = target_monster_idx(&room.monsters) else {
                break;
            };
            let monster = &mut room.monsters[target_idx];
            let mon_element = monster_element(&monster.type_name);
            let attune_mult = attunement_mult(&attunement, &mon_element);
            let effective_def =
                monster.scaled_stats.defense as f32 * reinforcement_mult * attune_mult;
            let taken_mult = monster_damage_taken_mult(monster);
            let elem_mult = element_multiplier(adv_element, &mon_element);
            if elem_mult > 1.0 {
                party_hit_strong = true;
            } else if elem_mult < 1.0 {
                party_hit_weak = true;
            }
            let damage = ((*attack as f32 - effective_def / 2.0).max(1.0)
                * taken_mult
                * elem_mult)
                .round()
                .max(1.0) as i32;
            monster.hp -= damage;
            if monster.hp <= 0 {
                monster.hp = 0;
                monster.alive = false;
                monster_kills.push((monster.type_name.clone(), monster.is_boss));
                if has_passive(monster, "SplitOnDeath") {
                    if let Some(spawn) = split_spawn(&monster.type_name, floor_num) {
                        split_spawns.push(spawn.type_name.clone());
                        room.monsters.push(spawn);
                    }
                }
            }
        }
    }

    for spawn_name in &split_spawns {
        state.add_log(LogEntry::combat(format!(
            "The slain monster splits — a {} emerges!",
            spawn_name
        )));
        state.push_effect(floor_num, room_pos, "Split!", EffectKind::Ability);
    }

    if party_hit_strong {
        state.push_effect(floor_num, room_pos, "Strong hit!", EffectKind::Ability);
    } else if party_hit_weak {
        state.push_effect(floor_num, room_pos, "Resisted", EffectKind::Ability);
    }

    // Phase 4: surviving monsters strike back
    let monster_strikes: Vec<MonsterStrike> = {
        let room = &state.floors[floor_idx].rooms[room_idx];
        let alive_count = room.monsters.iter().filter(|m| m.alive).count();
        let enemies_alive = state.adventurer_parties[party_idx]
            .members
            .iter()
            .filter(|a| a.alive)
            .count();
        room.monsters
            .iter()
            .filter(|m| m.alive)
            .map(|m| {
                let element = monster_element(&m.type_name);
                let attune_mult = attunement_mult(&attunement, &element);
                MonsterStrike {
                    monster_id: m.id,
                    attack: monster_attack_value(
                        m,
                        alive_count,
                        enemies_alive,
                        reinforcement_mult * attune_mult,
                    ),
                    element,
                    pierce: has_passive(m, "ArmorPierce"),
                    lifesteal: passive_value(m, "LifeStealPercent"),
                    mana_on_kill: passive_value(m, "ManaOnKill") as i32,
                }
            })
            .collect()
    };

    let mut adventurer_kills: Vec<(String, i32)> = Vec::new();
    let mut damage_to_party = 0;
    let mut monster_hit_strong = false;
    let mut lifesteal_heals: Vec<(u64, i32)> = Vec::new();
    let mut leeched_mana = 0;
    {
        let party = &mut state.adventurer_parties[party_idx];
        for strike in monster_strikes {
            let alive_idxs: Vec<usize> = party
                .members
                .iter()
                .enumerate()
                .filter(|(_, a)| a.alive)
                .map(|(i, _)| i)
                .collect();
            if alive_idxs.is_empty() {
                break;
            }
            let victim_idx = alive_idxs
                [macroquad_toolkit::rng::gen_range(0, alive_idxs.len())];
            let victim = &mut party.members[victim_idx];
            let elem_mult =
                element_multiplier(&strike.element, &adventurer_element(&victim.class_name));
            if elem_mult > 1.0 {
                monster_hit_strong = true;
            }
            let victim_def = if strike.pierce {
                0.0
            } else {
                victim.scaled_stats.defense as f32 / 2.0
            };
            let damage = ((strike.attack as f32 - victim_def).max(1.0) * elem_mult)
                .round()
                .max(1.0) as i32;
            victim.hp -= damage;
            damage_to_party += damage;
            if strike.lifesteal > 0.0 {
                let heal = (damage as f32 * strike.lifesteal).round() as i32;
                if heal > 0 {
                    lifesteal_heals.push((strike.monster_id, heal));
                }
            }
            if victim.hp <= 0 {
                victim.hp = 0;
                victim.alive = false;
                party.casualties += 1;
                adventurer_kills.push((victim.name.clone(), victim.level));
                leeched_mana += strike.mana_on_kill;
            }
        }
    }

    // Apply lifesteal heals now that the party borrow has ended.
    if !lifesteal_heals.is_empty() {
        let room = &mut state.floors[floor_idx].rooms[room_idx];
        for (monster_id, heal) in lifesteal_heals {
            if let Some(monster) = room
                .monsters
                .iter_mut()
                .find(|m| m.id == monster_id && m.alive)
            {
                monster.hp = (monster.hp + heal).min(monster.max_hp);
            }
        }
    }
    if leeched_mana > 0 {
        state.mana = (state.mana + leeched_mana).min(state.max_mana);
        state.add_log(LogEntry::combat(format!(
            "Mana Leech drains +{} mana from the fallen.",
            leeched_mana
        )));
    }

    if damage_to_party > 0 && adventurer_kills.is_empty() {
        state.push_effect(
            floor_num,
            room_pos,
            format!(
                "-{}{}",
                damage_to_party,
                if monster_hit_strong { "!" } else { "" }
            ),
            EffectKind::Damage,
        );
    }

    reward_monster_kills(state, party_idx, floor_idx, room_idx, &monster_kills);
    reward_adventurer_kills(state, party_idx, floor_idx, room_idx, &adventurer_kills);
    check_retreat(state, party_idx);
}

/// One monster's pending attack for the strike-back phase.
struct MonsterStrike {
    monster_id: u64,
    attack: i32,
    element: String,
    pierce: bool,
    lifesteal: f32,
    mana_on_kill: i32,
}

/// Whether a monster has a passive trait with the given effect type.
fn has_passive(monster: &Monster, effect_type: &str) -> bool {
    monster.active_traits.iter().any(|t| {
        get_trait(&t.id)
            .is_some_and(|d| d.trait_type == "Passive" && d.effect_type == effect_type)
    })
}

/// Summed value of a monster's passive traits with the given effect type.
fn passive_value(monster: &Monster, effect_type: &str) -> f32 {
    monster
        .active_traits
        .iter()
        .filter_map(|t| get_trait(&t.id))
        .filter(|d| d.trait_type == "Passive" && d.effect_type == effect_type)
        .map(|d| d.value)
        .sum()
}

/// Index of the monster adventurers hit next: taunters first, then the front.
fn target_monster_idx(monsters: &[Monster]) -> Option<usize> {
    monsters
        .iter()
        .position(|m| m.alive && has_passive(m, "Taunt"))
        .or_else(|| monsters.iter().position(|m| m.alive))
}

/// Build the tier-1 monster a slain splitter breaks into (half HP).
fn split_spawn(parent_type: &str, floor: i32) -> Option<Monster> {
    let parent = crate::data::monsters::get_monster_template(parent_type)?;
    let candidates: Vec<_> = crate::data::monsters::get_monster_templates()
        .into_iter()
        .filter(|t| t.species == parent.species && t.tier == 1)
        .collect();
    let template = candidates
        .iter()
        .find(|t| t.element == parent.element)
        .or_else(|| candidates.first())?
        .clone();

    let scaled = crate::data::get_scaled_stats(
        crate::game_state::Stats {
            hp: template.hp,
            attack: template.attack,
            defense: template.defense,
        },
        floor,
        false,
    );

    Some(Monster {
        id: macroquad_toolkit::rng::random_u64(),
        type_name: template.name.clone(),
        hp: (scaled.hp / 2).max(1),
        max_hp: scaled.hp,
        alive: true,
        is_boss: false,
        scaled_stats: scaled,
        active_traits: template
            .traits
            .iter()
            .map(|trait_id| crate::game_state::ActiveTrait {
                id: trait_id.clone(),
                name: get_trait(trait_id)
                    .map(|t| t.name)
                    .unwrap_or_else(|| trait_id.clone()),
                cooldown_timer: 0,
            })
            .collect(),
        experience: 0,
    })
}

/// Damage element of an adventurer, from their class. Empty = neutral.
fn adventurer_element(class_name: &str) -> String {
    crate::data::adventurers::get_adventurer_class(class_name)
        .map(|c| c.element)
        .unwrap_or_default()
}

/// Element of a monster, from its template. Empty = neutral.
fn monster_element(type_name: &str) -> String {
    crate::data::monsters::get_monster_template(type_name)
        .and_then(|t| t.element)
        .unwrap_or_default()
}

/// Stat multiplier a room attunement grants to a monster of `element`.
fn attunement_mult(attunement: &Option<(String, f32)>, element: &str) -> f32 {
    match attunement {
        Some((attuned, mult)) if !element.is_empty() && attuned == element => *mult,
        _ => 1.0,
    }
}

/// Damage multiplier from a monster's defensive passive traits.
fn monster_damage_taken_mult(monster: &Monster) -> f32 {
    let mut mult = 1.0;
    for trait_data in &monster.active_traits {
        if let Some(def) = get_trait(&trait_data.id) {
            if def.trait_type == "Passive"
                && def.applies_to == "OnDefense"
                && def.effect_type == "DamageReductionMult"
            {
                mult *= 1.0 - def.value;
            }
        }
    }
    mult
}

/// Effective attack including offensive passives and room reinforcement.
fn monster_attack_value(
    monster: &Monster,
    allies_alive: usize,
    enemies_alive: usize,
    reinforcement_mult: f32,
) -> i32 {
    let mut attack = monster.scaled_stats.attack as f32;
    for trait_data in &monster.active_traits {
        if let Some(def) = get_trait(&trait_data.id) {
            if def.trait_type == "Passive"
                && def.applies_to == "OnAttack"
                && def.effect_type == "AttackBonus"
            {
                match def.scaling_type.as_str() {
                    "PerAlly" => attack += def.value * allies_alive.saturating_sub(1) as f32,
                    "PerEnemy" => attack += def.value * enemies_alive as f32,
                    _ => attack += def.value,
                }
            }
        }
    }
    (attack * reinforcement_mult).round() as i32
}

/// Fire monsters' active abilities (currently OnCombatStart party damage).
fn resolve_abilities(
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

fn apply_trap_damage(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
    trap_mult: f32,
) {
    if !macroquad_toolkit::rng::chance(TRAP_TRIGGER_CHANCE) {
        return;
    }

    // Get trap name for log
    let trap_name = state.floors[floor_idx].rooms[room_idx]
        .upgrade
        .as_ref()
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "Trap".into());

    // Find an alive adventurer to damage
    // We scope the mutable borrow of party so we can log afterwards
    let log_msg = {
        let party = &mut state.adventurer_parties[party_idx];
        if let Some(victim) = party.members.iter_mut().find(|a| a.alive) {
            let damage = (10.0 * trap_mult) as i32;
            victim.hp -= damage;

            if victim.hp <= 0 {
                victim.hp = 0;
                victim.alive = false;
                party.casualties += 1;

                let mana_gain = victim.level * 10;
                Some((true, victim.name.clone(), mana_gain, damage))
            } else {
                Some((false, victim.name.clone(), 0, damage))
            }
        } else {
            None
        }
    };

    let floor_num = state.floors[floor_idx].number;
    let room_pos = state.floors[floor_idx].rooms[room_idx].position;

    if let Some((killed, victim_name, mana_gain, damage)) = log_msg {
        if killed {
            state.mana = (state.mana + mana_gain).min(state.max_mana);
            state.total_deaths += 1;
            state.add_log(LogEntry::combat(format!(
                "{} killed {} by {}! +{} mana",
                trap_name, victim_name, trap_name, mana_gain
            )));
            state.push_effect(floor_num, room_pos, "Trapped!", EffectKind::AdventurerDown);
        } else {
            state.add_log(LogEntry::combat(format!(
                "{} dealt {} damage to {}",
                trap_name, damage, victim_name
            )));
            state.push_effect(
                floor_num,
                room_pos,
                format!("-{}", damage),
                EffectKind::Damage,
            );
        }
    }
}

/// Grant loot/souls for monsters slain this tick and narrate the kills.
fn reward_monster_kills(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
    kills: &[(String, bool)],
) {
    if kills.is_empty() {
        return;
    }

    let floor_num = state.floors[floor_idx].number;
    let room_pos = state.floors[floor_idx].rooms[room_idx].position;
    let treasure_mult = state.floors[floor_idx].rooms[room_idx].treasure_multiplier();

    for (monster_name, is_boss) in kills {
        let base_gold = if *is_boss { 50 } else { 20 };
        let gold_reward = (base_gold as f32 * treasure_mult) as i32;
        let soul_reward = if *is_boss { 1 } else { 0 };

        state.adventurer_parties[party_idx].loot += gold_reward;
        if soul_reward > 0 {
            state.souls += soul_reward;
        }

        state.add_log(LogEntry::combat(format!(
            "{} defeated on floor {}, room {}! +{} gold{}",
            monster_name,
            floor_num,
            room_pos,
            gold_reward,
            if soul_reward > 0 {
                format!(", +{} soul", soul_reward)
            } else {
                String::new()
            }
        )));
        state.push_effect(
            floor_num,
            room_pos,
            format!("{} down", monster_name),
            EffectKind::MonsterDown,
        );
    }

    // Victory quote
    let victory_quotes = get_victory_quotes();
    if macroquad_toolkit::rng::chance(0.2) && !victory_quotes.is_empty() {
        let quote = &victory_quotes[macroquad_toolkit::rng::gen_range(0, victory_quotes.len())];
        if let Some(adv) = state.adventurer_parties[party_idx]
            .members
            .iter()
            .find(|a| a.alive)
        {
            state.add_log(LogEntry::adventure(format!(
                "{} says: \"{}\"",
                adv.name, quote
            )));
        }
    }
}

/// Grant mana/XP for adventurers slain this tick and narrate the deaths.
fn reward_adventurer_kills(
    state: &mut GameState,
    _party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
    kills: &[(String, i32)],
) {
    if kills.is_empty() {
        return;
    }

    let floor_num = state.floors[floor_idx].number;

    for (victim_name, victim_level) in kills {
        let mana_gain = victim_level * 10;
        state.mana = (state.mana + mana_gain).min(state.max_mana);
        state.total_deaths += 1;

        // Award XP to all surviving monsters in the room
        let room = &mut state.floors[floor_idx].rooms[room_idx];
        let room_pos = room.position;
        let evolution_mult = room.evolution_multiplier();
        let base_xp = victim_level * 5;
        let xp_gain = (base_xp as f32 * evolution_mult) as i32;

        for monster in room.monsters.iter_mut().filter(|m| m.alive) {
            monster.experience += xp_gain;
        }

        state.add_log(LogEntry::combat(format!(
            "{} has fallen on floor {}! +{} mana, +{} XP to monsters",
            victim_name, floor_num, mana_gain, xp_gain
        )));
        state.push_effect(floor_num, room_pos, "Slain!", EffectKind::AdventurerDown);
    }
}

/// Flag the party as retreating after heavy losses or a full wipe.
fn check_retreat(state: &mut GameState, party_idx: usize) {
    let party = &mut state.adventurer_parties[party_idx];
    if party.retreating {
        return;
    }
    let no_survivors = party.members.iter().all(|a| !a.alive);
    if no_survivors {
        party.retreating = true;
        state.add_log(LogEntry::adventure("The entire party has been wiped out!"));
    } else if party.casualties >= RETREAT_THRESHOLD {
        party.retreating = true;
        state.add_log(LogEntry::adventure(
            "Party is retreating due to heavy casualties!",
        ));
    }
}
