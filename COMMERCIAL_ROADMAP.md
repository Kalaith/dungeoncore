# Dungeon Core — Prototype → Commercial Release Gap Analysis

*Compiled 2026-07-14. Grounded in the current source tree, `ROADMAP.md` (Phases 0–6
shipped), `game_review.md` (2026-07-07 design review), and `feedback.md` (founder
playtest). This is the superset list: design, content, presentation, production,
platform, and business work needed to ship this as a paid commercial game.*

## Where the game actually stands

The simulation layer is **done and deep**: stat-driven tick combat, a 9-element
matchup matrix with room attunement, 47 monsters across 8 species and 4 tiers with
branching evolutions and boss uniques, 14 wired traits, 9 behavior-typed traps with
counterplay, adventurer races/classes/party AI, a persistent hero roster, and a
threat → siege → prestige endgame. All balance data lives in `assets/*.json` with
integrity tests. Save/load, tutorial, Codex, and title screen exist.

What does **not** exist, at all: audio, real art (everything is emoji + rects),
combat you can watch, a settings menu, accessibility support, localization,
platform integration, and every piece of business/production infrastructure.
The honest framing: **the game design is ~80% built; the game *product* is ~15% built.**

Sequencing principle (from the design review, and it's correct): make the
simulation *visible* before adding anything else. An invisible strength is
indistinguishable from a missing one.

---

## Tier 1 — Raid legibility (the game-maker; nothing else lands without it)

The single founder complaint across every playtest: "what's happening isn't clear."
The combat engine computes targeting, elements, traits, conditions, and trap procs
every tick — and surfaces it as log lines and floating strings.

- [x] **Live combat visualization.** HP bars on every combatant, damage numbers
      anchored to units (not floating in space), a visible "who is hitting whom"
      indication, rooms flashing/highlighting while fighting. The per-tick data
      already exists in `simulation/combat.rs` — this is rendering work, not sim work.
      *(Done: HP bars on defenders/invaders; combat effects now side-anchored —
      hits on defenders rise over the left zone, damage/deaths on invaders over
      the right — giving a clear who-hits-whom read; room-flash pulse and element
      feedback already existed. Possible future polish: per-disc anchoring and
      target lines, but the two-sides-trading-blows read is in.)*
- [ ] **Party token movement.** Adventurer parties visibly walk room-to-room
      through the dungeon instead of teleporting between combat resolutions.
      This one change makes the dungeon read as a *place*.
- [x] **Post-raid summary card.** After each party dies/retreats: who died and to
      what, damage by room/monster, MVP defender, income earned, loot lost.
      Deferred from Phase 6; it's the cheap half of legibility.
      *(Done: "Raid Report" card shows outcome, slain/escaped, and mana/gold/souls
      banked vs. defenders lost, via a per-raid tally.)*
- [x] **Monster icons in rooms** instead of "increments a count" (explicit founder
      request). Placed monsters render as individual units.
      *(Done: per-unit discs coloured by element with a class/monster initial cue;
      invaders labelled by class. Full sprite art is a Tier-3 item.)*
- [x] **Matchup hints at decision time.** Element strong/weak badges while placing
      monsters/traps; "counters Mage"-style tags; attunement synergy shown on
      hover. The Codex wheel is reference material — the funnel needs it at the
      moment of choice.
      *(Done: placement badge shows the selected monster's element and what it's
      strong against; rooms whose attunement matches show an "Attuned" synergy
      pill. Still open: live "counters the current invaders" tags during a raid,
      and applying the same to trap placement.)*
- [x] **Dramatized threat & siege.** Threat as visible mounting dread ("the realm
      marches in N days" countdown), a screen-state change + distinct presentation
      when the siege lands, not a log line reading "THE SIEGE BEGINS".
      *(Done: HUD threat slot now carries a rising "dread" meter toward the siege
      threshold; an active siege paints a pulsing red screen frame plus a bold
      "DEFEND THE CORE" banner. The siege is threshold- not time-triggered, so a
      dread meter replaces a literal day countdown.)*
- [ ] **Teach the depth, not just the buttons.** The 5-step tutorial covers
      controls; nothing teaches element counters, attunement, or trait synergy.
      Add contextual tips and one guided "your first synergy" beat.

## Tier 2 — Player agency & the shape of a full game

- [ ] **Mid-raid agency (at least one lever).** A core spell or manual trap
      trigger on cooldown so raids aren't pure spectator. Legend of Keepers is the
      model: watch *and slightly steer*.
- [ ] **Spatial strategy.** The dungeon is currently a linear queue of rooms.
      Branching paths, choke points, and sequence-dependent rooms (snare-before-
      damage, alarm-empowers-downstream) would make layout a real puzzle — the
      building phase is the confirmed "best part," so deepen it.
- [ ] **A shaped destination.** Prestige is an uncapped scaler with no arc. Add
      named prestige tiers, a milestone/achievement track, and/or a soft
      "ascension" win state. Commercial players need a reason to say "I finished it."
- [ ] **Expanded core-power tree.** 3 soul-bought powers is too shallow to carry
      the prestige loop; the `CORE_POWERS` framework already supports more —
      build a branching tree (10–20 nodes) so repeated prestiges diverge.
- [ ] **Difficulty options / modes.** One fixed difficulty today. Minimum: 2–3
      difficulty presets. Strong candidates for a management game: endless mode
      vs. structured campaign, challenge modifiers, seeded runs.
- [ ] **Remaining design-debt items from ROADMAP.md:**
  - [ ] Undead identity rule (no healing, cheap respawn) — requires a respawn-cost
        mechanic; all respawns are currently free.
  - [ ] Hero grudges/rivalries/bounties (data hooks exist) — *only after* heroes
        are visible actors in raids, per the design review.
- [ ] **Economy balance passes** flagged but never done:
  - [ ] Gold glut — sinks are one-time (species unlocks) or occasional
        (evolutions); late-game gold piles up dead. Add an ongoing sink (upkeep,
        rerolls, rush-building) or restructure.
  - [ ] Mana starvation — stat-driven combat cut death income (10/death);
        verify building doesn't stall across a long session.

## Tier 3 — Presentation: art, audio, game feel (zero → shipping quality)

This is the largest raw-cost block. The game currently ships with emoji glyphs,
flat rects, one AI title image, and **no audio of any kind**.

### Art
- [ ] **Visual identity / art direction decision.** Pick a style you can afford
      to execute across ~70 unique assets (pixel art is the realistic option for
      this scope). Everything below depends on this call.
- [ ] **Monster sprites** — 47 monsters (some sharing bases across evolution
      lines is acceptable). `image_prompts.json` is already prepped for generation.
- [ ] **Adventurer sprites** — 7 classes × 4 races (palette/part swaps fine).
- [ ] **Trap, upgrade, and room-attunement art** — 9 traps + 11 upgrades.
- [ ] **Room/environment art** — themed room interiors, floor depth theming so
      floor 5 looks different from floor 1; core room as a visual centerpiece.
- [ ] **Animation** — idle animations for placed monsters, walk cycles for
      parties, attack/hit/death flashes. Even 2-frame idles transform the board.
- [ ] **VFX** — hits, element procs (fire/frost/poison read differently),
      trap triggers, deaths, siege arrival, prestige moment.
- [ ] **UI art pass** — replace programmer-art panels/buttons with a cohesive
      styled kit; icons for resources, elements (distinct *shapes*, not just
      colors — see accessibility), classes, and statuses.
- [ ] **Title screen, game-over, and prestige screens** at commercial quality.
- [ ] **Marketing art** — store capsules (multiple sizes), logo/wordmark,
      screenshots, animated GIFs, a trailer (see Tier 6).

### Audio
- [ ] **SFX** — build/place/summon, per-element combat hits, trap triggers, UI
      clicks, deaths, coin/mana income ticks, threat-tier stings, siege alarm,
      core damage heartbeat, prestige fanfare. (~30–50 sounds minimum.)
- [ ] **Music** — build-phase theme, raid tension layer, siege track, title
      theme. Adaptive layering (calm → combat) suits this genre well.
- [ ] **Ambience** — dungeon room tone; deeper floors sound deeper.
- [ ] **Mixing + ducking + per-channel volume** (master/music/SFX sliders —
      requires the settings menu below).
- [ ] Verify `macroquad` audio behavior on both native and WASM targets early;
      web audio unlock-on-first-input is a known browser landmine.

## Tier 4 — Product/UX infrastructure (table stakes for a paid game)

None of this exists today.

- [ ] **Settings menu**: audio sliders, windowed/fullscreen + resolution
      (native), UI scale, game-speed defaults, autosave interval, colorblind
      mode, reduced motion, key rebinding.
- [ ] **Save system hardening** (`persistence.rs` is a single hardcoded slot):
  - [ ] Multiple save slots + save metadata (playtime, day, prestige count).
  - [ ] Explicit save-file versioning (a `migrate()` exists but no version
        field discipline) — every future patch must not eat old saves.
  - [ ] Corrupt-save handling: backup rotation, graceful failure with recovery,
        never a panic or silent reset on a paying customer's 40-hour dungeon.
  - [ ] Cloud save (Steam Cloud) if shipping on Steam.
- [ ] **Pause** as a first-class state (speed controls exist; verify true pause
      + pause-on-focus-loss + pause menu with resume/settings/save/quit).
- [ ] **Confirmations** on destructive actions (reset run, dismiss monster,
      overwrite save).
- [ ] **Tooltips everywhere** — every stat, cost, icon, and abbreviation
      hoverable. Management-game players expect total inspectability.
- [ ] **Scalability hardening** — large late-game dungeons: dungeon-view
      scrolling/zoom, log scrollback + filtering, drawer performance with 47
      unlocked monsters, 20-floor layouts staying legible.
- [ ] **Input**: full keyboard shortcut coverage with an in-game reference;
      decide explicitly whether gamepad/Steam Deck is supported at 1.0 (Steam
      Deck verification sells real copies in this genre — worth scoping).
- [ ] **Accessibility**:
  - [ ] Colorblind-safe element/threat communication (shapes + labels, not hue
        alone — the element system is currently 100% color-coded).
  - [ ] Text size options; minimum font sizes audited.
  - [ ] Reduced-motion / disable-screen-flash options.
  - [ ] No mechanic gated on reaction time (mid-raid lever should tolerate slow
        input or pause-to-cast).
- [ ] **Localization readiness**: all player-facing strings are currently
      hardcoded in Rust or spread through JSON. Externalize to a string table
      keyed by ID, audit UI for text expansion (German +30%), pick fonts with
      needed glyph coverage. Even if 1.0 ships English-only, retrofitting
      localization later is far more expensive than plumbing it now. (EFIGS +
      Simplified Chinese is the usual value order for this genre.)

## Tier 5 — Technical & platform work

- [ ] **Platform decision**: Steam is effectively mandatory for commercial PC
      indie; itch.io as secondary; keep the existing WebHatchery WASM build as a
      demo/marketing funnel rather than the product.
- [ ] **Steam integration**: Steamworks SDK from Rust (steamworks-rs) —
      achievements (~20–40, tied to the milestone track from Tier 2), cloud
      saves, rich presence; store-page plumbing.
- [ ] **Windows packaging**: proper icon/version metadata, code-signing
      decision (unsigned exes trip SmartScreen), installer or Steam-only.
- [ ] **macOS/Linux decision**: macroquad supports them; decide support tier
      now (Linux ~free via Proton on Steam Deck; native macOS is real QA cost).
- [ ] **Performance validation**: soak-test a max-size dungeon (many floors,
      dozens of monsters, multiple simultaneous parties, 4× speed) for frame
      time and memory; profile the per-tick sim; long-session (8h+) leak check.
- [ ] **Crash/panic handling**: a top-level panic hook that writes a crash log
      and preserves the save; opt-in crash reporting if desired (with privacy
      policy — see Tier 6).
- [ ] **Determinism & RNG audit**: seeded RNG per run would enable seeded
      challenge runs and reproducible bug reports — cheap now, painful later.
- [ ] **CI → release pipeline**: current CI runs fmt/clippy/test; add automated
      release builds per platform, versioned artifacts, and a build-stamping
      scheme (version visible on the title screen for bug reports).
- [ ] **Code-health debt** (per project standards): keep every file under the
      800-line limit as UI work lands (the dungeon view/drawer historically
      breach it); maintain the JSON-integrity test suite as content grows;
      add the input-state and dungeon-run scenario tests listed in README.

## Tier 6 — QA, balance, and business

### QA & balance
- [ ] **Structured playtesting program**: move beyond founder playtests —
      10–20 external testers across skill levels, with a feedback form and a
      build distribution channel (Steam Playtest works well).
- [ ] **Balance instrumentation**: log per-run stats (income curves, monster
      pick rates, trap usage, death causes, time-to-first-prestige) to a local
      file so balance passes are data-driven, not vibes-driven.
- [ ] **Full balance pass** across 8 species × difficulty modes once combat is
      visible (visibility will change what players actually do).
- [ ] **First-hour retention pass**: the design review flags the "I set things
      up and numbers scroll" plateau — validate with fresh players that the
      first session lands a "my combo crushed a party" moment.
- [ ] **Compatibility matrix**: min-spec definition, multiple GPU vendors,
      high-DPI displays, ultrawide, 60/144 Hz behavior of the tick sim.

### Business & launch
- [ ] **Positioning & pricing**: comparable set is Legend of Keepers / Loop
      Hero territory (~$15–20); decide scope-to-price honestly.
- [ ] **Steam page early** (wishlists are the whole marketing game): capsule
      art, 5+ real screenshots, description, tags — page live months before
      launch.
- [ ] **Trailer** (~60–90s, gameplay-first — requires Tier 3 art to exist).
- [ ] **Demo build** (Steam Next Fest is the highest-leverage indie marketing
      event; the WASM build could seed this).
- [ ] **Name/trademark check**: "Dungeon Core" is both generic and heavily
      used in LitRPG publishing — search Steam and USPTO/EUIPO before
      committing marketing spend; a distinctive subtitle may be enough.
- [ ] **Legal/admin**: business entity + tax setup for Steam payouts, EULA,
      privacy policy (required if any telemetry/crash reporting ships),
      third-party license audit (`cargo license` — macroquad stack is
      MIT/Apache, but verify fonts and any generated-art tool ToS for
      commercial use), age-rating questionnaires (IARC via Steam).
- [ ] **Post-launch plan**: patch cadence commitment, community channel
      (Discord/Steam forums), bug-report intake (the HTML bug-report feature
      exists for web — needs an equivalent for native), and a 1.x content
      roadmap (the deferred species — Plant/Fungal, Insect — and the
      grudge/bounty system are the natural post-launch content drops).

---

## What NOT to do (inherited from the design review, still correct)

- **No more monsters/species/elements** until combat is watchable — content
  is not the bottleneck; perception is.
- **No multiplayer.** No design pull; multiplies every legibility problem.
- **No deeper combat math** (crits, more formulas) — the sim already outruns
  what the screen conveys.
- **No equipment-loot economy for the player** — the January audits pushed
  toward this; it's plumbing, not the gap.

## Sequencing summary

| Stage | Contents | Gate to next stage |
|---|---|---|
| 1. Legible | Tier 1 (combat viz, summary card, icons, hints, siege drama) | A new player watching a raid can say *why* it went that way |
| 2. Complete game | Tier 2 (agency lever, spatial depth, milestone arc, difficulty, economy fixes) | External playtesters finish a prestige and want another |
| 3. Looks & sounds like a product | Tier 3 (art + audio) + Tier 4 (settings, saves, accessibility, l10n plumbing) | A stranger's first screenshot reads "real game," not "prototype" |
| 4. Shippable | Tier 5 (Steam, packaging, perf, crash handling) | Release-candidate build passes the compat matrix |
| 5. Launchable | Tier 6 (QA program, balance, store page, trailer, demo, legal) | Wishlist/demo metrics justify the launch date |

Rough scale honesty: Tiers 1–2 are weeks of focused work each. Tier 3 is the
big one — likely the majority of remaining calendar time and the only tier
that probably needs money spent (art/audio, whether commissioned or heavily
AI-assisted-then-curated). Tiers 4–6 are steady grind that can interleave.
