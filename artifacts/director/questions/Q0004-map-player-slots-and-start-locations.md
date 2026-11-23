# Q0004 — Map model: player slots, start locations, starting-base layout, pre-placed objects

- **Date**: 2026-11-23T11:38:05-05:00
- **Blocking**: no — proceeding on assumption: each player slot on a map has an optional **start location** (a tile where that slot's starting base will go) in addition to the optional starting camera position. The map editor can also pre-place any object for any owner (used for test scenarios). Spawning each faction's Starting Conditions around the start location waits for this answer.
- **Docs**: `core/camera.md > Starting Position`, `factions/gdo/economy.md > Starting Conditions`, `factions/cults/economy.md > Starting Conditions`, `factions/colonists/economy.md > Starting Conditions`, `core/map.md`

## Question
The user asked for a **map editor that supports all objects**. The design specifies what is on a map (tiles, presets, elevation, resources) and what each faction starts with. It doesn't specify how a map tells the game where those starting bases go. Could the design define:
1. **Player slots**: what a map defines per slot. A start location? Allowed factions? Teams? How many slots a map can have?
2. **Starting-base layout**: who places each faction's starting objects? The game could do it automatically around the slot's start location, but then by what rules? For example, which patches get GDO's two Extraction Plates "on Space Crystals patches near the starting base", and where do the Colonists' Egress Node, Robo Portal and four Drones go? Or does the map maker place a layout per faction in the editor?
3. **Pre-placed objects**: can maps contain units or structures beyond resources, either owned by players (scenario or custom maps) or neutral?
4. **Map metadata and limits**: name, description, minimum and maximum size, and whether maps may define custom tile presets with their own textures (`core/map.md > TilePreset` allows any flag combination).

## Context
The map editor (plan M0-C) saves maps as text files that the game loads, so it needs to know everything a map contains. Tiles, elevation and resources are fully specified; start locations and starting layouts are not. The faction Starting Conditions also can't be implemented without this.

## Options
- **A: start location + automatic layout**: one start-location tile per slot. The game places each faction's starting objects using rules in the faction docs (fixed offsets, or e.g. "plates go on the two patches nearest the start"). Maps stay simple and work for all four factions, but every faction needs layout rules that fit any terrain.
- **B: per-faction layouts placed by the map maker**: four layouts per slot, set in the editor. Full control, four times the work per map, and maps break when a faction's starting objects change.
- **C: A plus optional per-slot overrides** in the editor.
