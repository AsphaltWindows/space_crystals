# 0143 — Unit capacity is reserved when production starts (all factions)

- **Date**: 2026-11-24T17:55:00-05:00
- **Answers**: Q0041 (points 1 and 2)
- **Docs**: `factions/gdo/overview.md > Unit Control` ("Reserved when production starts", replaces "Reserved when queued"); `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts` (replaces "Reserved when queued"), `> Over capacity`; `factions/colonists/economy.md > Beacon Deployment > Beacon Capacity during deployment` (example); `factions/gdo/structures.md > Barracks` (Q/W availability; destroyed refund); `systems/controls/selection_and_hud.md > Group production with limited resources`

## What changed and why
**Replaces 0142 (Colonists reserved on queue) and 0082 (GDO Unit Control reserved on queue).** The user clarified: capacity is reserved only when the building **starts making progress** on the unit. Every faction now follows the Syndicate's Tunnel Space model:
- Queuing needs only the resource cost (paid at queue) and a queue slot; no capacity check.
- At production start the unit needs free capacity ≥ its cost; it then counts as used for its production and life.
- Not enough → the building waits at 0% with the unit at the front; starts automatically when room appears; cancel = full refund; notification (system TBD).
- A unit in production always finishes, even if capacity drops (Q0041 pt 2).

## Implementation impact
- GDO: remove queue-time Unit Control check/reservation; HUD "used" = alive + in-production. Barracks Q/W availability no longer depends on Unit Control. Destroyed producer releases UC only for the in-production unit (queued ones never held any).
- Colonists: same for Beacon Capacity. 0112's example now reads "a new Drone starts production".
- 0083 (group production with limited resources) no longer considers Unit Control.

## Verification
- GDO near the 200 cap: queue more Peacekeepers than fit — all queue (crystals paid); those that fit start, the next one waits at 0% with a notification; kill a unit and it starts.
- Colonists at 10/10: queue Drones freely; the portal waits; deploy a Beacon → production resumes.
- Destroy a Beacon while a Drone is at 60%: it finishes (over capacity shown red); the next one waits.
