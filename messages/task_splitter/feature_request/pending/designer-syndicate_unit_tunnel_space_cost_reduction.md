# syndicate_unit_tunnel_space_cost_reduction

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Reduced the Syndicate's per-unit Tunnel Space cost. Changed `artifacts/designer/design/syndicate_objects.md`.

## Change
- **Guard**: TunnelSpaceCost 2 -> 1
- **Agent**: TunnelSpaceCost 2 -> 1

Both of the Syndicate's currently-defined units now cost 1 Tunnel Space instead of 2. This effectively doubles the number of units a given amount of Tunnel Space can support, compounding with the earlier `syndicate_tunnel_space_from_unused_area` rework (Tunnel Space is now derived from unused tunnel-region rather than a flat per-tier value — see that feature_request for full context, still pending/unconsumed as of this message).

## QA Instructions

1. Select an Agent and a Guard. Confirm their InfoPanel/inspector both show TunnelSpaceCost 1 (not 2).
2. With 11 Tunnel Space available (the new starting-game value per the companion Tunnel Space rework), confirm you can train 11 Agents/Guards in any combination before production is blocked for insufficient Tunnel Space (previously this would have been 5, rounding down from 11/2).
3. Confirm the Tunnel Space usage display increments by exactly 1 per Agent or Guard trained, and decrements by 1 when one is destroyed.
