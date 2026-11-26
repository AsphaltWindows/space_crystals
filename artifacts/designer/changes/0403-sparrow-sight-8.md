# 0403 — Sparrow sight 8 GU so a lone Sparrow keeps strafing; Attack Move unchanged

- **Date**: 2026-11-27T14:10:00-05:00
- **Answers**: Q0148 (part 1)
- **Docs**: `factions/cults/units/sparrow.md > Placeholder stats`

## What changed and why
User, on Q0148 part 1:
- "Can't we increase its sight range?" The Sparrow's **SightRange goes from 6 to 8 GU** (512 su). Its full-speed loop takes it up to ~500 su from the target, so it keeps the target in sight. On a direct **Attack**, a lone Sparrow keeps making passes. Its speed may also be tuned later.
- **The Dragonfly and the Specter are unaffected:** each has a single shot and then goes home.
- **Attack Move is unchanged:** its 6 GU leash may still drop a looping Sparrow's target. The user: "it's probably also ok if you just target attack, and what attack move should mean for units that can't stop moving isn't entirely obvious at this point." With several Sparrows, their shared vision covers the loop. This is kept as an open design question.

## Implementation impact
- Sparrow SightRange 8.
- No rule changes.

## Verification
- A lone Sparrow ordered to Attack a stationary target, with no spotter, makes repeated passes until the target dies.
