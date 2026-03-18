# Design Update: Document Split (2026-03-06)

## Summary

Split the monolithic `design/design.md` (~2132 lines) into focused topic files for easier navigation and maintenance. Old-format bullet-point content separated into a dedicated conversion backlog file.

## Changes

### New Files Created (from formal content)

- **`design/scale.md`** — SimulationFrame, GridUnit, SpaceUnit definitions
- **`design/entities.md`** — Entity hierarchy (Entity, Invisible Entity, Faction, Player, Visible Entity, Tile, TilePreset, TilePlacement, Object Type, Vision, Structure Type, Object/Structure Instance, Resources)
- **`design/units.md`** — Unit type, Unit Instance, 5 MovementModels, 9 UnitBases, TurretAttributes
- **`design/combat.md`** — AttackAttributes, AttackPhases, AttackTypes, AttackSources, DamageCalculation, LocomotionOrientationConstraints
- **`design/control_system.md`** — ControlState, Selection, ControlGroups, ObjectInterfaceState, InterfaceTransitions, BasicCombatUnitInterfaceState, Unit Commands, BaseCommandState/BaseBehaviorState, TurretCommandState/TurretBehaviorState, all Behaviors (MovingToLocation, AttackingObject, etc.), BaseAutoTargeting, BaseActionChannels, TurretActionChannels
- **`design/factions.md`** — All 4 factions (GDO, Syndicate, Cults, Colonists) and their resource definitions
- **`design/gdo_objects.md`** — All concrete GDO objects (Peacekeeper, PowerPlant, Barracks, DeploymentCenter, GDOBuildArea, ExtractionFacility, ExtractionPlate, SupplyTower, SupplyChopper)

### Old-Format Content

- **`design/to_be_converted.md`** — All bullet-point format content from the original file that has not been fully converted to formal entity style. Most of this duplicates formal content but may contain unique details (e.g., Supply Chopper transport capability, Syndicate tunnel mechanics, Cults recruitment details, Colonist beacon/warp-in flow).

### Removed Files

- **`design/design.md`** — Replaced by the above split files

## No Design Changes

This update is purely organizational. No design decisions were added, modified, or removed. All formal content was preserved exactly as written.
