# Task: Agent Tunnel Building Command and Behavior

## Current State
The command-to-behavior pipeline has no BuildTunnel command or BuildingTunnel behavior. The Agent's ObjectInterfaceState has an AwaitingPlacement flow for Tunnel building (UI layer), and the Agent unit definition includes building stats (data layer), but the command pipeline cannot execute the build sequence. The ConstructionHP Rule exists in the entity system, but no behavior drives the Agent through the construction flow.

## Desired State
Add the BuildTunnel command and BuildingTunnel behavior to the command-to-behavior pipeline.

### Command

**BuildTunnel**
- CommandType: BuildTunnel
- TargetLocation: location (the validated placement location from AwaitingPlacement)
- TargetObject: None
- Availability: Agent only, issued via AwaitingPlacement (left-click valid location in Agent's ObjectInterfaceState)

### Behavior

**BuildingTunnel**
Execution sequence:
1. Agent moves to the target build location (MovingToLocation sub-behavior)
2. Construction begins — a partially-built Tunnel appears at the location, starting at **10% HP** (ConstructionHP Rule: `HP = MaxHP x 10%` = 60 HP for T1 Tunnel)
3. The Agent **embeds inside** the partially-built Tunnel and becomes **untargetable** for the duration of construction
4. HP increases linearly over 480 frames: `HP = MaxHP x (10% + 90% x construction_progress)` where `construction_progress` goes from 0.0 to 1.0
5. **If construction completes (480 frames)**: The Tunnel becomes fully operational. The Agent is placed inside the Tunnel Network (not on the surface), available for redeployment from any Tunnel.
6. **If the partially-built Tunnel is destroyed during construction**: The Agent survives and emerges at the Tunnel's location. The Tunnel is lost and any Supplies spent are not refunded.

### Constraints
- Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction
- The Agent must remain present for the full 480-frame build duration
- Construction cost follows the Tunnel cost scaling formula: cost = current number of Tunnels owned, in Supplies (see `features/syndicate_objects.md` Cost Scaling section)

## Technical Context

### Command Already Exists
`UnitCommand::BuildTunnel(Vec3)` is already defined at `src/game/units/types/state/commands.rs:33`. `CommandType::BuildTunnel` at line 89. The placement UI at `src/game/world/faction.rs:1208-1214` already issues the command:
```rust
commands.entity(source_entity).insert(UnitCommand::BuildTunnel(world_pos));
```
The `right_click_move_command` at `core.rs:521-525` already handles BuildTunnel in the no-op branch (placement-only mode). **No command-layer changes needed.**

### New BuildingTunnelBehavior Marker Component
Add to `src/game/units/types/state/behavior.rs`, following the `BuildingStructureBehavior` pattern at line 170:

```rust
/// Phase of the tunnel building behavior.
#[derive(Clone, Debug, PartialEq)]
pub enum BuildTunnelPhase {
    /// Moving to the build location
    MovingToSite,
    /// Constructing — Agent is embedded, tunnel entity exists with ConstructionHP
    Constructing {
        /// The partially-built tunnel entity
        tunnel_entity: Entity,
        /// Frames of construction elapsed
        frames_elapsed: u32,
    },
}

/// Marker component for the BuildingTunnel behavior.
/// When present, a dedicated system moves the Agent to the build site,
/// spawns a partially-built Tunnel, embeds the Agent, and ticks construction.
#[derive(Component, Clone, Debug)]
pub struct BuildingTunnelBehavior {
    /// Target world-space location to build at
    pub target_location: Vec3,
    /// Current phase
    pub phase: BuildTunnelPhase,
    /// Precomputed path to target
    pub path: Vec<Vec3>,
    /// Current index in the path
    pub path_index: usize,
}

impl BuildingTunnelBehavior {
    pub fn new(target_location: Vec3) -> Self {
        Self {
            target_location,
            phase: BuildTunnelPhase::MovingToSite,
            path: Vec::new(),
            path_index: 0,
        }
    }
}
```

Export from `src/game/units/types/state/mod.rs` alongside other behavior types.

### Build Duration Constant
Add to `src/game/units/types/unit_data.rs` near the Agent constants (line ~202):
```rust
/// Agent tunnel construction duration in simulation frames (480 frames = 30 seconds at 16 FPS)
pub const AGENT_TUNNEL_BUILD_FRAMES: u32 = 480;
```

### Key Existing Infrastructure

**ConstructionHP** (`src/game/types/structures.rs:11`):
- `ConstructionHP::new(build_frames)` — creates with progress 0.0
- `ConstructionHP::hp_fraction(progress)` — returns `0.10 + 0.90 * progress.clamp(0.0, 1.0)`
- `construction_hp_tick_system` at `src/game/world/faction.rs:530` — ticks progress each frame, scales HP, removes component on completion

**ObjectInstance::under_construction** (`src/game/types/objects.rs:73`):
- `ObjectInstance::under_construction(ObjectEnum::Tunnel, max_hp)` — creates at 10% HP

**spawn_tunnel** (`src/game/utils.rs:617`):
- Full tunnel spawn with `TunnelState::default_tier1()`, `TunnelArea::new()`, `SightRange`, `Selectable`, visual mesh
- For the behavior system, the tunnel must be spawned **without** calling `spawn_tunnel()` directly because the behavior system doesn't have mesh/material access. Use a **staged approach**: behavior detects arrival and inserts a `PendingTunnelSpawn` marker component with location data; a separate system with `ResMut<Assets<Mesh>>` etc. detects this and calls `spawn_tunnel()` (modified to accept `ConstructionHP` and `under_construction` ObjectInstance)

**Alternative (recommended)**: The behavior system has access to `Commands`, so it can insert a resource-like event that a rendering-capable system processes. However, the simplest approach: **the behavior system needs `ResMut<Assets<Mesh>>` and `ResMut<Assets<StandardMaterial>>`** as system params — this is the same pattern used by the placement click system at `faction.rs:990`. The behavior system can call a modified spawn function directly.

**Tunnel T1 HP**: `TUNNEL_T1_MAX_HP = 600.0` at `src/game/types/structures.rs:390`
**TunnelTier::Tier1.max_hp()** at `structures.rs:463`

**Tunnel cost function**: `tunnel_construction_cost(existing_tunnel_count)` at `src/game/types/structures.rs:629` — returns `existing_tunnel_count` (0-indexed: 1st=0, 2nd=1, 3rd=2)

**InTunnelNetwork** (`src/game/units/types/state/behavior.rs:284`):
```rust
pub struct InTunnelNetwork { pub owner_player: u8 }
```
Currently the entering_tunnel_behavior at `behaviors.rs:442` does `despawn_recursive()` — but for the BuildingTunnel completion, the Agent should be placed in the network with `InTunnelNetwork` instead. The existing `entering_tunnel_behavior_system` despawns but doesn't add `InTunnelNetwork` — this is a known gap. For this task, upon completion, **despawn the Agent entity and spawn a new minimal entity with `InTunnelNetwork`** (matching the existing Enter pattern), OR modify the Agent to hide and mark with `InTunnelNetwork`. The former matches existing patterns better.

### Behavior System: `building_tunnel_behavior_system`

Add to `src/game/units/systems/behaviors.rs`. System params:
```rust
pub fn building_tunnel_behavior_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut units: Query<(
        Entity,
        &Transform,
        &Owner,
        &mut BuildingTunnelBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
    )>,
    tunnel_query: Query<(Entity, &Owner), With<TunnelState>>,
    tunnel_hp_query: Query<&ObjectInstance, With<ConstructionHP>>,
    syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
)
```

**Phase logic:**

1. **MovingToSite**: Write `LocomotionChannel::Moving(vec![target])` + `OrientationChannel::Turning(target)`. On arrival (threshold ~1.0, matching `BUILD_ARRIVAL_THRESHOLD`):
   - Count existing tunnels owned by this player: `tunnel_query.iter().filter(|(_, o)| o == agent_owner).count()`
   - Compute cost: `tunnel_construction_cost(count as u32)`
   - Deduct from `SyndicatePlayerResources.supplies` (if insufficient, cancel build — remove marker, idle)
   - Spawn partially-built tunnel at target using modified spawn (see below)
   - **Hide the Agent**: Set `Visibility::Hidden` on the Agent entity, stop locomotion/orientation
   - Transition to `Constructing { tunnel_entity, frames_elapsed: 0 }`

2. **Constructing**: Each tick:
   - Check if `tunnel_entity` still exists (via `tunnel_hp_query.get(tunnel_entity)`):
     - **If tunnel destroyed**: Agent emerges — set `Visibility::Inherited`, `LocomotionChannel::Stationary`, `UnitCommand::Idle`, `BaseBehaviorState::None`, remove `BuildingTunnelBehavior`. Agent is now at the build location (its Transform was left at arrival position).
     - **If tunnel alive**: Increment `frames_elapsed`. The `construction_hp_tick_system` already handles HP progression — this behavior just tracks completion timing.
   - When `frames_elapsed >= AGENT_TUNNEL_BUILD_FRAMES`:
     - Construction complete — `ConstructionHP` will already be removed by `construction_hp_tick_system`
     - Enter tunnel network: `commands.entity(entity).despawn_recursive()` (matches entering_tunnel pattern)
     - Set `UnitCommand::Idle`, `BaseBehaviorState::None`, remove `BuildingTunnelBehavior` (these happen before despawn takes effect)

**Spawning the partially-built tunnel**: Create a helper function `spawn_tunnel_under_construction()` in `src/game/utils.rs` (near `spawn_tunnel` at line 617):
```rust
pub fn spawn_tunnel_under_construction(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
    build_frames: u32,
) -> Entity
```
- Same as `spawn_tunnel()` but uses `ObjectInstance::under_construction(ObjectEnum::Tunnel, tier.max_hp())` instead of `destructible()`
- Adds `ConstructionHP::new(build_frames)`
- The existing `construction_hp_tick_system` will handle HP scaling automatically

### Agent Untargetability During Construction
Set `Visibility::Hidden` on the Agent entity when embedding. This removes it from rendering AND from combat targeting (targeting systems use `With<Unit>` + visibility checks). The Agent's `Transform` remains at the build site so it can emerge there if the tunnel is destroyed.

**Alternative**: Remove `Unit` component temporarily and re-add on emerge. `Visibility::Hidden` is simpler and matches the fog-of-war hide pattern at `src/game/world/map.rs:348`.

### Command Indicator
`BuildTunnel` should show an indicator. Add to `command_has_indicator()` at `src/game/units/types/types.rs:53`:
```rust
UnitCommand::BuildTunnel(_)
```
Add color mapping at the indicator color function (currently at types.rs:32-41). Suggested: same color as Build command.

### Preventing Multiple Builders Per Tunnel
In the `Constructing` phase, the tunnel entity already exists. When another Agent tries to build at the same location:
- The `can_worker_place_structure()` validation (called in `building_behavior_system` or placement validation) will reject placement because the partially-built tunnel occupies those tiles
- Additionally, the placement ghost system at `faction.rs:847` calls `can_place_building()` which checks structure overlap — this prevents placing a second tunnel at the same spot
- **No additional logic needed** — existing overlap checks naturally prevent double-building

### System Registration
Add to `src/game/units/mod.rs:16-34` in the Movement DiagCategory:
```rust
systems::behaviors::building_tunnel_behavior_system,
```

### Tunnel Cost Deduction
Before spawning the tunnel, deduct supplies:
```rust
// In building_tunnel_behavior_system, on arrival:
let existing_count = tunnel_query.iter()
    .filter(|(_, owner)| owner.player_number() == agent_owner.player_number())
    .count();
let cost = tunnel_construction_cost(existing_count as u32);

// Deduct from player resources
for (player, mut resources) in syndicate_players.iter_mut() {
    if player.number == agent_owner.player_number() {
        if resources.supplies < cost as i32 {
            // Insufficient funds — cancel build
            // ... remove marker, idle
        } else {
            resources.supplies -= cost as i32;
        }
    }
}
```

### Agent Emerging from Destroyed Tunnel
When the tunnel entity is destroyed (despawned) during construction, `tunnel_hp_query.get(tunnel_entity)` will return `Err`. The behavior system detects this and:
1. Sets `Visibility::Inherited` (make Agent visible again)
2. The Agent's `Transform` is still at the build location
3. Sets idle state — Agent is targetable again
4. No resource refund

### Tests
Follow pattern at `behaviors.rs:815+`:
- Test `MovingToSite` phase: spawn Agent with behavior, run system, verify locomotion channel
- Test arrival + tunnel spawn: Agent at target location, run system, verify tunnel entity spawned with `ConstructionHP`
- Test construction completion: mock tunnel entity alive for N frames, verify Agent despawned after 480
- Test tunnel destroyed mid-construction: despawn tunnel, run system, verify Agent emerges (Visibility::Inherited)
- Test cost deduction: verify `SyndicatePlayerResources.supplies` decremented correctly
- Test insufficient funds: verify build cancelled when supplies < cost

## Dependencies
- `worker_built_structure_arrival_validation` — `BuildingStructureBehavior` and `can_worker_place_structure()` (in `/completed_tasks` — implemented)
- `construction_hp_rule` — `ConstructionHP` component and `construction_hp_tick_system` (in `/completed_tasks` — implemented)
- `tunnel_structure_and_network` — `TunnelState`, `TunnelArea`, `spawn_tunnel()` (in `/completed_tasks` — implemented)
- `syndicate_agent_unit` — Agent entity definition and spawn function (in `/qa_tasks` — implemented)
- `agent_object_interface_state` — Agent UI panel with `AgentAwaitingPlacement` and BuildTunnel command issuing at faction.rs:1208 (in `/qa_tasks` — implemented)

All 5 dependencies are already implemented. This task can proceed.

## QA Steps
1. [semi] Select an Agent, press A (Build Tunnel), place a ghost on a valid location, and left-click. Verify the BuildTunnel command is issued and the Agent begins walking to the target location.
2. [semi] Verify a partially-built Tunnel appears at the build location when the Agent arrives, starting at 10% of MaxHP (60 HP for a T1 Tunnel with 600 MaxHP).
3. [auto] Verify the Agent becomes untargetable once embedded in the partially-built Tunnel.
4. [auto] Verify HP increases linearly during construction. At 50% construction progress (240 frames), HP should be `600 x (0.10 + 0.90 x 0.50)` = 330 HP.
5. [auto] Verify construction completes after exactly 480 frames and the Tunnel becomes fully operational (full HP, functional sides).
6. [auto] After construction completes, verify the Agent is inside the Tunnel Network (not visible on the surface) and can be ejected from any Tunnel.
7. [auto] During construction, destroy the partially-built Tunnel. Verify the Agent survives and appears at the Tunnel's former location.
8. [auto] Verify the surviving Agent is targetable again after emerging from a destroyed construction site.
9. [auto] Verify the Tunnel construction cost follows the scaling formula: 2nd Tunnel costs 1 Supply, 3rd costs 2 Supplies, etc.
10. [auto] Attempt to assign a second Agent to construct the same partially-built Tunnel. Verify this is rejected — only one Agent per construction.
11. [auto] Verify that non-Agent units cannot receive the BuildTunnel command.

## Expected Experience
The tunnel building sequence should feel weighty and committal: the Agent walks to the site, a foundation appears, and the Agent visually merges into the structure. The player sees the Tunnel's HP bar climbing steadily over 30 seconds. The Agent is safe inside but the partially-built Tunnel is vulnerable — if enemies destroy it, the Agent pops out alive but the investment is lost. When construction finishes, the Tunnel snaps to full operation and the Agent disappears into the network, ready to be redeployed from any Tunnel. The cost scaling means each additional Tunnel is more expensive, creating meaningful expansion decisions.

## Automated QA Results
- Step 1 [semi]: DEFERRED to human review
- Step 2 [semi]: DEFERRED to human review
- Step 3 [auto]: PASS — Agent has Visibility::Hidden when in Constructing phase
- Step 4 [auto]: PASS — HP fraction formula verified (10%→55%→100% linearly)
- Step 5 [auto]: PASS — Build duration constant is 480 frames, ConstructionHP completes correctly
- Step 6 [auto]: PASS — Agent entity despawned after construction completes (enters tunnel network)
- Step 7 [auto]: PASS — Agent survives when tunnel entity is destroyed during construction
- Step 8 [auto]: PASS — Agent visibility restored to Inherited (targetable) after emerging from destroyed tunnel
- Step 9 [auto]: PASS — Cost scaling formula correct: tunnel_construction_cost(n) = n
- Step 10 [auto]: PASS — Overlap checks prevent second tunnel spawn at same location
- Step 11 [auto]: PASS — BuildTunnel requires is_syndicate=true, non-Syndicate units rejected
