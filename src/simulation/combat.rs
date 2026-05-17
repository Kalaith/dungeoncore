use crate::data::adventurers::get_victory_quotes;
use crate::data::constants::RETREAT_THRESHOLD;
use crate::game_state::{GameState, LogEntry};

/// Resolve combat between adventurers and monsters in a room
pub fn resolve_combat(state: &mut GameState, party_idx: usize, floor_idx: usize, room_idx: usize) {
    // Check if there are alive monsters
    let has_alive_monsters = state.floors[floor_idx].rooms[room_idx]
        .monsters
        .iter()
        .any(|m| m.alive);

    if !has_alive_monsters {
        return;
    }

    // Get room upgrade multipliers
    let trap_mult = state.floors[floor_idx].rooms[room_idx].trap_multiplier();
    let reinforcement_mult = state.floors[floor_idx].rooms[room_idx].reinforcement_multiplier();

    // Trap damage: if room has trap, deal extra damage to adventurers
    if trap_mult > 1.0 {
        apply_trap_damage(state, party_idx, floor_idx, room_idx, trap_mult);
    }

    // Adjust combat probabilities based on reinforcement
    // Reinforcement makes monsters harder to kill
    let mut adventurer_death_chance = 0.3 * reinforcement_mult;
    let mut monster_death_chance = 0.3 / reinforcement_mult;

    // Apply Monster Traits
    // We need to check traits of ALIVE monsters in the room
    // Note: We are iterating again, but doing it inside the combat loop for the specific room is okay

    // Phase 1: Ability Activation
    // We need to borrow room and party disjointly to modify both
    // and we need to collect logs to add them to state later (since state is split borrowed)
    let mut combat_logs: Vec<LogEntry> = Vec::new();

    {
        // Disjoint borrows
        let floor = &mut state.floors[floor_idx];
        let room = &mut floor.rooms[room_idx];
        let party = &mut state.adventurer_parties[party_idx];
        let monsters = &mut room.monsters;

        for monster in monsters.iter_mut().filter(|m| m.alive) {
            for trait_data in &mut monster.active_traits {
                // Check cooldown
                if trait_data.cooldown_timer <= 0 {
                    if let Some(trait_def) = crate::data::traits::get_trait(&trait_data.id) {
                        // Generic Active Ability Logic
                        if trait_def.trait_type == "Active"
                            && trait_def.applies_to == "OnCombatStart"
                        {
                            // Check effect type
                            if trait_def.effect_type == "DamageFlat"
                                && trait_def.target_type == "EnemyParty"
                            {
                                let damage = trait_def.value as i32;
                                trait_data.cooldown_timer = trait_def.cooldown;

                                // Deal damage to ALL adventurers in party
                                let mut total_hits = 0;
                                for adv in &mut party.members {
                                    if adv.alive {
                                        adv.hp -= damage;
                                        total_hits += 1;
                                        if adv.hp <= 0 {
                                            adv.alive = false;
                                            party.casualties += 1;
                                        }
                                    }
                                }

                                if total_hits > 0 {
                                    combat_logs.push(LogEntry::combat(format!(
                                        "{} uses {}! Dealt {} damage to {} adventurers.",
                                        monster.type_name, trait_def.name, damage, total_hits
                                    )));
                                }
                            }
                        }
                    }
                } else {
                    trait_data.cooldown_timer -= 1;
                }
            }
        }
    }

    // Apply deferred logs
    for log in combat_logs {
        state.add_log(log);
    }

    // Phase 2: Standard Combat Attacks
    // Recalculate chances (traits might have changed stats, or adventurers might have died)

    // We iterate again to collect passive bonuses.
    // This is read-only on monsters, so we can hold state?
    // Actually we mutate state later if someone dies (adventurer_dies / monster_dies).
    // So let's calculate the stats based on the Room immutable borrow?
    // But `resolve_combat` calls `monster_dies` which takes `&mut GameState`.
    // So we must finish all borrows before calling those.

    let mut swarm_bonus = 0.0;
    let mut total_defense_mult = 1.0;
    let mut monster_count = 0;

    // Scope for read-only monster analysis
    {
        let room = &state.floors[floor_idx].rooms[room_idx];
        for monster in room.monsters.iter().filter(|m| m.alive) {
            monster_count += 1;
            for trait_data in &monster.active_traits {
                if let Some(trait_def) = crate::data::traits::get_trait(&trait_data.id) {
                    if trait_def.trait_type == "Passive" {
                        if trait_def.effect_type == "DamageReductionMult"
                            && trait_def.applies_to == "OnDefense"
                        {
                            total_defense_mult *= 1.0 - trait_def.value;
                        }
                        if trait_def.effect_type == "AttackBonus"
                            && trait_def.applies_to == "OnAttack"
                            && trait_def.target_type == "Self"
                        {
                            // Scaling Logic
                            if trait_def.scaling_type == "PerAlly" {
                                swarm_bonus += 0.01 * trait_def.value;
                            }
                        }
                    }
                }
            }
        }
    }

    // Apply swarm bonus
    if monster_count > 1 {
        adventurer_death_chance += swarm_bonus * (monster_count - 1) as f32;
    }

    monster_death_chance *= total_defense_mult;

    let result = macroquad_toolkit::rng::rand();

    if result < adventurer_death_chance {
        // Adventurer takes fatal hit
        adventurer_dies(state, party_idx, floor_idx, room_idx);
    } else if result < adventurer_death_chance + monster_death_chance {
        // Monster dies
        monster_dies(state, party_idx, floor_idx, room_idx);
    }
    // else: stalemate, continue combat next tick
}

fn apply_trap_damage(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
    trap_mult: f32,
) {
    // 20% chance per tick for trap to trigger
    if macroquad_toolkit::rng::chance(0.2) {
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

    if let Some((killed, victim_name, mana_gain, damage)) = log_msg {
        if killed {
            state.mana = (state.mana + mana_gain).min(state.max_mana);
            state.add_log(LogEntry::combat(format!(
                "{} killed {} by {}! +{} mana",
                trap_name, victim_name, trap_name, mana_gain
            )));
        } else {
            state.add_log(LogEntry::combat(format!(
                "{} dealt {} damage to {}",
                trap_name, damage, victim_name
            )));
        }
    }
}

fn adventurer_dies(state: &mut GameState, party_idx: usize, floor_idx: usize, room_idx: usize) {
    let floor_num = state.floors[floor_idx].number;

    // Find alive adventurer and kill them
    let death_info = {
        let party = &mut state.adventurer_parties[party_idx];
        if let Some(victim_idx) = party.members.iter().position(|a| a.alive) {
            let victim = &mut party.members[victim_idx];
            let victim_name = victim.name.clone();
            let victim_level = victim.level;
            victim.alive = false;
            party.casualties += 1;

            let retreating = party.casualties >= RETREAT_THRESHOLD;
            if retreating {
                party.retreating = true;
            }

            Some((victim_name, victim_level, retreating))
        } else {
            None
        }
    };

    if let Some((victim_name, victim_level, retreating)) = death_info {
        // Award mana for adventurer death
        let mana_gain = victim_level * 10;
        state.mana = (state.mana + mana_gain).min(state.max_mana);

        // Award XP to all surviving monsters in the room
        let room = &mut state.floors[floor_idx].rooms[room_idx];
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

        // Check retreat condition
        if retreating {
            state.add_log(LogEntry::adventure(
                "Party is retreating due to heavy casualties!",
            ));
        }
    }
}

fn monster_dies(state: &mut GameState, party_idx: usize, floor_idx: usize, room_idx: usize) {
    // Find alive monster
    let monster_idx = match state.floors[floor_idx].rooms[room_idx]
        .monsters
        .iter()
        .position(|m| m.alive)
    {
        Some(idx) => idx,
        None => return,
    };

    let monster = &mut state.floors[floor_idx].rooms[room_idx].monsters[monster_idx];
    let monster_name = monster.type_name.clone();
    let is_boss = monster.is_boss;
    monster.alive = false;

    // Award rewards with treasure multiplier
    let treasure_mult = state.floors[floor_idx].rooms[room_idx].treasure_multiplier();
    let base_gold = if is_boss { 50 } else { 20 };
    let gold_reward = (base_gold as f32 * treasure_mult) as i32;
    let soul_reward = if is_boss { 1 } else { 0 };

    state.adventurer_parties[party_idx].loot += gold_reward;
    if soul_reward > 0 {
        state.souls += soul_reward;
    }

    let floor_num = state.floors[floor_idx].number;
    let room_pos = state.floors[floor_idx].rooms[room_idx].position;

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
