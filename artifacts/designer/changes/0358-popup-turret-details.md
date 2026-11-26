# 0358 — Pop-up turret details: visible when its Tunnel dies (high armor while still down), retract finishes, ground, Tier 1, Tunnel Space from the start

- **Date**: 2026-11-27T00:05:00-05:00
- **Answers**: Q0091
- **Docs**: `factions/syndicate/structures/laser_turret.md > Pop-up rules`, `> Conversion`; `factions/syndicate/tunnels.md > Losing Tunnels`

## What changed and why
User:
1. **When its Tunnel is destroyed with a unit on top, the turret becomes visible at once**, but keeps its **high underground armor**.
   - It stays down, not blocking and not firing, until the spot is clear; then it rises as an upright turret.
   - Generally, **every expansion building of a destroyed Tunnel becomes visible** immediately.
2. While retracting it **finishes retracting**, then rises for a new target (a).
3. A Tunnel-built turret needs **Buildable, flat and Drillable** ground (a).
4. **Tier 1** can build it (a).
5. **Both kinds of site take their Tunnel Space from the start** (c). This changes the Agent-built site, which took it at completion.

## Implementation impact
- Point 1: visibility on conversion, armor kept while blocked.
- Point 5: Agent-built sites reserve Tunnel Space at the start.
- The rest is as built.

## Verification
- Kill a Tunnel while an enemy tank sits on its lowered turret: the turret appears to all, takes little damage, and rises once the tank leaves.
