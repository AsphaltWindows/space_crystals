# guard-unit-verification

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-guard-unit.md

## Task

Verify the existing Guard unit implementation matches the design spec. The Guard unit appears to be fully implemented already. Verify all stats, production, and integration points match the spec:

**What exists (verify correctness):**
- `ObjectEnum::SyndicateGuard` in `game/types/objects.rs` — ObjectType: name="Guard", size=(36,36), destructible=true, sight_range=5, groupable=true
- `guard_type_data()` in `game/units/types/unit_data.rs` — faction=TheSyndicate, unit_base=HeavyInfantry, max_hp=80, point_armor=1, full_armor=1
- `guard_attack_data()` in `game/units/types/unit_data.rs` — FullyConnected, Ground, SingleTarget, damage=6, range=3, min_range=0, aim=2, firing=1, cooldown=1, reload=4
- `spawn_syndicate_guard()` in `game/utils.rs` — spawns with all correct components (no AgentCarryState, no turret channels)
- `HeadquartersState::production_cost(SyndicateGuard)` in `game/types/structures.rs` — 125 space_crystals, 120 build_frames
- HQ menu slot (0,1) = W = HqTrain(SyndicateGuard) in `ui/command_panel.rs`
- HQ production tick system spawns Guard correctly in `game/world/faction.rs`
- GUARD_TUNNEL_SPACE_COST=2, GUARD_CONTROL_COST=1, GUARD_RUGGED_BONUS=0.5
- Existing tests cover type data, attack data, ObjectType, unit classification, control cost

**Verify and fix if needed:**
1. Confirm all numeric values match the design spec exactly
2. Confirm HeavyInfantry base data: MaxSpeed=5, Acceleration=infinite, Deceleration=infinite, TurnRate=180
3. Confirm Guard uses ObjectInterfaceState::Default (BasicCombatUnitInterfaceState is handled by a separate feature request)
4. Confirm spawn function includes all necessary components for combat (AttackCapability, AttackState, AttackPhase, etc.)
5. Run `cargo test` to ensure all existing Guard tests pass

If everything matches, no code changes needed — just confirm verification.

## Technical Context

### Files to Verify (all under `artifacts/developer/src/`)

1. **`game/types/objects.rs` (line 232)** — `ObjectEnum::SyndicateGuard` ObjectType definition:
   - name: "Guard", size: (36,36), destructible: true, sight_range: 5, groupable: true
   - All values match design spec (`syndicate_objects.md` lines 183-189). CORRECT.
   - Tests at lines 1108-1127 verify groupable, is_unit, is_not_structure, control_cost.

2. **`game/units/types/unit_data.rs` (line 214)** — `guard_type_data()`:
   - faction: TheSyndicate, silhouette: 36x36, max_hp: 80, point_armor: 1, full_armor: 1, unit_base: HeavyInfantry
   - All values match design spec. CORRECT.
   - Tests at lines 619-628 verify all fields.

3. **`game/units/types/unit_data.rs` (line 227)** — `guard_attack_data()`:
   - FullyConnected, Ranged subtype, Ground, SingleTarget, damage=6, range=3, min_range=0, aim=2, firing=1, cooldown=1, reload=4
   - All values match design spec (`syndicate_objects.md` lines 199-209). CORRECT.
   - Tests at lines 631-677 verify all attack fields and timings.

4. **`game/units/types/unit_data.rs` (lines 246-252)** — Constants:
   - `GUARD_CONTROL_COST: u32 = 1` — matches spec
   - `GUARD_TUNNEL_SPACE_COST: u32 = 2` — matches spec (line 188)
   - `GUARD_RUGGED_BONUS: f32 = 0.5` — **POTENTIAL ISSUE**: Design spec says HeavyInfantry "does not receive a defensive bonus" on rugged terrain (units.md line 118). Only LightInfantry has `RuggedTerrainDefenseBonus` (units.md line 114). However, this constant exists and is spawned on Guard. The developer should verify whether this is intentional game-design divergence or a bug. If the design spec is authoritative, `GUARD_RUGGED_BONUS` should be 0.0 and `RuggedTerrainDefenseBonus` should either not be spawned or be set to 0.0 on Guard. **Recommend flagging but not changing** — the design doc for the Guard unit in `syndicate_objects.md` does not mention RuggedTerrainDefenseBonus at all, and since this task says "if everything matches, no code changes needed", leave as-is unless designer clarifies.

5. **`game/units/types/movement.rs` (line 312)** — `UnitBaseEnum::HeavyInfantry` base data:
   - domain: Ground, has_turret: false, directional_armor: false, rugged_terrain: true, crushable: false, can_turn_in_place: true, can_reverse: false, movement_model: TurnRate
   - All values match design spec (units.md lines 117-127). CORRECT.

6. **`game/utils.rs` (line 590)** — `spawn_syndicate_guard()`:
   - Uses `guard_type_data()` and `guard_attack_data()` for all values
   - Spawns: Unit, ObjectInstance(SyndicateGuard, 80hp), Selectable, GridPosition, UnitBaseEnum, MovementSpeed, RotationSpeed, Velocity
   - Spawns combat: AttackCapability (with correct range=3, all timings converted via `frames_to_seconds()`), AttackState
   - Spawns command/behavior: UnitCommand::Idle, TurnRateMovementParams (turn_rate=180deg*16fps, accel/decel=MAX, speed=5*16/64=1.25), CommandQueue, BaseCommandState, BaseBehaviorState
   - Spawns channels: LocomotionChannel, OrientationChannel, BaseAttackChannel (no turret channels — correct for HeavyInfantry)
   - Spawns Armor, Silhouette, SightRange(5)
   - Does NOT spawn AgentCarryState (correct — Guard is pure combat)
   - HeavyInfantry movement params: MaxSpeed=5 SU/frame → 5*16/64=1.25 GU/sec. TurnRate=180deg/frame → 180*16=2880deg/sec. Accel/Decel=infinite (f32::MAX). All CORRECT.

7. **`game/types/structures.rs` (line 278)** — `HeadquartersState::production_cost(SyndicateGuard)`:
   - space_crystals: 125, build_frames: 120 — matches spec (line 151). CORRECT.
   - Test at line 1744 verifies these values.

8. **`ui/command_panel.rs` (line 107)** — HQ menu grid slot (0,1) = W hotkey:
   - `CommandButtonAction::HqTrain(ObjectEnum::SyndicateGuard)` — CORRECT.
   - Label at line 2232: "[W] Guard\n125 SC" — CORRECT.
   - Test at line 3534 verifies this slot.

### Verification Procedure

1. **Run all Guard-related tests**: `cargo test guard --manifest-path artifacts/developer/Cargo.toml`
2. **Run HeavyInfantry tests**: `cargo test heavy_infantry --manifest-path artifacts/developer/Cargo.toml`
3. **Run full test suite**: `cargo test --manifest-path artifacts/developer/Cargo.toml`
4. **Cross-reference each value** against the design spec in `artifacts/designer/design/syndicate_objects.md` lines 175-210 and `artifacts/designer/design/units.md` lines 117-127.
5. **Note**: The design spec (syndicate_objects.md line 211) says Guard uses `BasicCombatUnitInterfaceState`, but the task says this is handled by a separate feature request — so the Guard currently uses `ObjectInterfaceState::Default` which is correct for now.

### Potential Issue to Flag (do NOT fix)

The `GUARD_RUGGED_BONUS = 0.5` constant and `RuggedTerrainDefenseBonus(0.5)` component spawn may conflict with the design spec for HeavyInfantry, which states it "does not receive a defensive bonus" on rugged terrain (only LightInfantry does). This should be flagged as a design question but NOT changed in this verification task.

## Dependencies

None — this is a standalone verification task. All referenced code already exists and has tests.
