# Q0190 — A lone Null Chamber-buffed unit engages sooner (sight from the centre, scans from the edge); does a new Shadebringer start with full tanks?

- **Date**: 2026-11-27T21:46:23-05:00
- **Blocking**: no. M6-SB built 0463 and the tanks; the game proceeds as built (option (a) on both points).
- **Docs**:
  - `factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff` (0463);
  - `core/vision.md` (SightRange); `systems/combat.md > Range Measurement` ("edge to edge");
  - `systems/controls/command_behaviors.md` (AttackMove: "Scan for ValidTarget enemies within SightRange");
  - `factions/syndicate/units/shadebringer.md > Charges and recharging`, `> Placeholder stats`.

## Question
1. **0463's Verification 1 holds only when another unit sees the target.** A buffed unit on its own engages about 30 su (6 ticks) sooner than an unbuffed one. Is that acceptable, or should the two measures be made to agree?
2. **Does a new Shadebringer start with full tanks,** or empty, to be filled in the network by a Vault?

## Context
**Point 1.** The game measures two things from different points:
- **Vision** is a disc of SightRange around the unit's **centre**: the tiles its owner sees.
- **A target scan** (idle targeting, AttackMove, Patrol) takes enemies within SightRange of the unit's **edge**, as weapon range is measured (`systems/combat.md`: "edge to edge"). It takes only enemies its owner can see.

So an unbuffed unit alone can't pick a target at its full scan reach: its own eye stops about half its width short. The buff's +2 GU reaches past that, so a lone buffed unit engages out to its full scan reach. That is about 30 su sooner for a Guard.

0463 says the scan keeps the unbuffed SightRange. It does: with a spotter that sees the target, buffed and unbuffed units engage at the same gap, and SB's test shows it. Only a unit alone differs, because its unbuffed eye limited it before.

The doc doesn't say where SightRange is measured from. The centre for vision and the edge for scans dates from the first slices (M1), and nothing showed the difference until now.

**Point 2.** The doc gives the tanks' size, drain and refill, but not what a new Shadebringer has. As built, it leaves production with both tanks full (100 each).
- The Vault is its TechPrerequisite, and it is trained into the Tunnel Network, so an empty one would fill there: 20 s for both tanks, with a free Vault slot.

## Options
**Point 1:**
- **(a) Accept it** (as built). With a spotter nothing changes; alone, the buff's vision lets a unit use its full scan reach.
- **(b) Scans from the centre**, like vision. A lone unit's scan then never reaches past its own eye, buffed or not. Every unit would engage about half its width later than now, so most goldens and the computer players' games would move.
- **(c) Vision from the edge**, like range: a unit sees SightRange past its Silhouette. Every unit would see a little farther, so every game's fog would change.

**Point 2:**
- **(a) Full** (as built).
- **(b) Empty:** it fills in the network, from the Vault, before its first use.
