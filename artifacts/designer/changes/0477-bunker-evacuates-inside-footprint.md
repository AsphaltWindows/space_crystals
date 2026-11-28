# 0477 — A destroyed Bunker's occupants come out inside its former footprint

- **Date**: 2026-11-28T12:30:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/gdo/structures/bunker.md > Garrison`
  - `systems/mechanics/garrisoning.md > Garrison structure`

## What changed and why
The user resolved the inconsistency flagged in 0475. When a **Bunker** is destroyed, its occupants come out all at once **inside its former footprint**, as the Cults Garrison's do. This replaces "around its footprint". It is now the rule for every garrison structure.

## Implementation impact
- Bunker evacuation now places units within the former 2x2 footprint, spread over it.
- Overlapping units ignore each other until they have separated.

## Verification
Destroy a full Bunker. All four occupants appear at once inside where it stood, unharmed and idle.
