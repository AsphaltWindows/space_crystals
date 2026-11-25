# Garrison

The Cults' defensive structure. Infantry enter the Garrison and fire from inside it: Recruits and Disciples fire **as if they were Militants** (using the Militant's attack — the weapons come from the Armory, which is why the Garrison requires one), while Militants and Gunners fire their own weapons. Without units inside, the Garrison does not attack.

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Armory
## GarrisonCapacity - 4 units

## Garrisoning
- **Who can enter**: Recruits, Militants, Gunners, and Disciples. Units enter with the Enter command (right-click own Garrison with eligible infantry selected), up to 4 units total.
- **Weapons**: a Recruit or Disciple inside attacks with the Militant's AttackAttributes (see Militant); Militants and Gunners use their own AttackAttributes. (Whether an Disciple can also cast spells from inside: TBD.)
- **Range bonus**: **Militants and Gunners** inside get **+1 grid unit of Range** on their attack. Recruits and Disciples (firing with the Militant's attack) do not get the bonus — they fire at the Militant's normal Range.
- Units inside are not consumed and still count toward Unit Control.
- Evacuation on Destruction applies: when the Garrison is destroyed, everyone inside comes out alive around its footprint.
- **Protection**: units inside cannot be targeted and take no damage while the Garrison stands.
- Sight: units inside see with the Garrison's SightRange (placeholder, below).

Size, armor and other stats: see Placeholder stats below. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Size 2x2; SymmetryType AAAA; MaxHP 400; PointArmor 2, FullArmor 8 (building armor rule, n = 2: sturdier as a defense); SightRange 6 grid units; Groupable true.
- Cost and time: Recruit > Constructs (150 SC, 400 frames).
- Units inside see with the Garrison's SightRange and fire at enemies the player sees within their (bonus) range.
- Units **leave through any side**, at the first clear spot next to the footprint (like Evacuation on Destruction).
- Panel: **A: Eject All** (everyone inside leaves); **X: Cancel construction** while being built. Right-click resolution: none (it has no rally point).
