# guard-unit

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the Guard unit as defined in `artifacts/designer/design/syndicate_objects.md` under 'Guard'.

The Guard is the Syndicate's basic combat infantry. A heavy infantry unit with a rapid-fire fully connected attack, tougher than the GDO Peacekeeper but with shorter range.

**Entity Type:** Unit
**Faction:** TheSyndicate
**UnitBase:** HeavyInfantry

**Stats:**
- Silhouette: 36x36
- MaxHP: 80
- PointArmor: 1
- FullArmor: 1
- SightRange: 5
- TunnelSpaceCost: 2
- Groupable: true

**UnitBaseAttributes[HeavyInfantry]:**
- MaxSpeed: 5 space units/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 degrees/frame

**TurretAttributes:** None

**AttackAttributes:**
- AttackType: FullyConnected
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 6
- Range: 3 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 1 frame
- CooldownDuration: 1 frame
- ReloadDuration: 4 frames

**ObjectInterfaceState:** BasicCombatUnitInterfaceState

**Production:** Built at Headquarters for 125 Space Crystals, 120 frames (7.5 seconds).

## QA Instructions

1. Build a Guard from a Headquarters (W command). Verify 125 crystals deducted and 7.5-second build time.
2. Eject the Guard from a Tunnel. Verify it appears at Side A.
3. Select the Guard — verify BasicCombatUnitInterfaceState commands (Q=Move, A=Attack, S=Patrol, E=HoldPosition, X=Stop).
4. Verify the Guard's silhouette is 36x36.
5. Order the Guard to attack an enemy unit. Verify it engages at range 3 (shorter than Peacekeeper's range 4).
6. Verify attack cycle: 2 frames aim, 1 frame fire, 1 frame cooldown, 4 frames reload.
7. Verify the Guard deals 6 damage per shot.
8. Verify the Guard has 80 HP (tougher than Peacekeeper's 50 HP).
9. Verify the Guard moves at 5 space units/frame.
10. Verify the Guard cannot be crushed (HeavyInfantry: Crushable=false).
11. Verify the Guard can traverse rugged terrain.
12. Select multiple Guards — verify they group together (Groupable=true).
