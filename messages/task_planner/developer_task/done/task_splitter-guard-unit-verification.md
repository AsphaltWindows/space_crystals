# guard-unit-verification

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
