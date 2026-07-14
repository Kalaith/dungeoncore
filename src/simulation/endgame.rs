use crate::game_state::{
    Adventurer, AdventurerParty, GameState, HeroRecord, HeroStatus, LogEntry, Stats,
};

/// A soul-bought permanent core power.
pub struct CorePower {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
}

/// The catalog of core powers the dungeon heart can awaken.
pub const CORE_POWERS: [CorePower; 3] = [
    CorePower {
        id: "deep_roots",
        name: "Deep Roots",
        description: "The core drinks deeper: +1 mana regen forever.",
        cost: 3,
    },
    CorePower {
        id: "bulwark_core",
        name: "Bulwark Core",
        description: "Reinforce the heart: +250 core HP.",
        cost: 4,
    },
    CorePower {
        id: "dread_aura",
        name: "Dread Aura",
        description: "Invaders lose their nerve one casualty sooner.",
        cost: 3,
    },
];

/// Purchase a permanent core power with souls.
pub fn buy_core_power(state: &mut GameState, id: &str) -> Result<(), String> {
    if state.has_core_power(id) {
        return Err("That core power is already awakened.".into());
    }
    let power = CORE_POWERS
        .iter()
        .find(|p| p.id == id)
        .ok_or("Unknown core power")?;
    if state.souls < power.cost {
        return Err(format!("Not enough souls! Need {}.", power.cost));
    }
    state.souls -= power.cost;
    state.core_powers.push(id.to_string());

    // Apply immediate, permanent effects.
    match id {
        "bulwark_core" => {
            state.core_max_hp += 250;
            state.core_hp += 250;
        }
        "deep_roots" => { /* applied each hour in regen */ }
        "dread_aura" => { /* read at retreat check */ }
        _ => {}
    }

    state.add_log(LogEntry::system(format!(
        "Core power awakened: {}!",
        power.name
    )));
    Ok(())
}

/// If the realm's fury has peaked (threat tier 4) and no siege is under way,
/// muster the army: one overwhelming elite party that marches on the core.
pub fn maybe_launch_siege(state: &mut GameState) {
    if state.siege_active || state.threat_tier() < 4 {
        return;
    }
    // Wait for a lull between ordinary raids so the siege is a clean event.
    if !state.adventurer_parties.is_empty() {
        return;
    }

    let deepest = state
        .floors
        .iter()
        .find(|f| f.is_deepest)
        .map(|f| f.number)
        .unwrap_or(1);

    let names = crate::data::adventurers::get_adventurer_names();
    let classes = crate::data::adventurers::get_adventurer_classes();
    let races = crate::data::adventurers::get_race_names();

    // Elites scale with how deep the dungeon runs and how many sieges repelled.
    let level = 5 + state.total_floors + state.prestige * 2;
    let size = 5usize;
    let mut members = Vec::with_capacity(size);
    for _ in 0..size {
        let class = macroquad_toolkit::rng::choose(&classes).unwrap();
        let name = macroquad_toolkit::rng::choose(&names).unwrap().clone();
        let race = macroquad_toolkit::rng::choose(&races)
            .cloned()
            .unwrap_or_else(|| "Human".to_string());
        let id = macroquad_toolkit::rng::random_u64();

        // Register siege champions in the ledger like anyone else.
        state.known_adventurers.push(HeroRecord {
            id,
            name: name.clone(),
            class_name: class.name.clone(),
            race: race.clone(),
            level,
            experience: 0,
            delves: 1,
            kills: 0,
            gold_stolen: 0,
            status: HeroStatus::Inside,
            death_floor: 0,
            death_day: 0,
        });

        let hp = class.hp + (level - 1) * 12;
        members.push(Adventurer {
            id,
            name,
            class_name: class.name.clone(),
            race,
            level,
            hp,
            max_hp: hp,
            alive: true,
            experience: 0,
            gold: 0,
            equipment: crate::data::equipment::recommended_loadout(&class.name, level),
            conditions: Vec::new(),
            scaled_stats: Stats {
                hp,
                attack: class.attack + (level - 1) * 3,
                defense: class.defense + (level - 1) * 2,
            },
        });
    }

    state.adventurer_parties.push(AdventurerParty {
        id: macroquad_toolkit::rng::random_u64(),
        members,
        current_floor: 1,
        current_room: 0,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: state.hour,
        target_floor: deepest,
        snared_ticks: 0,
        alarmed: false,
        sieging: true,
        prev_room: 0,
        move_anim: 0.0,
    });
    state.siege_active = true;

    state.status = crate::game_state::DungeonStatus::Open;
    state.add_log(LogEntry::system(
        "THE SIEGE BEGINS! The realm's army storms the dungeon, bound for your core.",
    ));
}

/// A siege party has reached the core. Trade blows: the party batters the
/// heart while the core's wards cut them down. Returns true if the party is
/// spent (repelled).
pub fn assault_core(state: &mut GameState, party_idx: usize) -> bool {
    let party_attack: i32 = state.adventurer_parties[party_idx]
        .members
        .iter()
        .filter(|a| a.alive)
        .map(|a| a.scaled_stats.attack)
        .sum();

    // The core bites back, harder the deeper and more prestigious it is.
    let core_attack = 18 + state.total_floors * 8 + state.prestige * 12;

    state.core_hp -= party_attack;
    if state.core_hp <= 0 {
        state.core_hp = 0;
        state.game_over = true;
        state.add_log(LogEntry::system(
            "THE CORE HAS FALLEN. Your dungeon is undone.",
        ));
        return true;
    }

    state.add_log(LogEntry::combat(format!(
        "The core takes {} damage! ({}/{})",
        party_attack, state.core_hp, state.core_max_hp
    )));

    // Core wards strike the assailants.
    let mut deaths: Vec<(u64, String)> = Vec::new();
    {
        let party = &mut state.adventurer_parties[party_idx];
        let alive_idxs: Vec<usize> = party
            .members
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .map(|(i, _)| i)
            .collect();
        // Spread the core's wrath across up to two defenders' worth of hits.
        for &idx in alive_idxs.iter().take(2) {
            let member = &mut party.members[idx];
            member.hp -= core_attack;
            if member.hp <= 0 {
                member.hp = 0;
                member.alive = false;
                party.casualties += 1;
                deaths.push((member.id, member.name.clone()));
            }
        }
    }

    let floor = state.adventurer_parties[party_idx].current_floor;
    for (id, name) in deaths {
        state.record_hero_death(id, floor);
        state.total_deaths += 1;
        state.add_log(LogEntry::combat(format!(
            "The core's wards strike down {}!",
            name
        )));
    }

    let repelled = state.adventurer_parties[party_idx]
        .members
        .iter()
        .all(|a| !a.alive);
    if repelled {
        state.adventurer_parties[party_idx].retreating = true;
    }
    repelled
}

/// The siege has been broken. Grant prestige, a permanent boon, and reset the
/// realm's fury so the long game can continue.
pub fn repel_siege(state: &mut GameState) {
    state.siege_active = false;
    state.prestige += 1;

    // Permanent boon: a sturdier, mana-richer heart.
    state.core_max_hp += 150;
    state.core_hp = state.core_max_hp;
    state.max_mana += 100;

    // The realm licks its wounds; the threat meter resets.
    state.total_deaths = 0;
    state.threat_warned = 0;

    state.add_log(LogEntry::system(format!(
        "THE SIEGE IS BROKEN! Prestige {} attained. The core swells with power (+150 HP, +100 max mana) and the realm retreats to lick its wounds.",
        state.prestige
    )));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn siege_musters_at_peak_threat() {
        let mut s = GameState::new();
        s.total_deaths = 100; // tier 4
        assert_eq!(s.threat_tier(), 4);
        maybe_launch_siege(&mut s);
        assert!(s.siege_active);
        assert_eq!(s.adventurer_parties.len(), 1);
        assert!(s.adventurer_parties[0].sieging);
        // A second call must not stack another siege.
        maybe_launch_siege(&mut s);
        assert_eq!(s.adventurer_parties.len(), 1);
    }

    #[test]
    fn repel_grants_prestige_and_resets_threat() {
        let mut s = GameState::new();
        s.total_deaths = 100;
        s.threat_warned = 4;
        s.siege_active = true;
        let hp_before = s.core_max_hp;
        repel_siege(&mut s);
        assert_eq!(s.prestige, 1);
        assert!(!s.siege_active);
        assert_eq!(s.total_deaths, 0);
        assert!(s.core_max_hp > hp_before);
    }

    #[test]
    fn buying_bulwark_raises_core_hp() {
        let mut s = GameState::new();
        s.souls = 10;
        let hp_before = s.core_max_hp;
        buy_core_power(&mut s, "bulwark_core").unwrap();
        assert!(s.core_max_hp > hp_before);
        assert!(s.has_core_power("bulwark_core"));
        // Can't buy twice.
        assert!(buy_core_power(&mut s, "bulwark_core").is_err());
    }
}
