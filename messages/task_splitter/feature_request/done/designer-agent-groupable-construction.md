# agent-groupable-construction

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement Agent's Groupable=false constraint and single-Agent construction enforcement, as defined in `artifacts/designer/design/syndicate_objects.md`.

**Groupable = false:** Each Agent instance occupies its own SelectionGroup when selected, even when multiple Agents are selected together. The command panel always displays a single Agent's interface. See `artifacts/designer/design/control_system.md` Selection constraints for how ungroupable objects behave.

**Single-Agent Construction Enforcement:** Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction. If an Agent is already building a Tunnel and another Agent is directed to the same site, the second Agent's command should be rejected or the Agent should idle.

## QA Instructions

1. Select two or more Agents by box-selecting or shift-clicking.
2. Verify the SelectionPanel shows individual portraits (not a merged group).
3. Verify the CommandPanel displays one Agent's interface (not a combined view).
4. Use Tab/Shift-Tab to cycle ActiveGroup between Agents — verify each Agent shows individually.
5. Order an Agent to build a Tunnel. While construction is in progress, order a second Agent to the same Tunnel.
6. Verify the second Agent does NOT join the construction — it should stop/idle instead.
7. Verify only one Agent is embedded in the Tunnel during construction.
