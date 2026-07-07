# Dungeon Core — Design Review

*Senior design / systems review, 2026-07-07. Based on source (`src/`), data (`assets/`), `ROADMAP.md`, the founder playtest notes (`feedback.md`), and the two January feature audits.*

> **Headline:** This is no longer a prototype. The systems layer is genuinely deep and, as of the July work, mechanically honest — combat reads the stats, traits fire, elements matter. The project's problem has completely inverted since the January audits. It is no longer "missing features." It is **a rich simulation the player cannot see or feel.** Every recommendation below points at that one gap.

---

# 1. Project Overview

## Project Name
**Dungeon Core**

## Genre
Dungeon-management / reverse tower-defense with light incremental-strategy pacing. Closest living relative: *Legend of Keepers* crossed with a Dungeon Keeper economy sim. Single-screen, real-time-with-speed-control, mouse-driven.

## Core Concept
You *are* the dungeon. You build rooms and floors branching from your core, summon monsters into them (paid in mana, sometimes souls), lace rooms with traps and elemental attunements, then **open the dungeon** and watch adventurer parties raid inward on a day/hour clock. Slain adventurers pay mana; slain monsters and stolen loot recovered pay gold/souls. You spend that income to unlock species, evolve monsters, deepen the dungeon, and buy permanent core powers. Killing adventurers raises the realm's **threat**; at peak threat the realm launches a **siege** on your core. Repel it and you **prestige** — a stronger core, higher stakes, the loop continues.

- **What the player does:** front-loaded spatial/economic puzzle (compose a lethal path within a mana budget), then a spectator-with-levers raid phase.
- **The fantasy:** being the malevolent, growing heart of a lair — Dungeon Keeper's "you're the villain" fantasy, scaled down to a management dashboard.
- **What makes it different:** the *elemental attunement + trait + trap combo* build space is deeper than most games in this niche attempt, and it's fully data-driven (all balance lives in `assets/*.json`).
- **Target player:** management/automation-sim and incremental-strategy players; the "set up a machine and watch it work" audience (Factorio-adjacent, *Legend of Keepers*, *Dungeon of the Endless* fans). Not an action or twitch audience.

## Current State
**Feature-complete vertical slice, systemically over-built, presentation-under-built.**

Six roadmap phases are shipped: stat-driven combat, a real elemental matchup system, 47 monsters across 8 species and 4 tiers, 14 functioning traits, 9 behavior-typed traps, a persistent hero roster with races/classes/party-AI, a core-HP → siege → prestige endgame, and a Codex screen. The January audits (which flagged dead traits, coin-flip combat, empty equipment, "75% complete") are **stale** — nearly everything they listed as missing or fake is now real and test-covered.

Why not "polished": the founder's own most recent feedback (`feedback.md`) is dominated by a single theme — *"what's happening isn't clear."* The simulation now computes far more than the screen communicates. That's a content-expansion-phase project wearing a presentation-prototype's UI.

---

# 2. Core Gameplay Analysis

## Main Gameplay Loop

> **Build path → Summon & attune defenders → Open dungeon → Watch raid resolve → Bank mana/gold/souls → Unlock / evolve / deepen / buy core powers → Threat climbs → Siege → Repel → Prestige → Repeat**

Two nested loops:
- **Minute-to-minute (raid loop):** open dungeon, a party enters, combat ticks resolve room-by-room, party dies or retreats, income lands.
- **Session loop (growth loop):** spend income on breadth (species/monsters/floors/core powers), push threat, survive the siege, prestige.

## Evaluation

| Question | Verdict |
|---|---|
| Is the loop clear? | **The growth loop, yes. The raid loop, no.** The player builds with clear costs and feedback, but the payoff moment — the raid — is narrated by floating text and a scrolling log, not shown. This is the central weakness. |
| Is it satisfying? | Building is satisfying (the founder confirms this is the best part). The raid resolution is *told, not felt* — the emotional beat of "my trap corridor just shredded a party" is buried in log lines. |
| Meaningful decisions? | **Yes, and this is the hidden strength.** Element matchup + room attunement + trait synergy + trap kind is a real optimization space. But the player can't *perceive* the decisions paying off, so they may never realize the depth exists. |
| Enough variety? | Systemically, abundant (8 species play differently by design). Experientially, it may feel same-y because the raid is visually identical whether a Fire Shrine + Flame Vent combo or a lone Goblin is doing the killing. |
| Long-term motivation? | Prestige gives an infinite scaler, but there's no shaped destination or milestone arc beyond "numbers go up." Threat/siege is the only real tension source and it resets cleanly. |

**Core diagnosis:** the loop is *well-designed on paper and under-communicated in play.* The decisions are meaningful; the feedback that would make them *feel* meaningful is missing.

---

# 3. Existing Systems Review

## Combat System

**Purpose:** Resolve raids; convert dungeon design quality into outcomes and income.

**Current implementation:** Tick-based (`resolve_combat` in `src/simulation/combat.rs`). Each tick: conditions tick (poison/burn) → trap fires → OnCombatStart abilities → adventurers focus the front monster → monsters strike back. Damage = `max(1, attack − defense/2) × element_mult × room_mults`. HP pools deplete; deaths at 0 HP. Taunt, lifesteal, armor-pierce, mana-on-kill, split-on-death, per-ally/per-enemy attack scaling are all wired.

**Strengths:** This is the make-or-break system and it's now genuinely good. Tier and stat differences change outcomes (the January coin-flip is gone). Elements swing damage 1.5×/0.67× both directions. The trait hooks are real and varied. It's deterministic-ish and test-covered.

**Weaknesses:** **The combat is invisible.** All this computation surfaces as floating strings ("Strong hit!", "-14", "Snared!") and log spam. The player cannot watch HP bars trade, see who's targeting whom, or understand *why* a fight went the way it did. A player cannot tell a good build from a lucky one. Secondary: `combat.rs` is 896 lines — over the project's own 800-line hard limit (`CODE_STANDARDS.md`); the resolve/trap/ability responsibilities should split into siblings.

**Improvement ideas:** A **per-room combat visualization** — HP bars on monsters and party members, target lines or a "who hit whom" flash, damage numbers anchored to combatants — is the single highest-value change in the whole project. A **post-raid summary card** (already deferred in Phase 6) is the cheaper half of the same fix.
- **Impact: Game-changing** · **Cost: Medium** (viz) / **Small** (summary card)

---

## Dungeon Building & Spatial Design

**Purpose:** The primary agency; the "fun part" per the founder.

**Current implementation:** Rooms branch left-to-right from entrance to core; floors stack downward; max 5 rooms/floor; room cost scales with total rooms. Rooms hold monsters plus one upgrade per type (trap/treasure/reinforcement/attunement).

**Strengths:** Clean, legible, the confirmed highlight. The one-upgrade-per-type rule creates layered rooms without decision paralysis. Attunement rooms make same-element placement *strictly* better — a clear, learnable optimization.

**Weaknesses:** Layout is nearly linear — a single path from entrance to core. There's little *spatial* strategy (choke points, branching, forcing party splits, kill-order sequencing across rooms). The path is a queue, not a maze. With many floors, single-screen legibility is a real risk (`dungeon_view.rs` is 1188 lines and does a lot of layout work).

**Improvement ideas:** Give the *sequence* of rooms meaning — e.g. a Snare room *before* a damage room, an Alarm room that empowers everything *after* it. Some of this exists (alarm is party-wide) but isn't taught or spatially expressed. Consider optional branching paths so the player shapes *where* parties go.
- **Impact: High** · **Cost: Medium**

---

## Economy (Mana / Gold / Souls)

**Purpose:** Pace growth; force trade-offs.

**Current implementation:** **Mana** = the spend currency (regen over time; summon + build + upgrade costs; leech/siphon). **Gold** = earned from loot and monster kills; sink is species unlocks + evolution gold-cost. **Souls** = earned from boss kills and soul-traps; sink is soul-gated summons/traps + permanent core powers.

**Strengths:** Three currencies with distinct emotional roles (mana = operating budget, gold = expansion, souls = prestige/permanence) is a clean mental model. Moving upgrades to mana (from the playtest round) fixed a currency-confusion complaint.

**Weaknesses:** Risk of **gold glut** — its only sinks are one-time (species unlocks) or occasional (evolutions). Late game, gold likely piles up meaninglessly. Souls are the interesting scarce currency; gold may be the weak link. Mana regen tuning is delicate: the roadmap itself flags that stat-driven combat *reduced* death income, so building can stall.

**Improvement ideas:** Give gold an ongoing sink (per-raid upkeep? re-roll/retrain? a gold-priced fast-track?) or fold it into mana and make souls the sole "special" currency. Watch mana-starvation in playtests.
- **Impact: Medium** · **Cost: Small**

---

## Progression: Species / Monsters / Evolution

**Purpose:** Long-term content unlock and build identity.

**Current implementation:** 8 species with identity rules (Demon = souls-costed, Slime = splits, Construct = no self-heal, Elemental = attunement-native). 47 monsters, tiers 1–4 with per-species boss uniques. Branching evolutions are a **manual** choice (the EVOLVE tab), directly answering the founder's "I don't want auto-evolve" note.

**Strengths:** Big, differentiated, and the manual-evolution fix respects player agency. Species-as-playstyle is the right framing for replayability.

**Weaknesses:** This is where **over-building** shows most. 47 monsters is a lot of content for a game whose combat the player can't watch — the differences between, say, a Blade Sentinel and a Stone Golem are real in the math but imperceptible on screen. The Codex helps, but reading a stat table is not the same as *seeing* a monster matter.

**Improvement ideas:** Don't add more monsters until the combat is legible enough to make the *existing* 47 feel distinct. The content isn't the bottleneck; perception is.
- **Impact: High (as a "stop" signal)** · **Cost: n/a**

---

## Traits & Elements

**Purpose:** The synergy/optimization layer.

**Current implementation:** 14 traits with real effect kinds (lifesteal, taunt, armor-pierce, mana-on-kill, split, frenzy, AoE actives). 9 elements in a balanced rock-paper-scissors matrix; room attunement multiplies matching-element monsters ×1.3; traps run element matchups too.

**Strengths:** This is the design crown jewel — a genuinely deep, internally-consistent, test-covered synergy system. The one-directional strong-against matrix (weakness derived as inverse) is an elegant guarantee against contradictory matchups.

**Weaknesses:** Almost entirely **invisible and untaught**. A new player has no way to learn that Cleric counters Undead, that a Fire Shrine boosts a Flame Vent, or that Taunt soaks focus-fire — except by reading the Codex or the log after the fact. The depth exists; the *legibility of the depth* does not.

**Improvement ideas:** Surface matchups at decision time — element badges with strong/weak coloring when hovering a room during placement, a "this room's monsters counter Mages" hint. The Codex element wheel is a good reference but arrives too late in the funnel.
- **Impact: High** · **Cost: Medium**

---

## Threat, Siege & Prestige (Endgame)

**Purpose:** Tension, fail-state, and the meta-loop.

**Current implementation:** Threat tier (0–4) scales off cumulative adventurer deaths. At tier 4, one elite siege party marches past loot straight to the core; the core has HP and fights back. Core falls → game over. Repel → prestige (bigger core, more mana, threat resets, siege elites scale with prestige count). 3 soul-bought permanent core powers.

**Strengths:** Elegant tension design — *being good at your job* (killing adventurers) is exactly what raises the threat that endangers you. That's a real, thematically-coherent risk/reward dial. Prestige gives the loop a heartbeat and a reason to survive.

**Weaknesses:** The threat meter (per `feedback.md`, a long-standing complaint) needs to *feel* like mounting dread, not a silent percentage bar. The siege is a strong beat but arrives via a log line ("THE SIEGE BEGINS"). Prestige is an uncapped scaler with no shaped arc — fine for an idle game, thin as a designed climax. Only 3 core powers means the prestige reward tree is shallow.

**Improvement ideas:** Dramatize the siege (screen state change, music sting, a distinct visual). Add more core powers so prestige has a branching upgrade path (the framework already supports it — `CORE_POWERS` is just an array). Consider a visible "days until the realm marches" so threat becomes anticipation.
- **Impact: High** · **Cost: Small–Medium**

---

## UI / Presentation

**Purpose:** Communicate state and outcome.

**Current implementation:** Single-screen dashboard: resource HUD across the top, left drawer (Build/Monsters/Traps/Evolve/Heroes tabs), center dungeon layout, right inspector, bottom event/stats panels. A 5-step tutorial. Emoji icons for monsters/traps.

**Strengths:** The dashboard is competent and readable *at rest* — costs, availability, and selection states are clear. The tutorial correctly teaches the build loop. The drawer/tab structure scales to the content.

**Weaknesses:** **The UI is optimized for the build phase and neglects the raid phase.** When the dungeon is open and a party is fighting — the emotional core of the game — the screen barely changes. The founder said it three ways: "what's happening isn't clear when adventurers fight," "placing minions just increments a count," "the UI still doesn't feel clear." Two files (`dungeon_view.rs` 1188, `side_drawer.rs` 1096) are over the 800-line hard limit and should be split.

**Improvement ideas:** Make the dungeon view come *alive* during a raid — animate the party token moving room to room, flash rooms in combat, show HP bars, replace "increments a count" monster display with icons (the founder asked for this explicitly).
- **Impact: Game-changing** · **Cost: Medium–Large**

---

# 4. Similar Games & Lessons

## Legend of Keepers *(the essential comparable)*
**Similar:** You are the dungeon; you place monsters and traps in rooms; hero parties raid room-by-room; you watch fights resolve.
**Does better:** The raid is a **readable turn-based encounter** with visible HP, initiative, and a *morale/fear* system that hands the player mid-fight levers. You *watch and slightly steer* — which is exactly the experience Dungeon Core's raid phase is missing.
**Adapt:** Make combat legible and give the player *one or two* mid-raid decisions (a core spell? triggering a trap manually?) so the raid isn't pure spectator.
**Don't copy:** Its full roguelite run-structure and hand-authored bosses — Dungeon Core's identity is the persistent, growing lair, not discrete runs.

## Dungeon Keeper / War for the Overworld
**Similar:** "You're the villain building a lair" fantasy; economy of minions and rooms.
**Does better:** *Physicality* — you see creatures walk, eat, fight, and slap them yourself. The lair feels alive and inhabited.
**Adapt:** Even minimal token movement and idle animation would transform Dungeon Core's static board into a living place.
**Don't copy:** Real-time RTS control and possession mode — out of scope for a dashboard game.

## Loop Hero
**Similar:** You set up a system, then largely watch it play out; incremental power growth; deliberate low-agency-during-combat loop.
**Does better:** It makes the *watching* legible and tense — you clearly see the hero's HP crater and feel the "do I push or retreat" decision. It proves a spectator loop can be gripping *if the stakes are visible.*
**Adapt:** The lesson is that spectator combat works only when the player can read the danger in real time.
**Don't copy:** Its card-placement metaphor.

## Dungeon of the Endless
**Similar:** Room-by-room defense, resource management, waves pushing toward a core objective.
**Does better:** Spatial tension — *where* you open doors and place defenders matters enormously.
**Adapt:** Argues for branching paths and choke-point strategy over Dungeon Core's near-linear room queue.
**Don't copy:** Its permadeath roguelike framing.

**Cross-cutting lesson:** every successful game in this niche solved *raid legibility and stakes visibility first*, then added content. Dungeon Core did the reverse.

---

# 5. Feature Improvement List

## Critical Improvements
| Priority | Feature | Description | Player Benefit | Cost |
|---|---|---|---|---|
| Critical | **Live combat visualization** | HP bars on combatants, damage anchored to units, room flashes when fighting, party token moves room-to-room | Turns the invisible core loop into a watchable, comprehensible one — the #1 founder complaint | Medium |
| Critical | **Post-raid summary card** | After a party leaves: who died, damage dealt, MVP room/monster, income earned | Player finally *sees* whether their build worked; makes the design space legible | Small |
| Critical | **Monster icons in rooms** | Replace "increments a count" with placed monster icons (founder asked directly) | Rooms read as inhabited, not as counters | Small |

## High Value Improvements
| Priority | Feature | Description | Player Benefit | Cost |
|---|---|---|---|---|
| High | **Matchup hints at decision time** | Element strong/weak badges when placing monsters/traps; "counters Mage" tags | Makes the deep element/trait system *discoverable* instead of Codex-only | Medium |
| High | **Dramatize threat & siege** | Threat as escalating dread (visual/audio), "realm marches in N days" countdown, siege screen-state change | Converts a silent % bar into the game's tension engine | Small–Medium |
| High | **Expand core powers** | More `CORE_POWERS` entries so prestige has a branching upgrade tree | Gives the meta-loop a reason to prestige repeatedly | Small |
| High | **Mid-raid agency (one lever)** | A core spell or manual trap trigger on cooldown during raids | Breaks the pure-spectator problem; small dose of Legend-of-Keepers steering | Medium |

## Nice To Have
| Priority | Feature | Description | Player Benefit | Cost |
|---|---|---|---|---|
| Low | Gold sink rework | Ongoing use for gold (upkeep/re-roll) or merge into mana | Removes late-game dead currency | Small |
| Low | Branching dungeon paths | Non-linear room layout, choke points | Adds spatial strategy | Medium |
| Low | Sound hooks | Combat/siege/build audio (hooks noted in roadmap) | Feedback and game-feel | Medium |
| Low | Split oversized files | `combat.rs`, `dungeon_view.rs`, `side_drawer.rs` exceed the 800-line limit | Maintainability | Small |

## Avoid / Do Not Add
| Feature | Why avoid |
|---|---|
| **More monsters / species / elements right now** | The bottleneck is perception, not content. Adding to 47 monsters before combat is legible deepens the invisible-depth problem. |
| **Multiplayer** | No design pull; would multiply the legibility problem across two players. |
| **Complex combat math (crits, more formulas)** | Combat is already deeper than the UI conveys; more math the player can't see is negative value. |
| **Deferred grudge/rivalry/bounty hero systems** | Data hooks exist, but this is more invisible simulation. Don't build it until heroes are *visible* actors in the raid. |
| **Manual save-slot UI / equipment loot economy** | Low player-value plumbing; not what's holding the game back. |

---

# 6. Missing Gameplay Elements

## Real-time combat readability
- **Expected?** Yes — every comparable shows the fight. **Needed?** Absolutely — it's the game's core payoff. **Implementation:** HP bars + moving party token + room combat flashes (the data already exists per tick). **Priority: Critical.**

## Player agency during the raid
- **Expected?** For an active game, yes. **Needed?** At least a small dose — pure spectator risks boredom, which the founder himself worried about ("depends if it's interesting to watch"). **Implementation:** one core ability on cooldown. **Priority: High.**

## A shaped goal / destination
- **Expected?** Players want a "win" or a milestone arc. **Needed?** Somewhat — prestige is an infinite scaler with no narrative or milestone structure. **Implementation:** named prestige tiers, an achievement/milestone track, or a soft "ascension" goal. **Priority: Medium.**

## Onboarding into the *depth* (not just the controls)
- **Expected?** The tutorial teaches buttons, not strategy. **Needed?** Yes — the element/trait/attunement system is the reason to play and nothing teaches it. **Implementation:** contextual tips, a "your first synergy" guided beat. **Priority: High.**

## What's genuinely *not* needed
- Equipment loot loops, condition system expansion, multi-save UI, more currencies. The January audits pushed toward these; they are not the gap.

---

# 7. Content & Replayability Analysis

**Reasons to keep playing today:** species-as-playstyle (8 distinct identities), the build/optimize puzzle, threat→siege→prestige escalation, unlock progression across 47 monsters and a soul-bought core-power tree.

**Assessment by axis:**
- **Variety:** High in the data, low in the *felt* experience (raids look identical regardless of build).
- **Progression:** Strong and multi-layered (monsters → species → floors → core powers → prestige).
- **Randomness:** Present (party composition, spawns, trait procs) but not chaotic — good.
- **Player choices:** Rich build-phase choices; near-zero raid-phase choices.
- **Different strategies:** Exist mechanically (attunement builds vs. trap corridors vs. taunt-tank walls) but the player can't *see* them differ.
- **Emergent gameplay:** The synergy system *should* generate it — the founder even hopes emergence is the hook — but emergence you can't observe isn't emergence you can enjoy.
- **Long-term goals:** Prestige only; no shaped destination.

**Improvement:** Replayability isn't a *content* problem here — it's a *perception* problem. Make the existing systems visible and the replayability that's already built will finally be felt. Then add a milestone/ascension arc to give the infinite prestige loop a shape.

---

# 8. Player Experience Review

## First 10 Minutes
The tutorial cleanly teaches build → place → trap → open → survive. The player understands the *controls* and the build fantasy (which lands — "building up your own dungeon" is the confirmed best part). What they **don't** understand: what actually happened when the first party fought, why anyone died, or that a deep synergy system even exists. The raid resolves as log spam and floating text. **Improve:** a slow, legible *first raid* with a summary card and visible HP.

## First Hour
The build/economy hook is real; unlocking a second species and a trap corridor is satisfying. Risk: the player plateaus into *"I set things up and watch numbers scroll"* without grasping the element/trait depth, and the raid's sameness starts to bore. Whether the hour lands hinges entirely on making raids readable and teaching one satisfying synergy. **Improve:** surface matchups at placement; land one "your combo just crushed a party" moment.

## Long-Term
Threat→siege→prestige gives a heartbeat, and species variety gives replay *reasons*. But with an invisible combat layer and only 3 core powers, long-term is currently "watch bigger numbers." **Improve:** visible combat makes each new monster/species feel different; a deeper core-power tree and a milestone arc give prestige a point.

---

# 9. Development Roadmap

> The in-repo `ROADMAP.md` (Phases 0–6) is essentially *done* and was the right sequence for building the machine. This roadmap covers what comes next: **make the machine visible, then give the player a hand on it, then shape the destination, then content.**

## Phase 1 — Make It Legible *(Make It Fun)*
**Goal:** The player can *see and understand* a raid. This is the whole ballgame.
**Features:** live combat visualization (HP bars, moving party token, room combat flashes); post-raid summary card; monster icons in rooms; matchup badges at placement time.
**Why first:** Everything already built is invisible. No new system will land until the existing ones can be perceived. This directly resolves the founder's #1, #2, and #4 feedback points.

## Phase 2 — Give the Player a Hand on the Machine *(Add Depth)*
**Goal:** Break the pure-spectator raid.
**Features:** one core ability on cooldown during raids (a core spell or manual trap trigger); dramatized threat meter and siege (countdown + screen state); teach one synergy contextually.
**Why second:** Once raids are *visible* (Phase 1), giving the player a lever makes them *tense and active*. Doing this before legibility would just add an invisible button.

## Phase 3 — Shape the Destination *(Add Content, the right kind)*
**Goal:** Give the infinite loop an arc.
**Features:** expand the core-power tree (branching prestige upgrades); milestone/ascension goals; rework the gold sink; *then* — only now — consider more monsters/species/branching paths.
**Why third:** New content is finally worth adding once the player can perceive and steer it. Deepening prestige rewards the loop this phase gives a point.

## Phase 4 — Polish
**Goal:** Game-feel and shipping quality.
**Features:** sound hooks; generated monster/trap art (`image_prompts.json` is prepped); split the three over-limit source files; scrolling/large-dungeon UI hardening; balance pass on mana-starvation and gold-glut.
**Why last:** Polish amplifies a loop that works; applied earlier it would gild an invisible core.

---

# 10. Final Assessment

## Strongest Idea
The **element-attunement + trait + trap synergy system** sitting on top of an honest, stat-driven combat engine — all fully data-driven. This is a genuinely deep, well-architected optimization space that most games in this niche never reach. It is the thing worth protecting.

## Biggest Risk
**The player never perceives the depth that's been built.** A rich simulation that resolves as scrolling log text reads, to a player, as a shallow idle game. The project has spent its budget building systems and almost none making them visible — and an invisible strength is indistinguishable from a missing one. The founder's own repeated feedback ("what's happening isn't clear") *is* this risk already surfacing.

## Missing Ingredient
**Raid legibility.** One thing: let the player *see* combat happen — HP trading, party moving, rooms fighting, a summary of what worked. This single change unlocks the value of every system already built.

## Unique Selling Point
"Be the growing heart of a lair and engineer an elemental death-machine that shreds hero parties — then survive the realm's revenge." The persistent, prestige-scaling *living lair* (vs. *Legend of Keepers'* discrete runs) plus the attunement/trait build depth is a defensible niche — **once the machine is visible enough to appreciate.**

## Recommendation
**Continue development — but hard-pivot the effort from systems to presentation.**

The hard part is done and done well: the simulation is deep, honest, and test-covered. Stop adding systems. The next chunk of work is not more content — it is making the existing content *visible, teachable, and steerable.* Ship Phase 1 (legibility) and the whole project's perceived quality will jump, because there's a rich game already underneath waiting to be seen. The single most important milestone, echoing the founder exactly, is: **a player watching a raid understands what happened and why — and wants to build a better dungeon because of it.**

*Do not* archive, reduce scope, or redesign systems — the systems are the asset. The gap is entirely in the last layer between the simulation and the player's eyes.
