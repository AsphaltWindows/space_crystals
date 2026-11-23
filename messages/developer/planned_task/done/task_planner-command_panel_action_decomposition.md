# command_panel_action_decomposition

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-command_panel_split.md

## Task

Decompose the ~560-line `execute_command_action()` function in `src/ui/command_panel/actions.rs` into 5 category-specific sub-dispatchers. The main function should become a thin match dispatcher (~30 lines) that delegates to the sub-functions.

### Sub-dispatchers to Create

All in `actions.rs` (or as `pub(crate)` helpers callable from actions.rs):

1. **`execute_dc_action()`** -- Deployment Center build/cancel/placement actions (CommandButtonAction variants related to DC construction menus)
2. **`execute_production_action()`** -- Barracks/HQ/SupplyTower/RC/Armory train/cancel actions (all production queue CommandButtonAction variants)
3. **`execute_unit_command_action()`** -- Move/Stop/Patrol/Attack/Hold/Reverse/Enter/Gather command actions (all unit command CommandButtonAction variants)
4. **`execute_tunnel_action()`** -- All tunnel-specific actions (delegates to functions already in tunnel_actions.rs)
5. **`execute_placement_action()`** -- Enter placement mode actions

### Implementation Steps

1. Identify the match arms in `execute_command_action()` and group them by category
2. Extract each group into its own function with the same parameters needed from the parent
3. Replace the extracted match arms in `execute_command_action()` with calls to the sub-dispatchers
4. The resulting `execute_command_action()` should be ~30 lines: a match on the action category delegating to the 5 sub-functions
5. All sub-dispatchers should be in actions.rs (tunnel actions can delegate to tunnel_actions.rs functions that already exist)

### Constraints

- No behavioral changes -- the exact same actions must execute for the exact same inputs
- This task depends on the directory split being completed first (actions.rs must exist)
- All existing tests for execute_command_action must continue to pass
- The sub-dispatchers do not need their own separate tests since existing tests cover the behavior through execute_command_action

### Verification

- `cargo check` must pass
- `cargo test --lib` must pass
- `cargo test --test qa` must pass
- `execute_command_action()` should be ~30 lines (thin dispatcher)

## Technical Context

### Target File

`artifacts/developer/src/ui/command_panel/actions.rs` -- This file will exist after the `command_panel_directory_split` task completes. It will contain `execute_command_action()` (currently at line 1260 of `command_panel.rs`) and `update_command_panel_progress()` (currently at line 1817), along with their tests.

### Current Function Signature (line 1260 of command_panel.rs)

```rust
fn execute_command_action(
    action: &CommandButtonAction,
    commands: &mut Commands,
    interface_state: &mut ResMut<ObjectInterfaceState>,
    panel_target: &Res<CommandPanelTarget>,
    dc_query: &mut Query<(&Owner, &mut DeploymentCenterState)>,
    bk_hq_query: &mut Query<(&Owner, Option<&mut BarracksState>, Option<&mut HeadquartersState>, Option<&mut RecruitmentCenterState>, Option<&mut ArmoryState>), Or<(With<BarracksState>, With<HeadquartersState>, With<RecruitmentCenterState>, With<ArmoryState>)>>,
    ef_query: &mut Query<(&Owner, &mut ExtractionFacilityState)>,
    st_query: &mut Query<(&Owner, &mut SupplyTowerState)>,
    tunnel_query: &mut Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    syndicate_players: &mut Query<(&Player, &mut SyndicatePlayerResources)>,
    placement_state: &mut ResMut<PlacementState>,
    players: &mut Query<(&Player, &mut GdoPlayerResources)>,
    selected_units: &Query<(Entity, &mut Velocity, Option<&mut CommandQueue>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    attack_states: &Query<&AttackState>,
    shift_held: bool,
    selection: &Selection,
)
```

### Match Arm Groupings (by category)

**Category 1: DC Actions (`execute_dc_action`)** -- Needs: `interface_state`, `panel_target`, `dc_query`, `players`
- `DcOpenBuildMenu` (lines 1279-1293) -- routes to DcBuildMenu/DcConstructing/DcReadyToPlace
- `DcBuild(object_type)` (lines 1294-1324) -- deducts GDO crystals, starts construction
- `DcCancel` (lines 1325-1349) -- refunds crystals, resets state

**Category 2: Production Actions (`execute_production_action`)** -- Needs: `interface_state`, `panel_target`, `bk_hq_query`, `syndicate_players`, `players`, `selection`, `commands`, `st_query`
- `BkTrain(unit_type)` (lines 1350-1382) -- iterates active group, queues unit
- `BkCancel` (lines 1383-1402)
- `HqTrain(unit_type)` (lines 1403-1429) -- uses syndicate resources
- `HqCancel` (lines 1430-1449) -- uses syndicate resources
- `RcCancel` (lines 1450-1459) -- resets RC production_progress
- `EfBuildPlate` (lines 1460-1481) -- deducts GDO crystals, starts plate construction
- `EfCancel` (lines 1482-1496) -- refunds, resets EF state
- `StTrain(unit_type)` (lines 1638-1664) -- iterates active group
- `StCancel` (lines 1665-1684)
- `StScheduleDeliveries` (line 1685-1688) -- sets AwaitingTarget
- `ArmoryTrainSoldier` (lines 1725-1749) -- uses CultsPlayerResources via deferred command
- `ArmoryTrainGunner` (lines 1751-1775) -- same pattern
- `ArmoryEjectAll` (lines 1777-1789) -- inserts ArmoryEjectionQueue

**Category 3: Unit Command Actions (`execute_unit_command_action`)** -- Needs: `interface_state`, `commands`, `selection`, `selected_units`, `attack_states`, `shift_held`, `placement_state`
- `UnitMove` (line 1528-1531) -- sets AwaitingTarget(Move)
- `UnitAttack` (line 1532-1535)
- `UnitAttackGround` (line 1536-1539)
- `UnitAttackMove` (line 1540-1543)
- `UnitPatrol` (line 1544-1547)
- `UnitHoldPosition` (lines 1548-1582) -- complex: command_target_entities, shift queue, removes MoveTarget/Path
- `UnitStop` (lines 1583-1613) -- same pattern as HoldPosition
- `UnitReverse` (line 1614-1617)
- `UnitEnter` (line 1618-1621)
- `UnitGather` (line 1622-1625)
- `ChopperPickUpSupplies` (line 1626-1629)
- `ChopperAttachToTower` (line 1630-1633)
- `ChopperDropOffSupplies` (line 1634-1637)
- `AgentBuildTunnel` (lines 1707-1720) -- sets placement_state and AgentAwaitingPlacement
- `AgentDropOff` (line 1721-1724)
- `RecruitConstruct` (line 1790-1793) -- opens CultsRecruitMenu
- `RecruitSelectBuilding(building_type)` (lines 1794-1804) -- sets placement_state, RecruitAwaitingPlacement
- `RecruitAssistConstruction` (line 1805-1808) -- sets AwaitingTarget
- `SetRallyPoint` (lines 1809-1812) -- sets AwaitingTarget(SetRallyPoint)

**Category 4: Tunnel Actions (`execute_tunnel_action`)** -- Needs: `interface_state`, `panel_target`, `tunnel_query`, `syndicate_players`, `placement_state`, `commands`
- `TunnelUpgrade` (line 1689-1691) -- delegates to `execute_tunnel_upgrade()`
- `TunnelOpenExpandMenu` (line 1692-1694)
- `TunnelOpenEjectMenu` (line 1695-1697)
- `TunnelSelectExpansion(expansion_type)` (lines 1698-1700) -- delegates to `execute_tunnel_select_expansion()`
- `TunnelCancelUpgrade` (lines 1701-1703) -- delegates to `execute_tunnel_cancel_upgrade()`
- `TunnelEjectUnit(unit_type)` (lines 1704-1706) -- delegates to `execute_tunnel_eject_unit()`

**Category 5: Navigation/Back Action (`execute_placement_action` or fold into dispatcher)**
- `Back` (lines 1497-1516) -- context-dependent state return
- `EnterPlacement` (lines 1517-1527) -- DcReadyToPlace->DcAwaitingPlacement, EfIdle->EfAwaitingPlacement

Note: `Back` and `EnterPlacement` are the "placement" category variants. They only need `interface_state`.

### Helper Functions Used

These are currently in `command_panel.rs` and will move to `command_panel/helpers.rs` during the directory split:
- `active_group_entities(selection)` (line 2524) -- returns Vec<Entity> for the active selection group
- `command_target_entities(action, selection, selected_units)` (line 2532) -- determines which entities receive a command (common vs group-specific)
- `find_player_resources_mut(owner, players)` (line 1964) -- finds GdoPlayerResources for an owner
- `find_syndicate_resources_mut(owner, syndicate_players)` (line 2419) -- finds SyndicatePlayerResources for an owner

After the directory split, these will be accessible via `use super::helpers::*` from `actions.rs`.

### Tunnel Action Delegates

These are currently in `command_panel.rs` and will move to `command_panel/tunnel_actions.rs`:
- `execute_tunnel_upgrade()` (line 1990)
- `execute_tunnel_cancel_upgrade()` (line 2054)
- `execute_tunnel_select_expansion()` (line 2108)
- `execute_tunnel_eject_unit()` (line 2150)

After the directory split, accessible via `use super::tunnel_actions::*` from `actions.rs`.

### Key Types and Imports Needed

From `crate::ui::types`: `CommandButtonAction`, `ObjectInterfaceState`, `StructureMenuState`, `AgentMenuState`, `CultsRecruitMenuState`, `CommandPanelTarget`, `PlacementState`
From `crate::game::types`: `DeploymentCenterState`, `BarracksState`, `HeadquartersState`, `RecruitmentCenterState`, `ArmoryState`, `ExtractionFacilityState`, `SupplyTowerState`, `TunnelState`, `Player`, `GdoPlayerResources`, `SyndicatePlayerResources`, `CultsPlayerResources`, `StructureInstance`, `Owner`
From `crate::game::types::cults_structure_stats`: `SOLDIER_TRAINING_COST`, `GUNNER_TRAINING_COST`
From `crate::game::units::types::commands`: `CommandType`, `HoldingPosition`, `UnitCommand`, `CommandQueue`
From `crate::game::units::types::movement`: `MoveTarget`, `Path`, `Velocity`
From `crate::game::combat::types`: `AttackState`
From `crate::types`: `Selection`, `Selected`, `Unit`
Bevy component: `ArmoryEjectionQueue` (from `crate::game::types`)

### Pattern for Sub-dispatcher Signatures

Each sub-dispatcher should accept only the parameters it actually uses. For example:

```rust
fn execute_dc_action(
    action: &CommandButtonAction,
    interface_state: &mut ResMut<ObjectInterfaceState>,
    panel_target: &Res<CommandPanelTarget>,
    dc_query: &mut Query<(&Owner, &mut DeploymentCenterState)>,
    players: &mut Query<(&Player, &mut GdoPlayerResources)>,
) { ... }
```

The main `execute_command_action()` becomes a thin match like:

```rust
fn execute_command_action(/* all params */) {
    match action {
        CommandButtonAction::DcOpenBuildMenu | CommandButtonAction::DcBuild(_) | CommandButtonAction::DcCancel => {
            execute_dc_action(action, interface_state, panel_target, dc_query, players);
        }
        CommandButtonAction::BkTrain(_) | CommandButtonAction::BkCancel | ... => {
            execute_production_action(action, ...);
        }
        CommandButtonAction::UnitMove | CommandButtonAction::UnitAttack | ... => {
            execute_unit_command_action(action, ...);
        }
        CommandButtonAction::TunnelUpgrade | ... => {
            execute_tunnel_action(action, ...);
        }
        CommandButtonAction::Back | CommandButtonAction::EnterPlacement => {
            execute_placement_action(action, interface_state);
        }
    }
}
```

### Callers of execute_command_action

Two call sites currently exist:
1. `handle_command_button_clicks()` (line 941) -- mouse click handler, passes `shift_held: false` always (no keyboard access)
2. `command_panel_hotkeys()` (line 1160) -- keyboard handler, computes `shift_held` from keyboard state

Both pass the same full parameter set. After the refactor, these callers do NOT change -- they still call `execute_command_action()` with the same signature.

### Categorization Decision for Edge Cases

- `SetRallyPoint` goes in **unit_command** (it sets AwaitingTarget, same pattern as UnitMove/UnitAttack)
- `EfBuildPlate`/`EfCancel` go in **production** (they handle EF construction, not DC)
- `AgentBuildTunnel`/`AgentDropOff`/`RecruitConstruct`/`RecruitSelectBuilding`/`RecruitAssistConstruction` go in **unit_command** (they are issued from unit selection, not structure)
- `StScheduleDeliveries` goes in **production** (it is a SupplyTower action)
- `Back` and `EnterPlacement` go in **placement** (navigation/placement mode transitions)

## Dependencies

- **command_panel_directory_split** (planned_task) -- This task CANNOT begin until the directory split is complete. The target file `actions.rs` does not exist until `command_panel.rs` is split into `command_panel/` directory modules. The function to decompose, its helpers, and its tests will all be in the new directory structure.
