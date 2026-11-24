# 0081 — Queue limit of 20 includes the unit in production

- **Date**: 2026-11-24T02:25:00-05:00
- **Answers**: Q0024 (point 1; points 2–7 still open)
- **Docs**: BuildQueue / TrainingQueue max in `factions/gdo/structures.md > Barracks`, `factions/gdo/economy.md` (Supply Tower), `factions/syndicate/structures.md`, `factions/colonists/structures.md` (both portals), `factions/cults/structures.md > Armory`

## What changed and why
The user chose A. The production queue limit of 20 is **20 in total**, counting the unit currently in production (CurrentBuild). A full queue shows 20. This applies to every faction's production queue.

## Implementation impact
- A building with CurrentBuild set and 19 waiting entries is full, and queuing more is refused.

## Verification
Queue Peacekeepers at a Barracks until it refuses. Exactly 20 are accepted: 1 in production and 19 waiting.
