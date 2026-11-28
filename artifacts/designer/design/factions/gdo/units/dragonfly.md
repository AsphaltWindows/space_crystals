# Dragonfly
GDO air-to-air fighter produced at the Airfield once the player owns Propulsion Labs. Fires **air-to-air missiles**. Carries **limited ammo** and rearms at its Airfield (factions/gdo/structures/airfield.md > Aircraft Support).

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Airfield
## TechPrerequisite - player owns at least one Propulsion Labs (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)
## UnitBase - Glider (must keep moving; circles when idle; wider turns at higher speed; with **vertical landing and take-off at the Airfield**, Q0162: factions/gdo/structures/airfield.md > Returning and rearming)

## AttackAttributes:
- Weapon: air-to-air missiles
- TargetDomain: Air
- Ammo: **1 missile**, then it must reload at its assigned Airfield
- RearmTimePerAmmo: placeholder below
- AttackType, Damage, Range, timings: placeholders below

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 40x40 space units; MaxHP 100; PointArmor 1, FullArmor 1; SightRange 8 grid units; Groupable true; UnitControlCost 2.
- GliderMovement: IdleSpeed 6, MaxSpeed **14 su/frame**, Acceleration 0.6, Deceleration 0.6, MaxCentripetalAcceleration 0.6 (about 330 su turn radius at full speed); idle circle radius 1 grid unit; SeparationRadius **48** su (raised from 32, Q0148; must exceed the 40 su Silhouette). No turret (fires forward).
- Attack: **HeadDisjointed** (homing air-to-air missile; user, 2026-11-27); Damage **90**; Range 6 grid units; ProjectileSpeed 20 su/frame; **Aim 2** (was 8), Firing 2, Cooldown 4, Reload 0 (1 missile); (user, 2026-11-28; Q0192: limited-ammo aircraft fire **from their base, with no turret**, so they never spend their payload on enemies they pass on a Move (base engagement isn't active on Move); they must **face the target** to fire, with no launch cone; a **short Aim** keeps it smooth; Aim value designer placeholder); **RearmTimePerAmmo 64 frames**.
- Cost: Airfield, 175 Space Crystals **+ 4 Supplies** (user, 2026-11-26; Q0169), 320 frames.
- Panel: BasicCombatUnitInterfaceState plus **F: Return to Base**.
