# Codebase Refactoring Priorities

## Date: 2026-03-23
## Status: Analysis complete, awaiting architecture_request approval for QA unblock first

## Overview

The Space Crystals RTS codebase totals ~50,000 lines of Rust across `artifacts/developer/src/`. Rapid feature development has created several structural issues that impede maintainability, increase bug risk, and slow down task planning and splitting. This document identifies the highest-priority refactoring targets with evidence and concrete proposals.

## Priority 1: `right_click_move_command` Monolith (units/systems/core.rs)

**File**: `src/game/units/systems/core.rs` (5612 lines)
**Function**: `right_click_move_command` (lines 243-600+)

### Problem

This single function handles ALL right-click and command-mode-left-click interactions for every unit type. It contains:
- A 12-parameter system signature with complex queries (9-tuple destructuring)
- Sequential `if command_type == CommandType::X` blocks, each ~30-50 lines, for: Attack, DropOff, Enter, PickUpSupplies, AttachToTower, DropOffSupplies, plus default right-click behaviors for choppers and agents
- Massive code duplication: the pattern `check_attack_interruptible -> clear_movement_state -> issue_or_queue_command` is repeated verbatim 10+ times
- The `target_info` query bundles `Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>, Option<&ArmoryState>` -- every new interactable structure requires adding another Option, which cascades into every destructure site
- Every new unit type or command type requires modifying this function

### Impact

The task_planner reports this is the single largest pain point: almost every combat or command task touches this function. The linear growth pattern means each new feature increases the risk of regression in unrelated command paths.

### Proposed Refactoring

**Command Dispatch Pattern**: Extract each command type handler into its own function, with a trait-based or match-based dispatch at the top level.

```rust
// Before: monolithic if-chain
if command_type == CommandType::DropOff { ... 30 lines ... }
if command_type == CommandType::Enter { ... 30 lines ... }
// ... 8 more blocks

// After: dispatch table
match command_type {
    CommandType::Attack => handle_attack_command(&ctx, &mut selected_units, &target_info),
    CommandType::DropOff => handle_dropoff_command(&ctx, &mut selected_units, &target_info),
    CommandType::Enter => handle_enter_command(&ctx, &mut selected_units, &target_info),
    // ...
}
```

**CommandContext struct**: Bundle the shared parameters (shift_held, local_player, cursor_target, interface_state) into a struct to reduce parameter passing.

**Target Info Trait**: Replace the 6-Option query tuple with a trait or enum:
```rust
enum TargetKind {
    SupplyStation,
    SupplyTower(SupplyTowerState),
    Tunnel(TunnelState),
    Armory(ArmoryState),
    CrystalPatch,
    Other,
}
```

### Testing Requirements

- Each extracted handler function should be unit-testable via TestHarness
- Existing unit tests for command issuance must continue passing
- Integration tests for attack, move, chopper commands, agent commands must pass
- New tests: one per extracted handler verifying the command is issued correctly

## Priority 2: `command_panel.rs` Monolith (ui/command_panel.rs)

**File**: `src/ui/command_panel.rs` (6357 lines -- largest file in the project)
**Functions**: `get_grid_slot_action` (lines 43-260) and `execute_command_action` (lines 1260-1600+)

### Problem

- `get_grid_slot_action`: 200+ line match tree mapping (ObjectInterfaceState, row, col) to CommandButtonAction. Every new structure menu state or command button adds branches.
- `execute_command_action`: 16+ parameters (hitting Bevy's SystemParam limit), handles all button actions. Separate queries for DC, Barracks/HQ/RC/Armory, EF, SupplyTower, Tunnel -- all to avoid borrow conflicts.
- The developer reports hitting the 16-param system limit and needing workarounds.

### Proposed Refactoring

**Data-driven grid layout**: Replace the match tree with a declarative grid layout per interface state:
```rust
fn grid_layout(state: &ObjectInterfaceState) -> &[GridSlot] {
    // Returns array of (row, col, action, condition) tuples
}
```

**Split execute by category**: Group related actions into sub-functions:
- `execute_dc_action` (DC build/cancel/placement)
- `execute_production_action` (Barracks/HQ/RC/Armory train/cancel)
- `execute_unit_action` (Move/Stop/Patrol/Attack/Hold)

**SystemParam bundle**: Use a custom SystemParam struct to group related queries and reduce parameter count.

### Testing Requirements

- Grid layout tests: verify each interface state produces the correct button layout
- Action execution tests: verify each action modifies the correct state
- Regression: all existing command panel tests must pass

## Priority 3: `faction.rs` Production System Duplication (world/faction.rs)

**File**: `src/game/world/faction.rs` (4387 lines)
**Functions**: `barracks_production_tick_system`, `headquarters_production_tick_system`, `supply_tower_production_tick_system`, `recruitment_center_production_system`, `armory_training_tick_system`

### Problem

Five production systems follow an identical pattern:
1. Check if current_build is None and queue is non-empty -> start next build
2. Tick progress (with power_ratio or fixed 1.0)
3. On completion: determine spawn position, call spawn helper, update unit control, issue rally command
4. Clear build state

The only differences are: which state component (BarracksState vs HeadquartersState vs...), which resource type (GdoPlayerResources vs SyndicatePlayerResources vs...), spawn position logic, and power scaling.

### Proposed Refactoring

**Production trait**: Define a trait that each production structure implements:
```rust
trait ProductionStructure {
    fn build_queue(&mut self) -> &mut Vec<ObjectEnum>;
    fn current_build(&mut self) -> &mut Option<ObjectEnum>;
    fn progress(&mut self) -> &mut Option<f32>;
    fn production_cost(unit: &ObjectEnum) -> Option<ProductionCost>;
    fn power_ratio(&self, ...) -> f32; // 1.0 for non-GDO
}
```

**Generic production tick**: One parameterized system using the trait, instead of five copy-pasted systems.

### Testing Requirements

- Existing production tests (1886 unit tests include production tick coverage) must pass
- Test each production structure variant through the generic system
- Verify power scaling differences are preserved

## Priority 4: Dual Movement Pipeline

**Components**: `MoveTarget`/`Path` (135 references) vs `LocomotionChannel`/`OrientationChannel` (378 references)

### Problem

Two movement systems coexist:
- **MoveTarget/Path**: Older pipeline used by command issuance (right_click_move_command, rally commands) and combat approach. Waypoint-based.
- **LocomotionChannel/OrientationChannel**: Newer channel-based pipeline used by unit behavior systems. Provides smoother motion with turn rates.

The task_planner identifies this as a source of subtle bugs: the attack approach path issue where AttackTarget relies on combat behaviors computing approach paths separately from inline path computation in the Move command.

### Proposed Refactoring

This is the most complex refactor and should be deferred until Priorities 1-3 are addressed. The approach would be:
1. Migrate all MoveTarget users to set LocomotionChannel instead
2. Remove MoveTarget/Path components
3. Unify approach path computation into the channel pipeline

### Testing Requirements

- All movement-related tests must pass
- Specific regression tests for: attack approach, rally point movement, patrol, chopper movement
- Integration tests for multi-step command sequences (move -> attack -> move)

## Priority 5: `target_info` Query Proliferation

**Location**: `right_click_move_command` in core.rs

### Problem

The `target_info` query bundles all possible interactable structure state components as Options. Adding a new structure type requires:
1. Adding `Option<&NewState>` to the query tuple
2. Updating every destructure site (10+ locations)
3. Risk of missing a destructure site, causing a compilation error or incorrect behavior

### Proposed Refactoring

This is addressed as part of Priority 1 (right_click_move_command refactor). The TargetKind enum approach eliminates the need for per-type Options in the query.

## Sequencing

1. **QA unblock first** -- the architecture_request for test compilation fixes is already pending. This must land before refactoring begins so that refactoring changes can be verified.
2. **Priority 1** (core.rs command dispatch) -- highest impact, most bug-prone, enables further refactoring
3. **Priority 2** (command_panel.rs split) -- second-highest impact, can proceed in parallel with Priority 1 since they are in different modules
4. **Priority 3** (faction.rs production trait) -- medium impact, cleaner but less bug-prone
5. **Priority 4** (movement pipeline) -- highest risk, defer until foundational refactors are stable

## Metrics

Current state:
- Top 4 files: 22,737 lines (45% of codebase in 4 files)
- `right_click_move_command`: ~360 lines, 12 parameters, 10+ command type branches
- `command_panel.rs`: 6,357 lines, largest file
- Production systems: 5 near-identical implementations totaling ~600 lines

Target state after refactoring:
- No file exceeds 2,000 lines
- No function exceeds 100 lines
- No system exceeds 12 parameters
- Production code deduplicated into one generic implementation
