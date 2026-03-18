# Feature Update: control_system (2026-03-06)

## Modified Feature File
`features/control_system.md` (NEW)

## Relevant Design Files
- `design/control_system.md`

## Summary
Initial feature specification created from formal design content. Defines client-side ControlState with Selection (SelectionGroups, ActiveGroup, constraints), ControlGroups (10 saved selections), CommandPanel (common vs group-specific commands), InterfaceTransitions (StateOnly vs CommandIssuing), DefaultState, AwaitingTarget[CommandType], CursorTarget resolution, and BasicCombatUnitInterfaceState template with full right-click and target resolution tables.
