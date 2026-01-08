# Dungeon Core 🏰

**Dungeon Core** is a dungeon management simulator where you play as the malevolent core of a dangerous dungeon. Build rooms, place monsters, and defend your core from waves of greedy adventurers.

## 🌟 Overview

Migrated from a React/PHP web application to **Rust** using the [Macroquad](https://macroquad.rs/) engine, Dungeon Core combines strategic base-building with tower defense mechanics.

## ✨ Features

- **Dungeon Building**: Construct specialized rooms like Lairs, Traps, and Treasure Rooms.
- **Monster Management**: Unlock different species (Goblins, Undead, Elementals) and place them to defend your floors.
- **Progression**:
  - Earn **Mana** to place monsters.
  - Earn **Gold** from slain adventurers to unlock new species.
  - Collect **Souls** to upgrade your dungeon's capabilities.
- **Deep Floor System**: Dig deeper to access more powerful mana veins and stronger enemies.
- **Adventurer AI**: Parties of Warriors, Thieves, Mages, and Clerics invade your dungeon, seeking loot and glory.

## 🎮 Controls

### General
| Key / Action | Function |
| :--- | :--- |
| **Mouse Click** | Select rooms, place monsters, interact with UI |
| **Space** | Toggle Game Speed (1x -> 2x -> 4x) |
| **D** | Toggle Dungeon Status (Open/Closed) |
| **R** | Respawn Monsters (if available) |
| **A** | Add New Room |
| **E** | Process Evolutions |

### UI Panels
- **Resource Panel (Left)**: View Mana, Gold, Souls, and available Monsters.
- **Controls (Left)**: Quick actions for dungeon management.
- **Game Log (Bottom)**: Real-time updates on combat and events.
- **Upgrade Panel (Right)**: Appears when a room is selected to purchase upgrades.

## 🛠️ Build & Run

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

### Development
```bash
cargo run
```

### Release Build
```bash
cargo build --release
```

### WebGL Build
```bash
# Using the provided PowerShell script
.\publish.ps1

# Or manually
cargo build --target wasm32-unknown-unknown --release
```

## 📂 Project Structure

- `src/main.rs`: Entry point and main game loop.
- `src/game_state.rs`: Core state management (resources, grid, monsters).
- `src/simulation/`: Game logic (combat, time, AI, spawning).
- `src/ui/`: UI rendering and interaction logic.
- `src/data/`: Static data loading (monsters, rooms, upgrades).
- `src/persistence/`: Save/Load functionality.
- `assets/`: JSON configuration files.

## 📜 License

MIT
