# Phantom
(user, 2026-11-25) Syndicate **hovering aircraft**. It **rises vertically out of a Tier 3 Tunnel's air door** (factions/syndicate/tunnels.md > Tunnel > Look by tier). Fires a **red laser** that hits **both air and ground**. **Relatively manoeuvrable.**

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - HoverCraft (enters and exits only through Tier 3 Tunnels' air doors)
## Produced at - Aerospace Assembly
## TechPrerequisite - player owns at least one **Optics Lab** (user, 2026-11-25)
## Groupable - true (confirmed, Q0071)

## AttackAttributes:
- AttackType: FullyConnected (laser like other Syndicate lasers: instant, cannot miss; confirmed, Q0071)
- TargetDomain: Universal (ground and air)
- TargetType: SingleTarget (confirmed, Q0071)

## Role
(user, 2026-11-25) **Comparable to the Guard** (a laser all-rounder against air and ground), but **somewhat squishier for its cost**, and **faster**, and of course it **flies** (over cliffs, buildings and ground units like any air unit): the airborne, mobile counterpart of the Guard.

## TurretAttributes:
- TurnAngle: **moderate, 60–90°** (user, 2026-11-25: the **total** cone, i.e. 30–45° to each side of its facing; placeholder below)

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 44x44 space units; MaxHP 70; PointArmor 1, FullArmor 1; SightRange 8 grid units; TunnelSpaceCost 2.
- HoverCraft (DragMovement; designer placeholder 2026-11-26, Q0154; "relatively manoeuvrable": more drag and more sideways thrust than the Hornet, so it starts, stops and strafes quicker): ForwardAcceleration 0.78; OmniDirectionalAcceleration 0.3; DragRatio 0.12 (top speed 9 su/frame); TurnRate 10°/frame.
- Turret: TurnAngle **75°** total; TurnRate 8°/frame.
- Laser: Damage 12; Range 5 grid units; Aim 2, Firing 1, Cooldown 3, Reload 10 frames (12 damage per second before armor).
- Cost: Aerospace Assembly, 175 Space Crystals, 320 frames.
- ObjectInterfaceState: BasicCombatUnitInterfaceState.
