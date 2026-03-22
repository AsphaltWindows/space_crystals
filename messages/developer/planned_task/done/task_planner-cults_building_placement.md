# cults_building_placement

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-cults_objects_formalized.md

## Task

Implement the Cults building placement flow — the command panel and placement system that allows Recruits to construct buildings.

### Overview

Cults buildings are built by Recruits (not by a Deployment Center). The flow is:
1. Select one or more CultsRecruit units
2. Select the Construct option (command panel button)
3. Select the desired building from a submenu
4. Left-click on the ground to place it
5. All selected Recruits are issued a walk-to-site command

Additionally, an Assist Construction command allows Recruits to join an in-progress Cults building.

### Recruit command panel additions

In command_panel.rs, when selected units are CultsRecruit and state is Default:
- Add a **Construct** button that transitions to a build submenu (new AgentMenu variant or a new CultsConstructMenu state)
- In the build submenu, show available Cults buildings:
  - **Q: Storage** — transitions to AwaitingPlacement state with Storage ghost
- Add a **Z: Back** button to return from submenu to Default
- Add an **Assist Construction** button (e.g., slot S or D) that transitions to AwaitingTarget mode for targeting in-progress Cults buildings

### Placement system

When in the Cults placement state (AwaitingPlacement for a Cults building):
- Show ghost building at cursor position (follow existing ghost placement pattern from DC/EF/Tunnel)
- Validate placement using can_place_building() (or Cults-specific variant — Cults don't use GdoBuildArea, they just need buildable terrain + no overlap)
- On left-click valid placement: spawn the building as under-construction (with ConstructionHP at 10% max HP), issue UnitCommand::Build(building_entity) or similar to all selected Recruits
- On right-click or Escape: cancel placement, return to Default

### Walk-to-site command

Define a new UnitCommand variant (e.g., UnitCommand::ConstructBuilding(Entity)) or reuse an existing pattern. When issued to a Recruit, it should cause the Recruit to walk toward the target building. The actual enter-and-build behavior will be handled by the cults_construction_system task.

### Assist Construction targeting

When in AwaitingTarget for Assist Construction:
- Left-click on an in-progress Cults building (has ConstructionHP, owned by same player): issue the same walk-to-building command to selected Recruits
- Left-click on anything else: cancel/ignore
- Escape: cancel, return to Default

### Notes
- CultsRecruit unit must exist (at least as a stub from the recruitment_center_auto_production task)
- Cults buildings do NOT require a build area — they can be placed on any valid buildable terrain
- The ghost/placement system should reuse existing patterns from DC AwaitingPlacement

## Technical Context

### Architecture Decision: CultsRecruitMenu (new AgentMenu-like state)

The Cults Recruit needs its own interface state, analogous to how SyndicateAgent uses `AgentMenu`. Create a **new top-level `ObjectInterfaceState` variant** `CultsRecruitMenu(CultsRecruitMenuState)` rather than overloading AgentMenu. This mirrors the Agent pattern but keeps faction-specific logic cleanly separated.

### New Types to Add

**1. `artifacts/developer/src/ui/types.rs`**

a) New `CultsRecruitMenuState` enum (after `AgentMenuState`, line ~232):
```rust
pub enum CultsRecruitMenuState {
    /// Recruit selected — DefaultState: Construct, Assist Construction
    RecruitDefault,
    /// Recruit selected — Build submenu: pick building to construct
    RecruitConstructMenu,
    /// Recruit selected — AwaitingPlacement: ghost preview for Cults building
    RecruitAwaitingPlacement,
}
```

b) New `ObjectInterfaceState` variant (line ~159, after AgentMenu):
```rust
CultsRecruitMenu(CultsRecruitMenuState),
```

c) Update `is_placement_mode()` (line ~164) to include:
```rust
ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement)
```

d) New `CommandButtonAction` variants (line ~307 area):
```rust
/// Recruit: Open Construct submenu
RecruitConstruct,
/// Recruit: Select a building to construct (carries the ObjectEnum)
RecruitSelectBuilding(crate::types::ObjectEnum),
/// Recruit: Assist Construction (enters AwaitingTarget)
RecruitAssistConstruction,
```

e) New `CommandType` variant in `game/units/types/state/commands.rs` (line ~100):
```rust
AssistConstruction,
```

f) New `UnitCommand` variant in `game/units/types/state/commands.rs` (line ~35):
```rust
/// Walk to a Cults building under construction and enter it (Recruit only)
ConstructBuilding(Entity),
```
Update `is_available()` to allow this for Cults units (add `is_cults: bool` param or just return `true` since the UI gates visibility).

### Files to Change

**1. `artifacts/developer/src/ui/command_panel.rs` (major changes)**

a) **`get_grid_slot_action()` (line ~52)** — Add match arms for all CultsRecruitMenu states:
```rust
ObjectInterfaceState::CultsRecruitMenu(crm) => match crm {
    CultsRecruitMenuState::RecruitDefault => match (row, col) {
        (0, 0) => Some(CommandButtonAction::RecruitConstruct),
        (1, 1) => Some(CommandButtonAction::RecruitAssistConstruction),
        _ => None,
    },
    CultsRecruitMenuState::RecruitConstructMenu => match (row, col) {
        (0, 0) => Some(CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage)),
        (2, 0) => Some(CommandButtonAction::Back),
        _ => None,
    },
    CultsRecruitMenuState::RecruitAwaitingPlacement => None, // handled by mouse + Escape
},
```

b) **`update_command_panel_state()` (line ~434)** — In the unit branch (after `active_is_agent` check at line 435), add CultsRecruit detection:
```rust
let active_is_cults_recruit = active_type == Some(ObjectEnum::CultsRecruit);
```
Then add a new branch (before the else at line 443):
```rust
else if active_is_cults_recruit {
    if !matches!(*interface_state, ObjectInterfaceState::CultsRecruitMenu(_) | ObjectInterfaceState::AwaitingTarget(_)) {
        *interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
        panel_target.entity = None;
    }
}
```
NOTE: `ObjectEnum::CultsRecruit` does not exist yet — it will be added by the `recruitment_center_auto_production` task. If it doesn't exist when you build, add a stub `CultsRecruit` variant to `ObjectEnum` in `shared/types.rs` (line ~335), `is_unit()` in `game/types/objects.rs` (line ~393), and `object_type()` (line ~214).

c) **`rebuild_command_panel_ui()` (line ~462)** — Add title/info section:
- Title match (line ~516): `ObjectInterfaceState::CultsRecruitMenu(_) => "Recruit"`
- Info panel: minimal — no progress bar needed for recruit menu
- Grid button spawning: follow the same loop pattern, use `get_grid_slot_action()` with the new CultsRecruitMenu states

d) **`execute_command_action()` (line ~1137)** — Add handlers:
```rust
CommandButtonAction::RecruitConstruct => {
    **interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
}
CommandButtonAction::RecruitSelectBuilding(building_type) => {
    placement_state.building_type = Some(*building_type);
    placement_state.source_entity = selected_units.iter().next().map(|(e, _, _)| e);
    placement_state.grid_pos = None;
    placement_state.is_valid = false;
    placement_state.rotation = StructureRotation::default();
    placement_state.flip_horizontal = false;
    placement_state.flip_vertical = false;
    **interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement);
}
CommandButtonAction::RecruitAssistConstruction => {
    **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::AssistConstruction);
}
```
Pattern: follow `AgentBuildTunnel` handler at line 1571.

e) **Escape handler in `command_panel_hotkeys()` (line ~910)** — Add:
```rust
ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => {
    *interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
}
ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu) => {
    *interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
}
```
Also update the `AwaitingTarget(_)` Escape handler (line ~935) to check for CultsRecruit:
```rust
let active_is_cults_recruit = selection.active_type() == Some(ObjectEnum::CultsRecruit);
if active_is_cults_recruit {
    *interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
}
```

f) **`object_type_supports_action()` (line ~2211)** — Add CultsRecruit support:
```rust
CommandButtonAction::RecruitConstruct |
CommandButtonAction::RecruitSelectBuilding(_) |
CommandButtonAction::RecruitAssistConstruction => matches!(obj, ObjectEnum::CultsRecruit),
```

g) **`grid_button_label()` (line ~2300 area)** — Add labels:
```rust
CommandButtonAction::RecruitConstruct => format!("[{}] Construct", hotkey),
CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage) => format!("[{}] Storage", hotkey),
CommandButtonAction::RecruitSelectBuilding(_) => format!("[{}] Build", hotkey),
CommandButtonAction::RecruitAssistConstruction => format!("[{}] Assist\nConstruct", hotkey),
```

h) **`grid_button_enabled_ext()` (line ~2117)** — Add the new variants as always-enabled.

i) **`is_unit_action()` (line ~2188 area)** — Add:
```rust
CommandButtonAction::RecruitConstruct |
CommandButtonAction::RecruitSelectBuilding(_) |
CommandButtonAction::RecruitAssistConstruction => true,
```

**2. `artifacts/developer/src/game/world/faction.rs` (placement systems)**

a) **`manage_placement_ghost()` (line 1194)** — Add CultsRecruit placement branch in the `building_type` match (line ~1213):
```rust
ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => {
    placement_state.building_type  // already set by execute_command_action
}
```
Import `CultsRecruitMenuState` at the top of faction.rs (line 12).

b) **`update_placement_ghost()` (line 1290)** — Add Cults validation branch in the validity check (line ~1382). Cults placement uses `can_worker_place_structure()` (same as Agent tunnel placement — no build area, no fog check):
```rust
else if matches!(*panel_state, ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement)) {
    crate::game::world::utils::can_worker_place_structure(
        grid_x, grid_z, size_x, size_z, &tiles, &structures,
    ).is_ok()
}
```

c) **`placement_click_system()` (line 1433)** — Add right-click cancel and left-click place branches:

Right-click cancel (line ~1456):
```rust
ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => {
    *panel_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
}
```

Left-click place (line ~1503): Spawn the building under construction, issue ConstructBuilding command to all selected recruits:
```rust
ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => {
    let world_pos = grid_to_world(grid_x, grid_z, 1.0);
    // Spawn building under construction (10% HP via ObjectInstance::under_construction)
    let building_entity = match building_type {
        ObjectEnum::CultsStorage => {
            let owner = /* get owner from first selected recruit */;
            spawn_cults_storage_under_construction(
                &mut commands, &mut meshes, &mut materials,
                grid_x, grid_z, owner, rotation, flip_h, flip_v,
            )
        }
        _ => { return; }
    };
    // Issue ConstructBuilding command to ALL selected recruits
    for (entity, _, _) in selected_units.iter() {
        commands.entity(entity).insert(UnitCommand::ConstructBuilding(building_entity));
    }
    *panel_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
}
```
NOTE: `placement_click_system` currently does NOT have a `selected_units` query. You need to add one: `selected_recruits: Query<Entity, (With<Unit>, With<Selected>)>`.

You also need a `spawn_cults_storage_under_construction()` function. Create it in `game/utils.rs` following the pattern of `spawn_tunnel_under_construction()` (line 734):
- Use `ObjectInstance::under_construction(ObjectEnum::CultsStorage, STORAGE_MAX_HP)` instead of `ObjectInstance::destructible()`
- Add `ConstructionHP::new(build_frames)` component (define `STORAGE_BUILD_FRAMES` constant)
- The existing `spawn_cults_storage()` (line 988) is the reference for mesh/material/components

d) **Assist Construction click handler** — Add a new system or extend `placement_click_system` to handle `AwaitingTarget(CommandType::AssistConstruction)`:
When left-click on an entity that has `ConstructionHP` + `ObjectInstance.object_type` is a Cults building + same Owner:
```rust
for entity in selected_recruits.iter() {
    commands.entity(entity).insert(UnitCommand::ConstructBuilding(target_entity));
}
*panel_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
```
This can be a new system or added as a branch in `right_click_move_command` (core.rs:179). The AwaitingTarget pattern for left-click handlers is used by `set_rally_point_click_system` (core.rs:952) — follow that pattern.

**3. `artifacts/developer/src/game/units/types/state/commands.rs`**

- Add `UnitCommand::ConstructBuilding(Entity)` variant (line ~35)
- Add `CommandType::AssistConstruction` variant (line ~100)
- Update `is_available()` — return `true` for `ConstructBuilding` (UI gates visibility)
- Add `name()` and `hotkey()` entries for `AssistConstruction`

**4. `artifacts/developer/src/shared/types.rs`** (if CultsRecruit doesn't exist yet)

- Add `CultsRecruit` to `ObjectEnum` enum (line ~335, in Cults section)
- This may already be added by the `recruitment_center_auto_production` task

**5. `artifacts/developer/src/game/types/objects.rs`** (if CultsRecruit doesn't exist yet)

- Add `ObjectEnum::CultsRecruit` entry in `object_type()` (line ~214)
- Add `CultsRecruit` to `is_unit()` match (line ~393)

### Key Existing Patterns to Follow

1. **Agent tunnel placement flow** (best reference):
   - `AgentBuildTunnel` action → sets `PlacementState` + transitions to `AgentAwaitingPlacement` (command_panel.rs:1571-1583)
   - `manage_placement_ghost` spawns ghost (faction.rs:1228 branch)
   - `update_placement_ghost` validates with `can_worker_place_structure` (faction.rs:1382-1389)
   - `placement_click_system` issues `UnitCommand::BuildTunnel` on left-click (faction.rs:1641-1661)
   - Right-click/Escape cancels back to `AgentDefault` (faction.rs:1466-1468, command_panel.rs:931-933)

2. **DC build submenu flow** (for submenu pattern):
   - `DcOpenBuildMenu` action → `DcBuildMenu` state (command_panel.rs)
   - `DcBuild(ObjectEnum)` action selects specific building
   - `Back` action returns to previous state

3. **Under-construction spawning**:
   - `spawn_tunnel_under_construction()` (utils.rs:734) — uses `ObjectInstance::under_construction()` + `ConstructionHP::new()`
   - `ConstructionHP` component (structures.rs:11) — `progress: f32`, `build_frames: u32`
   - Construction tick handled by `construction_hp_tick_system` — BUT for Cults, the sibling `cults_construction_system` task handles ticking via `CultsConstructionState`

### Constants Needed

- `STORAGE_MAX_HP` — already defined in `game/types/structures.rs` (search for it; if not, add ~200.0)
- `STORAGE_BUILD_FRAMES` — new constant, define near other build frame constants (e.g., 300 frames = ~18.75 seconds)
- `STORAGE_SC_COST` — per design: "SpaceCrystalsCost - TBD", use a placeholder (e.g., 0 — Cults buildings cost Recruits, not crystals)

### System Registration

- `manage_placement_ghost`, `update_placement_ghost`, `placement_click_system` — already registered in `FactionPlugin` (world/mod.rs:102-104). The new CultsRecruit branches are added within these existing systems, so no new registration needed.
- `update_command_panel_state`, `rebuild_command_panel_ui`, `command_panel_hotkeys` — already registered in `HudPlugin` (ui/mod.rs). Extended with new branches, no new registration.
- The AssistConstruction AwaitingTarget handler may need a new system registration if not added to an existing system. Consider adding it to `right_click_move_command` (core.rs, registered in UnitsPlugin) or creating a new system in the same plugin.

### Queries Needed in `placement_click_system`

Current params (faction.rs:1433-1448) don't include a selected units query. Add:
```rust
selected_recruits: Query<(Entity, &Owner), (With<Unit>, With<Selected>)>,
```
Use the first recruit's `Owner` to set the building's owner.

## Dependencies

- **`recruitment_center_auto_production` planned_task** (active, being worked on by developer) — This task adds `ObjectEnum::CultsRecruit` to `shared/types.rs` and `spawn_cults_recruit()` to `game/utils.rs`. If CultsRecruit doesn't exist when implementing this task, add it as a stub first. The auto_production task also ensures CultsRecruit entities exist in-game.

- **`recruitment_center_interface` planned_task** (pending) — Sibling in the same feature. No code overlap — RC interface is for RecruitmentCenter structures, this task is for Recruit units. Can be implemented independently.

- **`cults_construction_system` developer_task** (pending) — Sibling task that implements the actual construction tick mechanics (CultsConstructionState, recruit enter behavior, proportional speedup). This task spawns the building and issues UnitCommand::ConstructBuilding, but the sibling task makes the Recruit actually walk to and enter the building. They connect via the `UnitCommand::ConstructBuilding(Entity)` variant and the `ConstructionHP` component.

- **Existing `spawn_cults_storage()` function** (utils.rs:988) — Already exists. Provides the reference for creating `spawn_cults_storage_under_construction()`.

- **Existing placement ghost infrastructure** (faction.rs:1194-1665) — `manage_placement_ghost`, `update_placement_ghost`, `placement_click_system` are fully implemented for DC/EF/Tunnel/Agent placement. This task adds new branches within these existing systems.
