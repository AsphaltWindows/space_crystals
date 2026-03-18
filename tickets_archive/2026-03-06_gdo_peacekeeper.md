# Ticket: GDO Peacekeeper Unit

## Current State
No GDO combat units exist. There is no concrete unit definition using the LightInfantry base.

## Desired State
Implement the Peacekeeper as the first concrete GDO unit:

**Peacekeeper** (Unit - LightInfantry):
- Faction: GlobalDefenseOrdinance
- Silhouette: 24x24 space units (square)
- MaxHP: 50, PointArmor: 1, FullArmor: 1, SightRange: 5
- Destructible: true, Groupable: true
- UnitBase: LightInfantry, UnitControlCost: 1
- UnitBaseAttributes[LightInfantry]:
  - TurnRate: 180 degrees/frame
  - Acceleration: infinite
  - Deceleration: infinite
  - MaxSpeed: 4 space units/frame
  - RuggedTerrainDefenseBonus: 50%
- TurretAttributes: None (infantry — attacks via BaseAttackChannel)
- AttackAttributes:
  - AttackType: FullyConnected (Ranged)
  - TargetDomain: Ground
  - TargetType: SingleTarget
  - Damage: 10, Range: 4 grid units, MinRange: 0
  - AimDuration: 2 frames, FiringDuration: 1 frame, CooldownDuration: 2 frames, ReloadDuration: 12 frames
- ObjectInterfaceState: BasicCombatUnitInterfaceState

## Justification
Defined in `features/gdo_objects.md` (Peacekeeper section). The Peacekeeper is the only GDO combat unit in the initial spec and serves as the first concrete implementation of the LightInfantry unit base, the FullyConnected attack type, and the BasicCombatUnitInterfaceState.

## QA Steps
1. Produce a Peacekeeper from a Barracks. Verify it spawns with correct silhouette size (24x24 su).
2. Verify MaxHP is 50, PointArmor is 1, FullArmor is 1, SightRange is 5.
3. Select the Peacekeeper. Verify it uses BasicCombatUnitInterfaceState commands: HoldPosition, Stop, Attack, Move, Patrol. Verify AttackGround is NOT available (FullyConnected has CanTargetGround=false). Verify Reverse is NOT available (LightInfantry has CanReverse=false).
4. Issue a Move command. Verify the unit moves at MaxSpeed 4 su/frame with instant acceleration and deceleration.
5. Verify the unit can turn at 180 degrees/frame (effectively instant turning).
6. Right-click an enemy ground unit. Verify Attack command is issued and the Peacekeeper engages.
7. Verify attack sequence: 2 frames aiming, 1 frame firing, 2 frames cooldown, 12 frames reload. Verify FullyConnected attack hits instantly (no projectile).
8. Verify damage dealt is 10 per attack (before armor).
9. Verify TargetDomain is Ground — the Peacekeeper should not be able to target air units.
10. Verify the Peacekeeper has no turret (attacks via BaseAttackChannel, must face target to fire).
11. Place the Peacekeeper on rugged terrain. Verify RuggedTerrainDefenseBonus of 50% is applied.
12. Verify UnitControlCost is 1 (counts against player's unit control limit).

## Expected Experience
The Peacekeeper should feel like a responsive, fast-turning infantry unit. Movement should be snappy with no acceleration ramp-up (infinite accel/decel). Attacks should feel direct and immediate (FullyConnected — no projectile travel time). The attack rhythm should be noticeable: quick aim-fire-cooldown then a longer reload pause before the next shot. On rugged terrain, the unit should visually occupy a defensive position.
