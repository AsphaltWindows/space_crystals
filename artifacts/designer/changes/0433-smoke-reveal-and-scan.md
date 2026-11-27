# 0433 — Smoke: firing from it still reveals the attacker; it blocks Scan inside it

- **Date**: 2026-11-26T19:40:00-05:00
- **Answers**: Q0167
- **Docs**: `factions/syndicate/units/shadebringer.md > Cover (toggle)`

## What changed and why
- **1 (a)**: an attacker in or behind smoke is **revealed** after hitting your unit, as in the fog; the reveal isn't sight, so smoke doesn't block it.
- **2 (a)**: smoke **blocks Scan** for what is **inside** the smoke; a Scan has no viewpoint, so it isn't blocked "behind" smoke.

## Implementation impact
None: matches the placeholders (with "behind" clarified for Scan).

## Verification
- A Laser Battery firing from smoke at your unit becomes visible to you for 1.5 s after each hit.
- Scan an area half covered by smoke: units in the smoke stay hidden; units beyond the smoke inside the Scan circle are shown.
