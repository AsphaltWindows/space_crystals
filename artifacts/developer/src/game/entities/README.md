# Entities Module

Declarative entity definition system for Space Crystals.

## Files
- **types.rs** — Core types: `BaseStats`, `UnitStats`, `StructureStats`, `CommandSlotDef`, `SpawnParams`, `EntityKind`, traits (`EntityDefinition`, `UnitDefinition`, `StructureDefinition`), and `EntityRegistry`.
- **registry.rs** — `build_registry()` function and `EntitiesPlugin` (Bevy plugin).
- **utils.rs** — Shared helper functions (currently empty).

## Subdirectories
- **factions/** — Per-faction entity definitions (gdo, syndicate, cults, neutral).
