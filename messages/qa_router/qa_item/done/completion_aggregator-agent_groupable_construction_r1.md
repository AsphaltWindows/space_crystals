# agent_groupable_construction_r1

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

## Content

The Groupable=false constraint for Agents works correctly — individual portraits, single Agent command panel, Tab/Shift-Tab cycling all pass. However, Tunnel construction cannot be initiated at all, blocking verification of the single-Agent construction enforcement.

**What passed:**
- Selecting multiple Agents shows individual portraits in SelectionPanel
- CommandPanel displays a single Agent's interface
- Tab/Shift-Tab cycles ActiveGroup between individual Agents

**What failed:**
- Tunnel build command does not show a placement overlay — clicking the Tunnel build action does nothing visible, and no construction can be started. This blocks testing whether a second Agent is correctly rejected from joining an in-progress construction.

## QA Instructions

1. Select an Agent and click the Tunnel build command — verify a placement overlay appears showing where the Tunnel will be built.
2. Place and start Tunnel construction with one Agent. While construction is in progress, order a second Agent to the same Tunnel.
3. Verify the second Agent does NOT join the construction — it should stop/idle instead.
4. Verify only one Agent is embedded in the Tunnel during construction.
