# recruitment_center_interface

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-cults_objects_formalized.md

## Task

Implement the ObjectInterfaceState for RecruitmentCenter. This covers the command panel and right-click behavior when a Recruitment Center is selected.

### StructureMenuState variant

Add `StructureMenuState::RecruitmentCenterMenu` to the enum in `ui/types.rs`.

### Command panel grid (command_panel.rs)

When `StructureMenuState::RecruitmentCenterMenu` is active, display:
- **X (row=2, col=1): Cancel Production** — cancels the current Recruit in production, resetting `ProductionProgress` to None. Only available (button visible) when `RecruitmentCenterState.production_progress > 0` (i.e., production is active). Action: set `production_progress = 0` on the selected RecruitmentCenter entity.
- **C (row=2, col=2): Set Rally Point** — transitions to `ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint)`. Left-click on ground or object sets the rally point on the RecruitmentCenterState, then returns to `StructureMenuState::RecruitmentCenterMenu`.

### State detection (update_command_panel_state)

When the active selection is a single RecruitmentCenter (ObjectEnum::RecruitmentCenter), set `ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu)`. Follow the same pattern as HeadquartersMenu, BarracksMenu, etc.

### Right-click resolution (core.rs - right_click_move_command)

When the selected unit is a RecruitmentCenter and interface state is `RecruitmentCenterMenu`:
- Right-click Ground: set `RecruitmentCenterState.rally_point = Some(world_position)`
- Right-click Object: set `RecruitmentCenterState.rally_point = Some(object_position)`

Follow the same pattern used by HeadquartersMenu right-click rally (production_rally_point_system).

### AwaitingTarget resolution

When in `AwaitingTarget(SetRallyPoint)` and the selected entity is a RecruitmentCenter:
- Left-click ground: set rally point to clicked location, return to RecruitmentCenterMenu
- Left-click object: set rally point to object location, return to RecruitmentCenterMenu

Follow the existing `set_rally_point_click_system` pattern used by HQ/Barracks/SupplyTower.

## Technical Context

### Files to Change

**1. `artifacts/developer/src/ui/types.rs` (StructureMenuState enum)**
- Add `RecruitmentCenterMenu` variant to `StructureMenuState` enum (line ~221, after `HeadquartersMenu` or before `Inert`)
- Pattern: same as `HeadquartersMenu` — simple menu state, no sub-states needed

**2. `artifacts/developer/src/ui/command_panel.rs` (multiple change sites)**

a) **`get_grid_slot_action()` (line ~52)** — Add a new match arm for `StructureMenuState::RecruitmentCenterMenu`:
```
StructureMenuState::RecruitmentCenterMenu => match (row, col) {
    (2, 1) if <has_production> => Some(CommandButtonAction::RcCancel),
    (2, 2) => Some(CommandButtonAction::SetRallyPoint),
    _ => None,
},
```
NOTE: The cancel visibility requires knowing if `production_progress > 0`. Currently `get_grid_slot_action` receives `bk_has_queue: bool` as a param — you can repurpose or extend this (e.g., rename to something more general, or add a new `rc_has_production: bool` param). The `bk_has_queue` param is already used for HQ cancel visibility (line 100) even though it was originally for Barracks — so RC can follow the same convention, OR add a separate boolean parameter.

b) **Add `CommandButtonAction::RcCancel` variant** to the enum in `ui/types.rs` (line ~307 area):
```
/// Recruitment Center: Cancel current production (reset progress)
RcCancel,
```

c) **`update_command_panel_state()` (line ~299)** — Add `ObjectEnum::RecruitmentCenter` branch in the structure type match (line ~346). The `selected_structures` query needs `Option<&RecruitmentCenterState>` added to its tuple. Follow the `Headquarters` pattern (lines 407-415):
```
ObjectEnum::RecruitmentCenter => {
    let in_valid_state = matches!(*interface_state,
        ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu) |
        ObjectInterfaceState::AwaitingTarget(_)
    );
    if target_changed || !in_valid_state {
        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
    }
}
```
Import: Add `RecruitmentCenterState` to the `crate::game::types` import at line 3-9.

d) **`rebuild_command_panel_ui()` (line ~462)** — Add title and info section for RC:
- Title (line ~516 match): `StructureMenuState::RecruitmentCenterMenu => "Recruitment Center"`
- Info/progress text (line ~546 match): Show production progress % when `production_progress > 0`. RC uses integer frame counting — calculate pct from `production_progress` / total frames. Add a `rc_query: Query<&RecruitmentCenterState>` to the function params.
- Grid button spawning: need to compute `rc_has_production` bool from the RC query, pass to `get_grid_slot_action()`.

e) **`execute_command_action()` (line ~1137)** — Add handler for `CommandButtonAction::RcCancel`:
```
CommandButtonAction::RcCancel => {
    if let Some(entity) = panel_target.entity {
        // Reset production_progress to 0 on the RC entity
        // Need rc_query added to execute_command_action params
    }
}
```
NOTE: `execute_command_action` already receives many query params. You will need to add a mutable `RecruitmentCenterState` query parameter. Follow the pattern of `HqCancel` (line 1307).

f) **`grid_button_label()` (line ~2300)** — Add label for `RcCancel`:
```
CommandButtonAction::RcCancel => format!("[{}] Cancel\nProd", hotkey),
```

g) **`grid_button_enabled_ext()` (line ~2117)** — Add `RcCancel` — always enabled when visible (the grid slot conditional already gates visibility).

h) **`right_click_cancel_target()` (line ~1059)** — Add `RallyTargetKind::RecruitmentCenter` variant and corresponding return:
```
Some(RallyTargetKind::RecruitmentCenter) => Some(ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu)),
```

i) **`right_click_cancel_submenu()` (line ~1106)** — Add `rc_query: Query<(), With<RecruitmentCenterState>>` param and corresponding resolve branch.

j) **Escape handler in `command_panel_hotkeys()` (line ~935)** — The generic `AwaitingTarget(_)` arm at line 935 returns to `Default`. For RC, after Escape from `AwaitingTarget(SetRallyPoint)`, it should return to `RecruitmentCenterMenu`. You need to add a check: if `panel_target.entity` has `RecruitmentCenterState`, return to `RecruitmentCenterMenu`. Alternatively, since the `right_click_cancel_target()` helper already resolves the correct state, consider reusing that logic for Escape too.

**3. `artifacts/developer/src/game/units/systems/core.rs` (set_rally_point_click_system)**

The `set_rally_point_click_system` (line 952) handles left-click SetRallyPoint for BK/HQ/ST. Add an `rc_query: Query<&mut RecruitmentCenterState>` and a new branch:
```
else if let Ok(mut rc_state) = rc_query.get_mut(target_entity) {
    // RC rally_point is Option<Vec3> (NOT RallyTarget)
    let location = match &rally_target {
        RallyTarget::Object(e) => object_transforms.get(*e).ok().map(|t| t.translation),
        RallyTarget::Location(loc) => Some(*loc),
    };
    rc_state.rally_point = location;
    info!("Recruitment Center: Rally point set via command mode");
    spawn_or_update_rally_marker(...);
}
```
IMPORTANT: RC `rally_point` is `Option<Vec3>`, NOT `Option<RallyTarget>` like BK/HQ/ST. You must resolve the RallyTarget to a Vec3 before storing.

After setting, change the return state (line 1030 currently sets `Default`). This needs to use the same `RallyTargetKind` resolve pattern — but `set_rally_point_click_system` currently unconditionally sets `Default`. Modify it to resolve the correct return state. Check if `panel_target.entity` has `RecruitmentCenterState` → return to `RecruitmentCenterMenu`.

**4. `artifacts/developer/src/game/world/faction.rs` (production_rally_point_system)**

The `production_rally_point_system` (line 703) handles right-click rally for production structures. Add:
- `mut rc_query: Query<(Entity, &mut RecruitmentCenterState), With<Selected>>` param
- `StructureMenuState::RecruitmentCenterMenu` to the `is_production_menu` match (line 728)
- New match arm (after line 793):
```
ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu) => {
    for (entity, mut rc_state) in &mut rc_query {
        let location = match &rally_target {
            RallyTarget::Object(e) => potential_targets.iter()
                .find(|(te, _, _, _)| *te == *e)
                .map(|(_, t, _, _)| t.translation),
            RallyTarget::Location(loc) => Some(*loc),
        };
        rc_state.rally_point = location;
        info!("Recruitment Center: Rally point set");
        spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
    }
}
```
Import `RecruitmentCenterState` and `StructureMenuState::RecruitmentCenterMenu` at top of faction.rs.

### Key Types
- `RecruitmentCenterState` (structures.rs:499): `rally_point: Option<Vec3>`, `production_progress: u32`, `build_order: u64`
- `RallyTarget` (structures.rs:58): `Location(Vec3) | Object(Entity)` — used by BK/HQ/ST but NOT by RC (RC uses plain `Option<Vec3>`)
- `RallyPointMarker` (structures.rs:63): visual marker component — reuse `spawn_or_update_rally_marker()` for RC
- `ObjectEnum::RecruitmentCenter` — already exists in `game/types/objects.rs:302`
- `StructureMenuState` — enum in `ui/types.rs:191`
- `CommandButtonAction` — enum in `ui/types.rs:236`
- `RallyTargetKind` — private enum in `command_panel.rs:1095`

### Pattern to Follow
The `HeadquartersMenu` implementation is the closest pattern:
- Same single-state menu (no sub-states like DC)
- Same cancel + rally point grid layout (X=cancel, C=rally)
- Same `AwaitingTarget(SetRallyPoint)` flow
- Key difference: RC uses `Option<Vec3>` for rally, HQ uses `Option<RallyTarget>`

### System Registration
- `update_command_panel_state` and `rebuild_command_panel_ui` are already registered in `HudPlugin` (`ui/mod.rs`)
- `set_rally_point_click_system` is registered in `UnitsPlugin` (`game/units/mod.rs:84`)
- `production_rally_point_system` is registered in `FactionPlugin` (`game/world/mod.rs`)
- No new system registration needed — just extend existing systems

## Dependencies

- **`RecruitmentCenterState` component** — already exists in `game/types/structures.rs:499` with `rally_point: Option<Vec3>` and `production_progress: u32`. Spawned by `spawn_recruitment_center()` in `game/utils.rs:947`.
- **`ObjectEnum::RecruitmentCenter`** — already exists in `game/types/objects.rs:302`, already marked as `is_structure() == true`.
- **RC production system** (sibling task) — the Cancel button resets `production_progress` to 0, which only matters once the auto-production tick system exists. The interface can be built now and will work correctly once production is wired up.
- **`setup_cults_game_start()`** (faction.rs:172) — already spawns an RC at grid (50,50), so the RC entity exists in-game for testing.
