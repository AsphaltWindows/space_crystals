# Declarative Entity Definitions Architecture

## Status: APPROVED (team consensus reached 2026-03-25)

## Summary

Consolidate scattered entity data (stats, costs, command panel layouts, spawn logic) into per-entity Rust files organized by faction, using a trait-based definition pattern that preserves compile-time safety.

## Motivation

Entity data is currently spread across 5+ files totaling ~5,948 lines:

| File | Lines | Content |
|------|-------|---------|
| `game/utils.rs` | 1,397 | 21 spawn functions |
| `game/types/objects.rs` | 1,312 | ObjectType/StructureType definitions |
| `game/units/types/unit_data.rs` | 740 | UnitTypeData, AttackAttributesData, per-unit constants |
| `game/types/structures.rs` | 2,312 | Structure stat constants (3 faction modules), StructureCost match arms, state components |
| `ui/command_panel/grid_layout.rs` | 187 | Command panel button layout per entity/state |

Adding a new entity requires touching 4-6 files, adding enum variants, match arms, spawn functions, command panel entries, and stat functions. This scattered pattern is the root cause of several large-file problems.

## Proposed Directory Structure

```
src/game/entities/
    mod.rs              -- Trait definitions, EntityRegistry, shared types
    registry.rs         -- build_registry() with explicit registration
    gdo/
        mod.rs
        units/
            mod.rs
            peacekeeper.rs
        buildings/
            mod.rs
            deployment_center.rs
            power_plant.rs
            barracks.rs
            extraction_facility.rs
            extraction_plate.rs
            supply_tower.rs
    syndicate/
        mod.rs
        units/
            mod.rs
            agent.rs
            guard.rs
        buildings/
            mod.rs
            tunnel.rs
            headquarters.rs
    cults/
        mod.rs
        units/
            mod.rs
            recruit.rs
            soldier.rs
            gunner.rs
        buildings/
            mod.rs
            recruitment_center.rs
            storage.rs
            armory.rs
    neutral/
        mod.rs
        space_crystals_patch.rs
        supply_delivery_station.rs
```

## Trait Design

### Option A: Single Trait with Optional Fields (simpler)

```rust
pub trait EntityDefinition {
    const OBJECT: ObjectEnum;
    fn object_type() -> ObjectType;
    fn stats() -> EntityStats;
    fn cost() -> Option<StructureCost> { None }
    fn attack_data() -> Option<AttackAttributesData> { None }
    fn command_slots() -> &'static [CommandSlotDef] { &[] }
    fn spawn(commands: &mut EntityCommands, params: SpawnParams);
}
```

`EntityStats` uses Options for unit-only / structure-only fields.

### Option B: Trait Hierarchy (more type-safe, recommended)

```rust
pub trait EntityDefinition {
    const OBJECT: ObjectEnum;
    fn object_type() -> ObjectType;
    fn base_stats() -> BaseStats;  // HP, armor, faction
    fn cost() -> Option<StructureCost> { None }
    fn command_slots() -> &'static [CommandSlotDef] { &[] }
    fn spawn(commands: &mut EntityCommands, params: SpawnParams);
}

pub trait UnitDefinition: EntityDefinition {
    fn unit_stats() -> UnitStats;  // unit_base, silhouette, control_cost
    fn attack_data() -> Option<AttackAttributesData> { None }
    fn movement_params() -> MovementParams;
}

pub trait StructureDefinition: EntityDefinition {
    fn structure_stats() -> StructureStats;  // power, build_radius, symmetry
}
```

No Options needed — the type system enforces that units have unit fields and structures have structure fields. The registry can store `Box<dyn EntityDefinition>` with runtime downcasting to `UnitDefinition` or `StructureDefinition` via `Any`.

**Recommendation: Option B.** The additional type safety justifies the slight complexity.

## EntityRegistry

```rust
pub struct EntityRegistry {
    entries: HashMap<ObjectEnum, RegistryEntry>,
}

struct RegistryEntry {
    object_type: ObjectType,
    base_stats: BaseStats,
    cost: Option<StructureCost>,
    command_slots: &'static [CommandSlotDef],
    // Dynamic spawn function pointer
    spawn_fn: fn(&mut EntityCommands, SpawnParams),
    // Kind-specific data
    kind: EntityKind,
}

enum EntityKind {
    Unit {
        unit_stats: UnitStats,
        attack_data: Option<AttackAttributesData>,
        movement_params: MovementParams,
    },
    Structure {
        structure_stats: StructureStats,
    },
    Resource,
}
```

The registry is built once at startup and inserted as a Bevy `Resource`. Systems query it instead of calling scattered functions.

```rust
// Registration
pub fn build_registry() -> EntityRegistry {
    let mut reg = EntityRegistry::new();
    reg.register_unit::<gdo::units::Peacekeeper>();
    reg.register_structure::<gdo::buildings::DeploymentCenter>();
    // ... all ~20 entities
    reg
}
```

## What Migrates Into Entity Files

Per entity file contains:
1. Stats (from `unit_data.rs` functions and `structures.rs` constants)
2. Attack data (from `unit_data.rs` `*_attack_data()` functions)
3. Cost data (from `structures.rs` `construction_cost()` / `production_cost()` match arms)
4. Command panel slots (from `grid_layout.rs` match arms)
5. Spawn function (from `utils.rs` `spawn_*()` functions)
6. Entity-specific constants (e.g., `AGENT_MINING_DURATION`, `AGENT_CRYSTAL_CARRY`)

## What Does NOT Migrate

- **Behavior systems** (`behaviors/*.rs`) — ECS systems implementing how behaviors work
- **State machine transitions** — system logic in `core/*.rs`
- **Command availability** — `UnitCommand::is_available()` (capability-based, not per-entity)
- **Shared component types** — `UnitCommand`, `CommandQueue`, `AttackState`, `MovementSpeed`, etc.
- **Production queue logic** — generic queue/cancel in `faction/production.rs` (reads costs from registry)
- **Pathfinding, combat systems** — frame-level simulation logic

## Implementation Phases

### Phase 1: Foundation (2-3 tasks)
- Create `src/game/entities/mod.rs` with trait definitions, shared types, `EntityRegistry`
- Create `registry.rs` with `build_registry()` (initially empty)
- Add `EntityRegistry` as a Bevy Resource in the game plugin
- Unit tests for registry operations

### Phase 2: Pilot Entity (1 task)
- Migrate Peacekeeper end-to-end into `entities/gdo/units/peacekeeper.rs`
- Register in registry
- Update one consumer (e.g., `spawn_peacekeeper`) to use registry
- Validate pattern works, adjust trait design if needed

### Phase 3: GDO Entities (2-3 tasks)
- Migrate all GDO units and structures
- Each entity file is self-contained
- Old code remains as pass-through wrappers during transition

### Phase 4: Syndicate Entities (2 tasks)
- Migrate Tunnel (most complex — tiered stats, construction variants)
- Migrate HQ, Agent, Guard

### Phase 5: Cults Entities (2 tasks)
- Migrate RecruitmentCenter, Storage, Armory
- Migrate Recruit, Soldier, Gunner

### Phase 6: Wire Registry Consumers (3-4 tasks)
- Replace `get_grid_slot_action()` match with registry lookup
- Replace `DeploymentCenterState::construction_cost()` with registry lookup
- Replace production cost lookups with registry
- Replace spawn function calls with registry dispatch

### Phase 7: Cleanup (1-2 tasks)
- Remove emptied functions from `utils.rs`, `unit_data.rs`, `objects.rs`
- Remove `gdo_structure_stats`, `syndicate_structure_stats`, `cults_structure_stats` modules
- Remove `grid_layout.rs` match arms (replaced by registry)

**Total: ~15-18 tasks**

## Testing Strategy

### Per-Entity Tests
Each entity file should have a `#[cfg(test)]` module verifying:
- Stats match design doc values (HP, armor, costs)
- Attack data is valid (ranges, durations > 0 where expected)
- Command slots don't overlap (no duplicate row/col for same state)
- Spawn function produces an entity with expected components

### Registry Tests
- All ObjectEnum variants have a registered entity
- Registry lookup returns correct data for each entity
- No duplicate ObjectEnum registrations

### Integration Tests
- Existing spawn-based tests continue to pass (spawn wrappers delegate to registry)
- Command panel grid layout matches current behavior
- Production cost lookups return same values as before

### Regression Guarantee
During transition, both old and new paths coexist. Integration tests verify behavioral equivalence before old code is removed.

## Risks

1. **Spawn function complexity**: Some spawns (Tunnel, Agent) have conditional logic. `SpawnParams` must be flexible enough, or entity files can have private helper methods.
2. **Command slot conditions**: Some slots are conditional (`bk_has_queue`, `has_active_construction`). `SlotCondition` enum must capture these.
3. **Trait object limitations**: Can't call associated consts on `dyn EntityDefinition`. Registry stores concrete values extracted at registration time.
4. **Large migration surface**: ~20 entities, ~5,948 lines across source files. Phased approach mitigates risk.

## Relationship to Other Refactors

- **File splits (done)**: Complementary. Splits broke up monoliths by concern; this distributes the data portions by entity.
- **Production deduplication (deferred)**: Becomes natural — registry provides generic cost/queue access, eliminating per-building-type queue code.
- **Dual movement pipeline (deferred)**: Unrelated. Movement logic stays in behavior systems.
