# 0267 — A Tunnel-built Laser Turret uses the operation slot; detection comes after the faction rosters

- **Date**: 2026-11-25T15:05:00-05:00
- **Answers**: Q0071 (complete: 0260–0267)
- **Docs**: `factions/syndicate/structures.md > Laser Turret > Construction`

## What changed and why
User:
- A Laser Turret built from a Tunnel's ExpandMenu **uses the Tunnel's single operation slot**, like an expansion.
- **Detection** will be designed **after all faction units are done**. Until then hidden things (expansions, pop-up turrets, burrowed DrillUnits, mines) can only be hurt by area damage over them (0262). Schedule the Transport and other stealth-dependent items accordingly.

## Implementation impact
- The Tunnel operation slot covers turret construction.
- Detection is not coming soon.

## Verification
While a Tunnel builds a Laser Turret, its Expand and Upgrade are unavailable.
