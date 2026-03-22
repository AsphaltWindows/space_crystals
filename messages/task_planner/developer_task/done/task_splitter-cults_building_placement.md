# cults_building_placement

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
