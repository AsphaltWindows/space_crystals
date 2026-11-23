# agent_tunnel_building_r1

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

# agent_tunnel_building_r1

## Metadata
- **From**: manual_qa
- **To**: task_splitter

## Content

Agent Tunnel construction cannot be tested. The Build Tunnel (A) command does not produce a ghost placement overlay — no construction can be initiated. This blocks all 11 QA steps.

This is the same root cause as agent_interface_r1 and agent_groupable_construction_r1: the tunnel placement system is non-functional.

## QA Instructions

1. Select an Agent, press A — verify ghost Tunnel preview appears on cursor with green/red validity tinting.
2. Place a Tunnel — verify Agent walks to location, partially-built Tunnel appears at 10% HP, and construction completes in 30 seconds.
3. After completion, verify Agent enters Tunnel Network and Tunnel is operational.
4. Destroy a partially-built Tunnel — verify Agent survives and emerges, Supplies are not refunded.
5. Verify construction cost scales: 2nd Tunnel = 1 Supply, 3rd = 2 Supplies, etc.
