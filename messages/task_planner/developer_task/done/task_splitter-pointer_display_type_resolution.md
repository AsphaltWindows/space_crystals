# pointer_display_type_resolution

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-pointer_display_types.md

## Task

Define the PointerDisplayType enum and resolution system. This is a new system that determines which cursor display type to show each frame.

### What to implement:

1. **PointerDisplayType enum** in `ui/types.rs`:
   - Variants: Inactive, Move, Attack, AttackGround, Patrol, GatherResources, ReturnResources, Enter
   - Derive Default (Inactive)

2. **PointerDisplayType as a Resource** — add to `ui/types.rs`, init in `ui/mod.rs` plugin setup.

3. **resolve_pointer_display_type system** in `ui/command_panel.rs` (or a new `ui/pointer.rs` module):
   - Runs each frame, reads: ObjectInterfaceState, CursorTarget, CursorTargetEnum, Selection, SelectedUnitCapabilities, ActiveGroup data
   - Sets the PointerDisplayType resource based on these resolution rules:

   **When ObjectInterfaceState is placement mode** (`is_placement_mode()`): set to Inactive (rendering layer will hide it).

   **When ObjectInterfaceState::Default** (right-click preview):
   - Nothing selected → Inactive
   - Selection is production building (has BarracksState/HeadquartersState/SupplyTowerState/DeploymentCenterState/ExtractionFacilityState) → Move (rally)
   - CursorTarget is EnemyObject + selection has_attack → Attack
   - CursorTarget is resource node (SpaceCrystalPatch/SupplyDeliveryStation) + selection is resource gatherer (Agent/SupplyChopper) → GatherResources
   - CursorTarget is drop-off point + resource gatherer is carrying → ReturnResources
   - CursorTarget is own Tunnel + Syndicate unit + tier sufficient → Enter
   - CursorTarget is Ground/FriendlyObject/NeutralObject + selection can move → Move
   - Otherwise → Inactive

   **When ObjectInterfaceState::AwaitingTarget(cmd)**:
   - Attack + EnemyObject → Attack
   - Attack + Ground → Attack (AttackMove)
   - Attack + Friendly/Neutral → Inactive
   - Move + any → Move
   - Patrol + Ground → Patrol
   - Patrol + non-ground → Inactive
   - AttackGround + Ground → AttackGround
   - AttackGround + non-ground → Inactive
   - Reverse + Ground → Move
   - Reverse + non-ground → Inactive
   - ScheduleDeliveries + valid SDS target → GatherResources
   - ScheduleDeliveries + invalid → Inactive
   - SetRallyPoint + any → Move

4. **Unit tests** verifying resolution rules for each combination (DefaultState scenarios, AwaitingTarget scenarios, placement mode).

### Key references:
- Design: `artifacts/designer/design/control_system.md` — PointerDisplayType section
- CursorTarget resource: `ui/types.rs` line 118
- ObjectInterfaceState: `ui/types.rs` line 150
- Right-click resolution logic (mirror for DefaultState): `game/units/systems/core.rs` `right_click_move_command`
- SelectedUnitCapabilities: `ui/types.rs` line 369
- AgentCarryState: `game/units/types/state/types.rs`
