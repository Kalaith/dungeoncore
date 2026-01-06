# Rust Coding Standards for Dungeon Core

**Engine**: Macroquad + macroquad-toolkit  
**Language**: Rust  
**Genre**: Dungeon Management RPG

This document defines the coding standards for the Dungeon Core project. Its goal is to maintain long-term sanity for a dungeon management game with complex adventurer behaviors, monster spawning, and upgrade systems. The dungeon may be treacherous, but the code should be orderly.

These standards prioritize:  
- Readability over cleverness  
- Data-driven design over hardcoded values  
- Clean state management for dungeon operations  
- Modular services for game logic  
- A clear mental model for game phases and transitions  

## 1. Core Philosophy

### 1.1 Write for Maintainability
This is a dungeon management game with complex adventurer AI, monster behaviors, and procedural room generation. Code should be easy to debug and extend.  
- Prefer obvious, straightforward code  
- Avoid hidden state or side effects  
- If a junior Rust developer can understand the flow, you are doing it right.

### 1.2 Consistency Beats Preference
If a pattern already exists in the codebase, follow it even if you dislike it. A consistent codebase is more valuable than a perfect one.

### 1.3 Data-Driven Design
All game constants, balance values, adventurer data, monster stats, room layouts, traits, and upgrades should be defined in JSON files under `assets/`. Load this data at startup using Serde for easy balancing and iteration without recompiling code. Avoid hardcoding values in Rust code; reference loaded data structures instead.

### 1.4 No Unused Code
- Remove unused variables, fields, and functions immediately
- Never suppress unused warnings with `_` prefixes on struct fields
- If a field is unused, delete it - don't mark it as unused
- Parameter prefixes with `_` are acceptable only when required by trait signatures

## 2. Project Structure Rules

### 2.1 Module Responsibilities
Each module/subdirectory owns a single conceptual domain:

**Root Level:**
- `main.rs` – Entry point, game loop, phase transitions, and high-level coordination
- `game_state.rs` – Core game state structures
- `persistence.rs` – Save/load functionality

**Subdirectories:**
- `data/` – Data structures and JSON loading
  - Type definitions for adventurers, monsters, rooms, traits, upgrades
  - Constants and configuration structures

- `simulation/` – Game logic services (stateless where possible)
  - `adventure.rs` – Adventurer behavior and pathfinding
  - `combat.rs` – Combat resolution and damage calculations
  - `monsters.rs` – Monster spawning and AI
  - `rooms.rs` – Room generation and layout
  - `time.rs` – Time progression and scheduling
  - `upgrades.rs` – Upgrade application and effects

- `ui/` – User interface components
  - `controls.rs` – Input handling and controls
  - `dungeon_view.rs` – Dungeon visualization
  - `game_log.rs` – Event logging and display
  - `monster_selector.rs` – Monster selection interface
  - `resource_panel.rs` – Resource display
  - `species_selector.rs` – Species selection
  - `upgrade_panel.rs` – Upgrade management UI
  - Uses macroquad-toolkit for buttons and interactions

**Cross-Domain Rules:**
- ❌ UI must never mutate game state directly
- ❌ Simulation services should be stateless - receive state, return results
- ❌ Data module has no knowledge of simulation or UI
- ✅ All domains can read from `data/` types
- ✅ State mutations happen only in main.rs via clearly defined actions

### 2.2 File Size Guideline
- Target: 200–400 lines per file
- Soft limit: 600 lines
- Hard limit: 800 lines (main.rs excepted for game loop complexity)
- If a file grows beyond this, split by responsibility.

### 2.3 Folder Structure

```
dungeon_core/
├── Cargo.toml              # Project manifest
├── CODE_STANDARDS.md       # This file
├── index.html              # WebGL host page
├── publish.ps1             # Build and deploy script
├── assets/                 # Game data
│   ├── adventurers.json    # Adventurer definitions
│   ├── constants.json      # Balance values
│   ├── image_prompts.json  # Image generation prompts
│   ├── monsters.json       # Monster definitions
│   ├── traits.json         # Trait definitions
│   └── upgrades.json       # Upgrade definitions
├── src/
│   ├── main.rs             # Entry point, game loop, screen rendering
│   ├── game_state.rs       # Core game state
│   ├── persistence.rs      # Save/load functionality
│   ├── data/               # Data types and loading
│   │   ├── mod.rs          # Re-exports all data types
│   │   ├── adventurers.rs  # Adventurer types
│   │   ├── constants.rs    # Game constants
│   │   ├── monsters.rs     # Monster types
│   │   ├── rooms.rs        # Room types
│   │   ├── traits.rs       # Trait types
│   │   └── upgrades.rs     # Upgrade types
│   ├── simulation/         # Game logic services
│   │   ├── mod.rs          # Re-exports
│   │   ├── adventure.rs    # Adventurer simulation
│   │   ├── combat.rs       # Combat simulation
│   │   ├── monsters.rs     # Monster simulation
│   │   ├── rooms.rs        # Room simulation
│   │   ├── time.rs         # Time management
│   │   └── upgrades.rs     # Upgrade simulation
│   └── ui/                 # UI components
│       ├── mod.rs
│       ├── controls.rs
│       ├── dungeon_view.rs
│       ├── game_log.rs
│       ├── monster_selector.rs
│       ├── resource_panel.rs
│       ├── species_selector.rs
│       └── upgrade_panel.rs
└── .gitignore
```

## 3. Naming Conventions

### 3.1 General Rules
- Types: PascalCase  
- Functions & variables: snake_case  
- Constants: SCREAMING_SNAKE_CASE  
- Modules: snake_case  

Names should describe what the thing is, not how it works.

Good examples:  
```rust
Adventurer  
MonsterType  
calculate_damage  
spawn_monster  
generate_room  
```

Bad examples:  
```rust
do_thing  
temp2  
handle_stuff  
m  // use monster instead
```

### 3.2 Boolean Naming
Booleans should read like facts:  
```rust
is_hostile  
can_enter_room  
has_upgrade_unlocked  
should_spawn_boss  
```  
Avoid `flag`, `value`, or `state` in names.

### 3.3 Service Naming
Simulation services follow a naming pattern:
- `*Service` for stateless helpers (if any)
- Direct module names for simulation logic (`Adventure`, `Combat`, etc.)

## 4. Functions & Methods

### 4.1 Function Size
- Target: 20–50 lines  
- Absolute max: 100 lines  
- If a function needs scrolling, it probably needs refactoring.

### 4.2 Single Responsibility
Each function should answer one question or perform one action.

Bad:  
```rust
// Calculates damage, updates health, drops loot, checks achievements  
fn resolve_combat() { ... }  
```

Good:  
```rust
fn calculate_damage() -> u32 { ... }  
fn update_health() { ... }  
fn drop_loot() -> Option<Item> { ... }  
fn check_achievements() { ... }  
```

### 4.3 Argument Count
- Prefer ≤ 3 parameters  
- If more are needed, use a struct or reference to state  
- Services should take `&GameState` or `&ConstantsData` rather than many individual fields

### 4.4 Return Types
- Use `Option<T>` for potentially missing values  
- Use custom result structs for complex outcomes (e.g., `CombatResult`)
- Avoid returning multiple values via tuple; create a named struct instead

## 5. Data & State Management

### 5.1 Game State Ownership
- `GameState` owns the current dungeon state  
- Mutation happens through methods in main.rs or game_state.rs  
- Simulation modules return results; they don't mutate state directly  

### 5.2 Prefer Plain Data
Use structs with clear fields. Avoid overly clever enums with embedded logic unless they model a real state machine.  

Game data should be:  
- Serializable (Serde-friendly for save/load)  
- Easy to debug and inspect  
- Immutable after loading from JSON  

### 5.3 Data-Driven Design
- All game balance, adventurer stats, monster data, room layouts, traits, and upgrades in JSON under `assets/`
- Load data at application startup; data is embedded at compile time
- Use structs that mirror JSON structure for type safety
- Never hardcode magic numbers; reference loaded config data

### 5.4 Enums for Game Phases
Use enums to model distinct game states:
```rust
pub enum GamePhase {
    Loading,
    MainMenu,
    DungeonSetup,
    AdventurerArrival,
    Simulation,
    Combat,
    UpgradePhase,
    GameOver,
    Victory,
}
```

## 6. Error Handling

### 6.1 Prefer Option Over Panics
- `panic!` is acceptable only for truly unrecoverable states  
- Missing monsters or items should return `None`, not panic  
- Use:  
  - `Option<T>` for potentially missing values  
  - `Result<T, E>` for fallible I/O operations (save/load)  
  - Graceful degradation for missing data  

### 6.2 Logging Over Silent Failures
Use `eprintln!` for error conditions that should be visible during development but shouldn't crash the game.

## 7. UI Code (Macroquad-Toolkit)

### 7.1 UI Is Dumb
UI code:  
- Reads game state  
- Returns actions/intents  
- It should never contain game logic.  

Bad:  
```rust
// Calculating damage inside a button handler  
fn on_attack_button() { calculate_damage(); }  
```

Good:  
```rust
// Button returns UiAction::Attack
// main.rs handles the action and calculations
fn draw_attack_button() -> Option<UiAction> { ... }
```

### 7.2 Action Pattern
UI components return `Option<UiAction>` to signal user intent:
```rust
pub enum UiAction {
    StartGame,
    SelectMonster(MonsterType),
    UpgradeTrait(TraitId),
    PauseSimulation,
    // etc.
}
```

### 7.3 Component Organization
- Each UI module handles a specific panel or view
- Components are pure functions: `fn draw_panel(state: &State) -> Option<UiAction>`

### 7.4 Macroquad-Toolkit Usage

This project uses `macroquad-toolkit` for common UI patterns. Import via `use ui::*;` which re-exports all toolkit modules.

**Available Modules:** (same as before)
- `ui::button()` – Standard clickable button (fires on release)
- `ui::button_on_press()` – Button that fires on mouse down
- `ui::button_styled()` – Button with custom styling
- `ui::panel()` – Draws a panel with optional title
- `ui::progress_bar()` – Progress indicator
- `ui::colors::dark::*` – Standard dark theme colors
- `ui::input::*` – Mouse/keyboard input helpers

**Button Click Semantics:** (same)
```rust
// Standard button - fires on mouse RELEASE (safer, allows cancel)
if button(x, y, w, h, "Attack") {
    return UiAction::Attack;
}

// Press button - fires on mouse DOWN (instant feedback)
if button_on_press(x, y, w, h, "Emergency", &style) {
    // Immediate action
}
```

**Color Palette:** (same)
```rust
use macroquad_toolkit::colors::dark;

clear_background(dark::BACKGROUND);  // Standard background
draw_rectangle(x, y, w, h, dark::PANEL);  // Panel color
draw_text("Hello", x, y, 20.0, dark::TEXT);  // Text color
// Also: dark::ACCENT, dark::POSITIVE, dark::WARNING, dark::NEGATIVE
```

**Input Helpers:** (same)
```rust
use ui::input::*;

if is_hovered(x, y, w, h) { /* Mouse over area */ }
if was_clicked(x, y, w, h) { /* Left click released on area */ }
if was_pressed(x, y, w, h) { /* Left click pressed on area */ }
```

## 8. Deployment & Web Standards

### 8.1 Required Files
Every game must have these files for deployment:
- `publish.ps1` – Build and deploy script
- `index.html` – WebGL host page

### 8.2 Build Targets
The game must build for:
- **Windows**: `cargo build --release`
- **Web/WASM**: `cargo build --release --target wasm32-unknown-unknown`

### 8.3 WebGL Requirements
The `index.html` must:
- Load `mq_js_bundle.js` (Miniquad loader)
- Call `load("dungeon_core.wasm")`
- Include canvas with `id="glcanvas"`
- Use `image-rendering: pixelated` for pixel art

## 9. Game Phases & Transitions

### 9.1 Clear Phase Model
The game uses explicit phases:
1. **MainMenu** → Start game
2. **DungeonSetup** → Configure dungeon
3. **AdventurerArrival** → Adventurers enter
4. **Simulation** → Real-time dungeon simulation
5. **Combat** → Combat resolution
6. **UpgradePhase** → Spend resources on upgrades
7. **GameOver/Victory** → End conditions

### 9.2 Transition Clarity
Phase transitions should be explicit and obvious in code:
```rust
// Clear: one function, one transition
fn start_simulation(&mut self) {
    self.game_state.game_phase = GamePhase::Simulation;
}
```

## 10. Comments & Documentation

### 10.1 Comment Why, Not What
Code already explains what it does. Comments should explain why it exists.

Good:  
```rust
// Elite monsters have increased spawn rates during boss waves  
fn spawn_monster() { ... }  
```

Bad:  
```rust
// Spawn a monster  
fn spawn_monster() { ... }  
```

### 10.2 Module-Level Docs
Each module should contain a short `//!` comment explaining its purpose:
```rust
//! Adventurer behavior simulation and pathfinding.
```

## 11. Formatting & Tooling

### 11.1 rustfmt
- Always use `cargo fmt`  
- Never fight the formatter  

### 11.2 Clippy
- Run `cargo clippy` regularly  
- Fix warnings unless intentionally ignored  
- Document any `#[allow]` with a comment

### 11.3 Variable Shadowing
- Avoid variable shadowing (hiding)
- Do not declare a new variable with the same name as an existing one in the same scope

### 11.4 Unused Code
- Remove unused variables immediately
- Remove unused struct fields immediately  
- Never use `_` prefix on struct fields to suppress warnings
- `_` prefix on function parameters is acceptable when required by API

## 12. Testing Guidelines

### 12.1 What to Test
Focus tests on:  
- Damage calculations  
- Monster spawn logic  
- Adventurer pathfinding  
- State transitions  
- JSON data loading  
- UI and rendering generally do not need unit tests.

### 12.2 Test Style
- Tests should read like rules  
- Avoid complex setups  
- If a test is hard to write, the code is probably too tangled.

## 13. Final Rule

If a piece of code feels fragile, confusing, or brittle, it probably is. Refactor early. Leave the dungeon code more fortified than you found it.