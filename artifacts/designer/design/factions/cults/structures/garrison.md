# Garrison

The Cults' defensive structure. Infantry enter the Garrison and fire from inside it: Recruits fire **as if they were Militants** (using the Militant's attack — the weapons come from the Armory, which is why the Garrison requires one), while Militants and Gunners fire their own weapons. Without units inside, the Garrison does not attack.

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Armory
## GarrisonCapacity - 4 units

## Garrisoning
Uses **Garrisoning, Garrison structure** (systems/mechanics/garrisoning.md), with:
- **Capacity 4**. **Who**: Recruits, Militants, Gunners, Disciples, and Disciples' **illusions** (factions/cults/units/disciple.md > Mirage).
- **Weapons inside**: a **Recruit** fires the **Militant's** attack (see Militant; the weapons come from the Armory); **Militants and Gunners** fire their own; **Disciples and illusions don't fire** (user, 2026-11-27; replaces "Disciples fire as Militants"): they are only sheltered. **Augmentations don't change the borrowed attack** (Q0115).
- **Range bonus**: **Militants and Gunners** get **+1 grid unit**; Recruits (with the Militant's attack) don't.
- **Sight**: occupants see with the Garrison's SightRange (placeholder, below).
- **Destroyed**: Evacuation on Destruction (factions/cults/economy.md): everyone inside comes out alive inside its former footprint.

Size, armor and other stats: see Placeholder stats below. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Size 2x2; SymmetryType AAAB, **the door on side B** (one door, entrance and exit; user, 2026-11-26; Q0116; side placeholder); MaxHP 400; PointArmor 2, FullArmor 8 (building armor rule, n = 2: sturdier as a defense); SightRange 6 grid units; Groupable true.
- Cost and time: Recruit > Constructs (150 SC, 400 frames).
- Units inside see with the Garrison's SightRange and fire at enemies the player sees within their (bonus) range.
- **Leaving** (user, 2026-11-26; Q0115/Q0116): units leave **through the door**; on **Eject All** they all come out **at once**, ignoring collision with each other until they are clear of the footprint. (Evacuation on Destruction puts them all at once inside its former footprint; factions/cults/economy.md > Evacuation on Destruction.)
- Panel: **A: Eject All** (everyone inside leaves); **X: Cancel construction** while being built. Right-click resolution: none (it has no rally point).
