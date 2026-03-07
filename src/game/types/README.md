# game/types/

Game-level type definitions shared across game subsystems.

## Files

- **factions.rs** — Faction definitions (FactionEnum, FactionMember, Player) and per-faction player resource types (GdoPlayerResources, SyndicatePlayerResources, etc.)
- **objects.rs** — Object type hierarchy: ObjectType (static data), StructureType, ObjectInstance (ECS component), StructureInstance, and ObjectEnum lookup methods
- **structures.rs** — GDO structure state components (DeploymentCenterState, BarracksState, SupplyTowerState, SupplyChopperState), Syndicate structure types (TunnelState, TunnelTier, TunnelArea, TunnelOperation, TransitTier), PowerValue, BuildRadiusExtension, StructureCost, RallyTarget, cost functions, and stat constants
- **types.rs** — Shared type aliases (convention file)
- **utils.rs** — Shared utility functions (convention file)
