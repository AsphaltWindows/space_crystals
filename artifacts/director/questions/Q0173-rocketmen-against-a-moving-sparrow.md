# Q0173 — Rocketmen can't hit a moving Sparrow, and can't lock a strafing one without a spotter

- **Date**: 2026-11-26T22:53:16-05:00
- **Blocking**: no. M5-K is built to the docs, and the Rocketman stays as built until the answer.
- **Docs**:
  - `factions/gdo/units/rocketman.md` (`> Target Lock`, `> Placeholder stats`);
  - `systems/combat.md > DoublyDisjointed`;
  - `factions/cults/units/sparrow.md > Placeholder stats`;
  - design change 0356 (Verification).

## Question
0356's Verification reads: "Rocketmen can shoot down a Sparrow. A circling Sparrow can **sometimes** dodge the normal missile; a locked one can't."

Built to the docs, a Sparrow in flight **always** dodges the normal missile. A lone group of Rocketmen also can't complete a lock on a Sparrow that strafes them. Is that the intended balance? Or should one of the options below change it?

## Context
Measured by slice M5-K:
- **Normal missiles:** 0 hits in 70 shots at a strafing Sparrow over 60 s, 0 in 15 at a circling one, and 0 in 6 in the new golden.
- **Strafing:** three Rocketmen against one strafing Sparrow left it untouched after 60 s. With a building's sight over them, three Rocketmen killed it by 10 s.
- **Circling:** locks killed circling Sparrows 3 times out of 3.

**Why the normal missile always misses:**
- DoublyDisjointed "travels to the location the target was at at the end of the Aiming phase", and hits whatever is there when it lands. It aims without leading.
- The missile flies at 16 su/frame after 2 frames of Firing. Even at MinRange, 2 GU (128 su), it lands 10 frames after aiming ends.
- A Sparrow never stops. At its IdleSpeed of 4 su/frame it has moved at least 40 su by then, at 10 su/frame 100 su. Its silhouette is 40x40, so it has always left the spot.
- Only a Sparrow flying straight at a Rocketman from close by could be hit, and even then it has passed the spot.

**Why a lone group can't lock a strafing Sparrow:**
- Locking needs the target within LockRange (7 GU) and visible for 48 frames in a row. The Rocketman's SightRange is only 6 GU.
- A strafing Sparrow loops out until its target's centre is out of its own range, then comes back at 10 su/frame. So it is within the group's sight for less than 48 frames each pass, and every lock breaks.
- A building or another unit that sees the Sparrow keeps it visible, and the lock completes.

## Options
1. **Keep it** (as built). The normal missile only hits slow or standing targets: ground units and landed aircraft. Against air, Rocketmen rely on Target Lock, and on a spotter against a Sparrow that strafes them. 0356's "sometimes dodges" becomes "always dodges".
2. **Let a lone group lock a strafer:** raise the Rocketman's SightRange to at least its LockRange (7, or 8 to match the Sparrow's), or shorten LockOnDuration so a lock fits in one pass (at most about 30 frames). Either is a number change.
3. **Let the normal missile lead its target:** it aims where the target will be when the missile lands, if it keeps its speed and heading. A Sparrow flying straight is then hit, and one that turns, circling or looping, can dodge. That is what "sometimes dodges" suggests.
   - It is a new aiming rule for the Rocketman, or for DoublyDisjointed in general.
   - The Hand of Heaven's launcher is DoublyDisjointed too, so it would also lead, unless the rule is the Rocketman's alone.
4. **A faster normal missile** (ProjectileSpeed 16 → for example 40 su/frame) narrows the window, but a circling Sparrow still moves 4 su/frame, so hits stay rare unless the missile also leads.
