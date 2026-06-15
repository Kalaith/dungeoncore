# Dungeon Core

Dungeon Core is a dungeon management and defense game where you are the heart of a hostile underground lair.

Adventurers are coming for treasure and glory. Your job is to build rooms, place monsters, improve defenses, and make sure they never reach the core.

## Gameplay

- Add rooms and shape the dungeon layout.
- Place monsters where they can delay or defeat invaders.
- Earn mana, gold, and souls from dungeon activity.
- Unlock stronger monster types and deeper floors.
- React to adventurer parties with different roles.

## Goal

Protect the dungeon core while expanding into a stronger and more dangerous lair.

## Controls

- Mouse: select rooms and monsters.
- Click: place monsters and add rooms.

## Current Scope

Playable dungeon-building and wave-defense loop with rooms, monsters, adventurer parties, resources, unlocks, and upgrades.
# Practical Future Improvements

- Add input-state tests for pause, focus, tooltip blocking, resource panel updates, and log message ordering.
- Move resource panel calculations into pure helpers with fixtures for edge cases such as zero income and capped resources.
- Add small dungeon-run scenarios that verify controls, game-log output, and theme-driven UI states together.
- Extract repeated drawing constants into toolkit-backed theme helpers shared by controls, logs, and resource panels.

