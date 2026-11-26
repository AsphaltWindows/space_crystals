# Missile Turret
GDO defensive structure. Attacks both air and ground targets (TargetDomain: Universal — see systems/combat.md).

## Faction - GlobalDefenseOrganization
## Constructed by - DefenseSystem
## TechPrerequisite - None

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 1x1; SymmetryType AAAA; MaxHP 250; PointArmor 2, FullArmor 2; SightRange 7 grid units; Groupable true; Power −10; BuildRadiusExtension 0.
- Attack: **HeadDisjointed** (homing missiles, cannot miss; user, 2026-11-27), TargetDomain Universal, SingleTarget; Damage 18; Range 6 grid units; MinRange 0; ProjectileSpeed 16 su/frame; Aim 4, Firing 2, Cooldown 4, Reload 14 frames (one missile per 24 frames).
- Targeting as the Laser Turret's (BaseAutoTargeting, closest, keeps its target).
