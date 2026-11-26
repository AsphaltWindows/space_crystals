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
- AttackType, Damage, Range, timings: placeholders below

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 48x48 space units; MaxHP 120; PointArmor 1, FullArmor 1; SightRange 7 grid units; Groupable true; UnitControlCost 2.
- HoverCraft movement: MaxSpeed 9 su/frame; Acceleration 0.6; Deceleration 0.6; TurnRate 8°/frame. No turret (fires forward; it turns in place to aim).
- Attack: **DoublyDisjointed** (not homing: the missile flies to where the target was, so it can be dodged; it can Attack Ground. user, 2026-11-27); Damage 30; Range 5 grid units; ProjectileSpeed 14 su/frame; Aim 4, Firing 2, Cooldown 4, Reload 22 frames; **RearmTimePerAmmo 32 frames** (5 missiles in 10 s).
- Cost: Airfield, 150 Space Crystals, 320 frames.
- Panel: BasicCombatUnitInterfaceState plus **F: Return to Base**.
