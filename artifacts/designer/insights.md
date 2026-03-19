# Designer Insights

## Table of Contents — Design Documents

| File | Summary |
|------|---------|
| `camera.md` | Camera system design |
| `combat.md` | Attack attributes, attack phases, damage calculation, locomotion/orientation constraints |
| `control_system.md` | ControlState, Selection, ObjectInterfaceState, CommandPanel, SelectionPanel, unit commands, base/turret behaviors |
| `designer_notes.md` | Designer notes |
| `design_questions.md` | Open design questions |
| `entities.md` | Entity hierarchy (tiles, objects, structures, resources), fog of war, placement validation |
| `factions.md` | All 4 factions (GDO, Syndicate, Cults, Colonists) and their resources |
| `gdo_objects.md` | GDO units/structures: Peacekeeper, PowerPlant, Barracks, DeploymentCenter, ExtractionFacility, ExtractionPlate, SupplyTower, SupplyChopper |
| `scale.md` | Simulation frame rate (16 FPS), grid units, space units (64 per grid unit) |
| `syndicate_objects.md` | Syndicate units/structures: Tunnel Network, Tunnel, Headquarters, Guard, Agent |
| `to_be_converted.md` | Content awaiting conversion to design doc format |
| `units.md` | Unit base types, movement models, turret attributes, collision |

## Urgent Forum Questions

### 1. DC DefaultState Cancel Command (NEW DESIGN ELEMENT)
- **Forum topics**: `dc-ef-construction-submenu-rework`, `gdo-structures-guard-unit`
- **Proposal**: Add Cancel (X) at slot (2,1) in DC DefaultState, conditionally visible when `current_construction.is_some()` or `ready_to_place.is_some()`. Full refund during construction, 75% refund when ready-to-place.
- **Why it needs user input**: The current DC design only has "Build: enters BuildMenu" in DefaultState. This is a new UX element that shortcuts Cancel to avoid entering the BuildMenu. Affects player flow.
- **Present to user with**: "Should the DC show a Cancel (X) button directly in its default view when a construction is in progress or ready to place? Currently the player must enter the BuildMenu to find Cancel."

### 2. BasicCombatUnit Command Grid Layout (NEW DESIGN ELEMENT)
- **Forum topic**: `unit-control-selection-ui`
- **Proposal**: Specific slot assignments for combat unit commands:
  ```
  [Q] Move    [W] Attack    [E] AtkGround*
  [A] AtkMove [S] Patrol    [D] HoldPos
  [Z] Stop    [X] Reverse*
  ```
- **Why it needs user input**: The design docs define the commands but not their grid positions. This layout affects muscle memory and accessibility.
- **Present to user with**: "We need to assign the combat unit commands to specific grid slots. Here's a proposed layout — does this feel right?"

### 3. Supply Tower Schedule Deliveries Error Feedback
- **Forum topic**: `gdo-structures-guard-unit`
- **Question**: When clicking a non-SDS entity in AwaitingTarget[ScheduleDeliveries], current design says "no action." Should there be error feedback (sound/message)?
- **Low priority** — can be deferred.

## Pending Design Review

### Feature Requests Ready to Produce (pending user confirmation)
All items below are **already fully documented** in the design docs and match the forum descriptions exactly. They just need to be converted into feature_request messages for the pipeline. Confirm with user before sending.

1. **Agent ObjectInterfaceState** — Agent interface with Build Tunnel (A), Drop Off (B), right-click resolution, 7 unit commands. Source: `syndicate-agent-core-gameplay` topic items 1, 2.
2. **Agent Groupable & Construction Enforcement** — Groupable=false, single-agent-per-tunnel construction. Source: `syndicate-agent-core-gameplay` topic item 3.
3. **Agent Resource Gathering** — GatheringResource and DroppingOffResources behaviors, 48-frame durations, carry capacities. Source: `syndicate-agent-core-gameplay` topic item 4.
4. **Agent Tunnel Building** — BuildTunnel behavior, 480 frames, ConstructionHP Rule, agent embeds. Source: `syndicate-agent-core-gameplay` topic item 5.
5. **Worker-Built Arrival Validation** — Two-phase validation for worker-built structures. Source: `syndicate-agent-core-gameplay` topic item 6.
6. **Tunnel ObjectInterfaceState** — 4-state interface with Upgrade/Expand/Eject/AwaitingPlacement. Source: `syndicate-tunnels-underground` topic item 1.
7. **Underground Expansions Surface Walkability** (bug fix) — Underground structures shouldn't block surface movement. Source: `syndicate-tunnels-underground` topic item 2.
8. **HQ Production Interface** (bug fix) — HQ should show production commands, not unit commands. Source: `syndicate-tunnels-underground` topic item 3.
9. **Enter Command & EnteringTunnel Behavior** — Enter command, walk to Side A, despawn into network. Source: `syndicate-tunnels-underground` topic item 4.
10. **Rally Point Behavior for Syndicate Production** — Surface rally = auto-eject, no rally = stay in network. Source: `syndicate-tunnels-underground` topic item 5.
11. **BasicCombatUnitInterfaceState** — Full command set, right-click resolution, AwaitingTarget resolutions. Source: `unit-control-selection-ui` topic item 1. *Blocked on grid layout question (#2 above).*
12. **SelectionPanel** — Portrait grid with click interactions and ActiveGroup highlight. Source: `unit-control-selection-ui` topic item 2.
13. **CommonCommand vs GroupCommand Classification** (bug fix) — Commands only Common if all selected entities support them. Source: `unit-control-selection-ui` topic item 3.
14. **DC/EF Construction Submenu Fix** — Stop auto-forcing, EF flat interface, real-time progress bar. Source: `dc-ef-construction-submenu-rework`. *Blocked on DC Cancel question (#1 above).*
15. **Supply Tower ObjectInterfaceState** — Q/S/X/C commands, Schedule Deliveries. Source: `gdo-structures-guard-unit` topic item 2.
16. **Guard Unit Implementation** — Full Guard stats and production integration. Source: `gdo-structures-guard-unit` topic item 3.

## Loose Ends

- Alt-click camera behavior: should it pan smoothly or snap instantly? (from `unit-control-selection-ui`)
- Agent UnitBase is documented as HeavyInfantry but forum topic 2 mentions LightInfantry in the key questions. Design says HeavyInfantry — confirm this is intentional.
- Tunnel transit tier requirements: design says "Tier 1+: Infantry (Heavy Infantry)" but the parenthetical only mentions Heavy Infantry, not Light Infantry. The BasicCombatUnitInterfaceState right-click resolution for Enter says "Syndicate units only" — does this include all Syndicate unit bases or just those meeting tier requirements?
