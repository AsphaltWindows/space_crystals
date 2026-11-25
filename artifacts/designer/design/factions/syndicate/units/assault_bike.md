# Assault Bike
(user, 2026-11-25) Syndicate fast attack vehicle. The **fastest wheeled vehicle in the game**, with a **relatively wide turning radius** for a fast vehicle (and, being wheeled, it can't turn on the spot). Armed with a **mortar**: like the Grenadier's grenade (slow, dodgeable explosive projectile with area damage) but with **slightly longer range**. The mortar sits on a **turret with a narrow turn angle**: it can aim a little to either side of the bike's facing, but the bike must still roughly point toward its target. Ground only (assumed, like the Grenadier); can Attack Ground.

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - WheeledVehicle (exits only through Tier 2+ Tunnels)
## Produced at - Vehicle Assembly
## Groupable - true (assumed)

## TurretAttributes:
- TurnAngle: narrow (value TBD)

## AttackAttributes:
- AttackType: DoublyDisjointed (mortar; slow projectile to the target's position at the end of Aiming; can be dodged)
- TargetDomain: Ground (assumed)
- TargetType: AoE
- Range: slightly longer than the Grenadier's
- CanTargetGround: true (derived)

All numbers (HP, armor, sight, speed, turn radius, turret turn angle and rate, TunnelSpaceCost, damage, AoE radius, range, projectile speed, timings, cost, build time): TBD.

## ObjectInterfaceState: BasicCombatUnitInterfaceState
