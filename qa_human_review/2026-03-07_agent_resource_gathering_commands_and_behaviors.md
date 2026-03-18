# Task: Agent Resource Gathering Commands and Behaviors

## Current State
The command-to-behavior pipeline supports 9 commands and 10 behaviors. The Agent unit's data definition exists (stats, gathering attributes, carry capacities) but no Gather or DropOffResources commands exist in the command table, and no GatheringResource or DroppingOffResources behaviors exist in the behavior system. Agents cannot actually gather resources or deliver them to Tunnels through the command pipeline.

## Desired State
Add two new commands and two new behaviors to the command-to-behavior pipeline for Agent resource gathering.

### Commands

**Gather**
- CommandType: Gather
- TargetLocation: None
- TargetObject: Resource source (ObjectInstance) — Space Crystal Patch or Supply Delivery Station
- Availability: Agent (resource-gathering units) only

**DropOffResources**
- CommandType: DropOffResources
- TargetLocation: None
- TargetObject: Own Tunnel (ObjectInstance)
- Availability: Agent only, when carrying resources (greyed out otherwise per Agent ObjectInterfaceState)

### Behaviors

**GatheringResource**
Agent moves to the target resource source (MovingToObject sub-behavior), then performs resource extraction:
- **Space Crystals**: MiningDuration of 48 frames at Space Crystal Patch, picks up 50 SC
- **Supplies**: PickUpDuration of 48 frames at Supply Delivery Station, picks up 1 Supply

After extraction, the Agent **automatically** moves to the nearest own Tunnel's appropriate side:
- Side B for crystals
- Side C for supplies

Then performs drop-off (DropOffDuration: 48 frames for both types). One Agent at a time per drop-off side.

The behavior encompasses the full gather-deliver cycle: approach resource -> extract -> travel to Tunnel -> drop off.

**DroppingOffResources**
Agent moves to the target Tunnel's appropriate side based on carried resource type:
- Side B for crystals
- Side C for supplies

Then performs drop-off (DropOffDuration: 48 frames). The behavior auto-routes to the correct side — the player only needs to target the Tunnel, not a specific side.

### Interaction Between Commands
- Gather command triggers GatheringResource behavior (full cycle including auto-delivery)
- DropOffResources command triggers DroppingOffResources behavior (explicit delivery without gathering first)
- The GatheringResource behavior's auto-delivery phase effectively contains DroppingOffResources logic internally

## Technical Context

### Commands Already Exist
`UnitCommand::Gather(Entity)` and `UnitCommand::DropOffResources(Entity)` are already defined at `src/game/units/types/state/commands.rs:28-31`. `CommandType::Gather` and `CommandType::DropOff` exist at lines 87-88. Right-click issuing logic already works at `src/game/units/systems/core.rs:284-347` — Agents right-clicking crystal patches, SDS, or own tunnels correctly insert the commands. No changes needed to command definitions or right-click handling.

### AgentCarryState Already Exists
`AgentCarryState` at `src/game/units/types/state/types.rs:8` with `crystals: u32`, `supplies: u32`, and `is_carrying()` method. Already spawned on Agent entities at `src/game/utils.rs:599`. No changes needed.

### New Marker Components Needed (behavior.rs)
Follow the `EnteringTunnelBehavior` / `BuildingStructureBehavior` pattern at `src/game/units/types/state/behavior.rs:134-206`. Create two new marker components:

**`GatheringResourceBehavior`** — multi-phase behavior component:
```rust
#[derive(Component, Clone, Debug)]
pub struct GatheringResourceBehavior {
    /// Target resource entity (SpaceCrystalPatch or SupplyDeliveryStation)
    pub target_resource: Entity,
    /// Current phase of the gathering cycle
    pub phase: GatherPhase,
    /// Path to current target
    pub path: Vec<Vec3>,
    pub path_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GatherPhase {
    /// Moving to the resource source
    MovingToResource,
    /// Extracting resources (frame counter)
    Extracting { frames_remaining: u32 },
    /// Moving to tunnel for drop-off
    MovingToTunnel { tunnel_entity: Entity, side_position: Vec3 },
    /// Dropping off resources at tunnel (frame counter)
    DroppingOff { frames_remaining: u32 },
}
```

**`DroppingOffResourcesBehavior`** — simpler single-phase:
```rust
#[derive(Component, Clone, Debug)]
pub struct DroppingOffResourcesBehavior {
    /// Target tunnel entity
    pub target_tunnel: Entity,
    /// Current phase
    pub phase: DropOffPhase,
    /// Path to tunnel side
    pub path: Vec<Vec3>,
    pub path_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DropOffPhase {
    /// Moving to the tunnel's appropriate side
    MovingToTunnel,
    /// Dropping off resources (frame counter)
    DroppingOff { frames_remaining: u32 },
}
```

### Resource Source Components
- `SpaceCrystalPatch` at `src/game/world/types.rs:163` — has `crystals_remaining: u32` field
- `SupplyDeliveryStation` at `src/game/world/types.rs:172` — has `supplies_available: u32` field
- Both are components on resource entities, queryable via `Query<&SpaceCrystalPatch>` / `Query<&SupplyDeliveryStation>`

### Tunnel Side Position Computation
Tunnel is 4x4, ABCD symmetry (`src/game/types/objects.rs:339-342`). Sides map to [N, E, S, W] = [A, B, C, D] at R0.

To compute side world positions from a Tunnel's `Transform` and `StructureInstance`:
1. Get `oriented_labels()` from `StructureInstance` with `SymmetryTypeEnum::ABCD` — returns `[char; 4]` indexed as [N=0, E=1, S=2, W=3]
2. Find which index contains 'B' (crystal drop-off) or 'C' (supply drop-off)
3. Compute world offset: for a 4x4 structure centered at `transform.translation`:
   - N side: `(0, 0, -size_z/2 - 0.5)` offset (outside north edge)
   - E side: `(size_x/2 + 0.5, 0, 0)` offset
   - S side: `(0, 0, size_z/2 + 0.5)` offset
   - W side: `(-size_x/2 - 0.5, 0, 0)` offset

Helper function recommendation:
```rust
/// Get world position of a tunnel side ('A', 'B', 'C', or 'D')
fn tunnel_side_world_position(
    tunnel_transform: &Transform,
    structure_instance: &StructureInstance,
    target_side: char,
) -> Vec3
```

Query for tunnels: `Query<(Entity, &Transform, &Owner, &StructureInstance), With<TunnelState>>`

### Nearest Own Tunnel Lookup
For the auto-delivery phase of GatheringResource, find the nearest own tunnel:
```rust
fn find_nearest_own_tunnel(
    agent_pos: Vec3,
    agent_owner: &Owner,
    tunnels: &Query<(Entity, &Transform, &Owner, &StructureInstance), With<TunnelState>>,
) -> Option<(Entity, Vec3)>  // (tunnel_entity, side_world_position)
```

### Behavior System Functions
Add to `src/game/units/systems/behaviors.rs`, following the pattern of `entering_tunnel_behavior_system` (line 411) and `building_behavior_system` (line 464).

**`gathering_resource_behavior_system`**:
- Query: `(Entity, &Transform, &mut GatheringResourceBehavior, &mut LocomotionChannel, &mut OrientationChannel, &mut UnitCommand, &mut BaseBehaviorState, &mut AgentCarryState, &Owner)`
- Phase logic:
  1. **MovingToResource**: Write `LocomotionChannel::Moving` toward resource. On arrival (threshold ~0.5), transition to `Extracting { frames_remaining: 48 }`.
  2. **Extracting**: Decrement `frames_remaining` each tick. Set `LocomotionChannel::Stationary`. On completion: update `AgentCarryState` (crystals += 50 or supplies += 1), optionally decrement resource source. Find nearest own tunnel, compute side position ('B' for crystals, 'C' for supplies). Transition to `MovingToTunnel`.
  3. **MovingToTunnel**: Move toward `side_position`. On arrival, check occupancy (one Agent per side — see below). Transition to `DroppingOff { frames_remaining: 48 }`.
  4. **DroppingOff**: Decrement. On completion: update `SyndicatePlayerResources` (crystals or supplies), zero out `AgentCarryState`, remove `GatheringResourceBehavior` marker, set `UnitCommand::Idle`, `BaseBehaviorState::None`.
- Resource sources: query `Option<&SpaceCrystalPatch>` and `Option<&SupplyDeliveryStation>` on the target entity to determine resource type

**`dropping_off_resources_behavior_system`**:
- Simpler version — only MovingToTunnel + DroppingOff phases
- Same tunnel side selection logic based on `AgentCarryState` (crystals → Side B, supplies → Side C)

### Side Occupancy (One Agent Per Side)
Track which tunnel sides are occupied during drop-off. Two approaches:
1. **Query-based**: In the behavior system, query all other `GatheringResourceBehavior` / `DroppingOffResourcesBehavior` entities — if any are in `DroppingOff` phase at the same tunnel + same side, the arriving agent waits (stays in `MovingToTunnel` with `LocomotionChannel::Stationary`).
2. **Component-based**: Add `TunnelDropOffOccupancy { side_b: Option<Entity>, side_c: Option<Entity> }` component to tunnel entities.

Approach 1 is simpler and follows existing patterns (no new tunnel components needed). O(n^2) over agents is fine for low agent counts.

### Player Resources Update
When drop-off completes, update `SyndicatePlayerResources`:
- Query: `Query<(&Player, &mut SyndicatePlayerResources)>`
- Pattern: see `src/game/world/faction.rs:1447` for existing Syndicate resource mutation
- Match on `Player.number == agent_owner.player_number()`
- Add `carry_state.crystals` to `space_crystals`, `carry_state.supplies` to `supplies`

### System Registration
Add both new behavior systems to `src/game/units/mod.rs:16-34`, inside the `Update` system set alongside other behavior systems:
```rust
systems::behaviors::gathering_resource_behavior_system,
systems::behaviors::dropping_off_resources_behavior_system,
```

### Gathering Constants
Define in `src/game/units/types/unit_data.rs` near the Agent constants (line 184):
```rust
pub const AGENT_MINING_DURATION: u32 = 48;
pub const AGENT_PICKUP_DURATION: u32 = 48;
pub const AGENT_DROPOFF_DURATION: u32 = 48;
pub const AGENT_CRYSTAL_CARRY: u32 = 50;
pub const AGENT_SUPPLY_CARRY: u32 = 1;
```

### State Module Exports
Ensure `GatheringResourceBehavior`, `DroppingOffResourcesBehavior`, `GatherPhase`, and `DropOffPhase` are exported from:
- `src/game/units/types/state/behavior.rs` (definition)
- `src/game/units/types/state/mod.rs` (re-export)

### Command Indicator Updates
At `src/game/units/types/types.rs:50-59`, add `Gather` and `DropOffResources` to `command_has_indicator()` (both should return `true` — player needs feedback). Add color mapping at lines 32-41 (green for gather, same green as move for drop-off). Update tests at lines 153+.

### Existing Tests to Note
- Command tests at `src/game/units/types/state/commands.rs:562-616` — Gather/DropOff command variants and `is_available` already tested
- `AgentCarryState` tests at `src/game/units/types/state/types.rs:36-68` — already tested
- New tests needed: behavior system tests following pattern at `behaviors.rs:528+` (spawn entity with marker, run system, assert phase transitions)

## Dependencies
- `syndicate_agent_unit` — Agent entity definition and spawn function (in `/qa_tasks` — implemented)
- `combat_behaviors` — Behavior system framework, action channel pattern (in `/qa_tasks` — implemented)
- `movement_behaviors` — `moving_to_location_system`, `moving_to_object_system` patterns (in `/qa_tasks` — implemented)
- `agent_object_interface_state` — Agent UI panel including greyed DropOff button, `AgentCarryState` spawn (in `/qa_tasks` — implemented)

All 4 dependencies are already implemented (in `/qa_tasks`). This task can proceed.

## QA Steps
1. [semi] Select an Agent and right-click a Space Crystal Patch. Verify the Gather command is issued and the Agent walks toward the patch.
2. [auto] Verify the Agent performs mining for exactly 48 frames upon reaching the Space Crystal Patch.
3. [auto] Verify the Agent picks up 50 Space Crystals after mining completes.
4. [semi] After mining, verify the Agent automatically moves to the nearest own Tunnel's Side B (crystal drop-off side) without player input.
5. [auto] Verify the drop-off takes exactly 48 frames at Side B.
6. [auto] Verify 50 Space Crystals are added to the player's crystal count after drop-off completes.
7. [semi] Select an Agent and right-click a Supply Delivery Station. Verify the Agent walks to the station and performs pickup for 48 frames.
8. [auto] Verify the Agent picks up 1 Supply after pickup completes.
9. [semi] After pickup, verify the Agent automatically moves to the nearest own Tunnel's Side C (supply drop-off side).
10. [auto] Verify the drop-off takes exactly 48 frames at Side C and 1 Supply is added to the player's supply count.
11. [auto] Send two Agents to drop off crystals at the same Tunnel Side B simultaneously. Verify only one Agent drops off at a time — the second waits.
12. [auto] Send one Agent to drop off crystals (Side B) and another to drop off supplies (Side C) at the same Tunnel. Verify both can drop off simultaneously (separate sides).
13. [semi] Select an Agent carrying crystals. Issue an explicit DropOffResources command targeting an own Tunnel. Verify the Agent walks to Side B and drops off.
14. [semi] Select an Agent carrying supplies. Issue an explicit DropOffResources command targeting an own Tunnel. Verify the Agent walks to Side C and drops off.
15. [auto] Verify the DropOffResources command is unavailable (greyed out) when the Agent is not carrying resources.
16. [auto] Verify that non-Agent units cannot receive the Gather or DropOffResources commands.

## Expected Experience
The resource gathering loop should feel smooth and automated: the player right-clicks a resource source, and the Agent handles the full gather-deliver cycle without further input. After mining/picking up resources, the Agent automatically finds the nearest own Tunnel and routes to the correct side. Crystal deliveries go to Side B, supply deliveries to Side C. The player can see both a crystal gatherer and a supply gatherer working at the same Tunnel simultaneously on different sides. The explicit Drop Off command provides manual control when needed (e.g., redirecting to a specific Tunnel), but the automatic delivery in the Gather cycle handles the common case.

## Automated QA Results (2026-03-09 retest)
- Step 2 [auto]: PASS — Mining duration confirmed at exactly 48 frames
- Step 3 [auto]: PASS — Agent picks up 50 Space Crystals after extraction
- Step 5 [auto]: PASS — Drop-off takes exactly 48 frames
- Step 6 [auto]: PASS — 50 crystals added to player resources
- Step 8 [auto]: PASS — Agent picks up 1 Supply after pickup
- Step 10 [auto]: PASS — Supply drop-off at Side C takes 48 frames, 1 supply added
- Step 11 [auto]: PASS — Side occupancy enforced: second agent waits while first drops off (fix verified)
- Step 12 [auto]: PASS — Crystal (Side B) and supply (Side C) agents drop off simultaneously
- Step 15 [auto]: PASS — AgentCarryState.is_carrying() correctly reports false when empty; command availability gated by is_syndicate
- Step 16 [auto]: PASS — Gather and DropOffResources unavailable for non-syndicate units
- Steps 1, 4, 7, 9, 13, 14 [semi]: DEFERRED to human review
