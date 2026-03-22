# unit_crushing_mechanic

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-unit_bases_movement_collision_r1.md

## Task

Implement the unit crushing mechanic: TrackedVehicle and Mech units crush enemy LightInfantry on contact.

### Behavior

Per design doc, crushing occurs when a unit with the `can_crush` property (TrackedVehicle, Mech — see `UnitBaseData`) moves over an enemy unit with the `crushable` property (LightInfantry only).

### Implementation

1. Create a `crushing_system` that runs in Phase 3 (after movement systems) or Phase 4:
   - Query all units with `UnitBaseEnum`, `Transform`, `Owner`, `Silhouette`
   - For each unit whose `UnitBaseData` has a can-crush property (TrackedVehicle data has `crushable: false` but that's the crushable flag — need to check which bases CAN crush), check overlap with enemy LightInfantry units
   - Overlap check: AABB overlap between crusher silhouette and crushee silhouette at their current positions
   - On overlap: instantly kill the crushable unit (set HP to 0 or despawn via existing `remove_dead_entities_system`)
   - Only crush ENEMY units (different `Owner`)

2. The crushing property mapping from the design doc:
   - TrackedVehicle: crushes LightInfantry
   - Mech: crushes LightInfantry
   - LightInfantry: crushable (already `crushable: true` in `UnitBaseData`)
   - HeavyInfantry: NOT crushable (`crushable: false`)
   - All others: neither crush nor are crushable

3. Register the system in `UnitsPlugin` after movement systems complete.

### Notes
- The `UnitBaseData` struct already has a `crushable` field. You may need to add a `can_crush` field or derive it from the unit base enum directly (TrackedVehicle and Mech are the only crushers per design).
- Check `UnitBaseEnum::data()` in `movement.rs` for the existing `crushable` field.

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/game/units/types/movement.rs`** — Add `can_crush: bool` field to `UnitBaseData` struct (line 287). Set `can_crush: true` for `TrackedVehicle` (line 332) and `Mech` (line 362), `false` for all others. Update all 9 existing unit tests (`test_light_infantry_data` etc.) to assert `can_crush`. Add a new `only_tracked_and_mech_can_crush` test mirroring the existing `only_light_infantry_is_crushable` test (line 653).

2. **`artifacts/developer/src/game/units/systems/core.rs`** — Add the `crushing_system` function. This file already contains all movement systems and collision logic. Pattern to follow: look at `air_unit_separation_system` (line ~1795) for a similar dual-query system that compares pairs of units.

3. **`artifacts/developer/src/game/units/mod.rs`** — Register `crushing_system` in `UnitsPlugin`. Place it after Phase 3 movement systems but before Phase 4 `grid_position_sync_system`. Add an `.after()` constraint on all movement systems so crushing checks happen on final positions.

### Key Types and Components

- **`UnitBaseEnum`** (Component): Enum with 9 variants. Use `.data()` to get `UnitBaseData` with `crushable` and the new `can_crush` fields.
- **`UnitBaseData`** (struct in `movement.rs:287`): Static data per unit base. Already has `crushable: bool`, add `can_crush: bool`.
- **`Silhouette`** (Component in `combat/types.rs:194`): Has `width: f32` and `height: f32` — use for AABB overlap between crusher and crushee.
- **`Owner`** (Component in `shared/types.rs:32`): `Owner(Option<u8>)`. Compare with `!=` to determine enemy units.
- **`ObjectInstance`** (Component): Has `apply_damage(amount)` method (line 104) which reduces HP. Call with max HP to instantly kill. Also has `is_alive()` to check if unit is still alive.
- **`Unit`** (marker Component in `shared/types.rs:28`): Use `With<Unit>` filter in queries.
- **`Transform`**: Use `translation` for position. Crushing is AABB overlap in XZ plane.
- **`InTunnelNetwork`** (marker Component): Units inside the tunnel network should NOT be crushable — filter with `Without<InTunnelNetwork>`.

### Crushing System Design

```rust
pub fn crushing_system(
    mut commands: Commands,
    crushers: Query<(&Transform, &UnitBaseEnum, &Owner, &Silhouette), (With<Unit>, Without<InTunnelNetwork>)>,
    mut crushables: Query<(Entity, &Transform, &UnitBaseEnum, &Owner, &Silhouette, &mut ObjectInstance), (With<Unit>, Without<InTunnelNetwork>)>,
) {
    for (c_transform, c_base, c_owner, c_sil) in crushers.iter() {
        if !c_base.data().can_crush { continue; }
        for (entity, t_transform, t_base, t_owner, t_sil, mut obj) in crushables.iter_mut() {
            if !t_base.data().crushable { continue; }
            if c_owner == t_owner { continue; }  // only crush enemies
            if !obj.is_alive() { continue; }
            // AABB overlap check in XZ plane
            let cx = c_transform.translation.x;
            let cz = c_transform.translation.z;
            let tx = t_transform.translation.x;
            let tz = t_transform.translation.z;
            let overlap_x = (c_sil.width/2.0 + t_sil.width/2.0) - (cx - tx).abs();
            let overlap_z = (c_sil.height/2.0 + t_sil.height/2.0) - (cz - tz).abs();
            if overlap_x > 0.0 && overlap_z > 0.0 {
                // Crush! Apply lethal damage
                obj.apply_damage(f32::MAX);
            }
        }
    }
}
```

**Note on query overlap**: The two queries above (`crushers` and `crushables`) both query `With<Unit>` but the `crushables` query has `&mut ObjectInstance` which makes it a mutable query. Since a TrackedVehicle is NOT crushable (crushable=false), the `continue` on `t_base.data().crushable` handles the logical overlap. However, Bevy may still enforce disjointness — if it does, make the crushers query also have `&ObjectInstance` (immutable), or use a single query and iterate with `iter_combinations`, or check if the entity is the same and skip. Alternatively, split by `can_crush` being a component marker rather than a data field, but the simplest approach is to use `Query::iter()` for crushers and `Query::get_mut()` for crushables.

### Death Pipeline

When `obj.apply_damage(f32::MAX)` sets HP to 0, the existing `remove_dead_entities_system` (in `combat/systems/core.rs:757`) will pick up the entity on the next frame (checks `!obj.is_alive()`) and despawn it, handling unit control cost cleanup. No additional despawn logic needed.

### System Registration Pattern

In `mod.rs`, add after the Phase 3 movement block (around line 57):

```rust
// Phase 3.5: Crushing check — after movement, before grid sync
systems::core::crushing_system
    .after(systems::core::unit_movement_system)
    .after(systems::core::turn_rate_movement_system)
    .after(systems::core::fixed_turn_radius_movement_system)
    .after(systems::core::speed_turn_radius_movement_system)
    .after(systems::core::channel_turnrate_locomotion_system)
    .after(systems::core::channel_fallback_locomotion_system),
```

Also add `.after(systems::core::crushing_system)` to `grid_position_sync_system` to maintain ordering.

### Testing

Write tests in `core.rs` test module (starts at line ~1375). Follow existing test patterns:
1. **Crush occurs**: Spawn a TrackedVehicle and LightInfantry with overlapping positions, different owners. Assert LightInfantry HP goes to 0 after system runs.
2. **No friendly crush**: Same setup but same owner. Assert LightInfantry survives.
3. **Non-crushable survives**: TrackedVehicle overlaps HeavyInfantry. Assert HeavyInfantry survives.
4. **Non-crusher no effect**: WheeledVehicle overlaps LightInfantry. Assert LightInfantry survives.
5. **No overlap no crush**: TrackedVehicle and LightInfantry far apart. Assert LightInfantry survives.
6. **Mech crushes**: Same as test 1 but with Mech unit base.

Use `Silhouette { width: 1.0, height: 1.0 }` for test entities (matching existing test patterns at line 4318).

## Dependencies

- **Movement systems** (existing, Phase 3): Crushing checks must run AFTER all movement systems complete, so units are at their final positions for the frame. The system uses `.after()` constraints on all 6 movement systems.
- **`remove_dead_entities_system`** (existing, in `combat/systems/core.rs:757`): Handles despawning crushed units on subsequent frames. No code changes needed — it already checks `!obj.is_alive()`.
- **`UnitBaseData.can_crush` field** (new, added in this same task): The `crushing_system` depends on this field existing. Both changes are part of this single task.
