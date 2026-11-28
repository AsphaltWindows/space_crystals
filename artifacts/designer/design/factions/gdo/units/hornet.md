# Hornet
GDO air-to-ground attack aircraft produced at the Airfield. A HoverCraft (a chopper) that fires **air-to-ground missiles**. Carries **limited ammo** and rearms at its Airfield (factions/gdo/structures/airfield.md > Aircraft Support).

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Airfield
## TechPrerequisite - none beyond owning the Airfield that produces it
## UnitBase - HoverCraft (can hover in place, turns in place)

## AttackAttributes:
- Weapon: air-to-ground missiles
- TargetDomain: Ground
- Ammo: **5 missiles**, then it must reload at its assigned Airfield
- RearmTimePerAmmo: placeholder below
- **TargetType: AoE** (user, 2026-11-27; Q0182, as the Rocketman's: factions/gdo/units/rocketman.md): the missile **bursts at the spot** it flies to (systems/combat.md > AoEDamage; friendly fire applies), hitting **ground (and underground) objects only**, as an air-to-ground weapon. If nothing is in the circle, it bursts harmlessly.
- AttackType, Damage, Range, timings: placeholders below

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 48x48 space units; **SeparationRadius 56 su** (designer placeholder, 2026-11-27; Q0181: Silhouette side + 8 su); MaxHP 120; PointArmor 1, FullArmor 1; SightRange 7 grid units; Groupable true; UnitControlCost 2.
- HoverCraft movement (DragMovement, as the HoverCraft base; designer placeholder 2026-11-26, Q0154: high drag and high thrust, like the Supply Chopper): ForwardAcceleration 0.8; OmniDirectionalAcceleration 0.1; DragRatio 0.1 (top speed 9 su/frame); TurnRate 8°/frame. No turret (fires forward; it turns in place to aim).
- Attack: **DoublyDisjointed** (not homing: the missile flies to where the target was, so it can be dodged; it can Attack Ground. user, 2026-11-27); **Damage 45 total over an AoERadius of 32 su** (designer placeholder, 2026-11-27; Q0182: was 30 single-target; about 33 to a 64x40 vehicle hit square-on, 8 to a 24x24 infantry unit, before armor (corrected 2026-11-28, Q0189)); Range 5 grid units; ProjectileSpeed 14 su/frame; Aim 4, Firing 2, Cooldown 4, Reload 22 frames; **RearmTimePerAmmo 32 frames** (5 missiles in 10 s).
- Cost: Airfield, 150 Space Crystals **+ 2 Supplies** (user, 2026-11-26; Q0169), 320 frames.
- Panel: BasicCombatUnitInterfaceState plus **F: Rearm** (renamed from Return to Base, 2026-11-28).
