# 0385 — Cults economy rerun: an 8-patch line mined from both sides (~32 Recruits); One Hall against Two Halls, both starting with one Storage

- **Date**: 2026-11-27T11:30:00-05:00
- **Answers**: Q0144 (in part; the 2:48 target is deferred to the new figures)
- **Docs**: `factions/cults/economy.md > Economy check`

## What changed and why
The user, on Q0144:
- **"We should only start with one Storage, that's for sure."** "Ideal mining" was given its Storages for free at the start, which made it too generous.
- **"In both cases additional Storages need to be added to accommodate additional Recruits mining."**
- **"Rerun these two tests with 1 and 2 Recruitment Halls."**
- **"A contiguous line of 8 crystals should support … at least 32"** Recruits, mined from both long sides, with **no Storages at the ends**.

The new definitions:
- **Reference field:** 8 patches in a contiguous line, mined from both long sides, seating ~32 Recruits (8 tiles × 2 sides × ~2 per tile). Storages go along the long sides only.
  - Recruits reach the far side through the scenario's orders, as a player would send them (e.g. Move round the line, then Gather). There is no automatic spreading; Recruit > Gathering is unchanged.
- **Both scenarios** start with the one starting Storage, and add Storages as needed: whenever mining Recruits outnumber the room at the existing Storages. Each is paid for and built by Recruits taken off mining.
  1. **One Hall.**
  2. **Two Halls:** the user's line. The second Hall is ordered as early as affordable and built by the farthest Recruits; then Storages as needed. Its 40 Recruits now have ~32 seats, so the second Hall can pay off.
- **Target:** Two Halls should overtake One Hall by about 4:00 (0381).
- **Same report as Q0144:**
  - checkpoints at 1:30, 2:48, 4:00, 5:00 and 6:00;
  - income/s, total mined, bank and spent;
  - the GDO and the Colonists alongside.

This **replaces** 0381's "ideal mining" and "realistic opening", and Q0144's one-sided field.

## Implementation impact
- New reference layout and two scripted build lines.
- The Q0144 pins are superseded.
- Report the new table as a question. Nothing is tuned until the user has seen it.
- Also report the **seats measured** on the two-sided line (target ~32).

## Verification
- In the report, One Hall levels off at 20 Recruits mining, below the field's ~32 seats.
- Two Halls reaches ~32 mining, and builds roughly 5–6 Storages along both sides.
