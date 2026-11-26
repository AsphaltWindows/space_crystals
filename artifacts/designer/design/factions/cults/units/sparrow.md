# Sparrow

(Name: user, 2026-11-26, "ok for now".)

(user, 2026-11-26) A **small glider** built at the Airstrip, with a **machine-gun-style** attack that hits **both air and ground**.

## Entity Type - Unit
## Faction - TheCults
## UnitBase - Glider (must keep moving; circles when idle; systems/units.md)
## Produced at - Airstrip (consumes **1 Recruit**; Unit Control cost **1**). On completion it **takes off from the Airstrip** and **never lands** again (user, 2026-11-26).

## TurretAttributes:
- TurnAngle: **45° total cone** (22.5° to each side of its facing; user, 2026-11-26)

## AttackAttributes:
- AttackType: FullyConnected (machine gun similar to the Jackal's; **cannot miss**)
- TargetDomain: **Universal** (air and ground)
- TargetType: SingleTarget
- Rapid fire, low damage per shot
- **Unlimited ammo** (user, 2026-11-26): unlike GDO aircraft it never returns to re-arm.

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Silhouette 40x40 space units; MaxHP 50; PointArmor 0, FullArmor 0; SightRange 6 grid units; Groupable true.
- GliderMovement: IdleSpeed 4 su/frame; MaxSpeed 10 su/frame; Acceleration 0.5 su/frame²; Deceleration 0.5 su/frame²; MaxCentripetalAcceleration 1 su/frame² (turn radius about 100 su at full speed).
- SeparationRadius 32 space units (Q0120).
- **Idle circle**: radius **1 grid unit (64 su)** at IdleSpeed (placeholder; Q0120: at its full centripetal limit an idle circle would be only 16 su, nearly a spin).
- **Flight rules** (user, 2026-11-26; Q0120):
  - **Map edge**: it may overshoot the edge as far as its turn needs (no extra cap).
  - **Takeoff**: it appears over the runway's B end facing C at IdleSpeed and speeds up along the runway.
  - **No rally point**: it circles 2 grid units past the runway's C end.
  - **Strafing**: after a pass it flies on until the target is out of range, then loops back at full speed.
  - **Stop**: it circles where it is. **Hold Position**: it circles there and fires only at what comes into range, without chasing.
- Turret: TurnAngle 45° (total), TurnRate 8°/frame.
- Attack: Damage 3; Range 3 grid units; MinRange 0; Aim 1, Firing 1, Cooldown 1, Reload 1 frames (12 damage per second before armor).
- Cost: Airstrip, 1 Recruit, 75 Space Crystals, 320 frames.
- ObjectInterfaceState: BasicCombatUnitInterfaceState.
