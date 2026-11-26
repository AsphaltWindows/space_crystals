# Syndicate: Overview

## TheSyndicate
The Syndicate is a covert faction that builds underground Tunnel networks and uses Agents as its worker units. Army size is limited by Tunnel Space, which comes from keeping Tunnel Area underground space unused — building more Tunnels/upgrades grows it, but building expansions inside that space consumes it.

### Value - TheSyndicate
### Name - "The Syndicate"

### DisplayHud:
- Space Crystals: current amount
- Supplies: current amount
- Tunnel Space: current used / current available (max 200)

## SyndicateResources
The resources available to a Syndicate player.

### Space Crystals
Core resource. Gathered from Space Crystals Patches by Agents. Spent on buildings, units, and upgrades.

### Supplies
Resource necessary for expanding and teching. Gathered from Supply Delivery Stations by Agents. Spent on Tunnel construction, Tunnel upgrades, and research.

### Tunnel Space
Unit control resource. Derived from unused underground space rather than a flat per-Tunnel value: Tunnel Space = floor((sum of unused tunnel-region grid units across all owned Tunnels) / 4), capped at a hard maximum of 200 (see factions/syndicate/tunnel_area_and_space.md > Tunnel Space for the full formula and definitions of tunnel-region/unused). Each Syndicate unit has a Tunnel Space cost. Units can always be queued, but a unit only starts production if its Tunnel Space cost fits within the free Tunnel Space at that moment; otherwise production waits (see factions/syndicate/tunnel_area_and_space.md > Tunnel Space > Production and Tunnel Space). Building an expansion permanently reduces available Tunnel Space by converting unused tunnel-region into occupied space.
