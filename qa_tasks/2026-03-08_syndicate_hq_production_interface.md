# Ticket: Syndicate Headquarters Must Show Agent Production Commands

## Current State
When selecting the Syndicate Headquarters (underground expansion), the command panel shows unit commands (Move, Attack, etc.) instead of Agent production commands. This prevents the player from producing Agents, breaking the core Syndicate production loop.

## Desired State
The Headquarters expansion must have an ObjectInterfaceState that shows Agent production in its command panel. When selected:

### DefaultState commands:
- **A: Produce Agent** — CommandIssuingTransition. Costs 100 SC, takes 160 frames (10 seconds). Greyed out if insufficient Supply Credits or if the production queue is full. Queues an Agent for production; produced Agent emerges from the parent Tunnel or enters the Tunnel Network depending on rally point.

The Headquarters should NOT show Move, Attack, or other unit commands — it is a structure, not a unit.

## Justification
Discovered during QA session 2026-03-08 (forum topic `qa_session_2026_03_08_issues.md`, issue #3). `features/syndicate_objects.md` specifies "Headquarters produces Agent (100 SC, 160 frames / 10 seconds)" but no explicit ObjectInterfaceState was defined for the Headquarters expansion. The intent is clear: the HQ is the Syndicate's primary unit-producing structure, analogous to GDO Barracks. Without this interface, Syndicate faction gameplay is non-functional — the player cannot produce any units.

Note: The feature spec may need a formal Headquarters ObjectInterfaceState section added by the product analyst. This ticket covers the implementation of the production interface based on existing spec intent.

## Technical Context

### Root Cause
`update_command_panel_state()` at `src/ui/command_panel.rs:274` matches on `obj_instance.object_type`. `ObjectEnum::Headquarters` falls through to the `_ =>` wildcard at line 343, which sets `ObjectInterfaceState::Default` (unit commands). Fix requires adding an explicit match arm for Headquarters.

### Existing Infrastructure (Already Implemented)
- **`HeadquartersState`** component at `src/game/types/structures.rs:231-276` — FULLY DEFINED with:
  - `build_queue: Vec<ObjectEnum>`, `current_build: Option<ObjectEnum>`, `current_build_progress: Option<f32>`
  - `MAX_QUEUE_SIZE: usize = 5`
  - `production_cost(ObjectEnum::SyndicateAgent) -> Some(StructureCost { space_crystals: 100, build_frames: 160 })`
  - `try_queue()`, `cancel_last()` methods
- **`spawn_headquarters()`** at `src/game/utils.rs:660-702` — spawns HQ with `HeadquartersState::default()` at line 697
- **`spawn_syndicate_agent()`** at `src/game/utils.rs:516-600` — spawn target for produced Agents
- **HQ stats** at `src/game/types/structures.rs:388-409` (`syndicate_structure_stats` module): `HQ_MAX_HP: 200.0`

### Template: Barracks Production Pattern
The GDO Barracks is the closest analogue. Follow these files as templates:

1. **`barracks_production_tick_system()`** at `src/game/world/faction.rs:221-289`:
   - Queries `(Entity, &Owner, &GridPosition, &mut BarracksState, &StructureInstance)`
   - If no current build but queue has items → starts next build (deducts cost from `GdoPlayerResources`)
   - Ticks `current_build_progress` by 1.0 per frame (power ratio applied)
   - When `progress >= build_frames` → calls `spawn_peacekeeper()`, issues rally command, resets state
   - For HQ: use `SyndicatePlayerResources` instead of `GdoPlayerResources`, `spawn_syndicate_agent()` instead of `spawn_peacekeeper()`

2. **BarracksMenu in `get_grid_slot_action()`** at `command_panel.rs:71-75`:
   ```rust
   StructureMenuState::BarracksMenu => match (row, col) {
       (0, 0) => Some(CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper)),
       (0, 1) if bk_has_queue => Some(CommandButtonAction::BkCancel),
       _ => None,
   },
   ```

3. **BarracksMenu match arm in `update_command_panel_state()`** at `command_panel.rs:299-304`:
   ```rust
   ObjectEnum::Barracks => {
       let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
       if *interface_state != new_state || target_changed {
           *interface_state = new_state;
       }
   }
   ```

### Changes Required

#### 1. UI Types (`src/ui/types.rs`)
- **Add `HeadquartersMenu` variant** to `StructureMenuState` enum (after line 207/`SupplyTowerMenu`):
  ```rust
  /// Headquarters selected — production menu
  HeadquartersMenu,
  ```
- **Add `HqTrain(ObjectEnum)` and `HqCancel` variants** to `CommandButtonAction` enum (after line 268/`StCancel`):
  ```rust
  /// Headquarters: Train an Agent
  HqTrain(crate::types::ObjectEnum),
  /// Headquarters: Cancel last queued item
  HqCancel,
  ```

#### 2. Command Panel (`src/ui/command_panel.rs`)

**a. `get_grid_slot_action()` — add match arm** (after line 93, before TunnelIdle):
```rust
StructureMenuState::HeadquartersMenu => match (row, col) {
    (0, 0) => Some(CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent)),
    (0, 1) if bk_has_queue => Some(CommandButtonAction::HqCancel),
    _ => None,
},
```
Note: The `bk_has_queue` variable is reused for all production buildings — rename it to something generic like `has_queue`, or add an `hq_has_queue` check. Currently (line 798-805), it checks both `bk_query` and `st_query_mut`. Add `hq_query` check to this chain.

**b. `update_command_panel_state()` — add match arm for Headquarters** (before the `_ =>` wildcard at line 343):
```rust
ObjectEnum::Headquarters => {
    let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
    if *interface_state != new_state || target_changed {
        *interface_state = new_state;
    }
}
```

**c. Structure query in `rebuild_command_panel_ui()`** (line 220 area):
- Add `Option<&HeadquartersState>` to the `selected_structures` query tuple
- Add a dedicated `hq_query: Query<(&Owner, &HeadquartersState)>` for production state reads (follows pattern of `bk_query`)

**d. Title mapping in `rebuild_command_panel_ui()`** (around line 397):
- Add: `StructureMenuState::HeadquartersMenu => "Headquarters"`

**e. Production progress UI** (after line 441, following BarracksMenu pattern):
- Add match arm for `HeadquartersMenu` that renders:
  - "[Q] Train Agent" button (enabled when `SyndicatePlayerResources.supply_credits >= 100` AND queue not full)
  - "[W] Cancel" button (visible only when `build_queue` is not empty)
  - Production progress bar (using `current_build_progress / build_frames`)

**f. `execute_command_action()` — add match arms** (in the `match action` at line 834):
```rust
CommandButtonAction::HqTrain(object_type) => {
    let Some(target_entity) = panel_target.entity else { return };
    let Ok((owner, mut hq)) = hq_query.get_mut(target_entity) else { return };
    // Check cost affordability against SyndicatePlayerResources
    // Call hq.try_queue(*object_type)
}
CommandButtonAction::HqCancel => {
    let Some(target_entity) = panel_target.entity else { return };
    let Ok((_, mut hq)) = hq_query.get_mut(target_entity) else { return };
    hq.cancel_last();
}
```
- Add `hq_query: &mut Query<(&Owner, &mut HeadquartersState)>` parameter to `execute_command_action()`

**g. Keyboard hotkey handler** (line 798-805):
- Add HQ queue check to the `bk_has_queue` chain:
  ```rust
  || panel_target.entity
      .and_then(|e| hq_query.get(e).ok())
      .map(|(_, hq)| !hq.build_queue.is_empty())
      .unwrap_or(false);
  ```

#### 3. Production Tick System (`src/game/world/faction.rs`)

**Add `headquarters_production_tick_system()`** — new system function after `barracks_production_tick_system()` (after line 289). Template:

```rust
pub fn headquarters_production_tick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hq_query: Query<(Entity, &Owner, &crate::types::GridPosition, &mut HeadquartersState, &StructureInstance)>,
    mut players: Query<(&Player, &mut SyndicatePlayerResources)>,
) {
    for (entity, owner, grid_pos, mut hq, structure) in hq_query.iter_mut() {
        // Skip if under construction
        // If no current_build and queue non-empty: pop front, deduct cost, start build
        // Tick current_build_progress += 1.0
        // When complete: spawn_syndicate_agent() at tunnel exit position, reset state
    }
}
```

Key differences from Barracks:
- Uses `SyndicatePlayerResources` (not `GdoPlayerResources`)
- Calls `spawn_syndicate_agent()` (not `spawn_peacekeeper()`)
- Agent emerges from parent Tunnel, not from Barracks grid position — HQ is underground, the spawn position needs to be the parent Tunnel's surface position. The `spawn_headquarters()` takes a `parent_tunnel: Entity` param. Store/query the parent tunnel to determine spawn location.
- No power ratio (Syndicate doesn't have a power grid system yet)

**Register the system** in the plugin's `build()` method — add to `FixedUpdate` schedule alongside `barracks_production_tick_system`.

#### 4. Imports
- `src/ui/command_panel.rs`: Add `use crate::game::types::structures::HeadquartersState;`
- `src/game/world/faction.rs`: `HeadquartersState` import (check if already imported via structure types)

### Agent Spawn Location Note
The HQ is an underground structure — produced Agents should emerge from the **parent Tunnel** (surface entity). The `spawn_headquarters()` function at utils.rs:660 receives `parent_tunnel: Entity`. Either:
1. Store the parent tunnel entity reference in `HeadquartersState` (add field), or
2. Query the tunnel's `GridPosition` via the HQ's parent relationship

The Barracks spawns units at its own grid position offset. The HQ should spawn at the parent tunnel's grid position instead.

## Dependencies
- `command_panel_and_interface_state_machine` — ObjectInterfaceState and command panel infrastructure
- `syndicate_agent_unit` — SyndicateAgent ObjectEnum variant and spawn function
- `tunnel_structure_and_network` — TunnelState, parent tunnel relationship
- `faction_resource_definitions` — SyndicatePlayerResources

## QA Steps
1. [human] Start a game as Syndicate — select the Headquarters expansion inside the starting Tunnel
2. [human] Verify the command panel shows "Produce Agent" (A) — NOT Move/Attack/unit commands
3. [human] Click Produce Agent with sufficient Supply Credits — verify an Agent is queued for production
4. [human] Wait 160 frames (10 seconds) — verify the Agent is produced and emerges from the parent Tunnel
5. [human] Verify the cost (100 SC) is deducted when production begins
6. [human] Reduce Supply Credits below 100 — verify Produce Agent is greyed out / unavailable
7. [human] Select the Headquarters again while an Agent is in production — verify production progress is visible

## Expected Experience
Selecting the Headquarters shows a clean production interface with a single "Produce Agent" button. Clicking it begins visible production. After 10 seconds, an Agent unit appears from the parent Tunnel. The interface feels like a standard RTS production building — queue, wait, unit emerges. No unit movement or attack commands are shown.

## QA Results — 2026-03-09
- Step 1 [human]: PASS — HQ selectable
- Step 2 [human]: PASS — Command panel shows production commands (Agent, Guard), not unit commands
- Step 3 [human]: PASS — Agent queued successfully
- Step 4 [human]: FAIL — Agent emerges under the HQ and is stuck (HQ tiles not walkable — see forum topic `syndicate_hq_blocks_agent_movement`). Should emerge from parent Tunnel's surface position.
- Step 5 [human]: PASS — 100 SC deducted on production start
- Step 6 [human]: UNTESTABLE — Could not reduce SC below 100 (no spending mechanism available)
- Step 7 [human]: PASS — Production progress visible while unit is building
