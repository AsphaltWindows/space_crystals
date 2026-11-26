# 0331 — MaxHP increases raise HP by the same amount (refines 0330)

- **Date**: 2026-11-26T19:10:00-05:00
- **Answers**: Q0114 (point 1, refined)
- **Docs**: `factions/cults/augmentation.md > Increments` (HP when MaxHP changes)

## What changed and why
User:
- **When MaxHP goes up, HP goes up by the same amount**, so the damage stays the same. A full unit stays full: 25/40 becomes 45/60.
- **When MaxHP goes down, HP is unchanged**, unless it is above the new max, in which case it is capped: 8/40 becomes 8/10, and 40/40 becomes 10/10.

## Implementation impact
This replaces 0330's "on an increase, HP stays where it was".

## Verification
- A full Recruit given Enhanced Constitution (+20 MaxHP at L1 with +2 increments, whatever the current values) is still full afterwards.
- A hurt one keeps the same missing HP.
