# Map Editor

The map editor creates and edits maps. It supports **every object** in the game (user's request), so maps can contain terrain, elevation, resources and starting bases, and anything the game can place. Maps are saved as files the game loads.

What a map contains is defined in core/map.md (tiles, tile presets, corner heights, resources). This file covers editor-specific features.

## Per-object Resource Values
Patch amounts and Supply Delivery Station settings are set per object in the editor (core/map.md > SpaceCrystalsPatch, core/map.md > SupplyDeliveryStation).

## Starting Layouts
Starting bases are **placed by the map maker per faction** (user, 2026-11-23). For every player start position the map defines a separate starting layout for **each faction**: where each of that faction's starting buildings and units goes (the starting objects themselves are defined in factions/*/economy.md > Starting Conditions). At game start, only the layout of the faction that player picked is used.

### Overlay editing
The editor can show **all factions' starting layouts at the same position simultaneously, overlaid on each other**, so the map maker can define and compare the positions of all buildings and units for all factions in one place.

TBD (asked, not yet answered): whether the editor drops in a default layout per faction to adjust or the maker places everything from scratch; whether objects from different factions' layouts may overlap (each layout presumably only needs to be valid on its own); per-faction layer toggles; what happens to existing maps when a faction's Starting Conditions change (flag vs auto-fix).

## Symmetry Tools
The editor can make maps symmetric automatically (user, 2026-11-23). Supported symmetries:
- **Rotational, 180°** (two-fold).
- **Rotational, 90°** (four-fold). A 90° rotation maps the map onto itself only if it is **square**.
- **Mirror, horizontal axis.**
- **Mirror, vertical axis.**
- **Mirror, both axes** (user expects this to be rarely useful but wants it available).

TBD (asked, not yet answered): whether symmetry applies live while painting (edit one part, the rest updates) or as a one-shot "generate the rest from this part"; whether everything is mirrored: tiles, corner heights, patches, stations, start positions with their layouts, and pre-placed objects with their orientation transformed to match.
