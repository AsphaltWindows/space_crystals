# Task 002: Define Scale Constants and Core Entity Type Hierarchy

## Status
**Completed** - 2026-02-22

Added FRAMES_PER_SECOND (16) and SPACE_UNITS_PER_GRID_UNIT (64) constants. Defined 9 identity enums: FactionEnum, ObjectEnum, UnitBaseEnum, MovementModelEnum, AttackTypeEnum, TargetDomainEnum, TargetTypeEnum, DomainEnum, VisibilityStateEnum. Renamed Faction → FactionEnum with updated variant names (TheSyndicate, TheCults). Existing UnitBase and AttackType kept for runtime compatibility; new identity enums coexist.

## Description
Define the foundational type system that mirrors the top of the design document's concept hierarchy. This includes the simulation scale constants, the abstract Entity hierarchy, and the core enums (FactionEnum, ObjectEnum) that will be referenced throughout the codebase.

## Why Needed
The design document defines a formal concept hierarchy starting with Scale (SimulationFrame, GridUnit, SpaceUnit), then Entity → Invisible Entity / Visible Entity. These foundational types are prerequisites for all other game types. The current codebase has ad-hoc versions of some of these — they need to be replaced with design-aligned definitions.

## Acceptance Criteria
- Scale constants defined and accessible:
  - `FRAMES_PER_SECOND: u32 = 16`
  - `SPACE_UNITS_PER_GRID_UNIT: u32 = 64`
- `FactionEnum` enum with variants: `GlobalDefenseOrdinance`, `TheSyndicate`, `TheCults`, `Colonists`
- `ObjectEnum` enum with variants for all 6 GDO objects: `Peacekeeper`, `PowerPlant`, `Barracks`, `DeploymentCenter`, `ExtractionFacility`, `ExtractionPlate` (plus `SpaceCrystalsPatch` and `SupplyDeliveryStation` for the neutral resource objects)
- `UnitBaseEnum` enum with all 9 variants: `LightInfantry`, `HeavyInfantry`, `WheeledVehicle`, `TrackedVehicle`, `DrillUnit`, `HoverVehicle`, `Mech`, `HoverCraft`, `Glider`
- `MovementModelEnum` enum with 5 variants: `TurnRate`, `FixedTurnRadius`, `SpeedTurnRadius`, `Drag`, `Glider`
- `AttackTypeEnum` enum with 4 variants: `FullyConnected`, `HeadDisjointed`, `TailDisjointed`, `DoublyDisjointed`
- `TargetDomainEnum` enum: `Ground`, `Air`, `Universal`
- `TargetTypeEnum` enum: `SingleTarget`, `AoE`
- `DomainEnum` enum: `Ground`, `Air`, `Underground`
- `VisibilityStateEnum` enum: `Unexplored`, `Explored`, `Visible`
- Replace existing `Faction` enum in faction.rs with the new `FactionEnum`
- All enums derive appropriate Bevy traits: `Component`, `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash` as appropriate
- Project compiles and runs after changes

## Relevant Files/Components
- Current `src/faction.rs` — has existing `Faction` enum
- Current `src/combat.rs` — has existing `AttackType` and `AttackPhase` enums
- Current `src/units.rs` — has existing `UnitBase` enum
- Whichever types.rs file is appropriate after task_001 restructuring

## Technical Considerations
- These are data-definition enums — they define the vocabulary of the game. Place them in the appropriate `types.rs` per the reorganized directory structure.
- The existing `Faction` enum has 4 variants already but may have different naming. Align exactly with design: `GlobalDefenseOrdinance`, `TheSyndicate`, `TheCults`, `Colonists`.
- The existing `UnitBase` enum is a complex enum with inline data. The new design separates the enum identity (UnitBaseEnum) from the attribute data. Replace the existing enum with the clean identity enum; attributes will be defined in a later task.
- The existing `AttackType` enum has inline data (projectile_speed, etc.). The new design separates enum identity from attributes. Replace with clean identity enum.
- `ObjectEnum` will grow as more factions' objects are defined. For now include only GDO objects + neutral resources.
- Update all existing code that references the old enums to use the new ones. This may require temporary adapter code.

## Prerequisites
- [ ] `task_001.md` — Directory structure must be in place for proper file placement

## Complexity
Medium
