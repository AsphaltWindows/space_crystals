# entity-registry-spawn-dispatch

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Replace per-entity spawn function calls throughout the codebase with registry-based spawn dispatch.

**Current state:** Callers use specific functions like `spawn_peacekeeper()`, `spawn_syndicate_agent()`, `spawn_tunnel()`, etc. These are currently thin wrappers delegating to entity definitions (set up by migration tasks).

**After this task:** A single generic spawn path via EntityRegistry:
1. Add a `spawn(&self, object: &ObjectEnum, commands: &mut EntityCommands, params: SpawnParams)` method to EntityRegistry that dispatches to the registered spawn_fn
2. Update all spawn call sites (production systems in faction/, construction systems, initial game setup) to use `registry.spawn()` instead of per-entity functions
3. The thin wrapper functions in utils.rs and unit_data.rs can remain temporarily (removed in cleanup task) but all callers should go through the registry

**Call sites to update:**
- Production tick systems (barracks_production_tick, headquarters_production_tick, etc.) — spawn produced units via registry
- Construction completion (deployment_center construction) — spawn completed structures via registry
- Initial game setup / scenario spawning — use registry
- Any test helpers that spawn entities

**Tests:**
- All spawn call sites use registry dispatch
- Spawned entities have identical components to pre-refactor
- All existing spawn-related tests pass
- Production and construction systems work identically

**Reference:** Architecture doc Phase 6 (Wire Registry Consumers)

## Technical Context

### 1. EntityRegistry — Add `spawn()` convenience method

**File:** `artifacts/developer/src/game/entities/types.rs` (lines 120-207)

`EntityRegistry` currently has `get()`, `len()`, `is_empty()`. Add a `spawn()` method:

```rust
/// Spawn an entity of the given type using the registered spawn function.
/// Returns the entity id. Panics if `object` is not registered.
pub fn spawn(&self, object: &ObjectEnum, commands: &mut Commands, params: SpawnParams) -> Entity {
    let entry = self.entries.get(object)
        .unwrap_or_else(|| panic!("No registry entry for {:?}", object));
    let entity = commands.spawn_empty().id();
    (entry.spawn_fn)(&mut commands.entity(entity), params);
    entity
}
```

Note the signature difference: `RegistryEntry.spawn_fn` is `fn(&mut EntityCommands, SpawnParams)` (line 110). The `spawn()` method must first `commands.spawn_empty()` then pass `&mut commands.entity(entity)` to the spawn_fn. This mirrors how the existing `spawn_*` functions in `utils.rs` call `commands.spawn((...))`.

### 2. SpawnParams gap — extra parameters

**File:** `artifacts/developer/src/game/entities/types.rs` (lines 54-62)

Current `SpawnParams` has: `position: Vec3`, `owner: Owner`, `player: u8`, `rotation: Option<StructureRotation>`, `under_construction: bool`.

Several spawn call sites pass extra parameters not in SpawnParams:
- `spawn_headquarters()` needs `parent_tunnel: Entity` — for `TunnelExpansionMarker { parent_tunnel }`
- `spawn_extraction_plate()` needs `attached_patch: Entity` — for `ExtractionPlateState { attached_patch, ... }`
- `spawn_tunnel_under_construction()` needs `build_frames: u32` — for `ConstructionHP::new(build_frames)`
- `spawn_cults_storage_under_construction()` needs flip params — already handled by rotation + under_construction

**Approach:** For entities needing extra params (HQ, EP, Tunnel under construction), the caller should:
1. Call `registry.spawn()` to get the base entity
2. Then `commands.entity(entity).insert(ExtraComponent)` for the additional components

This means the entity definition's `spawn()` must NOT require those extra params. Check the existing entity definition files:
- `ExtractionPlate` definition (in `artifacts/developer/src/game/entities/factions/gdo/buildings/extraction.rs`) — if its spawn fn inserts `ExtractionPlateState`, it will need `Entity::PLACEHOLDER` for the patch, which the caller overwrites
- Headquarters, Tunnel entity definitions don't exist yet (Syndicate buildings stub at `artifacts/developer/src/game/entities/factions/syndicate/buildings/mod.rs` is empty)

**Important:** Syndicate buildings (Tunnel, HQ) and Cults buildings (RC, Storage, Armory) do NOT have entity definition files yet — only stub `mod.rs` files exist. These were covered by migration tasks (`entity-migration-syndicate-buildings`, `entity-migration-cults-buildings`) which are in `done/`. Verify these entity definitions exist before wiring spawn dispatch. If they don't yet have `spawn()` implementations, this task must add them or the registry won't have entries for these ObjectEnum variants.

### 3. Production systems — spawn call sites

**File:** `artifacts/developer/src/game/world/faction/production.rs`

All production systems need `Res<EntityRegistry>` added to their system params.

#### Barracks production (lines 17-86)
- Currently: `spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, spawn_x, spawn_z, *owner)` (line 53)
- After: Compute world position from grid, call `registry.spawn(&ObjectEnum::Peacekeeper, &mut commands, SpawnParams { position, owner, .. })`
- Grid→world: Use `grid_to_world(spawn_x, spawn_z, 1.0)` from `game::world::utils` (returns Vec3 with y=0, but unit spawn wants y=0.5; the entity definition's `spawn()` already adds the Y offset — see peacekeeper.rs line 131: `params.position + Vec3::new(0.0, 0.5, 0.0)`)
- After spawning, still need `commands.entity(unit_entity).insert(...)` for rally command — unchanged
- The `meshes`/`materials` params become unnecessary for the spawn call itself (visual components are NOT inserted by entity def spawn — those are deferred to a separate visual system or omitted per architecture doc)

**CRITICAL OBSERVATION:** The entity definition `spawn()` does NOT insert `Mesh3d`/`MeshMaterial3d` (no asset handles in SpawnParams). But the current `spawn_peacekeeper()` in `utils.rs` DOES insert mesh/material. If production systems switch to registry spawn, entities will lack visual components. This is an intentional design choice per the architecture doc, but it means:
- Either a separate "visual spawning" system must exist that adds Mesh3d/MeshMaterial3d to newly spawned entities
- Or the entity definition `spawn()` methods must be updated to include visuals (which they currently don't, since `SpawnParams` has no asset handles)

**Resolution:** Check if a visual spawn system already exists. If not, the `meshes`/`materials` params stay on the production systems and the caller does `registry.spawn()` for gameplay components, then inserts visual components separately. This is the pattern from the architecture doc — "visual components (Mesh3d, MeshMaterial3d) need caller-side or separate system handling."

#### HQ production (lines 187-286)
- Lines 228-236, 261-269: Match on `ObjectEnum::SyndicateAgent`/`SyndicateGuard` to pick spawn function
- After: `registry.spawn(&unit_type, &mut commands, SpawnParams { position, owner, .. })` — no match needed
- Same visual component concern applies

#### Supply Tower production (lines 289-367)
- Line 325: `spawn_supply_chopper(...)` — same pattern

#### RC production (lines 376-446)
- Line 412: `spawn_cults_recruit(...)` — same pattern

#### Armory training (lines 448-500)
- Lines 475-483: Match on `CultsSoldier`/`CultsGunner` — replaced by single `registry.spawn()`

### 4. Construction completion — spawn call site

**File:** `artifacts/developer/src/game/world/faction/construction.rs` (line 184)
- `spawn_headquarters(...)` with `tunnel_entity` param
- After: `registry.spawn(&ObjectEnum::Headquarters, &mut commands, params)` then `commands.entity(hq).insert(TunnelExpansionMarker { parent_tunnel: tunnel_entity })`

### 5. Placement systems — spawn call sites

**File:** `artifacts/developer/src/game/world/faction/placement.rs` (lines 340-536)

Major call sites:
- Lines 358-403: DC placement — `spawn_power_plant`, `spawn_barracks`, `spawn_extraction_facility`, `spawn_supply_tower`, `spawn_supply_chopper`
- Lines 426-432: EF placement — `spawn_extraction_plate` (needs `attached_patch` post-insert)
- Lines 518-522: Cults placement — `spawn_cults_storage_under_construction`

All can switch to `registry.spawn()` + post-insert for extra components. The rotation/flip params are already in `SpawnParams.rotation` (but `flip_horizontal`/`flip_vertical` are NOT in SpawnParams — these are in `StructureInstance` which the entity def's spawn fn constructs from `params.rotation`). 

**NOTE:** `SpawnParams` currently lacks `flip_horizontal`/`flip_vertical` fields. These are used by `StructureInstance { rotation, flip_horizontal, flip_vertical }` in many structure spawn functions. Either:
- Add `flip_horizontal: bool, flip_vertical: bool` to `SpawnParams`
- Or have callers post-insert `StructureInstance` with flips after the registry spawn

Recommend adding flip fields to `SpawnParams` since most structures need them.

### 6. Initial game setup — spawn call sites

**File:** `artifacts/developer/src/game/world/faction/setup.rs`
- Lines 115-118: `spawn_deployment_center(...)` 
- Lines 145-148: `spawn_tunnel(...)`
- Lines 154-158: `spawn_headquarters(...)` (needs `tunnel_entity` post-insert)
- Lines 182-185: `spawn_recruitment_center(...)` (needs `build_order` — currently a param of `spawn_recruitment_center`)
- Lines 269: `spawn_peacekeeper(...)` in test enemy spawn

Setup systems need `Res<EntityRegistry>` added. The `meshes`/`materials` system params may still be needed for visual components.

### 7. Tunnel construction behavior — spawn call site

**File:** `artifacts/developer/src/game/units/systems/behaviors/construction.rs` (line 206)
- `spawn_tunnel_under_construction(...)` with `build_frames` param
- After: `registry.spawn(&ObjectEnum::Tunnel, &mut commands, SpawnParams { under_construction: true, .. })` then `commands.entity(tunnel).insert(ConstructionHP::new(build_frames))`

### 8. Test utility spawner

**File:** `artifacts/developer/src/game/units/systems/core/utility.rs` (lines 22-36)
- `spawn_peacekeeper`, `spawn_syndicate_agent`, `spawn_syndicate_guard` calls
- Add `Res<EntityRegistry>` to `spawn_test_units` system params

### 9. Test harness

**File:** `artifacts/developer/src/shared/testing/harness.rs` (lines 61-213)
- `spawn_unit_at_grid()` and `spawn_structure_at_grid()` dispatch to `spawn_*` functions
- These should be updated to use `EntityRegistry` from the test app's world
- Pattern: `let registry = self.app.world().resource::<EntityRegistry>();` then call `registry.spawn()`
- **Caveat:** Harness methods take `&mut self` and call `self.app.world_mut().run_system_once(...)` — the registry lookup needs to happen inside the system closure, not outside

### 10. Test-local spawn helpers (NOT updated)

Files like `behaviors/resource_gathering.rs` (line 519) and `behaviors/tunnel_interaction.rs` (line 280) have private test helpers like `fn spawn_tunnel(world: &mut World, ...)`. These create minimal entities for unit tests and do NOT go through game spawn functions. Leave these alone — they are test fixtures that deliberately create minimal entities.

### Key Patterns to Follow

1. **Grid-to-world conversion:** For units: `grid_to_world(grid_x, grid_z, 1.0)` from `game::world::utils` gives the center of a 1x1 cell. For structures with size > 1: `(grid_x as f32 - 32.0) + (size_x as f32) / 2.0` for X (and similar for Z). The entity definition spawn fns already expect world-space `position` in SpawnParams.

2. **Mesh/Material handling:** Entity definition `spawn()` fns do NOT insert visual components. Callers still need `meshes`/`materials` system params to add visuals after the registry spawn. This is by design — visual component assembly will eventually move to a dedicated system.

3. **Post-insert pattern:** For entities needing extra components beyond what `SpawnParams` covers:
   ```rust
   let entity = registry.spawn(&object, &mut commands, params);
   commands.entity(entity).insert(ExtraComponent { ... });
   ```

4. **System param addition:** Every system that spawns entities needs `Res<EntityRegistry>` added. This is a Bevy resource, automatically available after `EntitiesPlugin` runs.

### Files to modify (summary)

| File | Changes |
|------|---------|
| `game/entities/types.rs` | Add `spawn()` method to `EntityRegistry`; add `flip_horizontal`/`flip_vertical` to `SpawnParams` |
| `game/world/faction/production.rs` | Add `Res<EntityRegistry>` to 5 systems; replace spawn function calls |
| `game/world/faction/construction.rs` | Add `Res<EntityRegistry>` to `tunnel_expansion_tick_system`; replace `spawn_headquarters` |
| `game/world/faction/placement.rs` | Add `Res<EntityRegistry>` to `placement_click_system`; replace 7+ spawn calls |
| `game/world/faction/setup.rs` | Add `Res<EntityRegistry>` to 4 setup systems; replace 5+ spawn calls |
| `game/units/systems/behaviors/construction.rs` | Add `Res<EntityRegistry>` to `building_tunnel_behavior_system`; replace `spawn_tunnel_under_construction` |
| `game/units/systems/core/utility.rs` | Add `Res<EntityRegistry>` to `spawn_test_units`; replace 3 spawn calls |
| `shared/testing/harness.rs` | Update `spawn_unit_at_grid`/`spawn_structure_at_grid` to use registry |
| Entity definition files for Syndicate buildings + Cults buildings | Verify spawn impls exist; add if missing |

### Bevy ECS Considerations

- `Res<EntityRegistry>` is read-only — no mut needed, no system ordering concerns
- `EntitiesPlugin` inserts the registry at startup (Startup schedule) — available in all Update/FixedUpdate systems
- `commands.spawn_empty()` + `commands.entity(id).insert(...)` is equivalent to `commands.spawn((...))` — both create deferred commands applied between system sets
- No ordering changes needed — the registry is just data lookup, not a system

## Dependencies

1. **entity-migration-syndicate-buildings** (done) — Syndicate building entity definitions must exist with `spawn()` implementations for Tunnel, Headquarters
2. **entity-migration-cults-buildings** (done) — Cults building entity definitions must exist with `spawn()` implementations for RecruitmentCenter, Storage, Armory
3. **entity-migration-gdo-buildings** (done) — GDO building entity definitions already have spawn implementations (verified: DC, PP in `gdo/buildings/base.rs`)
4. **entity-migration-gdo-units** (done) — GDO unit entity definitions already have spawn implementations (verified: Peacekeeper in `gdo/units/peacekeeper.rs`)
5. **entity-migration-syndicate-units** (done) — Syndicate unit spawn impls needed
6. **entity-migration-cults-units** (done) — Cults unit spawn impls needed
7. **entity-migration-neutral** (done) — Neutral resource entity spawn impls needed
8. **entity-definition-foundation** (done) — EntityRegistry, traits, SpawnParams exist

**Warning:** Despite migration tasks being marked done, the Syndicate buildings and Cults buildings `mod.rs` files are stubs with no entity definition files. The developer should verify these exist before proceeding, and create them if needed (following the pattern from `gdo/buildings/base.rs`).
