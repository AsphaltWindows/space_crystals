# Task Splitter Insights

## Upcoming Features (from forum topics, 2026-03-19)

Operator has posted 6 forum topics requesting designer to produce feature_requests. When these arrive, key splitting notes:

### Dependencies to track
- dc_ef_no_auto_enter_construction_submenu must complete BEFORE dc_default_state_cancel_commands
- QA re-tagging depends on automated_qa_ui_state_queries being completed first

### Large feature areas incoming
1. **Syndicate Agent** (6 subsystems): Agent interface state, spawn/commands, groupability, resource gathering (GatheringResource + DroppingOffResources behaviors), tunnel building (BuildTunnel behavior), worker-built arrival validation. Split along behavior/system boundaries.
2. **Syndicate Tunnels** (5 items): Tunnel interface (4 states: Default/Expand/Eject/AwaitingPlacement), underground occupancy bug fix (blocker!), HQ production interface, Enter command + EnteringTunnel behavior, rally point system.
3. **Unit Control** (3 items): BasicCombatUnitInterfaceState (command panel + right-click), SelectionPanel (multi-select UI), CommonCommand classification fix.
4. **GDO Structures** (3 items): DC cancel commands, Supply Tower interface, Guard unit.
5. **DC/EF Rework** (3 concerns): Stop auto-enter construction submenu, EF flat interface redesign, real-time progress bar fix.
6. **Technical** (2 items): Viewport black line visual glitch (debugging), QA tag re-tagging (markdown-only).

### Splitting heuristics
- Each behavior (GatheringResource, DroppingOffResources, BuildTunnel, EnteringTunnel) = separate task
- Each ObjectInterfaceState = separate task
- Bug fixes that are blockers = single focused task
- New unit implementations (Guard) = self-contained task (ObjectEnum + type data + spawn function)
