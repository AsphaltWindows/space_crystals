# Feature Update: Syndicate Objects - Agent Control Panel

**Feature file**: `features/syndicate_objects.md`
**Design sources**: `design/syndicate_objects.md`
**Design update**: `design_updates/2026-03-06_agent_control_panel.md`

## Modifications

### Agent ObjectInterfaceState (new section)
Added complete Agent ObjectInterfaceState with:
- **DefaultState commands**: A (Build Tunnel via AwaitingPlacement) and B (Drop Off Resources, greyed when not carrying)
- **Unit Commands list**: Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel
- **Right-Click Resolution table**: Context-sensitive based on target type and Agent carry state (crystal field -> Gather, supply source -> Gather, own Tunnel carrying -> Drop Off, own Tunnel not carrying -> Enter, enemy -> Attack, ground -> Move)
- **Multi-select note**: Right-click commands issued to all selected Agents despite ungroupable status

### Groupable fix
Corrected `Groupable: true` to `Groupable: false` — Agent is ungroupable per design source. Previous value was incorrect.

### Building section update
Added explicit single-Agent construction rule: only one Agent may construct a given Tunnel, multiple Agents cannot speed up construction.

### Open questions updated
- Removed resolved: "Agent gathering commands/behaviors" and "Agent build command: how is it initiated?"
- Added new: CommandIndicators for Agent commands, Gathering behavior algorithm, Drop Off auto-return-to-gathering
