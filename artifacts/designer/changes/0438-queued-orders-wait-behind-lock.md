# 0438 — Shift-queued orders wait behind a Target Lock or a Shroud channel (from the docs)

- **Date**: 2026-11-26T20:15:00-05:00
- **Answers**: Q0171
- **Docs**: `factions/gdo/units/rocketman.md > Target Lock`; `factions/cults/units/disciple.md > Spells > Shroud` (wording clarified)

## What changed and why
Answered from the docs (option 1): `systems/controls/commands.md > Unit Command` says a Shift-queued command is appended to the queue and never replaces the current one. "Any order" in the Rocketman's and the Disciple's docs meant orders given **without Shift**; the wording now says so. A Shift-queued order waits and runs when the lock or the channel ends (target lost or dead, out of range/vision, out of energy).

## Implementation impact
None: matches the build (T76, M5-N4b, M5-K).

## Verification
- A Rocketman locking a building, Shift+right-click ground: the lock continues; when the building dies, it moves there.
- A Disciple channelling Shroud, Shift+Move: it keeps channelling until out of energy, then moves. A plain right-click ends the channel at once.
