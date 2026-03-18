# Task: Rally Point Behavior for Syndicate Production Expansions

## Original Ticket
Implement rally point behavior for Syndicate production expansions (Headquarters). When a unit finishes production, the rally point determines whether it auto-ejects from the parent Tunnel's Side A and moves to the rally point, or stays in the Tunnel Network.

## Technical Context

### Current Behavior (Incorrect)
The `headquarters_production_tick_system` at `src/game/world/faction.rs:371-436` currently spawns ALL produced units on the surface at the parent Tunnel's grid position and calls `issue_rally_command`. When no rally point is set, the unit stays idle on the surface. **This is wrong** — units without a surface rally should enter the Tunnel Network instead.

### 1. Modify `headquarters_production_tick_system` — Conditional Spawn vs InTunnelNetwork
- **File**: `src/game/world/faction.rs:371-436`
- **Current**: Always spawns unit at tunnel surface position (lines 402-416), then calls `issue_rally_command` (lines 419-425)
- **New behavior after production completes** (line 400 onwards):
  - **If `rally_point` is `Some(RallyTarget::Location(_))` (surface location)**:
    - Compute Side A world position using `tunnel_side_world_position()` from `src/game/units/systems/behaviors.rs:543`
    - Need to query `(&Transform, &StructureInstance)` on `expansion_marker.parent_tunnel` to call this function
    - Convert Side A world position to grid coords using `world_to_grid()` from `src/game/units/utils.rs:13`
    - Spawn unit at Side A grid position (not tunnel center)
    - Call `issue_rally_command` with the rally point
  - **If `rally_point` is `Some(RallyTarget::Object(entity))` where entity == `expansion_marker.parent_tunnel`**:
    - Treat as "no rally point" — unit stays in network (see below)
  - **If `rally_point` is `None` or rally target is parent tunnel**:
    - Spawn the unit entity (still needed for ECS) but **do not place it on the surface**
    - Insert `InTunnelNetwork { owner_player: owner.player_number().unwrap_or(0) }` component
    - Remove `Transform` / set it to off-screen, or spawn without visible mesh (the unit should not appear on the map)
    - Do NOT call `issue_rally_command`
    - Log: "Headquarters: Produced {:?}, entered Tunnel Network"

#### System Signature Changes
The system needs additional queries to get Side A position:
```rust
tunnel_data: Query<(&Transform, &StructureInstance, &GridPosition), With<TunnelState>>,
```
Replace the current `tunnel_positions: Query<&GridPosition, With<TunnelState>>` with this broader query.

### 2. "Rally on Parent Tunnel" = Clear Rally Point
- **File**: `src/game/world/faction.rs` — in the generalized rally point system (created by `standard_bottom_row_commands` dependency)
- When right-clicking to set rally point on a Headquarters:
  - If the clicked entity is the `TunnelExpansionMarker.parent_tunnel` of the selected HQ → set `rally_point = None` (clear it)
  - This requires querying `TunnelExpansionMarker` on the selected HQ entity
- **Alternatively**: Add this check in the generalized `production_rally_point_system` (step 9 of standard_bottom_row_commands). When processing HQ right-click rally and clicked entity matches the parent tunnel, clear rally instead of setting `RallyTarget::Object(entity)`.

### 3. Side A Position Calculation
- **Function**: `tunnel_side_world_position()` at `src/game/units/systems/behaviors.rs:543-575`
- Signature: `fn tunnel_side_world_position(transform: &Transform, instance: &StructureInstance, side: char) -> Vec3`
- Call with side = `'A'` for unit ejection
- The function uses the tunnel's `StructureInstance.symmetry_labels` and rotation to determine cardinal direction
- **Important**: This function is `pub` and already used by entering-tunnel and ejection systems

### 4. InTunnelNetwork Marker
- **File**: `src/game/units/types/state/behavior.rs:324-327`
- ```rust
  pub struct InTunnelNetwork {
      pub owner_player: u8,
  }
  ```
- Units with this marker are invisible on the map and available for manual ejection via Tunnel EjectMenu
- The `unit_movement_system` already excludes `InTunnelNetwork` units (core.rs:1138: `Without<InTunnelNetwork>`)

### 5. Visual Rally Point Marker
- **Pattern**: Follow `MoveTargetMarker` at `src/game/units/types/movement.rs:23` — simple marker component on a mesh entity
- Create a `RallyPointMarker { owner_structure: Entity }` component
- When rally point is set on a production structure:
  - Despawn any existing `RallyPointMarker` for that structure
  - Spawn a small visual indicator (cylinder or flag mesh) at the rally location
  - Attach `RallyPointMarker { owner_structure: hq_entity }` to it
- When rally point is cleared: despawn the marker entity
- **Where to spawn/despawn**: In the rally point setting system (generalized production_rally_point_system and AwaitingTarget[SetRallyPoint] click handler)
- The marker should be visible only when the structure is selected (or always visible — design spec says "visual indicator at the rally point location when set")

### 6. `issue_rally_command` Hardcoded Unit Base
- **File**: `src/game/world/faction.rs:323`
- Currently hardcodes `UnitBaseEnum::LightInfantry` for pathfinding regardless of unit type
- For Syndicate units (Agent, Guard), this should use the actual unit's `UnitBaseEnum`:
  - Agent → `UnitBaseEnum::LightInfantry` (happens to match)
  - Guard → `UnitBaseEnum::LightInfantry` (also matches — Guard is light infantry)
- **No change needed now**, but note this for future vehicle/hovercraft production

### Key Files
| File | Purpose |
|------|---------|
| `src/game/world/faction.rs:371-436` | `headquarters_production_tick_system` — main modification target |
| `src/game/world/faction.rs:299-362` | `issue_rally_command` — called for surface rally |
| `src/game/world/faction.rs:500-572` | `barracks_rally_point_system` — pattern for rally setting (will be generalized by dependency) |
| `src/game/units/systems/behaviors.rs:543-575` | `tunnel_side_world_position()` — Side A calculation |
| `src/game/units/types/state/behavior.rs:324-327` | `InTunnelNetwork` marker component |
| `src/game/types/structures.rs:235-278` | `HeadquartersState` (rally_point field at line 237) |
| `src/game/types/structures.rs:226-229` | `TunnelExpansionMarker` (parent_tunnel field) |
| `src/game/types/structures.rs:58-61` | `RallyTarget` enum |
| `src/ui/types.rs:368-376` | `EjectionQueue` component (on Tunnel entities) |
| `src/game/units/types/movement.rs:23` | `MoveTargetMarker` — pattern for visual marker |

### Patterns to Follow
- **Barracks production tick** (`faction.rs:228-296`): Pattern for spawn + rally integration. HQ follows the same flow but with conditional InTunnelNetwork insertion.
- **Barracks rally system** (`faction.rs:500-572`): Pattern for right-click rally setting. The generalized version (from dependency) extends this to HQ.
- **EnteringTunnelBehavior** (`behavior.rs:129-142`): Pattern for behavior marker components referencing tunnel entities.
- **tunnel_side_world_position usage** (`behaviors.rs:580`): How the function is called with transform, instance, and side char.

### Tests
- Test that `headquarters_production_tick_system` inserts `InTunnelNetwork` when `rally_point` is `None`
- Test that `headquarters_production_tick_system` spawns unit at Side A position when `rally_point` is `Some(Location(...))`
- Test that setting rally point to parent tunnel entity clears `rally_point` to `None`
- Test that `RallyPointMarker` entity is spawned at rally location when set
- Test that `RallyPointMarker` entity is despawned when rally is cleared

## Dependencies
- **`2026-03-09_standard_bottom_row_commands.md`** — **MUST complete first**. This task creates the `SetRallyPoint` command type, the C hotkey binding, the AwaitingTarget[SetRallyPoint] handler, and generalizes the rally point right-click system to all production structures (including HQ). Without it, there's no UI mechanism to set rally points on the Headquarters. This rally_point_behavior task modifies the *production outcome* based on rally state, and adds the "parent tunnel click = clear" special case.

## QA Steps
1. [human] Select a Headquarters and right-click a surface location — verify a rally point marker appears at that location
2. [human] Produce a unit with the surface rally point set — verify the unit auto-ejects from the parent Tunnel's Side A and moves to the rally point
3. [human] Right-click the parent Tunnel while the Headquarters is selected — verify the rally point is cleared (no marker visible)
4. [human] Produce a unit with no rally point set — verify the unit stays in the Tunnel Network (does not eject, appears in Eject menu)
5. [human] Set a rally point, then right-click a new surface location — verify the rally point moves to the new location
6. [human] Produce multiple units with a surface rally point — verify each auto-ejects sequentially from Side A

## Expected Experience
Setting a rally point works like standard RTS production rally points: right-click a location, see a visual marker, and produced units automatically move there. The Syndicate twist is that units emerge from the Tunnel entrance (not the building itself, since it's underground). When no rally point is set, units silently enter the Tunnel Network — the player manages them later via the Eject menu. The system feels intuitive to RTS veterans while supporting the Syndicate's underground staging gameplay.
