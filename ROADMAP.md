# Dungeon Core — Review & Slice-to-Full-Game Roadmap (2026-07-05)

## Where the game stands

The vertical slice is real and complete: build rooms/floors → summon monsters →
open the dungeon → parties raid on a clock → earn mana/gold/souls → buy room
upgrades and evolve defenders. Tutorial, threat warnings, save/load, species
unlocks, and the clarity UI from the playtest feedback are all in. Content
volume today:

| Axis | Current |
|---|---|
| Species (player races) | 4 (Goblinoid, Slime, Undead, Draconic) |
| Monsters | 15, tiers 1–3 |
| Traits | 4 defined (**currently dead — see bugs**) |
| Elements | 11 labels on monsters, **no mechanics** |
| Traps | 3, implemented as room-upgrade multipliers |
| Room upgrades | 11 across 4 types |
| Adventurer classes | 4, no races, 20 names |
| Equipment | 15 items (stats feed adventurer spawns) |
| Threat system | 4 warning tiers, no consequence |

## Bugs / dead systems found in this review

These block the expansion work and should be fixed first.

1. **All monster traits are silently disabled.** `assets/traits.json` is
   truncated (ends at line 210 without closing `]` `}`) and also still contains
   ~9 legacy entries in an old schema (numeric ids, `effects` maps).
   `get_all_traits()` in `src/data/traits.rs` swallows the parse error and
   returns an empty list, so regeneration, Fire Breath, Swarm Tactics, and
   Undead Resilience never fire. Fix the JSON, delete the legacy entries, and
   make the loader panic (or at least log loudly) on parse failure — silent
   fallback hid this completely.
2. **Inverted probabilities.** `macroquad_toolkit::rng::chance(p)` returns true
   with probability *p*, but two call sites use `if chance(p) { return; }`:
   - `src/simulation/combat.rs` `apply_trap_damage`: comment says "20% chance
     per tick to trigger" — traps actually fire 80% of ticks.
   - `src/simulation/adventure.rs` `spawn_party`: `spawn_chance = 0.3` actually
     spawns a party 70% of the time.
   Either invert the conditions or rename the constants to match reality after
   rebalancing.
3. **Combat ignores stats.** Resolution is a flat 30%/30% instant-kill coin
   flip modified by reinforcement/defense multipliers. Monster attack/defense,
   scaled_stats, adventurer attack/defense, and equipment bonuses are computed
   but never used in the exchange (HP only matters vs traps and Fire Breath).
   The `element` field is pure flavor.
4. **Threat tier 4 promises an army that never comes.** Warnings escalate but
   there is no raid event, no consequence, no `Maintenance`-status trigger.

## Design keystone: stat-driven combat first

Every requested expansion axis — more monsters, elements, races, traps — only
creates *meaningful* variety if combat reads the numbers. With coin-flip
resolution, a Tier-3 Lich and a Tier-1 Goblin die at the same rate. So the
roadmap starts there.

Proposed model (keeps the tick cadence, ~10 lines of real math):
- Each combat tick, one attacker per side exchanges hits:
  `damage = max(1, attack − defense/2) × element_mult × room_mults`.
- HP pools deplete over ticks; deaths happen when HP ≤ 0 (no instant kills).
- Keeps existing hooks: trap damage, OnCombatStart abilities, retreat at
  casualty threshold. Reinforcement boosts stats instead of skewing a coin.

---

## Phase 0 — Foundation fixes (DONE 2026-07-05)

- [x] Repair `traits.json`, purge legacy schema entries, hard-fail on parse error.
- [x] Fix the two inverted `chance()` call sites (traps now 20%/tick, parties
      spawn at the configured 30%).
- [x] Replace coin-flip combat with the stat-driven exchange above.
- [x] Add `#[test]`s that parse every embedded JSON asset plus cross-reference
      checks (species/trait/evolution integrity) — `src/data/mod.rs`.
- [x] Delete dead paths: `src/data/rooms.rs` placeholder and the unreachable
      `DungeonStatus::Maintenance` variant.

Follow-up to watch in playtests: with far fewer instant adventurer deaths, the
dungeon's mana income (10/death) is leaner — if building stalls, raise base
mana regen or add mana-on-damage.

**Exit criteria:** tier/stat differences visibly change fight outcomes; traits
fire and show in the log; asset JSON is test-covered.

## Phase 1 — Elements as a real system

Elements already exist as data (`Fire, Water, Earth, Air, Nature, Spirit,
Body, Arcane, Bone, Blood, Death…`). Consolidate to ~8 and give them rules.

- [ ] `assets/elements.json`: element list + effectiveness matrix (1.5× strong,
      0.67× weak, 1.0 neutral). Loader + `element_multiplier(atk, def)` in data
      layer.
- [ ] Monsters keep their element; give each adventurer class a damage element
      (Warrior=Body, Mage=Arcane, Cleric=Spirit, Rogue=Air) and later per-hero
      variance.
- [ ] Room element attunement: a new upgrade type that tints a room (Fire
      shrine etc.) boosting matching monsters — makes placement a puzzle.
- [ ] UI: element badge on monster chips, effectiveness feedback in combat log
      and floating effects ("2× !").

**Exit criteria:** placing a Water monster against a Fire-heavy party is a
visibly better choice.

## Phase 2 — Monster & species expansion

Target: 8 species, 4 tiers, 45–60 monsters, branching evolutions.

- [ ] New species (one shipped at a time, each with a tier-1→4 tree):
      Beast (starter candidate), Demon, Elemental, Construct, Plant/Fungal,
      Insect swarm. Keep 3 starters total (Goblinoid, Slime, Beast).
- [ ] Branching evolutions: at tier 2→3 offer two paths (e.g. Skeleton →
      Skeleton Knight *or* Bone Mage) — `evolution_trees.json` already supports
      multiple paths from one monster; the UI needs a chooser.
- [ ] Trait pool: grow 4 → ~20. The engine is already data-driven
      (effect_type/scaling_type); add effect types incrementally:
      `LifeStealPercent`, `FirstStrike`, `ArmorShred`, `ManaOnKill`,
      `TauntAggro`, `SplashDamage`. Every monster gets 1–2 traits.
- [ ] Species identity rules: Undead don't heal but respawn cheap; Slimes split
      on death (spawn a tier-1 at half HP); Demons cost souls, not mana; ties
      species choice to playstyle instead of just different art.
- [ ] Boss uniques: one named boss form per species (Goblin King, Slime
      Empress…), summonable only in Boss rooms.

**Exit criteria:** two playthroughs with different species feel mechanically
different, not just cosmetically.

## Phase 3 — Traps as first-class content

Today traps are 3 entries in the single room-upgrade slot. Promote them.

- [ ] Separate trap slot per room (room keeps its upgrade slot; traps get 1–2
      dedicated slots) — `Room { traps: Vec<Trap> }`.
- [ ] `assets/traps.json` with categories:
      **Damage** (spike, boulder), **DoT/status** (poison, burn — needs a
      simple status-effect list on Adventurer, the `conditions: Vec<String>`
      field already exists and is unused), **Control** (snare: party loses a
      tick; alarm: buffs monsters in next room), **Economy** (mana siphon, gold
      steal).
- [ ] Elemental traps that combo with Phase 1 (flame vent in a Fire-attuned
      room, etc.).
- [ ] Counterplay: Rogues get a disarm chance per trap; disarmed traps need
      gold to re-arm — makes party composition matter to the defender.
- [ ] Trap tiers gated by souls, giving the soul economy a real sink.

**Exit criteria:** a trap-focused dungeon (few monsters, deadly corridors) is a
viable strategy.

## Phase 4 — Adventurer depth (the opposing content)

- [ ] Adventurer races: Human (balanced), Elf (high attack, low HP, trap
      detection), Dwarf (high defense, trap-immune-ish), Halfling (steals extra
      gold, retreats early). Race + class grid multiplies variety from 4 to 16
      loadouts using the equipment/stat plumbing that already exists.
- [ ] New classes: Ranger (targets back-line monster), Paladin (element:
      Spirit, resists Death/Blood), Alchemist (throws elemental flasks).
- [ ] Named recurring heroes: survivors persist, level up between raids, and
      come back with grudges — dungeon-keeper genre's best trick, and
      `Adventurer { experience, level }` fields are already there.
- [ ] Party AI: greed vs caution (loot targets vs retreat thresholds per
      race/class), so watching a raid tells a story.

## Phase 5 — Threat, raids, endgame

- [ ] Make threat real: each tier raises party level/size ranges; tier 3 sends
      elite hero squads; tier 4 sends the army — a scripted multi-party siege
      targeting the core room. Core has HP; if it falls, run ends.
- [ ] Surviving the tier-4 siege = prestige event: reset with a permanent core
      perk (new species discount, +mana regen …) — the long-term loop.
- [ ] Dungeon-level upgrades (spend souls): core skills like Fear Aura (raise
      retreat threshold), Illusion Floor (fake treasure), Deep Roots (mana).
- [ ] Reputation as a dial, not just a doom meter: low threat = many weak
      greedy parties (farm gold), high threat = few elite parties (farm souls).
      Player steers via open/close and kill/release decisions.

## Phase 6 — Presentation & retention polish

- [ ] Monster/trap icons (image_prompts.json is already prepped for asset
      generation; the UI currently uses emoji).
- [ ] Bestiary/codex screen: discovered monsters, evolution trees, element
      wheel — doubles as the missing reference documentation.
- [ ] Raid summary panel after each party leaves (kills, loot lost/gained,
      XP earned) instead of log-only.
- [ ] Sound hooks (macroquad audio): combat hits, trap triggers, evolution.

---

## Suggested order & sizing

| Phase | Size | Why this order |
|---|---|---|
| 0 Foundation | S | Everything else is meaningless on coin-flip combat; traits are currently OFF |
| 1 Elements | M | Multiplier that makes every later monster/trap more interesting |
| 2 Monsters/species | L (incremental) | Biggest content ask; ship one species at a time |
| 3 Traps | M | Needs elements + status conditions from earlier phases |
| 4 Adventurers | M | Opposition variety; reuses element/race tech |
| 5 Threat/endgame | M | Gives the loop a destination; needs tougher adventurers first |
| 6 Polish | ongoing | Slot alongside each phase |

All new content stays in `assets/*.json` (the established pattern) so balance
passes never require recompiles of logic — plus the Phase-0 JSON test keeps the
data honest.
