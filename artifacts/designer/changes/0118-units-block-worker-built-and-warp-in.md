# 0118 — Units block Worker-Built and Warp In placement

- **Date**: 2026-11-24T14:00:00-05:00
- **Answers**: Q0040 (point 2)
- **Docs**: `core/entities.md > Placement Validation > Worker-Built Structures`, `> Warp In`

## What changed and why
"Unoccupied" includes units for every placement method, as in Direct Placement (0094): a ground unit of any player on the footprint invalidates it. The building worker itself (the deploying Drone; likewise an Agent or Recruit building) doesn't count. Air and underground units never block.

## Implementation impact
Worker-Built arrival validation and the Warp In ghost check ground units. Applies to Drone deploy, Agent Tunnels/Laser Turrets, Cults buildings, and Warp In.

## Verification
- Park an enemy ground unit on a Deploy site: on arrival the order fails, Drone idles.
- Stand own units beside a Beacon inside a larger building's footprint: Warp In ghost is red there.
- An aircraft over the site doesn't block.
