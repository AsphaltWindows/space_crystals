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
- AttackType: FullyConnected (machine gun similar to the Seeker's; **cannot miss**)
- TargetDomain: **Universal** (air and ground)
- TargetType: SingleTarget
- Rapid fire, low damage per shot
- **Unlimited ammo** (user, 2026-11-26): unlike GDO aircraft it never returns to re-arm.

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Silhouette 40x40 space units; MaxHP 50; PointArmor 0, FullArmor 0; SightRange **8 grid units** (user, 2026-11-27; Q0148: was 6; its full-speed loop carries it up to ~500 su from its target, so 8 GU = 512 su keeps the target in sight through every pass and a lone Sparrow keeps strafing on an Attack. Its speed may also be tuned in the numbers pass. **Attack Move is unchanged** (user, 2026-11-27): its 6 GU leash may still drop a looping Sparrow's target; use a direct Attack for repeated passes. With several Sparrows, their shared vision covers the loop anyway. What Attack Move should mean for units that can't stop is an open question); Groupable true.
- GliderMovement: IdleSpeed 4 su/frame; MaxSpeed 10 su/frame; Acceleration 0.5 su/frame²; Deceleration 0.5 su/frame²; MaxCentripetalAcceleration **0.5 su/frame²** (was 1, "too much grip", user 2026-11-26; tightest turn radius = speed² / 0.5: **200 su (about 3 GU) at full speed**, a full loop in about 8 s; 32 su at IdleSpeed).
- SeparationRadius **48** space units (Q0120; raised from 32, 2026-11-27, Q0148: it must exceed the 40 su Silhouette). Gliders push apart slowly (systems/units.md > Air Collision).
- **Idle circle**: radius **1 grid unit (64 su)** at IdleSpeed (placeholder; Q0120: at its centripetal limit an idle circle would be only 32 su, nearly a spin, so idle circling uses a gentler turn).
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
