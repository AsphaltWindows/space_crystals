# entity-migration-syndicate-buildings

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate both Syndicate structures into per-entity definition files.

**Create 2 files:**
- src/game/entities/syndicate/buildings/tunnel.rs
- src/game/entities/syndicate/buildings/headquarters.rs

For each, implement EntityDefinition + StructureDefinition traits.

**Tunnel is the most complex entity.** It has:
- Tiered stats (tier 1-3 with different HP, capacity)
- Construction variants (under_construction with ConstructionHP)
- Multiple interface states (TunnelIdle, TunnelExpandMenu, TunnelEjectMenu, TunnelAwaitingPlacement)
- Tunnel-specific constants (upgrade costs, tier capacity, transit tier filtering)
- The SpawnParams.under_construction flag drives conditional component composition

Migrate from:
- Structure stats from structures.rs syndicate_structure_stats module
- Construction costs (tunnel_construction_cost scaling)
- Command panel slots from grid_layout.rs (all Tunnel interface states)
- Spawn logic from spawn_tunnel(), spawn_tunnel_under_construction(), spawn_headquarters() in utils.rs
- ObjectType data from objects.rs

Register both in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current syndicate_structure_stats values
- Tunnel tier stats are correct for all tiers
- Command slots match current grid_layout.rs entries for all Tunnel states
- HQ production costs match current values
- Spawn produces same components (including under_construction variant for Tunnel)

**Reference:** Architecture doc Phase 4 (Syndicate Entities)

## Technical Context

### Files to Create

1. **`artifacts/developer/src/game/entities/factions/syndicate/buildings/tunnel.rs`** — Tunnel entity definition
2. **`artifacts/developer/src/game/entities/factions/syndicate/buildings/headquarters.rs`** — Headquarters entity definition

### Files to Modify

3. **`artifacts/developer/src/game/entities/factions/syndicate/buildings/mod.rs`** — Add `pub mod tunnel;` and `pub mod headquarters;` (currently empty, just a doc comment at line 1)
4. **`artifacts/developer/src/game/entities/registry.rs`** — Add `use` imports for the new types and `registry.register_structure::<Tunnel>();` + `registry.register_structure::<Headquarters>();` in `build_registry()`. Update test assertion from `registry.len() == 2` to `4`. Currently imports only Peacekeeper and SupplyChopper (lines 5-6).

### Pattern to Follow — Peacekeeper (existing pilot)

Follow the Peacekeeper definition pattern at `artifacts/developer/src/game/entities/factions/gdo/units/peacekeeper.rs`:
- Constants section at top
- Static `COMMAND_SLOTS: &[CommandSlotDef]` array
- Marker struct (e.g., `pub struct Tunnel;`)
- `impl EntityDefinition for Tunnel { ... }` with `object_type()`, `base_stats()`, `cost()`, `command_slots()`, `spawn()`
- `impl StructureDefinition for Tunnel { ... }` with `structure_stats()` (structures use `StructureDefinition` instead of `UnitDefinition`)
- `#[cfg(test)] mod tests { ... }` with per-field assertions

### Tunnel Entity — Source Data Locations

**Stats** — `artifacts/developer/src/game/types/structures.rs` lines 442-465, `syndicate_structure_stats` module:
- `TUNNEL_T1_MAX_HP = 600.0`, `TUNNEL_T2_MAX_HP = 800.0`, `TUNNEL_T3_MAX_HP = 1000.0`
- `TUNNEL_POINT_ARMOR = 1`, `TUNNEL_FULL_ARMOR = 16`
- `TUNNEL_T1_SPACE = 20`, `TUNNEL_T2_SPACE = 30`, `TUNNEL_T3_SPACE = 40`
- `TUNNEL_T1_AREA_RADIUS = 3`, `TUNNEL_T2_AREA_RADIUS = 4`, `TUNNEL_T3_AREA_RADIUS = 5`
- `TUNNEL_SIGHT_RANGE = 5`
- `TUNNEL_CONSTRUCTION_FRAMES = 480`, `TUNNEL_UPGRADE_FRAMES = 480`

**ObjectType** — `artifacts/developer/src/game/types/objects.rs` line 289-295:
- name: "Tunnel", size: (4, 4), destructible: true, sight_range: 5, groupable: false

**Cost functions** — `artifacts/developer/src/game/types/structures.rs` lines 831-844:
- `tunnel_construction_cost(existing_count)` — returns `existing_count` (in Supplies)
- `tunnel_t2_upgrade_cost(existing_t2_plus_count)` — `2 + 2 * count`
- `tunnel_t3_upgrade_cost(existing_t3_count)` — `3 + 3 * count`
- Note: These are dynamic cost functions (depend on existing tunnel counts), NOT a fixed StructureCost. The `cost()` method should return `None` since tunnel costs aren't a fixed StructureCost. The dynamic cost functions should be re-exported as public functions from the entity file.

**Command panel slots** — `artifacts/developer/src/ui/command_panel/grid_layout.rs` lines 95-131:
- `TunnelIdle`: (0,0)=TunnelUpgrade, (0,1)=TunnelOpenExpandMenu, (0,2)=TunnelOpenEjectMenu, (2,1 if upgrading)=TunnelCancelUpgrade
- `TunnelExpandMenu`: (0,0)=TunnelSelectExpansion(Headquarters), (2,0)=Back
- `TunnelEjectMenu`: (2,0)=Back
- `TunnelAwaitingPlacement`: None (placement mode)
- Note: The TunnelIdle cancel slot is conditional (`tunnel_is_upgrading`). Currently the CommandSlotDef has a TODO for `condition: Option<SlotCondition>` — for now, include the conditional slot in the static array and add a comment noting it needs the condition mechanism.

**Spawn logic** — `artifacts/developer/src/game/utils.rs` lines 694-779:
- `spawn_tunnel()` (line 694): Inserts Transform, ObjectInstance::destructible, StructureInstance::default(), Owner, Selectable, SelectionBounds::from_dimensions(4.0, 1.0, 4.0), GridPosition, TunnelState::default_tier1(), TunnelArea::new(), SightRange(TUNNEL_SIGHT_RANGE), EjectionQueue::default(). Also spawns children (structure_label, side_labels) — these are VISUAL, skip in entity definition spawn.
- `spawn_tunnel_under_construction()` (line 738): Same as above but uses ObjectInstance::under_construction() and adds ConstructionHP::new(build_frames).
- Entity definition `spawn()` should use `params.under_construction` flag to decide between `ObjectInstance::destructible` vs `ObjectInstance::under_construction` and whether to insert `ConstructionHP`.

**Key component imports for spawn:**
- `ConstructionHP` — `crate::game::types::structures::ConstructionHP` (defined at structures.rs:11)
- `TunnelState` — `crate::game::types::structures::TunnelState`
- `TunnelArea` — `crate::game::types::structures::TunnelArea`
- `TunnelTier` — `crate::game::types::structures::TunnelTier`
- `EjectionQueue` — `crate::ui::types::EjectionQueue`
- `StructureInstance` — `crate::game::types::objects::StructureInstance`
- `ObjectInstance` — `crate::game::types::ObjectInstance` (re-export)
- `Selectable`, `SelectionBounds`, `GridPosition`, `SightRange` — `crate::shared::types`
- `Owner` — `crate::types::Owner`

**StructureStats:**
- power_value: 0 (Tunnels don't generate/consume power)
- build_radius: 0 (Tunnels provide build area through TunnelArea, not build_radius)
- symmetry: `SymmetryTypeEnum::ABCD`

### Headquarters Entity — Source Data Locations

**Stats** — `artifacts/developer/src/game/types/structures.rs` lines 459-465:
- `HQ_MAX_HP = 400.0`, `HQ_POINT_ARMOR = 1`, `HQ_FULL_ARMOR = 4`
- `HQ_SC_COST = 200`, `HQ_BUILD_FRAMES = 400`

**ObjectType** — `artifacts/developer/src/game/types/objects.rs` lines 296-302:
- name: "Headquarters", size: (2, 2), destructible: true, sight_range: 0 (underground), groupable: false

**Cost** — Fixed cost: `StructureCost { space_crystals: 200, build_frames: 400 }`

**Production costs** — `HeadquartersState::production_cost()` at structures.rs:272-284:
- SyndicateAgent: StructureCost { space_crystals: 100, build_frames: 160 }
- SyndicateGuard: StructureCost { space_crystals: 125, build_frames: 120 }
- Note: These are production costs of units the HQ builds — NOT the HQ's own construction cost. Keep these on HeadquartersState for now; the registry stores the HQ's own `cost()`.

**Command panel slots** — grid_layout.rs lines 88-94 (`HeadquartersMenu`):
- (0,0)=HqTrain(SyndicateAgent), (0,1)=HqTrain(SyndicateGuard), (2,1 if has_queue)=HqCancel, (2,2)=SetRallyPoint
- Note: HqCancel is conditional (`bk_has_queue` parameter) — same TODO for conditions as Tunnel.

**Spawn logic** — `artifacts/developer/src/game/utils.rs` lines 783-820:
- Inserts Transform, ObjectInstance::destructible(Headquarters, HQ_MAX_HP), StructureInstance::default(), Owner, Selectable, SelectionBounds::from_dimensions(2.0, 1.0, 2.0), GridPosition, DomainEnum::Underground, TunnelExpansionMarker { parent_tunnel }, HeadquartersState::default(), SightRange(3)
- Important: `parent_tunnel: Entity` is required but NOT in SpawnParams. Use `Entity::PLACEHOLDER` as default and document that callers must set TunnelExpansionMarker separately, OR skip inserting TunnelExpansionMarker in the entity definition spawn (leave it for the caller).
- Children: spawn_structure_label("Headquarters") — visual, skip.

**Additional HQ component:** `DomainEnum::Underground` — from `crate::shared::types::DomainEnum`

**StructureStats:**
- power_value: 0 (underground, no power grid interaction)
- build_radius: 0
- symmetry: `SymmetryTypeEnum::AAAA` (2x2 symmetric)

### Registration in registry.rs

Add to `build_registry()` (at `artifacts/developer/src/game/entities/registry.rs`):
```rust
use super::factions::syndicate::buildings::tunnel::Tunnel;
use super::factions::syndicate::buildings::headquarters::Headquarters;
// ...
registry.register_structure::<Tunnel>();
registry.register_structure::<Headquarters>();
```

Update the existing test `test_build_registry_has_peacekeeper` assertion from `assert_eq!(registry.len(), 2)` to `assert_eq!(registry.len(), 4)` and add lookups for Tunnel and Headquarters entries.

### Spawn Function Caveat — SpawnParams Limitations

`SpawnParams` (types.rs:56) has: position, owner, player, rotation, under_construction. It does NOT have:
- `parent_tunnel: Entity` (needed by HQ's TunnelExpansionMarker)
- `build_frames: u32` (needed by ConstructionHP::new() for under-construction tunnel)

**Recommended approach:** 
- For Tunnel: use `TUNNEL_CONSTRUCTION_FRAMES` constant for the default build_frames in ConstructionHP when `under_construction` is true.
- For HQ: skip inserting `TunnelExpansionMarker` in the spawn function — callers will insert it separately. Document this clearly.
- Do NOT modify SpawnParams in this task — a future task may add extensibility if needed.

### Test Structure

Each entity file should have `#[cfg(test)] mod tests` following the Peacekeeper pattern:
- `tunnel_base_stats()` — verify T1 HP (600), armor (1/16), faction (Syndicate)
- `tunnel_object_type()` — verify name, size (4,4), destructible, sight_range 5, not groupable
- `tunnel_structure_stats()` — verify power=0, build_radius=0, symmetry=ABCD
- `tunnel_command_slots_count()` — count all static slots across all states
- Individual slot position tests for key slots
- `tunnel_cost_is_none()` — verify cost() returns None (dynamic cost functions)
- `tunnel_registry_lookup()` — register and look up in registry
- `headquarters_base_stats()`, `headquarters_object_type()`, `headquarters_cost()` etc.
- `headquarters_cost_value()` — verify space_crystals=200, build_frames=400

## Dependencies

- **Entity definition foundation** (already complete) — The `EntityDefinition`, `StructureDefinition` traits, `EntityRegistry`, `SpawnParams`, `CommandSlotDef`, `BaseStats`, `StructureStats` types all exist in `artifacts/developer/src/game/entities/types.rs`. The `StructureDefinition` registration path (`register_structure::<T>()`) is implemented and tested.
- **Peacekeeper + SupplyChopper pilots** (already complete) — These establish the pattern to follow. See `artifacts/developer/src/game/entities/factions/gdo/units/peacekeeper.rs` and `supply_chopper.rs`.
- No dependency on other migration tasks (GDO buildings, Cults, Neutral) — migrations are independent of each other.
