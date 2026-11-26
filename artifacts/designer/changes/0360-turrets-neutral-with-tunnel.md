# 0360 — Laser Turrets go neutral with their Tunnel like expansions; no upright form

- **Date**: 2026-11-27T00:45:00-05:00
- **Answers**: — (completes 0359)
- **Docs**:
  - `factions/syndicate/structures/laser_turret.md` (intro; Pop-up turret; When its Tunnel is lost, replacing Conversion);
  - `factions/syndicate/tunnels.md > Losing Tunnels`; `core/game_rules.md > Defeat Condition`;
  - `factions/syndicate/structures/README.md`; `README.md`.

## What changed and why
User: turrets "can only be active in the tunnel area of a tunnel, so we don't have to worry about turrets that can't retract". When their Tunnel is lost, "it behaves the same way as other expansions: neutral, visible, still high armor, slowly decaying."
- **The upright turret form and the Conversion rules are removed.** A turret is always a pop-up and active only inside its Tunnel's area.
- **On Tunnel loss:**
  - neutral, visible to all, high armor (a raised turret retracts at once), never fires;
  - decays at 1% of MaxHP per second (placeholder);
  - reclaimable by a new Tunnel covering it, Tier 1 suffices.
- This replaces Q0091 point 1 (0358: stays down until clear, then rises upright) and the upright-turret parts of 0296–0298.
- **Defeat:** turrets no longer keep a Syndicate player alive after its last Tunnel. Losing its last Tunnel defeats the Syndicate unless it owns some other structure.

## Implementation impact
- Delete the upright form and conversion.
- Turrets join the expansion neutral, decay and reclaim path.
- Defeat no longer counts turrets after their Tunnel dies.

## Verification
- Destroy a Tunnel with a raised turret: it drops, appears to everyone as neutral, stops firing, and loses about 3 HP/s.
- A new Tunnel covering it makes it yours and active again.
