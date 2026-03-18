# Feature Update: unit_system (2026-03-06)

## Modified Feature File
`features/unit_system.md` (NEW)

## Relevant Design Files
- `design/units.md`
- `design/combat.md` (LocomotionOrientationConstraints)

## Summary
Initial feature specification created from formal design content. Defines Unit type with Silhouette/Armor/UnitBase, Unit Instance with continuous rotation and command queue, 9 UnitBases (LightInfantry through Glider) with boolean property matrix, TurretAttributes (TurnAngle, TurnRate), 5 MovementModels with full parameter sets, and LocomotionOrientationConstraints per movement model.
