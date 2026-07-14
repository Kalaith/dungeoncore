//! Player-driven economy sinks. Currently: channelling surplus gold into mana.
//!
//! Gold used to pile up dead once species were unlocked — its only sinks were
//! one-time. "Channel the Hoard" turns that stagnant gold into an ongoing sink
//! *and* a mana safety-valve: a deliberately inefficient conversion so slain-
//! invader mana stays the primary income, but hoarded gold can still be poured
//! back into building when mana runs short.

use crate::game_state::{GameState, LogEntry};

/// Gold spent per Channel the Hoard transaction.
pub const GOLD_CHANNEL_COST: i32 = 100;
/// Mana granted per Channel the Hoard transaction (5:1 — intentionally lossy).
pub const GOLD_CHANNEL_MANA: i32 = 20;

/// Can the player channel gold into mana right now?
pub fn can_channel_gold(state: &GameState) -> bool {
    state.gold >= GOLD_CHANNEL_COST && state.mana < state.max_mana
}

/// Channel a fixed chunk of gold into mana (capped at max mana). Returns `Ok`
/// on success or a short reason it could not proceed.
pub fn channel_gold_to_mana(state: &mut GameState) -> Result<(), String> {
    if state.mana >= state.max_mana {
        return Err("Mana is already full.".into());
    }
    if state.gold < GOLD_CHANNEL_COST {
        return Err(format!("Not enough gold! Need {}.", GOLD_CHANNEL_COST));
    }
    state.gold -= GOLD_CHANNEL_COST;
    let before = state.mana;
    state.mana = (state.mana + GOLD_CHANNEL_MANA).min(state.max_mana);
    let gained = state.mana - before;
    state.add_log(LogEntry::system(format!(
        "Channelled {} gold into {} mana.",
        GOLD_CHANNEL_COST, gained
    )));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_converts_gold_to_mana() {
        let mut s = GameState::new();
        s.gold = 250;
        s.mana = 0;
        s.max_mana = 500;
        channel_gold_to_mana(&mut s).unwrap();
        assert_eq!(s.gold, 150);
        assert_eq!(s.mana, GOLD_CHANNEL_MANA);
    }

    #[test]
    fn channel_never_overfills_mana() {
        let mut s = GameState::new();
        s.gold = 500;
        s.max_mana = 100;
        s.mana = 90;
        channel_gold_to_mana(&mut s).unwrap();
        // Capped at max even though 20 was offered.
        assert_eq!(s.mana, 100);
        // Gold is still spent (the transaction is fixed-cost).
        assert_eq!(s.gold, 400);
    }

    #[test]
    fn channel_blocked_when_poor_or_full() {
        let mut s = GameState::new();
        s.gold = 50;
        s.mana = 0;
        s.max_mana = 100;
        assert!(channel_gold_to_mana(&mut s).is_err());
        assert!(!can_channel_gold(&s));
        s.gold = 500;
        s.mana = 100;
        assert!(channel_gold_to_mana(&mut s).is_err());
        assert!(!can_channel_gold(&s));
    }
}
