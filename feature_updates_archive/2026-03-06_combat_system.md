# Feature Update: combat_system (2026-03-06)

## Modified Feature File
`features/combat_system.md` (NEW)

## Relevant Design Files
- `design/combat.md`

## Summary
Initial feature specification created from formal design content. Defines AttackAttributes, 4 attack phases (Aiming/Firing/Cooldown/Reloading) with interruptibility and allowed actions, 4 attack types (FullyConnected/HeadDisjointed/TailDisjointed/DoublyDisjointed) with CanMiss/CanTargetGround derivations, TargetDomain (Ground/Air/Universal), ValidTarget filter, and damage calculation for SingleTarget (PointArmor) and AoE (area-based FullArmor) including directional armor.
