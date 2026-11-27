# 0459 — Vehicles ejected from a Tunnel drive clear of the exit

- **Date**: 2026-11-27T14:30:00-05:00
- **Answers**: Q0183
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Structure Sides > Side A` (new "Vehicles clear the exit")

## What changed and why
The user chose option (a), the same fix as the Chop Shop and the Factory (0446, 0451). An ejected wheeled or tracked vehicle with nowhere to go (no rally point, no order) no longer stops at the end of the exit path. Instead it drives to a stop point **3 GU straight out from the end of the exit path** (placeholder). If that point is taken, it parks at the nearest free spot around it. Idle vehicles don't yield (0440), so this keeps the exit path clear and every ejected vehicle gets out.

## Implementation impact
- **New:** a per-Tunnel stop point for ejected vehicles without an order, and parking around it when it is taken (the 0451 rule).
- **Unchanged:**
  - auto-ejected units, which still go to their rally point (parking around it when taken, 0451);
  - infantry;
  - aircraft at the Tier 3 doors.
- **Note:** this is not a player-settable rally point. Tunnels' rally points stay on production expansions, as before.

## Verification
1. Put five Assault Bikes in the network and eject all five from a Tier 2 Tunnel with no orders. All five come out and cluster about 3 GU out from the exit path. None stays stuck inside.
2. Eject five Peacekeeper-style infantry. They stop at the end of the path and step aside as before.
