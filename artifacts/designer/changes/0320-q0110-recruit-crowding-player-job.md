# 0320 — Recruit crowding at a Storage face: the player adds Storages (no give-way rule)

- **Date**: 2026-11-26T16:20:00-05:00
- **Answers**: Q0110
- **Docs**: `factions/cults/units/recruit.md > Gathering` (Crowding)

## What changed and why
User chose **(d)**: "For cults let's try adding storages when we want more recruits to be mining simultaneously."
- Recruits keep collision, and there is no give-way rule.
- Overloading a face can block Recruits head-on for a long time. Adding Storages, and spreading Recruits over patches, is the player's job.
- This includes the Hall rally-on-patch case.

## Implementation impact
- No movement change for Recruits.
- The reference layouts and the Cults economy check should stay within the design's 4–5 Recruits per face.
- Q0109 (Agents) is unaffected: its first step (0315) stands, pending your re-measure.

## Verification
—
