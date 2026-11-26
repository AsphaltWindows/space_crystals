# 0349 — Evacuation: everyone appears at once inside the destroyed building's footprint

- **Date**: 2026-11-26T22:50:00-05:00
- **Answers**: Q0106
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`; `factions/cults/structures/garrison.md`

## What changed and why
User:
1. A destroyed building is just gone. The animation has no gameplay debris. Units **appear inside the former footprint**.
2. They **all appear at the same time**. Overlaps resolve by the existing rules: overlapping units ignore each other until separated, and idle units push apart.
3. A unit that had finished training comes out as the trained unit, with no refund (default).
4. Evacuated units idle (default).

## Implementation impact
- Replaces the ring search around the footprint.
- No placement order.

## Verification
- Destroy an Armory holding 10 Recruits: all 10 appear on its tiles in the same frame, then spread.
