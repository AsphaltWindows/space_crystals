# infopanel_fog_visibility_gate

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-infopanel_fog_of_war_visibility.md

## Task

Add fog-of-war visibility gating to InfoPanel fields in ui/hud.rs.

Currently, update_selected_units_grid_system renders all InfoPanel fields for SpaceCrystalPatch and SupplyDeliveryStation regardless of fog state. Two fields must be gated:

1. SpaceCrystalPatch RemainingAmount (line ~729-733): Only display when the patch is in VisibilityStateEnum::Visible for the local player. When in Explored state, replace with a placeholder (e.g., '???') or omit the remaining/initial/percentage lines.

2. SupplyDeliveryStation CurrentSupplies (line ~807-812): Only display when the SDS is in VisibilityStateEnum::Visible. DeliverySize and DeliveryInterval always display. When in Explored state, omit the CurrentSupplies line and the 'Next delivery' countdown.

Implementation:
- Add FogOfWarMap as a Res parameter to update_selected_units_grid_system
- Add GridPosition to the selected_scp and selected_sds query tuples
- Add a query or resource for the local player's PlayerNumber (needed to index into FogOfWarMap)
- Before rendering gated fields, look up the entity's grid position in FogOfWarMap for the local player. If the tile is not Visible, skip those fields or show '???'
- The ExtractionPlate info panel (line ~487-500) also shows remaining_amount from the attached patch — apply the same visibility gate there (check the EP's own grid position)

Reference: artifacts/designer/design/entities.md lines 197-207 for the InfoPanel visibility rules.

## Technical Context

### Files to modify

**`artifacts/developer/src/ui/hud.rs`** — the only file that needs changes.

### System signature changes (line 223)

`update_selected_units_grid_system` needs three new parameters:

1. `fog_map: Res<FogOfWarMap>` — import from `crate::game::world::types::FogOfWarMap` (add to line 5 import: `use crate::game::world::types::{SpaceCrystalPatch, SupplyDeliveryStation, FogOfWarMap};`)
2. `local_player: Res<LocalPlayer>` — already available via `crate::types::*` (line 2 wildcard import includes `LocalPlayer` which is defined in `shared/types.rs:18`)
3. Query changes to add `&GridPosition`:

Current queries (lines 228-230):
```rust
sc_patches: Query<&SpaceCrystalPatch>,
selected_scp: Query<(Entity, &SpaceCrystalPatch, &ObjectInstance), (With<Selected>, Without<Unit>, Without<StructureInstance>)>,
selected_sds: Query<(Entity, &SupplyDeliveryStation, &ObjectInstance), (With<Selected>, Without<Unit>, Without<StructureInstance>, Without<SpaceCrystalPatch>)>,
```

Change to:
```rust
sc_patches: Query<(&SpaceCrystalPatch, &GridPosition)>,
selected_scp: Query<(Entity, &SpaceCrystalPatch, &ObjectInstance, &GridPosition), (With<Selected>, Without<Unit>, Without<StructureInstance>)>,
selected_sds: Query<(Entity, &SupplyDeliveryStation, &ObjectInstance, &GridPosition), (With<Selected>, Without<Unit>, Without<StructureInstance>, Without<SpaceCrystalPatch>)>,
```

Both SCP and SDS entities are spawned with `GridPosition` (resources.rs lines 66 and 596), so this query addition is safe.

### FogOfWarMap API

- `FogOfWarMap` is a `Resource` in `game/world/types.rs:351`
- `fog_map.get(player_id: u8, x: i32, z: i32) -> VisibilityStateEnum` — returns Unexplored if player has no map or position is OOB
- `LocalPlayer(pub u8)` — the `u8` value is the player_id to pass to `fog_map.get()`
- `VisibilityStateEnum` has variants: `Unexplored`, `Explored`, `Visible` (shared/types.rs:435, already in scope via line 2 wildcard import)

### Visibility helper pattern

Create a small helper closure or inline the check at each gate site:

```rust
let is_visible = |gp: &GridPosition| -> bool {
    fog_map.get(local_player.0, gp.x, gp.z) == VisibilityStateEnum::Visible
};
```

### Change site 1: SCP info panel (lines 728-741)

Currently at line 669: `if let Some((res_entity, scp, obj_instance)) = selected_scp.iter().next()`
After query change, destructure becomes: `if let Some((res_entity, scp, obj_instance, grid_pos)) = selected_scp.iter().next()`

Lines 728-741 (the "Resources" section inside the stats_parent closure):
- Name (line 723) and Type (line 726) always display
- Gate the remaining amount, has_plate, and depleted lines:

```rust
if is_visible(grid_pos) {
    // existing lines 728-741 unchanged
    let remaining = scp.remaining_amount;
    let initial = scp.initial_amount;
    let pct = if initial > 0 { (remaining as f32 / initial as f32 * 100.0).min(100.0) } else { 0.0 };
    spawn_stat_text(stats_parent, "Resources", 12.0, Color::srgb(0.5, 0.9, 1.0), None);
    spawn_stat_text(stats_parent, &format!("  Remaining: {} / {} ({:.0}%)", remaining, initial, pct), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
    if scp.has_plate { ... }
    if remaining == 0 { ... }
} else {
    spawn_stat_text(stats_parent, "Resources", 12.0, Color::srgb(0.5, 0.9, 1.0), None);
    spawn_stat_text(stats_parent, "  Remaining: ???", 13.0, Color::srgb(0.6, 0.6, 0.6), None);
}
```

### Change site 2: SDS info panel (lines 803-812)

Currently at line 744: `if let Some((res_entity, sds, obj_instance)) = selected_sds.iter().next()`
After query change: `if let Some((res_entity, sds, obj_instance, grid_pos)) = selected_sds.iter().next()`

Lines 803-812 (inside stats_parent closure):
- Name (line 798), Type (line 801), DeliverySize (line 805), DeliveryInterval (line 806) always display
- Gate CurrentSupplies (line 807) and Next delivery countdown (lines 809-812):

```rust
// Always show:
spawn_stat_text(stats_parent, &format!("  Size: {} supplies", sds.delivery_size), 13.0, ...);
spawn_stat_text(stats_parent, &format!("  Interval: {:.0}s", sds.delivery_interval), 13.0, ...);

// Gated:
if is_visible(grid_pos) {
    spawn_stat_text(stats_parent, &format!("  Current: {} supplies", sds.current_supplies), 13.0, ...);
    if sds.current_supplies == 0 {
        let time_left = sds.time_until_next_delivery;
        spawn_stat_text(stats_parent, &format!("  Next delivery: {:.1}s", time_left), 13.0, ...);
    }
} else {
    spawn_stat_text(stats_parent, "  Current: ???", 13.0, Color::srgb(0.6, 0.6, 0.6), None);
}
```

### Change site 3: ExtractionPlate mining section (lines 487-500)

This section is inside the structures info panel. At line 489: `if let Ok(patch) = sc_patches.get(ep.attached_patch)`
After query change: `if let Ok((patch, patch_grid_pos)) = sc_patches.get(ep.attached_patch)`

Gate the remaining amount display using the patch's grid position:

```rust
if let Ok((patch, patch_grid_pos)) = sc_patches.get(ep.attached_patch) {
    if is_visible(patch_grid_pos) {
        // existing lines 490-496 unchanged
    } else {
        spawn_stat_text(stats_parent, "  Remaining SC: ???", 13.0, Color::srgb(0.6, 0.6, 0.6), None);
    }
}
```

Note: The `is_visible` closure captures `fog_map` and `local_player` — define it once at the top of the function body (after the `let Ok(grid_entity)` early return on line 238), so it is available in all three closure scopes. Since closures passed to `with_children` capture by reference, and `is_visible` only borrows `fog_map` and `local_player` immutably, this works. However, since `is_visible` is used inside `with_children` closures (which take `&mut ChildSpawnerCommands`), you may need to inline the check or use a local bool instead of the closure to avoid borrow conflicts with `commands`. Safest pattern: compute a `let patch_visible: bool` before entering the `with_children` closure, then use that bool inside.

### Import addition (line 5)

Change:
```rust
use crate::game::world::types::{SpaceCrystalPatch, SupplyDeliveryStation};
```
To:
```rust
use crate::game::world::types::{SpaceCrystalPatch, SupplyDeliveryStation, FogOfWarMap};
```

`VisibilityStateEnum` is already in scope via `use crate::types::*;` on line 2.

### System ordering

`update_selected_units_grid_system` already runs in `Update` schedule under `DiagCategory::UiHud` (ui/mod.rs:41). No ordering changes needed — `FogOfWarMap` is updated in `FixedUpdate` by `update_fog_of_war` (world/map.rs:326), which runs well before `Update`.

## Dependencies

None. This is a standalone UI change. FogOfWarMap and LocalPlayer resources already exist and are initialized during game setup. Both SCP and SDS entities already have GridPosition components.
