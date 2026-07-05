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

## Phase 1 — Elements as a real system (DONE 2026-07-05)

- [x] `assets/elements.json`: 9 elements — Fire/Water/Nature/Earth/Air in a
      balanced 2-strong/2-weak pentagram, Spirit>Death>Arcane triangle, Body
      neutral. One-directional `strong_against` lists (weakness derived as the
      inverse, so a matchup can never be strong both ways).
      `element_multiplier(atk, def)` in `src/data/elements.rs` (1.5×/0.67×/1×).
- [x] Consolidated monster elements (Bone, Blood → Death); adventurer classes
      got damage elements (Warrior=Body, Rogue=Air, Mage=Arcane, Cleric=Spirit
      — Cleric now counters undead, undead counter Mages).
- [x] Room element attunement: new `attunement` upgrade type (Fire Shrine,
      Spring Altar, Standing Stones, Ossuary) — matching-element monsters in
      the room get ×1.3 attack and defense.
- [x] Feedback: "Strong hit!" / "Resisted" floating effects when the party has
      an elemental edge (or lacks one); party damage numbers get "!" when
      monsters hit a weak class. Element name shows on monster drawer rows.
- [x] Matrix integrity tests: no self-strength, no mutual strength, all
      references valid.

Deferred to later phases: per-hero element variance, elemental trap combos
(Phase 3).

**Exit criteria met:** matchups shift damage 1.5×/0.67× both directions, and
attuned rooms make same-element placement strictly better.

## Phase 2 — Monster & species expansion (first drop DONE 2026-07-05)

Target: 8 species, 4 tiers, 45–60 monsters, branching evolutions.

Shipped in the first drop (15 → 27 monsters, 4 → 6 species):
- [x] **Beast** starter species (3 starters now: Goblinoid, Slime, Beast):
      Wolf/Giant Rat/Wild Boar → Dire Wolf/Alpha Boar → Werewolf/Behemoth.
- [x] **Demon** species with the souls identity rule: summons cost souls on
      top of mana (`MonsterTemplate.souls_cost`). Imp → Hellhound *or*
      Shadow Fiend → Pit Fiend.
- [x] Branching evolutions live (data + `get_evolutions_for_monster`):
      Imp and Skeleton each branch two ways; all qualifying branches unlock
      for summoning (chooser-free: player picks what to summon).
      Undead extended: Skeleton → Vampire | Bone Mage, both → Lich.
- [x] Trait pool 4 → 14 with new wired effect types: LifeStealPercent,
      ArmorPierce, Taunt (soaks focus-fire), AttackBonus PerEnemy (frenzy),
      ManaOnKill (mana leech), SplitOnDeath (slimes), single-target actives
      (Shadow Bolt), plus data variants (pack_hunter, stone_skin,
      regenerate_major). Every monster now has 1–2 traits; existing roster
      retrofitted (Vampire steals life, Orc taunts, big slimes split).
- [x] Slime identity rule: tier-2+ slimes split into a tier-1 at half HP on
      death (guarded by a no-tier-1-splitters test).

Shipped in the second drop (27 → 47 monsters, 6 → 8 species):
- [x] **Elemental** species (attunement-native: Ember/Frost/Gale Wisp →
      Flame/Tide/Storm Elemental → Primal Elemental) and **Construct** species
      (armored, no self-healing: Animated Armor/Clay Golem → Blade
      Sentinel/Stone Golem → Iron Colossus). New `tempest` AoE active.
- [x] Tier-4 **boss uniques**, one per species, reachable by evolving the
      species' top tier: Goblin King, Slime Empress, Grave King, Elder Dragon,
      Fenrir, Archfiend (8 souls), Elemental Overlord, Ancient Guardian.
      `boss_only` templates can only be summoned in Boss rooms and skip the
      boss-room 2× mana surcharge (they price it in).
- [x] Max mana now grows +50 per new floor so tier-4 summons stay affordable
      (was flat 200 forever — even the old Dragon was unaffordable in deep
      boss rooms).
- [x] Species selector modal scrolls (8 species overflowed the fixed list).

Still to do in later drops:
- [ ] Undead identity rule (no healing, cheap respawn) — needs a respawn-cost
      mechanic first; all respawns are currently free.
- [ ] Optional species beyond 8 (Plant/Fungal, Insect swarm) if content
      breadth is wanted after playtesting.

**Exit criteria:** two playthroughs with different species feel mechanically
different, not just cosmetically.

## Playtest feedback round (DONE 2026-07-05)

- [x] Room upgrades cost **mana** (not gold) — gold is what the dungeon
      earns, mana is what it spends. `mana_cost` in upgrades.json.
- [x] Rooms hold **one upgrade per type** (trap + treasure + … can coexist);
      old single-slot saves migrate on load.
- [x] Traps/treasure placeable from a new left-drawer **TRAPS** tab, same
      select-then-click-a-room flow as monsters. Inspector panel still works
      per-room with per-upgrade remove buttons.
- [x] Monster drawer lists only unlocked monsters.
- [x] Fixed: parties stopped spawning after day 1 (`next_party_spawn` was
      hour-of-day and broke at the midnight wrap; now absolute hours).
- [x] Mana regen bonus per adventurer inside raised 0.1 → 0.5/hour and no
      longer lost to integer truncation.

## Phase 3 — Traps as first-class content (DONE 2026-07-05)

- [x] Trap catalog 3 → 9 with behavior kinds (`effect_kind` on upgrades):
      **Damage** (Spike Trap, Boulder Trap, Abyssal Maw), **DoT** (Poison
      Dart, Flame Vent — `Condition {kind, ticks, power}` list on Adventurer
      ticks each combat round), **Control** (Frost Snare freezes the party's
      attacks for N ticks; Alarm Gong makes every defender hit the flagged
      party 25% harder), **Economy** (Mana Siphon feeds the core, Gold
      Snatcher claws loot back mid-raid).
- [x] Elemental traps combo with Phase 1: trap damage runs the element
      matchup against the victim's class, and a matching room attunement
      multiplies trap power (Flame Vent in a Fire Shrine room).
- [x] Counterplay: a party with a living Rogue has a 30% chance to disarm a
      triggering trap for the rest of the raid; traps auto re-arm between
      raids for 25% of their mana cost (skipped if the core can't afford it).
- [x] Soul-gated top tier (Boulder 1S, Mana Siphon/Gold Snatcher 1S,
      Abyssal Maw 3S) keeps the soul sink.

**Exit criteria met:** a corridor of damage + DoT + snare traps kills without
monsters; Rogue parties counter it, alarms and attunements deepen the build
space.

## Phase 4 — Adventurer depth (DONE 2026-07-06)

### 4a. Persistent, viewable heroes (requested)
- [x] `GameState.known_adventurers: Vec<HeroRecord>` — every hero who enters
      is logged with name, class, race, level/XP, delves, kills, gold stolen,
      and status (Alive / Inside / Dead on floor N, day D). Persisted.
- [x] Returning survivors: ~half of each party's slots draw from living
      veterans, who bank XP on escape and level up (to L10), returning
      stronger via the level-scaled loadout.
- [x] Kill attribution credits the hero who lands the killing blow.
- [x] HEROES drawer tab: scrollable roster sorted raiders → veterans →
      graves, with delves, kills, and cause of death.

### 4b. Variety and behavior
- [x] Races (Human/Elf/Dwarf/Halfling) with stat modifiers in JSON.
- [x] New classes: Ranger (Nature), Paladin (Spirit), Alchemist (Fire).
- [x] Party AI: retreat threshold varies by composition — Halflings bail
      early, Dwarves and Paladins hold; siege fanatics never break.

Deferred hooks: grudges/rivalries/bounties (data is in place for them).

## Phase 5 — Threat, raids, endgame (DONE 2026-07-06)
- [x] Threat scales party level/size (low threat = larger weak bands, high
      threat = small elite squads).
- [x] Core HP; tier-4 **siege**: one overwhelming elite party marches past
      looting to assault the core. The core's wards strike back, so a
      wounded siege can still be repelled at the heart. Core destroyed →
      game-over overlay with run stats and restart.
- [x] Repelling a siege → **Prestige**: +core HP, +max mana, threat resets,
      the long game continues (siege elites scale with prestige count).
- [x] Soul-bought permanent **core powers** (Build tab): Deep Roots
      (+mana regen), Bulwark Core (+250 core HP), Dread Aura (invaders break
      one casualty sooner).
- [x] Reputation dial via threat-scaled party shape.

Deferred: Illusion Floor and other flavor core skills (the framework is in
place — just more `CORE_POWERS` entries).

## Phase 6 — Presentation & retention polish (partial DONE 2026-07-06)
- [x] Bestiary/**Codex** screen (press C): element effectiveness wheel +
      every discovered monster with tier, species, element, stats, traits.
      Doubles as the reference documentation.
- [ ] Raid summary panel after each party leaves — the event log narrates
      this today; a dedicated card is a nice-to-have.
- [ ] Monster/trap icons — needs generated art (image_prompts.json is
      prepped); the UI uses emoji, which read fine.
- [ ] Sound hooks — needs audio assets.

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
