# Dungeon Core Expansion Plan

## 1. Make Progression Visible

Expand the existing Evolution tab and inspector so players can see monster XP, ready evolutions, locked requirements, traits, tier, and species paths.

Primary files:
- `src/ui/side_drawer.rs`
- `src/ui/upgrade_panel.rs`
- `src/simulation/monsters.rs`

## 2. Finish Room Upgrade Depth

Expose all trap, treasure, reinforcement, and evolution upgrades in the room inspector. Add clear effect previews, affordability states, and enough selection space for the full upgrade catalog.

Primary files:
- `src/ui/upgrade_panel.rs`
- `src/simulation/upgrades.rs`
- `src/simulation/combat.rs`
- `assets/upgrades.json`

## 3. Use Room State Properly

Wire up existing `Room.explored` and `Room.loot`: mark rooms explored as parties advance, accumulate loot in rooms, let treasure upgrades modify room rewards, and show room state in the board/inspector.

Primary files:
- `src/game_state.rs`
- `src/simulation/adventure.rs`
- `src/simulation/combat.rs`
- `src/ui/dungeon_view.rs`
- `src/ui/upgrade_panel.rs`

## 4. Connect Adventurer Equipment

Use the existing `assets/equipment.json` data by assigning gear by adventurer level/class, applying stat bonuses, and showing gear in party/combat UI.

Primary files:
- `src/data/equipment.rs`
- `src/simulation/adventure.rs`
- `src/simulation/combat.rs`
- `src/ui/upgrade_panel.rs`

## 5. Deepen Combat Without Replacing It

Keep the current fast tick-based combat loop, but make the odds depend more clearly on attack, defense, HP, equipment, room upgrades, monster traits, and conditions. Add status effects through the existing `conditions` field, starting with Poison Dart.

Primary files:
- `src/simulation/combat.rs`
- `src/game_state.rs`
- `assets/traits.json`
- `assets/upgrades.json`

## 6. Improve Control/UI Usability

Implement actual pause in the speed control, scroll/filter the event log, group monsters by species/tier, and make the dungeon board handle larger floor counts cleanly.

Primary files:
- `src/main.rs`
- `src/ui/shell.rs`
- `src/ui/event_toast.rs`
- `src/ui/side_drawer.rs`
- `src/ui/dungeon_view.rs`

## 7. Clean Up As We Go

Remove stale placeholders, update outdated feature docs, and add small tests around combat/evolution/equipment calculations once those functions are isolated enough to test cleanly.

Primary files:
- `src/data/rooms.rs`
- `FEATURE_REQUIREMENTS.md`
- `FEATURE_ANALYSIS.md`
