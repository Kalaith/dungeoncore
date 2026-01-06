# Dungeon Core: Feature Requirements

Complete list of features from the original React/TypeScript + PHP web application that need to be implemented in the Rust version.

---

## Core Game State

### Resources
- [x] Mana (current/max)
- [x] Mana regeneration with deep core bonus
- [x] Gold
- [x] Souls

### Time System
- [x] Day/Hour tracking
- [x] Time advancement with speed multiplier (1x/2x/4x)
- [x] Hour-based event triggers

### Dungeon Status
- [x] Open/Closed/Closing/Maintenance states
- [x] Status toggle logic

---

## Dungeon Structure

### Floors
- [x] Multiple floors with floor numbers
- [x] "Deepest" floor tracking
- [x] Floor creation with core room transfer
- [x] Deep core mana bonus (scales with depth)

### Rooms
- [x] Room types: Entrance, Normal, Boss, Core
- [x] Room positions within floors
- [x] Room cost calculation (scales with total rooms)
- [x] Max rooms per floor limit (5 + entrance)
- [x] Room upgrades (trap/treasure/reinforcement/evolution)
- [ ] Room special properties (`spawnCostReduction` in `RoomType` for Monster Lairs)
- [ ] Room explored state tracking (used for adventurer progression)
- [ ] Room loot accumulation

---

## Monster System

### Monster Templates
- [x] Base stats (HP, Attack, Defense)
- [x] Mana cost
- [x] Species classification
- [x] Tier levels (1, 2, 3)
- [x] Emoji/icon display
- [x] Monster descriptions
- [ ] Monster special abilities
- [ ] Boss abilities (bossAbility from MonsterType)
- [ ] Monster traits system

### Monster Placement
- [x] Place monster in room
- [x] Mana cost calculation with floor scaling
- [x] Boss room cost multiplier
- [x] Species unlock requirement check
- [ ] Spawn cost reduction from room type

### Monster Progression
- [x] Species unlocking with gold cost
- [ ] Monster experience tracking (logic in `monsterActions.ts`)
- [ ] Tier unlocks via experience
- [ ] Evolution trees (MonsterList.evolution_trees)
- [ ] Starting monster per species

### Monster Traits
From `MonsterTrait` interface:
- [x] Trait description
- [x] Trait type classification
- [x] Target type (who it affects)
- [x] Applies to (when it triggers)
- [x] Mana cost for trait activation
- [x] Cooldown turns

---

## Adventurer System

### Adventurer Classes
- [x] Warrior/Rogue/Mage/Cleric
- [x] Base stats per class
- [x] Level scaling for stats
- [ ] Class colors for display (e.g., Warrior #d4af37)
- [ ] Class special abilities


### Adventurer Generation
- [x] Random name assignment
- [x] Random class assignment
- [x] Level range based on floor depth
- [x] Party size (2-4 members)

### Adventurer Parties
- [x] Party spawning with chance check
- [x] Current floor/room tracking
- [x] Retreat logic (casualties threshold)
- [x] Target floor (exploration depth goal)
- [x] Loot accumulation
- [x] Entry time tracking
- [ ] Retreat chance when underleveled (RETREAT_CHANCE_UNDERLEVELED)

### Equipment System
From `Equipment` and `EquipmentData`:
- [ ] Weapon equipment
- [ ] Armor equipment
- [ ] Accessory equipment
- [ ] Equipment stat bonuses
- [ ] Available equipment pools (weapons[], armor[], accessories[])

### Conditions/Status Effects
- [ ] Condition system (conditions array on Adventurer)
- [ ] Status effect application
- [ ] Status effect clearing

---

## Combat System

### Basic Combat
- [x] Random combat resolution
- [x] Adventurer death (mana reward)
- [x] Monster death (gold/soul reward)
- [x] Retreat trigger on casualties
- [x] **Active Traits**: Monsters have traits (e.g. `regenerate_minor`) that do things.
    - [x] Define traits in `traits.json`.
    - [x] Logic for trait triggers (Hourly, OnCombatStart, OnAttack, OnDefense).
- [x] **Trait Effects**:
    - [x] `regenerate_minor`: Heals 5% HP per hour.
    - [x] `undead_resilience`: 20% damage reduction.
    - [x] `fire_breath`: AoE active ability.
    - [x] `swarm_tactics`: Attack bonus per ally.

### Room Upgrade Effects
- [x] Trap damage to adventurers
- [x] Reinforcement defense boost
- [x] Treasure gold multiplier
- [ ] Evolution experience multiplier

### Advanced Combat
- [ ] Attack/Defense calculations using scaled stats
- [ ] Damage formulas
- [ ] Critical hits
- [ ] Monster abilities activation
- [ ] Class-specific abilities
- [ ] Equipment stat application

---

## UI Components

### Resource Panel
- [x] Mana bar with current/max
- [x] Mana regen rate display
- [x] Gold display
- [x] Souls display

### Dungeon View
- [x] Floor layout with rooms
- [x] Room type visualization (colors/icons)
- [x] Monster count indicators
- [x] Adventurer party indicators
- [x] Room selection
- [ ] Room tooltips with detailed info
- [ ] Scrollable for large dungeons

### Monster Selector
- [x] List available monsters
- [x] Show locked/unlocked status
- [x] Monster cost display
- [x] Selection state
- [ ] Monster tier grouping
- [ ] Monster trait display

### Upgrade Panel
- [x] Available upgrades list
- [x] Apply upgrade
- [x] Remove upgrade
- [x] Cost display (gold/souls)
- [ ] Upgrade effect preview

### Game Controls
- [x] Speed toggle
- [x] Dungeon status toggle
- [x] Respawn monsters button
- [x] Add room button
- [x] Reset game button
- [ ] Pause/Resume

### Game Log
- [x] Log entry display
- [x] Color-coded by type
- [x] Entry limit
- [ ] Scrollable log
- [ ] Log filtering

### Species Selection Modal
- [x] Species list with descriptions
- [x] Free first species selection
- [x] Species monster preview
- [x] Unlock cost display
- [x] Purchase confirmation

### Time Display
- [x] Day counter
- [x] Hour display
- [x] Speed indicator

---

## Floor Scaling

### Per-Floor Scaling
- [x] Mana cost multiplier
- [x] Monster stat boost percentage
- [x] Adventurer level range

### Deep Floor Scaling
- [x] Multiplier increase per floor beyond defined list
- [x] Monster boost increase
- [x] Adventurer level increase

---

## Persistence

### Save System
- [x] JSON save file
- [x] Auto-save
- [x] Load on startup
- [ ] Multiple save slots
- [ ] Save/Load UI

---

## Data Management

### JSON Data Files
- [x] constants.json - Game constants and floor scaling
- [x] monsters.json - Monster templates and species
- [x] adventurers.json - Classes, names, quotes
- [x] upgrades.json - Room upgrade templates
- [x] image_prompts.json - Asset generation prompts

### Missing Data Files
- [ ] room_types.json - Room type definitions with special properties
- [ ] equipment.json - Weapons, armor, accessories
- [ ] traits.json - Monster trait definitions
- [ ] evolution_trees.json - Monster evolution paths

---

## Visual/Audio (Future)

- [ ] Sprite textures from image_prompts.json
- [ ] Monster sprites
- [ ] Room backgrounds
- [ ] Adventurer sprites
- [ ] UI icons
- [ ] Sound effects
- [ ] Background music
- [ ] Combat animations
- [ ] Particle effects

---

## Summary

### Implemented
- Core game loop and state management
- Dungeon floor/room structure
- Basic monster placement and spawning
- Adventurer party system
- Basic combat resolution
- Room upgrades with effects
- All data loaded from JSON
- Save/Load persistence
- Full UI with panels

### Not Yet Implemented
- Monster traits and abilities
- Monster evolution trees
- Equipment effects
- Detailed combat formulas
- Species selection modal
- Scrolling UI panels
- Visual assets
- Audio
