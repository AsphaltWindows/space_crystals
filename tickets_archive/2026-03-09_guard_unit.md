# Ticket: Guard Unit Implementation

## Current State
The Guard unit does not exist in the codebase. The Syndicate faction only has the Agent unit.

## Desired State
Implement the Guard as a new Syndicate combat unit with the following specification:

- **Faction**: TheSyndicate
- **UnitBase**: HeavyInfantry
- **Silhouette**: 36x36 space units
- **MaxHP**: 80, **PointArmor**: 1, **FullArmor**: 1
- **SightRange**: 5
- **TunnelSpaceCost**: 2
- **Groupable**: true (unlike Agent — Guard can be added to control groups)

### Movement (TurnRateMovement)
- MaxSpeed: 5 su/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 deg/frame

### Attack (FullyConnected Ranged)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 6
- Range: 3 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 1 frame
- CooldownDuration: 1 frame
- ReloadDuration: 4 frames

### ObjectInterfaceState
BasicCombatUnitInterfaceState (standard combat unit — Move, Stop, Attack commands).

### Required Implementation
- Add `SyndicateGuard` (or equivalent) variant to `ObjectEnum`
- Create `spawn_syndicate_guard()` function following the pattern of `spawn_syndicate_agent()`
- Register Guard in unit base definitions (HeavyInfantry)
- Guard must be selectable, groupable, and respond to standard combat unit commands

## Justification
`features/syndicate_objects.md` — Guard unit section. The Guard is the Syndicate's first dedicated combat unit, filling the basic infantry role analogous to the GDO Peacekeeper. Required for Syndicate to have any combat capability beyond Agent melee.

## QA Steps
1. [auto] Verify `ObjectEnum` has a Guard/SyndicateGuard variant
2. [auto] Verify Guard spawn function exists and sets correct HP (80), armor (1/1), speed (5), and attack stats
3. [human] Spawn a Guard in-game — verify it appears with correct silhouette size (36x36)
4. [human] Select the Guard — verify BasicCombatUnitInterfaceState commands appear (Move, Stop, Attack)
5. [human] Add Guard to a control group (Ctrl+1) — verify it is groupable
6. [human] Order Guard to attack an enemy unit — verify ranged attack fires at range 3 with correct damage (6)
7. [human] Verify Guard moves at visibly slower speed than Agent (5 vs 6 su/frame)

## Expected Experience
The Guard appears as a Syndicate heavy infantry unit. When selected, it shows standard combat commands. It can be grouped with other groupable units. In combat, it fires a rapid ranged attack (fast cycle time) at moderate range. It moves noticeably slower than Agents but is tougher (80 HP vs 75 HP) and deals ranged damage instead of melee.
