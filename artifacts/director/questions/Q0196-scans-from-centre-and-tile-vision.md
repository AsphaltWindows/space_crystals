# Q0196 — Target scans from the centre: tile-based vision lets a sight buff still change a lone unit's engagement, and some units don't fire at an enemy in weapon range

- **Date**: 2026-11-28T04:05:36-05:00
- **Blocking**: no. Proceeding on option (a) for both points, as now built: scans measure from the unit's centre to the target's edge, the Director's choice under 0467's Verification 2.
- **Docs**:
  - `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`;
  - `core/vision.md > Vision` and `> Visible`;
  - `systems/combat.md > Range Measurement`;
  - change 0467 (Verification 1 and 2), and 0463 (the principle that buffs don't change behaviour).

## Question
1. **When is a tile "within the SightRange"?** The answer decides whether a sight buff can still change when a lone unit engages.
2. **Should an idle unit fire at a visible enemy that is within its weapon range but just outside its scan?**

## Context
**Point 1: a buff still matters for a lone unit.**
- The design says a tile is Visible when it is "within the SightRange" of one of the player's objects, a disc around the object's centre.
- The game reads that as **the tile's centre** lying within the disc. Nothing in the design says so, and the choice was never marked as a placeholder.
- An object is seen when it covers a Visible tile, and a scan picks only targets its player sees.
- A scan now measures from the unit's centre to the target's edge. So it can reach a target before any eye sees it, by up to half a tile's diagonal (about 45 su). Until then, a lone unit with no other eye on the target waits.
- A sight buff widens the unit's eye, so it sees the target sooner. In a test, a lone buffed Guard engaged 10 su and 2 ticks before an unbuffed one. 0467's Verification 1 ("A lone Guard engages at the same distance buffed and unbuffed") holds only where the target is already seen when the scan reaches it.
- With any other eye on the target, buffed and unbuffed units engage together.

**Point 2: a band where the weapon reaches but the scan doesn't.**
- Weapon range is edge to edge, so a unit's weapon reaches its Range plus its own half width from its centre. Its scan reaches its SightRange from its centre.
- For the units whose Range equals their SightRange, that leaves a band of their half width:
  - the Gunner: 12 su;
  - the Rocketman: 12 su;
  - the Twister: 26 su.
- An idle unit doesn't fire at a visible enemy in that band, for instance one in a second row behind a firing line. This is 0467's "about half its width later".
- Before 0467, the scan measured edge to edge, and such an enemy was always picked.
- For the artillery (the Artillery Rig, the Hand of Heaven, the Laser Battery), Range exceeds SightRange by design. That was so before 0467 too.

## Options
**Point 1:**
- **(a) As built: a tile is Visible when its centre is within the disc.**
  - A lone unit's buff can make it engage up to about 45 su earlier, and only where no other eye sees the target.
  - No further change.
- **(b) A tile is Visible when any part of it is within the disc.**
  - A target within a unit's scan is then always on a Visible tile. The scan and the unit's eyesight always agree, and a buff never changes when a unit engages.
  - Every unit and structure would see up to half a tile's diagonal further at the edge of its sight.
  - Most goldens and the computer players' games move again.
- **(c) The scan is the unit's own unbuffed eye, by vision's tile test.**
  - The scan and eyesight agree exactly, and a buff never matters.
  - Units engage up to another half tile later than in (a).
  - About 20 tests built on the old reach need re-deriving. This was built once, and replaced by (a).

**Point 2:**
- **(a) As built:** such a unit waits until the enemy is within SightRange of its centre.
- **(b) The idle scan also picks a visible enemy within weapon range**, whichever reaches further. Units then never stay idle with a visible enemy in range.
  - A sight buff still changes nothing, since weapon range isn't buffed by sight.
  - A small code change, and the goldens move.
