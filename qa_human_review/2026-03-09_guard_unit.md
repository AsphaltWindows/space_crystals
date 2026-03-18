# Task: Guard Unit Implementation

## Original Ticket
The Guard unit does not exist in the codebase. Implement it as a new Syndicate combat unit.

## Spec Summary
- **Faction**: TheSyndicate
- **UnitBase**: HeavyInfantry
- **Silhouette**: 36x36 space units
- **MaxHP**: 80, **PointArmor**: 1, **FullArmor**: 1
- **SightRange**: 5
- **TunnelSpaceCost**: 2
- **Groupable**: true
- **Movement**: TurnRate — MaxSpeed 5 su/frame, Acceleration infinite, Deceleration infinite, TurnRate 180 deg/frame
- **Attack**: FullyConnected Ranged, Ground, SingleTarget — Damage 6, Range 3 GU, MinRange 0, Aim 2 frames, Fire 1 frame, Cooldown 1 frame, Reload 4 frames
- **ObjectInterfaceState**: BasicCombatUnitInterfaceState (Move, Stop, Attack)

## Technical Context

### Change Sites (in order)

#### 1. Add `SyndicateGuard` variant to `ObjectEnum`
- **File**: `src/shared/types.rs` line 312
- Add `SyndicateGuard` below `SyndicateAgent` in the `ObjectEnum` enum (line ~312, under `// Syndicate Units`)

#### 2. Add `object_type()` match arm for `SyndicateGuard`
- **File**: `src/game/types/objects.rs` line 225
- Add match arm after `SyndicateAgent` (line ~231). Pattern:
  ```rust
  ObjectEnum::SyndicateGuard => ObjectType {
      name: "Guard".to_string(),
      size: (36, 36),
      destructible: true,
      sight_range: 5,
      groupable: true, // Unlike Agent — Guard IS groupable
  },
  ```

#### 3. Add `SyndicateGuard` to `is_unit()` match
- **File**: `src/game/types/objects.rs` line 358
- Add `| ObjectEnum::SyndicateGuard` to the `matches!()` expression

#### 4. Add `SyndicateGuard` to `unit_control_cost()`
- **File**: `src/game/types/objects.rs` line 368
- Add match arm: `ObjectEnum::SyndicateGuard => GUARD_CONTROL_COST,`
- Import `GUARD_CONTROL_COST` from unit_data (created in step 5)

#### 5. Create Guard type data and constants in `unit_data.rs`
- **File**: `src/game/units/types/unit_data.rs` (after line ~209, after Agent constants)
- Follow the pattern of `agent_type_data()` / `agent_attack_data()`:

```rust
// === Syndicate Guard Definition ===

pub fn guard_type_data() -> UnitTypeData {
    UnitTypeData {
        faction: FactionEnum::TheSyndicate,
        silhouette_width: 36,
        silhouette_height: 36,
        max_hp: 80,
        point_armor: 1,
        full_armor: 1,
        unit_base: UnitBaseEnum::HeavyInfantry,
    }
}

pub fn guard_attack_data() -> AttackAttributesData {
    AttackAttributesData {
        attack_type: AttackTypeEnum::FullyConnected,
        fc_subtype: Some(FullyConnectedSubtype::Ranged), // RANGED, not Melee like Agent
        target_domain: TargetDomainEnum::Ground,
        target_type: TargetTypeEnum::SingleTarget,
        aoe_radius: None,
        damage: 6,
        range: 3, // 3 grid units (unlike Agent's 0 for melee)
        min_range: 0,
        projectile_speed: None, // FullyConnected = instant hit
        aim_duration: 2,
        firing_duration: 1,
        cooldown_duration: 1,
        reload_duration: 4,
    }
}

pub const GUARD_CONTROL_COST: u32 = 1;
pub const GUARD_TUNNEL_SPACE_COST: u32 = 2;
pub const GUARD_RUGGED_BONUS: f32 = 0.5;
```

#### 6. Create `spawn_syndicate_guard()` function
- **File**: `src/game/utils.rs` (after `spawn_syndicate_agent()` ending at line 579)
- **Follow `spawn_syndicate_agent()` closely** (lines 485-579) with these differences:
  - Use `guard_type_data()` and `guard_attack_data()` instead of agent
  - ObjectEnum: `ObjectEnum::SyndicateGuard`
  - UnitType name: `"Guard"`
  - Max HP: 80 (vs Agent 75)
  - Groupable: true (set via ObjectEnum, no component change needed)
  - **Attack range**: Use `attack_data.range as f32` (= 3.0 GU), NOT `MELEE_RANGE` — Guard is ranged
  - **Movement speed**: `5.0 * 16.0 / 64.0 = 1.25 GU/sec` (vs Agent 1.5)
  - **TurnRate**: same formula `180.0_f32.to_radians() * 16.0`
  - **NO `AgentCarryState`** — Guard is pure combat, doesn't carry resources
  - Keep `TunnelSpaceCost(GUARD_TUNNEL_SPACE_COST)`, `RuggedTerrainDefenseBonus(GUARD_RUGGED_BONUS)`, `UnitControlCost(GUARD_CONTROL_COST)`
  - Mesh: Use same capsule size as Agent (Capsule3d(0.28, 0.8)) — both have 36x36 silhouette

### Key Differences from Agent (summary)
| Property | Agent | Guard |
|---|---|---|
| MaxHP | 75 | 80 |
| Groupable | false | true |
| Attack | Melee (MELEE_RANGE) | Ranged (3 GU) |
| Speed | 6 su/frame (1.5 GU/s) | 5 su/frame (1.25 GU/s) |
| Fire duration | 4 frames | 1 frame |
| Reload duration | 9 frames | 4 frames |
| AgentCarryState | Yes | No (pure combat) |

### Patterns to Follow
- **Speed conversion**: `su_per_frame * FRAMES_PER_SECOND / SPACE_UNITS_PER_GRID_UNIT` — constants at `src/simulation/mod.rs`
- **Frame-to-seconds**: `frames_to_seconds()` at `src/game/units/types/unit_data.rs:212`
- **AttackCapability construction**: See Agent spawn lines 507-521 — but use `attack_data.range as f32` for ranged instead of `MELEE_RANGE`
- **Ranged FullyConnected subtype reference**: See `spawn_peacekeeper()` at `src/game/utils.rs:388` for the Ranged pattern

### Tests
- Add unit tests in `unit_data.rs` matching existing patterns (e.g., test guard_type_data values, guard_attack_data values)
- Add `SyndicateGuard` to `object_type()` tests in `src/game/types/objects.rs` (test block starts at line 379)
- Test `is_unit()` returns true for `SyndicateGuard`
- Test `unit_control_cost()` returns correct value

## Dependencies
None — this is a standalone unit implementation.

## QA Steps
1. [auto] Verify `ObjectEnum` has a `SyndicateGuard` variant
2. [auto] Verify Guard spawn function exists and sets correct HP (80), armor (1/1), speed (1.25 GU/s), and attack stats (damage 6, range 3)
3. [human] Spawn a Guard in-game — verify it appears with correct silhouette size (36x36)
4. [human] Select the Guard — verify BasicCombatUnitInterfaceState commands appear (Move, Stop, Attack)
5. [human] Add Guard to a control group (Ctrl+1) — verify it is groupable
6. [human] Order Guard to attack an enemy unit — verify ranged attack fires at range 3 with correct damage (6)
7. [human] Verify Guard moves at visibly slower speed than Agent (1.25 vs 1.5 GU/s)

## Automated QA Results
- Step 1 [auto]: PASS — ObjectEnum::SyndicateGuard exists with correct name, size (36x36), sight_range (5), groupable (true), is_unit (true)
- Step 2 [auto]: PASS — spawn_syndicate_guard sets HP=80, armor=1/1, speed=1.25 GU/s, damage=6, range=3 GU, attack timings correct
- Steps 3-7 [human]: Results below

## Human QA Results — 2026-03-09
- Step 3 [human]: PASS — Guard appears in-game with visible model, distinguishable from Agent
- Step 4 [human]: PASS — BasicCombatUnitInterfaceState commands shown (Move, Stop, Attack)
- Step 5 [human]: PASS — Guard is groupable (Ctrl+1 assigns, 1 recalls correctly)
- Step 6 [human]: BLOCKED — Guard stuck on HQ tiles (underground expansion not walkable), cannot move to engage enemies
- Step 7 [human]: BLOCKED — Cannot compare movement speed, Guard is stuck
- **Additional bugs found**:
  1. Guard spawns stuck on HQ tiles (same as Agent — HQ footprint not walkable, see forum topic)
  2. Without a rally point set on HQ, Guard spawns at top-left corner of map (world origin 0,0) instead of Tunnel Side A exit — default spawn position is wrong

## Expected Experience
The Guard appears as a Syndicate heavy infantry unit. When selected, it shows standard combat commands. It can be grouped with other groupable units. In combat, it fires a rapid ranged attack (fast cycle time) at moderate range. It moves noticeably slower than Agents but is tougher (80 HP vs 75 HP) and deals ranged damage instead of melee.
