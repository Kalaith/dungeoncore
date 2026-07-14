# Dungeon Graph — Design & Data-Model Plan (Tier 2: Spatial Strategy)

*Status: design proposal. Implements the still-open half of the Tier 2 "Spatial
strategy" item — **branching paths / choke points**. The sequence-dependent-room
half (snare→killbox, alarm/poison downstream, undefended-trap rooms) already
shipped (commit `eae0805`). This document is the plan for turning the dungeon's
**linear room queue** into a **branching graph**.*

---

## 1. Goal & non-goals

**Goal.** Make dungeon layout a *puzzle*: a floor becomes a small branching map
from its Entrance to its Core, with forks (branches) and joins (choke points),
so where you build and how you route intruders matters — deepening the build
phase, the confirmed "best part," without hurting legibility (the Tier 1 rule).

**Non-goals (v1).**
- No whole-dungeon graph across floors — floors stay chained by descent; each
  *floor* is its own small graph.
- No party **splitting** at forks — one party is one token that *chooses* a
  path. (Splitting multiplies every combat/legibility problem; defer.)
- No free-form maze editor — branch topology is constrained (see §4) to stay
  legible and keep rendering/pathfinding tractable.

---

## 2. Current model (what we're changing)

Grounded in the source as of `eae0805`:

- `game_state.rs`: `Floor { rooms: Vec<Room> }`, `Room { position: usize,
  floor_number, room_type, monsters, upgrades, … }`. **Adjacency is implicit**:
  the room at `position p` is followed by `position p+1`.
- Layout is a single horizontal row per floor: `dungeon_view.rs::draw_floor_rooms`
  sorts rooms by `position` and draws them left→right with a connector between
  each consecutive pair. Entrance = position 0, then Normal(1..5), Boss, Core last.
- Traversal: `adventure.rs::advance_party` moves `current_room → current_room+1`;
  at the last position it descends to `target_floor` or assaults the Core.
- `position` is the **intra-floor key everywhere**: `state.selected_room =
  (floor_number, position)`, `place_monster(floor, position)`,
  `adventurers_in_room` (`party.current_room == room.position`), party transit
  (`prev_room`/`current_room` are positions), build inserts before Core.
- Building: `rooms.rs::add_room` inserts before the Core and shifts the Core's
  position; a full floor spawns a new floor.
- Persistence: serde on `Floor`/`Room`; `GameState::migrate()` runs post-load.

**Design consequence:** the cheapest, lowest-churn path is to **keep `position`
as a stable per-floor node key** (it stops implying linear order) and add an
explicit edge list. Everything keyed by `(floor, position)` keeps working;
only *traversal* and *rendering* change.

---

## 3. Target data model

Single new **persisted** field — the graph edges. Layout coordinates are
**computed**, not stored (one source of truth = the edges).

```rust
pub struct Room {
    pub id: u64,
    pub room_type: RoomType,
    /// Stable per-floor node key. No longer implies linear order — it is just
    /// this room's identity within the floor (used by selection, placement,
    /// party position, and as the endpoints of `exits`).
    pub position: usize,
    /// Child rooms this room routes into (DAG edges within the floor). The
    /// Entrance has >= 1 exit; a fork has >= 2; the Core (sink) has 0.
    /// `#[serde(default)]` so pre-graph saves load with none, then `migrate()`
    /// rebuilds the linear chain.
    #[serde(default)]
    pub exits: Vec<usize>,
    pub floor_number: i32,
    pub monsters: Vec<Monster>,
    pub upgrades: Vec<RoomUpgrade>,
    pub explored: bool,
    pub loot: i32,
}
```

`AdventurerParty` already stores `current_room`/`prev_room` as positions — these
stay, but now index the graph by node key instead of implying `+1`.

**Invariants (checked by a `Floor::validate_graph()` helper + a test):**
1. Exactly one Entrance (single source) and one Core (single sink) per floor.
2. Every non-Core room has ≥1 exit; every room is reachable from the Entrance;
   every path from the Entrance reaches the Core (no dead ends, no orphans).
3. Acyclic (it's a DAG — enforced by construction: exits only point to
   strictly-deeper rooms; see §4).

**Layout coordinates (computed for rendering, not persisted):**
- `depth(room)` = longest path length from the Entrance (layered / topological
  layer) → the render **column** (x).
- `lane(room)` = a vertical slot so parallel branches don't overlap → the render
  **row** (y). Assigned by the layout pass (§6).

---

## 4. Key design decision: constrain topology to **series-parallel** ("diamonds")

Rather than a free DAG, constrain each floor to a **series-parallel graph** with
a single source (Entrance) and single sink (Core): paths may **fork** and later
**reconverge**, but every branch eventually rejoins. Concretely, the only build
operations are:

- **Extend**: append a room after a leaf on the path to Core (linear growth —
  today's behavior).
- **Fork**: branch a room off an existing room; the new branch **reconverges**
  at that room's existing successor (or at the Core). This creates a *diamond*:
  `X → A → Y` becomes `X → {A, B} → Y`.

Why this constraint:
- **Choke points fall out for free** — a *join* (a room with ≥2 parents) is a
  natural choke where branches funnel; the Core is the ultimate choke.
- **Pathfinding is trivial** — every path reaches the Core, so the party can
  never get "stuck"; path-selection is a local scoring problem (§5), not a
  search.
- **Rendering is bounded** — series-parallel graphs have small, predictable lane
  counts; a max-children rule (recommend **2**) plus the existing
  `MAX_ROOMS_PER_FLOOR` keeps a floor legible.
- It is exactly the roadmap's ask ("branching paths, choke points") without the
  cost/ambiguity of an arbitrary maze.

*(Open question O3 below asks whether to allow a freer DAG later.)*

---

## 5. Traversal & path-selection

`advance_party` changes from `current_room + 1` to *follow an exit*:

```rust
let exits = &current_room.exits;
match exits.len() {
    0 => { /* at the Core sink → descend / assault core (unchanged) */ }
    1 => advance_to(exits[0]),                 // linear: today's behavior
    _ => advance_to(choose_exit(state, exits)) // a fork: pick a branch
}
```

**`choose_exit` — the rule that makes branching *fun*.** Two **modes** (decided
with the founder):

- **Greedy (default).** A legible score over each candidate child room —
  adventurers are greedy for loot and shy of obvious danger:
  ```
  score(child) =  loot_appeal(child)   // treasure upgrade / stored loot  (+)
                - visible_threat(child) // count/strength of live monsters (−)
                + core_bias(child)      // closer-to-core breaks ties      (+)
  ```
  The player's puzzle: bait the party down a treasure branch that is *actually*
  a snare→poison→killbox, or under-defend a branch and watch it slip past.

- **Beeline (desperation).** When the realm is losing adventurers **too fast**,
  they stop looting and **rush the Core to destroy the dungeon** — score flips to
  pure shortest-path-to-Core, ignoring loot and danger. Trigger reuses the
  existing threat system: `threat_tier() >= 3` ("Hunted"/"Besieged"), the
  already-existing `party.sieging` flag, or a future per-party
  `beeline`/event/quest flag (special events & quests can force this too). This
  makes the threat meter *mean* something spatially: push the realm too hard and
  the heroes come straight for your heart.

Deterministic; state-owned RNG only breaks exact ties (keeps runs reproducible).
Surface the mode on the board/log ("The party eyes the gold and takes the left
path" vs. "Enraged, the party storms straight for the Core") so it's legible.

This composes directly with the already-shipped sequence mechanics: "downstream"
now means *the branch the party actually chose*.

---

## 6. Rendering (the biggest lift)

Replace the single-row `draw_floor_rooms` with a **layered layout**:

1. **Layout pass** (`ui/dungeon_view/layout.rs`, new): from `exits`, compute
   `depth` (column) via longest-path-from-Entrance, and assign `lane` (row) per
   room so branches in the same column don't collide. Series-parallel ⇒ a simple
   lane allocator suffices. Memoize per floor per frame.
2. **Draw**: place each room at `(col=depth, lane)` within the floor's row band;
   draw a connector **per edge** in `exits` (diagonals when lanes differ).
3. **Party token** rides the specific edge `prev_room → current_room` (extend
   `party_transit_progress` to look up the edge, not `pos → pos+1`).
4. **Legibility:** highlight the party's *chosen* path; keep the Tier-1 combat
   viz per room unchanged.

Watch the **800-line file limit**: `dungeon_view.rs` (390) and `room_art.rs`
(~430) are already sizable; the layout logic goes in a *new* `layout.rs` up
front, not bolted on.

---

## 7. Build UX

Today "Build" appends the next linear room. New flow:

- Selecting a room shows a **"Branch from here"** action (in the inspector /
  build tab) in addition to the existing extend-toward-core build.
- **Extend** = today's behavior (append on the path to Core).
- **Branch** = create a parallel room off the selected one that reconverges at
  its successor (the diamond of §4), gated by: max **2** children/room,
  `MAX_ROOMS_PER_FLOOR`, and mana cost (reuse `get_room_cost`, perhaps a small
  fork surcharge).
- A short **tutorial beat** teaches the fork once (mirrors the Tier-1
  "learn the elements" beat).

---

## 8. Migration & save compatibility

- `exits` is `#[serde(default)]` → old saves load with empty exits.
- `GameState::migrate()` gains: for each floor, if every room's `exits` is empty,
  rebuild the **linear** chain by sorted `position` (`p.exits = [next_p]`, Core
  gets `[]`). Idempotent; guarded so it never double-runs on graph saves.
- Add a migration test: a hand-built old-style linear floor loads and validates
  as a legal graph with identical traversal order.

---

## 9. Phasing (each phase = one clean, shippable commit)

| Phase | Scope | Risk | Rough size |
|---|---|---|---|
| **A. Model + migration (no behavior change)** | Add `exits`; migrate linear→chain; `advance_party` follows `exits` (still linear); `validate_graph` + tests. Board still renders one row. | Low — pure refactor, behavior identical | M |
| **B. Path-selection** | Implement `choose_exit` scoring; still linear graphs so no visible change, but the machinery + tests land. | Low | S |
| **C. Build a fork** | "Branch from here" build op with the series-parallel constraint, limits, cost; graph now branches in data. | Med | M |
| **D. Layered rendering** | New `layout.rs`; multi-lane draw; per-edge connectors; party rides the chosen edge. | **High** (UI) | L |
| **E. Polish & balance** | Bait-choice feedback on the board, choke-point bonus tuning, tutorial beat, a full playtest pass. | Med | M |

Phases A–B are safe to land immediately (invisible refactor + logic). C gates on
D for visibility but can merge behind the existing renderer (a forked floor just
renders its rooms in `position` order until D lands). This ordering keeps every
commit green and the game shippable throughout.

---

## 10. Test plan

- **Graph invariants:** `validate_graph` rejects orphans/dead-ends/cycles;
  accepts valid diamonds.
- **Migration:** old linear save → legal graph, same visit order.
- **Traversal:** single-exit = linear (regression); multi-exit picks the
  scored branch; every path terminates at the Core.
- **`choose_exit`:** prefers loot, avoids threat, deterministic tie-break.
- **Build op:** fork respects max-children / max-rooms / mana; result validates.
- **Layout (Phase D):** depth/lane assignment collision-free on sample graphs.
- Keep the JSON-integrity + existing 52 tests green throughout.

---

## 11. Risks & mitigations

- **Rendering complexity (highest).** → Series-parallel constraint bounds lanes;
  isolate in `layout.rs`; land behind the old renderer until ready.
- **`position`-as-order assumptions elsewhere.** → Audit every `position + 1` /
  `sort_by position` use (grep) in Phase A; convert to edge-following.
- **Path-selection feels unfair or pointless.** → Make scoring legible on the
  board; playtest in Phase E; keep the rule data-tunable.
- **Save migration eats old dungeons.** → `#[serde(default)]` + idempotent
  migrate + explicit test (ties into the Tier-4 save-versioning debt).
- **File-size limit breaches.** → New `layout.rs` from the start; watch
  `dungeon_view.rs`/`room_art.rs`.

---

## 12. Open questions — RESOLVED (2026-07-14)

- **O1 — Path-selection.** ✅ **Loot-bait + threat-shy by default, escalating to
  beeline-for-core when adventurers die too fast** (high threat) or an
  event/quest forces it. See §5 ("Greedy" vs "Beeline" modes).
- **O2 — Party splitting.** ✅ **Deferred** — one party, one token that chooses a
  path (v1). Splitting is a post-launch stretch.
- **O3 — Topology.** ✅ **Series-parallel with reconvergence** (every path reaches
  the Core; the Entrance→Core route is always maintained), now permitting up to
  3-way splits.
- **O4 — Branch budget.** ✅ **Up to 3 exits per room** (3-way splits allowed) so
  a floor can grow *wide* as well as deep. Invariant: there is always a route
  from the Entrance to the Core (build ops may never orphan either). Revisit the
  `MAX_ROOMS_PER_FLOOR` cap during Phase E if wide floors need more room.

**Status: Phase A in progress.**
