# Grenadier
(user, 2026-11-25) Syndicate explosives infantry. Fires an **explosive grenade**: a **slow projectile** that flies to where the target was when the Grenadier finished aiming and explodes there, dealing **moderate-high area damage**. **Ground only.** Moving units can dodge it; it punishes units standing still or clumped together. Like every attack that can miss, it can also **Attack Ground** (systems/combat.md > AttackType > CanTargetGround).

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - HeavyInfantry (assumed, like all Syndicate infantry)
## Produced at - Hideout
## TechPrerequisite - player owns at least one **Chemical Plant** (explosives)
## Groupable - true (assumed)

**Placeholder stats** (designer, 2026-11-25, Q0071; tune later):
## Silhouette - 36x36
## MaxHP - 70
## PointArmor - 1
## FullArmor - 1
## SightRange - 5
## TunnelSpaceCost - 1

## UnitBaseAttributes[HeavyInfantry]:
- MaxSpeed: 5 space units/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 degrees/frame

## AttackAttributes:
- AttackType: DoublyDisjointed (slow projectile to the target's position at the end of Aiming; can be dodged)
- TargetDomain: Ground
- TargetType: AoE (moderate-high damage)
- CanTargetGround: true (derived)
- Damage: 100 (total over the circle; systems/combat.md > AoEDamage: a 36x36 infantry fully inside takes about 26 before armor)
- AoE radius: 40 space units (0.625 grid units)
- Range: 4 grid units
- MinRange: 0
- ProjectileSpeed: 8 space units/frame (a 4 GU throw takes 2 s, so moving units can step out of the blast)
- AimDuration: 6 frames; FiringDuration: 2 frames; CooldownDuration: 4 frames; ReloadDuration: 28 frames (one grenade per 40 frames, 2.5 s)

## Cost - 150 Space Crystals; Build Time - 160 frames (10 seconds)

## ObjectInterfaceState: BasicCombatUnitInterfaceState
