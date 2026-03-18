# Feature Update: unit_commands_and_behaviors (2026-03-06)

## Modified Feature File
`features/unit_commands_and_behaviors.md` (NEW)

## Relevant Design Files
- `design/control_system.md`

## Summary
Initial feature specification created from formal design content. Defines 8 unit commands (Move through Reverse), BaseCommandState/BaseBehaviorState (parameterized by UnitBase), TurretCommandState/TurretBehaviorState, 3 base action channels (Locomotion/Orientation/BaseAttack), 2 turret action channels, 9 base behaviors (MovingToLocation through StoppingBehavior) with Glider exceptions, TurretAutonomousScanning with priority rules, and BaseAutoTargeting (active during Idle with 4gu leash and HoldPosition; inactive during Move/AttackTarget/AttackGround/Stop).
