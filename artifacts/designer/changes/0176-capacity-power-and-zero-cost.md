# 0176 — Capacity check: no start without progress; zero-cost units always pass

- **Date**: 2026-11-24T22:25:00-05:00
- **Answers**: Q0047 (points 2 and 3; Q0047 done)
- **Docs**: `factions/gdo/overview.md > Unit Control`; `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts`

## What changed and why
- Pt 2 (follows from 0143's "reserved when the building starts making progress"): a building that can't make progress (GDO at 0 power generated) doesn't take the room; another may. Current build, confirmed.
- Pt 3 (user, B): a unit with capacity cost 0 always passes the check, even when over capacity.

## Implementation impact
Pt 2 none. Pt 3: check is `cost == 0 || free >= cost`. Applies to all factions (Syndicate Tunnel Space included).

## Verification
Pt 3 has no unit yet; unit-test the predicate.
