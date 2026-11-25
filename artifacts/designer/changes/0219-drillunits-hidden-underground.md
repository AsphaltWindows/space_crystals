# 0219 — All Drill Units are hidden underground unless detected

- **Date**: 2026-11-25T06:05:00-05:00
- **Answers**: —
- **Docs**: `systems/units.md > DrillUnit`; `factions/syndicate/units.md > Subterranean Transport > Underground: hidden unless detected`

## What changed and why
User: the rule from 0216/0218 applies to **every DrillUnit**, not just the Subterranean Transport. Underground, a DrillUnit is hidden unless revealed by a detector and targetable only then; AoE damage still affects it. This replaces the base's "visible while subterranean". Few other Drill Units are expected.

## Implementation impact
Base-level rule; it needs the detection system (TBD).

## Verification
—
