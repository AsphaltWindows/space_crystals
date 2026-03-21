# peacekeeper-unit

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the Peacekeeper unit as defined in `artifacts/designer/design/gdo_objects.md`.

The Peacekeeper is the GDO's basic infantry unit — a LightInfantry soldier with a fully connected ground attack.

**Entity Type:** Unit
**Faction:** GlobalDefenseOrdinance
**UnitBase:** LightInfantry
**UnitControlCost:** 1

**Stats:**
- Silhouette: 24x24 space units (square)
- MaxHP: 50
- PointArmor: 1
- FullArmor: 1
- SightRange: 5 grid units
- Destructible: true
- Groupable: true

**UnitBaseAttributes[LightInfantry]:**
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 4 space units/frame
- RuggedTerrainDefenseBonus: 50%

**TurretAttributes:** None

**AttackAttributes:**
- AttackType: FullyConnected (Ranged)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 10
- Range: 4 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 1 frame
- CooldownDuration: 2 frames
- ReloadDuration: 12 frames

**ObjectInterfaceState:** BasicCombatUnitInterfaceState

**Production:** Built at Barracks for 50 Space Crystals, 80 frames (5 seconds).

## QA Instructions

1. Build a Peacekeeper from a Barracks (Q command) — verify 50 crystals deducted and 5-second build time.
2. Select the Peacekeeper — verify BasicCombatUnitInterfaceState commands appear (Q=Move, A=Attack, S=Patrol, E=HoldPosition, X=Stop). W (Reverse) and D (AttackGround) should NOT appear.
3. Verify silhouette is 24x24 space units.
4. Order the Peacekeeper to attack — verify it engages at range 4 grid units.
5. Verify attack cycle: 2 frames aim, 1 frame fire, 2 frames cooldown, 12 frames reload.
6. Verify 10 damage per shot (minus target's PointArmor).
7. Verify MaxHP is 50.
8. Verify movement speed of 4 space units/frame with instant accel/decel.
9. Move Peacekeeper onto Rugged Terrain — verify 50% defense bonus (damage reduction).
10. Drive a TrackedVehicle or Mech into the Peacekeeper — verify it gets crushed (LightInfantry: Crushable=true).
11. Select multiple Peacekeepers — verify they group together (Groupable=true).
12. Verify UnitControlCost is 1 — building Peacekeepers increments the GDO Unit Control used count.
