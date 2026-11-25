# Q0087 — Does the Agent that finishes a Tunnel wait out the transit delay?

- **Date**: 2026-11-25T07:46:19-05:00
- **Blocking**: no. Proceeding on the assumption that it can leave at once, as M4-E built it.
- **Docs**: `factions/syndicate/units/agent.md > Tunnel Construction Flow`; `factions/syndicate/tunnels.md > Tunnel Network` (the transit delay)

## Question
When an Agent finishes building a Tunnel, it is inside the Tunnel Network. Can it be ejected at once, or only after the 2-second transit delay that applies to units that enter?

## Context
- `agent.md > Tunnel Construction Flow`, step 3: "The Tunnel becomes operational and the Agent is inside the Tunnel Network, available for redeployment from any Tunnel."
- `tunnels.md > Tunnel Network`: "after a unit **enters** the network, there is a **small delay before it can exit**: **2 seconds (32 frames), placeholder**", and "Units **produced** inside the network (by expansions) have **no delay**".

The builder neither walks in nor is produced, so neither rule clearly covers it.

## Options
- (a) **Built:** no delay, as for a produced unit, reading "available for redeployment from any Tunnel" literally. The player can eject it at the new Tunnel the moment it completes.
- (b) The 2-second delay, as if it had walked in, counted from the completion.
