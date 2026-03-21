# supply-chopper-command-panel

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-supply_chopper_commands.md

## Task

Implement the SupplyChopper command panel grid with target commands, AwaitingTarget resolutions, and availability gating.

### Changes needed:

**1. Add CommandButtonAction variants** (`ui/command_panel.rs`):
- Add `ChopperPickUpSupplies` — target command, enters AwaitingTarget[PickUpSupplies]
- Add `ChopperAttachToTower` — target command, enters AwaitingTarget[AttachToTower]  
- Add `ChopperDropOffSupplies` — target command, enters AwaitingTarget[DropOffSupplies]
- Follow the pattern of existing target commands (AgentGather, AgentDropOff, UnitEnter)

**2. Add SupplyChopper command panel grid**:
Grid layout (row, col):
  - (0, 0) = Move (Q) — target command, enters AwaitingTarget[Move]
  - (0, 1) = Pick Up Supplies (W) — target command, availability: NOT carrying units
  - (0, 2) = Attach to Tower (E) — target command, availability: NOT carrying units
  - (1, 0) = Drop Off Supplies (A) — target command, availability: carrying supplies > 0
  - (1, 2) = Hold Position (D) — immediate command
  - (2, 1) = Stop (X) — immediate command

**3. Add `object_type_supports_action` entries**:
- SupplyChopper supports: UnitMove, UnitHoldPosition, UnitStop, ChopperPickUpSupplies, ChopperAttachToTower, ChopperDropOffSupplies
- SupplyChopper does NOT support: UnitAttack, UnitAttackMove, UnitAttackGround, UnitPatrol, UnitReverse, UnitEnter, AgentGather, AgentDropOff

**4. Implement AwaitingTarget resolutions**:
- AwaitingTarget[PickUpSupplies]: left-click SDS -> issue PickUpSupplies command, return to Default
- AwaitingTarget[AttachToTower]: left-click own SupplyTower with no attached chopper -> issue AttachToTower command, return to Default
- AwaitingTarget[DropOffSupplies]: left-click own SupplyTower with no attached chopper -> issue DropOffSupplies command, return to Default
- AwaitingTarget[Move]: left-click ground/object -> issue Move command, return to Default

**5. Add button availability checks**
**6. Add button labels**
**7. Add CommandType variants if needed**

## Technical Context

### What Already Exists (DO NOT re-implement)

Significant implementation already exists. The developer must audit what's present before making changes:

**CommandButtonAction variants** (`artifacts/developer/src/ui/types.rs` lines 307-310):
- `ChopperPickUpSupplies` — ALREADY EXISTS
- `ChopperAttachToTower` — ALREADY EXISTS
- `ChopperDropOffSupplies` — DOES NOT EXIST, must be added

**CommandType variants** (`artifacts/developer/src/game/units/types/state/commands.rs` lines 94-95):
- `CommandType::PickUpSupplies` — ALREADY EXISTS
- `CommandType::AttachToTower` — ALREADY EXISTS
- `CommandType::DropOffSupplies` — DOES NOT EXIST, must be added (distinct from `CommandType::DropOff` which is agent-specific)

**SelectedUnitCapabilities** (`artifacts/developer/src/ui/types.rs` line 382):
- `is_chopper: bool` — ALREADY EXISTS, set in `compute_selected_unit_capabilities()` (command_panel.rs:1699)
- Missing: fields for chopper carrying state (`chopper_has_supplies: bool`) needed for DropOffSupplies availability gating

**Chopper grid** (`artifacts/developer/src/ui/command_panel.rs` lines 141-149):
- ALREADY EXISTS but with WRONG layout: currently (0,0)=Move, (0,1)=PickUp, (0,2)=Attach, (1,0)=Stop, (1,1)=HoldPosition
- Must be UPDATED to match spec: (0,0)=Move, (0,1)=PickUp, (0,2)=Attach, (1,0)=DropOff, (1,2)=HoldPosition, (2,1)=Stop

**execute_command_action handlers** (command_panel.rs:1478-1485):
- `ChopperPickUpSupplies` -> AwaitingTarget(PickUpSupplies) — ALREADY EXISTS
- `ChopperAttachToTower` -> AwaitingTarget(AttachToTower) — ALREADY EXISTS
- `ChopperDropOffSupplies` -> AwaitingTarget(DropOffSupplies) — DOES NOT EXIST, must be added

**AwaitingTarget resolution in core.rs** (`artifacts/developer/src/game/units/systems/core.rs`):
- `CommandType::PickUpSupplies` handling (lines 348-375) — ALREADY EXISTS
- `CommandType::AttachToTower` handling (lines 377-404) — ALREADY EXISTS
- `CommandType::DropOffSupplies` handling — DOES NOT EXIST, must be added (pattern: left-click own SupplyTower -> issue command, see AttachToTower handler as template)

**object_type_supports_action** (command_panel.rs:2237-2239):
- `ChopperPickUpSupplies | ChopperAttachToTower => matches!(obj, ObjectEnum::SupplyChopper)` — ALREADY EXISTS
- `ChopperDropOffSupplies` — DOES NOT EXIST, must be added to this match arm

**is_action_active** (command_panel.rs:2401-2402):
- PickUpSupplies and AttachToTower entries — ALREADY EXISTS
- DropOffSupplies — DOES NOT EXIST, must be added

**Labels** (command_panel.rs:2349-2350):
- ChopperPickUpSupplies and ChopperAttachToTower labels — ALREADY EXISTS
- ChopperDropOffSupplies — DOES NOT EXIST, must be added

**is_unit_action** (command_panel.rs around 2193-2195):
- ChopperPickUpSupplies and ChopperAttachToTower — ALREADY EXISTS
- ChopperDropOffSupplies — DOES NOT EXIST, must be added

**Tests** (command_panel.rs lines ~5013-5071):
- Grid tests, label tests, is_action_active tests for PickUp/Attach — ALREADY EXISTS
- Must be updated/extended to cover DropOffSupplies and the corrected grid layout

### Files to Change

1. **`artifacts/developer/src/ui/types.rs`** — Add `ChopperDropOffSupplies` to `CommandButtonAction` enum (after line 310). Add `chopper_has_supplies: bool` to `SelectedUnitCapabilities` (after line 382).

2. **`artifacts/developer/src/game/units/types/state/commands.rs`** — Add `DropOffSupplies` variant to `CommandType` enum (line ~96). Add `name()` and `hotkey()` entries. Note: there is NO `UnitCommand::DropOffSupplies` variant — the chopper DropOff should reuse or add one. Currently `UnitCommand::DropOffResources` is agent-specific. Consider adding a new `UnitCommand::DropOffSupplies(Entity)` variant for choppers, or reuse the existing `DropOffResources` with appropriate filtering. The design doc says DropOff targets own SupplyTower, similar to AttachToTower.

3. **`artifacts/developer/src/ui/command_panel.rs`** — Multiple change sites:
   - **Grid layout** (line 141-149): Fix to match spec — add DropOff at (1,0), move HoldPosition to (1,2), move Stop to (2,1)
   - **execute_command_action** (around line 1485): Add `ChopperDropOffSupplies => AwaitingTarget(CommandType::DropOffSupplies)` handler
   - **object_type_supports_action** (line 2237-2239): Add `ChopperDropOffSupplies` to the match arm
   - **is_unit_action** (line 2193-2195): Add `ChopperDropOffSupplies`
   - **is_action_active** (line 2401-2402): Add `(ChopperDropOffSupplies, CommandType::DropOffSupplies)`
   - **grid_button_label** (line 2349-2350): Add `ChopperDropOffSupplies => "[A] Drop Off\nSupplies"`
   - **grid_button_enabled** (line 2354-2384): Add availability checks — `ChopperDropOffSupplies` disabled when chopper has 0 supplies (needs access to SupplyChopperState or use SelectedUnitCapabilities)
   - **compute_selected_unit_capabilities** (line 1669-1704): Add logic to read `SupplyChopperState.carried_supplies > 0` into a new `chopper_has_supplies` cap. Note: the query at line 1670 includes `Option<&AgentCarryState>` but NOT `Option<&SupplyChopperState>` — the query must be extended, or the function's query parameter type must be broadened.

4. **`artifacts/developer/src/game/units/systems/core.rs`** — Add AwaitingTarget resolution for `CommandType::DropOffSupplies` (after the AttachToTower block at line 404). Pattern: same as AttachToTower handler but issues a different UnitCommand. Also needs a `UnitCommand` variant for the chopper DropOff action.

### Key Patterns to Follow

**Target command flow** (see AgentDropOff as pattern):
1. Button press -> `execute_command_action` sets `ObjectInterfaceState::AwaitingTarget(CommandType::X)`
2. User clicks target -> `right_click_move_command` in core.rs matches `command_type == CommandType::X` and issues `UnitCommand::X(entity)`
3. After issuing command, resets to `ObjectInterfaceState::Default`

**Chopper guard in right-click handler** (core.rs:255-256):
```rust
let has_selected_choppers = selected_units.iter().any(|(_, _, _, _, _, chopper_state, _, _, _)| chopper_state.is_some());
```
The selected_units query already includes `Option<&SupplyChopperState>` — use `chopper_opt.is_some()` to filter chopper units.

**SelectedUnitCapabilities approach for grid_button_enabled**:
- Currently `grid_button_enabled()` does NOT receive `SelectedUnitCapabilities` — it uses static cost checks
- For chopper availability gating, either: (a) pass `unit_caps` to `grid_button_enabled()`, or (b) add the check directly in the grid slot function with a guard condition like existing `if caps.has_attack`
- Option (b) is simpler: add `if caps.chopper_has_supplies` guard on the DropOff grid slot at (1,0)

**Query for SupplyChopperState in compute_selected_unit_capabilities**:
The function signature at line 1669-1670 takes a query parameter. To read SupplyChopperState, the query must be extended:
```rust
// Current:
selected_units: &Query<(Entity, Option<&AttackCapability>, &UnitBaseEnum, &ObjectInstance, Option<&AgentCarryState>), ...>
// Needed:
selected_units: &Query<(Entity, Option<&AttackCapability>, &UnitBaseEnum, &ObjectInstance, Option<&AgentCarryState>, Option<&SupplyChopperState>), ...>
```
This also requires updating the query in `update_command_panel_state` (line 294) where it's defined, and any destructuring patterns.

### UnitCommand for Chopper DropOff

The codebase does NOT have a `UnitCommand::DropOffSupplies` variant. Options:
1. **Add `UnitCommand::DropOffSupplies(Entity)`** — cleanest, mirrors PickUpSupplies/AttachToTower pattern
2. Reuse `UnitCommand::DropOffResources(Entity)` — bad idea, semantically different (agent crystals vs chopper supplies)

Recommendation: Add `UnitCommand::DropOffSupplies(Entity)` to the UnitCommand enum in commands.rs, with `is_available()` returning true (same as PickUpSupplies/AttachToTower — chopper-specific UI handles visibility).

### Imports

- `SupplyChopperState` is in `crate::game::types::structures` — import in command_panel.rs if needed for query extension
- `SupplyDeliveryStation` is already imported in core.rs target_info query

## Dependencies

- **No blocking dependencies** — all prerequisite types (CommandType::PickUpSupplies, CommandType::AttachToTower, ChopperPickUpSupplies, ChopperAttachToTower, is_chopper cap, AwaitingTarget resolution for PickUp/Attach) already exist. This task adds the missing DropOffSupplies flow and corrects the grid layout.
