# Assault Bike
(user, 2026-11-25) Syndicate fast attack vehicle. The **fastest wheeled vehicle in the game**, with a **relatively wide turning radius** for a fast vehicle (and, being wheeled, it can't turn on the spot). Armed with a **mortar**: like the Grenadier's grenade (slow, dodgeable explosive projectile with area damage) but with **slightly longer range**. The mortar sits on a **turret with a narrow turn angle**: it can aim a little to either side of the bike's facing, but the bike must still roughly point toward its target. Ground only (assumed, like the Grenadier); can Attack Ground.

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - WheeledVehicle (exits only through Tier 2+ Tunnels)
## Produced at - Vehicle Assembly
## Groupable - true (assumed)

## TurretAttributes:
- TurnAngle: narrow (placeholder below)

## AttackAttributes:
- AttackType: DoublyDisjointed (mortar; slow projectile to the target's position at the end of Aiming; can be dodged)
- TargetDomain: Ground (assumed)
- TargetType: AoE
- Range: slightly longer than the Grenadier's
- CanTargetGround: true (derived)

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 56x32 space units; MaxHP 100; PointArmor 1, FullArmor 1; SightRange 7 grid units; TunnelSpaceCost 2.
- FixedTurnRadiusMovement: MinimumTurnRadius **128 su** (wide for its speed); ForwardMaxSpeed **12 su/frame** (the fastest wheeled vehicle); ForwardAcceleration 1; ReverseMaxSpeed 3; ReverseAcceleration 0.5; Deceleration 1.
- Turret: TurnAngle **40°** total; TurnRate 8°/frame.
- Mortar: Damage 120 total over the circle; AoERadius 48 su; Range **5 grid units** (Grenadier: 4); **MinRange 1 grid unit** (user, 2026-11-27; Q0184: so it never fires close enough for its own blast to hit it; value placeholder, just beyond the 48 su blast; was 0); ProjectileSpeed 8 su/frame; Aim 8, Firing 2, Cooldown 4, Reload 34 frames (one round per 48 frames).
- Cost: Vehicle Assembly, 125 Space Crystals, 240 frames.

## ObjectInterfaceState: BasicCombatUnitInterfaceState
