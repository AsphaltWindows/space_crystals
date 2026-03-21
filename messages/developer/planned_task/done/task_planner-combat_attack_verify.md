# combat-attack-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-combat-attack-system.md

## Task

**Verification-only task** — the combat and attack system is already fully implemented. Verify that all components, systems, and calculations match the design spec in `artifacts/designer/design/combat.md` and ensure tests pass.

### Verification steps:

1. Run `cargo test` — all combat tests should pass
2. Confirm AttackType enum derived properties match design doc (CanMiss, CanTargetGround, RequiresProjectileSpeed)
3. Confirm phase constraints match design doc tables (Aiming: move=false turn=true for UnitBase; Firing/Cooldown: both false; Reloading: both true; Turret source: always both true)
4. Confirm AoE damage formula: damage_share = Damage x (overlap/AoE_area), effective_armor = FullArmor x (overlap/unit_area)
5. Confirm directional armor uses dot product of negated attack direction vs facing

## Technical Context

### Files to verify (read, do NOT rewrite):

1. **`artifacts/developer/src/game/combat/types.rs`** — Core combat types
   - `AttackType` enum (line 18): FullyConnected{subtype}, TailDisjointed{projectile_speed, projectile_visual}, HeadDisjointed{effect_radius}, DoublyDisjointed{projectile_speed, projectile_visual, effect_radius}
   - `AttackCapability` component (line 36): damage, range, min_range, aim_time, fire_time, cooldown_time, reload_time, attack_type, target_domain, target_type, aoe_radius
   - `AttackPhase` enum (line 94): None, Aiming, Firing, Cooldown, Reloading
   - `AttackPhase::is_interruptible()` (line 114): Aiming+Reloading+None = true; Firing+Cooldown = false — matches design doc
   - `AttackPhase::base_action_constraints()` (line 120): UnitBase source: Aiming(move=F,turn=T), Firing/Cooldown(move=F,turn=F), Reloading(move=T,turn=T); Turret source: always(move=T,turn=T) — matches design doc
   - `AttackState` component (line 137): phase, time_in_phase, current_target
   - `Armor` component (line 182): point_armor, full_armor, directional_armor
   - `Silhouette` component (line 193): width, height for AoE overlap
   - Directional armor constants (lines 173-179): FRONT_MULTIPLIER=1.5, REAR_MULTIPLIER=0.5, thresholds at +/-0.5 dot product
   - Existing tests: lines 400-622 (interruptibility, phase constraints, asset cache, explosion animation)

2. **`artifacts/developer/src/shared/types.rs`** — AttackTypeEnum derived properties
   - `AttackTypeEnum` impl (line 380): `can_miss()`, `can_target_ground()`, `requires_projectile_speed()`, `allows_location_target()`
   - FullyConnected/HeadDisjointed: can_miss=false, can_target_ground=false
   - TailDisjointed/DoublyDisjointed: can_miss=true, can_target_ground=true
   - HeadDisjointed/DoublyDisjointed: requires_projectile_speed=true
   - TailDisjointed/DoublyDisjointed: allows_location_target=true
   - All match the design doc table exactly

3. **`artifacts/developer/src/game/combat/systems/core.rs`** — Core combat systems
   - `attack_command_system` (line 16): Accepts new targets only if interruptible — correct
   - `attack_phase_system` (line 38): Full Aiming→Firing→Cooldown→Reloading cycle with:
     - FullyConnected: instant hit via DamageEvent::SingleTarget + spawn_attack_line (line 119)
     - TailDisjointed: spawn_projectile with no effect_radius (line 139) — homing
     - HeadDisjointed: apply_aoe_damage + spawn zero-travel projectile for visual (line 157) — instant AoE
     - DoublyDisjointed: spawn_projectile with effect_radius (line 186) — traveling AoE projectile
     - Elevation-adjusted range via `elevation_modifier()` (line 78), melee exempt — matches design
   - `directional_armor_multiplier()` (line 454): Uses dot product of NEGATED attack direction vs target forward — matches design doc requirement
   - `apply_damage_system` (line 474):
     - SingleTarget: damage - (point_armor * directional_multiplier), floored at 0 — matches design
     - AoE: damage_share = damage * (overlap/aoe_area), effective_armor = full_armor * (overlap/unit_area), directional modifier applied to effective_armor — matches design doc formula exactly
   - `base_auto_target_system` (line 355): Auto-targets for Idle/HoldPosition/AttackMove, records IdleOrigin
   - `idle_leash_system` (line 430): Disengage at IDLE_LEASH_DISTANCE (4.0 GU)
   - `turret_autonomous_scanning_system` (line 265): Priority: threatening > least rotation > closest — matches design
   - `remove_dead_entities_system` (line 554): Despawn dead entities, decrement unit cap, handle ExtractionPlate cleanup

4. **`artifacts/developer/src/game/combat/utils.rs`** — Utility functions
   - `is_domain_compatible()` (line 262): Ground→Ground+Underground, Air→Air, Universal→all — matches design
   - `is_valid_target()` (line 278): destructible + visible + domain-compatible — matches ValidTarget spec
   - `circle_rect_overlap_area()` (line 191): Approximated AoE-silhouette overlap
   - `spawn_projectile()` (line 76), `spawn_explosion_effect()` (line 116), `spawn_attack_line()` (line 137): Visual effect spawners using CombatAssetCache

5. **`artifacts/developer/src/game/combat/projectile.rs`** — Projectile lifecycle
   - `projectile_movement_system` (line 8): Linear interpolation toward target_position
   - `projectile_impact_system` (line 30): On arrival (distance < 0.2): AoE projectiles insert DamageEvent::AreaOfEffect on all enemies in radius; single-target find closest enemy near impact
   - `explosion_effect_system` (line 139): Scale animation using base_scale * (1 + progress*2)
   - `attack_line_decay_system` (line 122), `target_highlight_decay_system` (line 101): Timed despawn

6. **`artifacts/developer/src/game/combat/mod.rs`** — Plugin registration
   - `CombatPlugin`: All combat systems in DiagCategory::Combat
   - `TurretPlugin`: Turret aiming/rotation in DiagCategory::Turrets
   - `ProjectilePlugin`: Projectile movement/impact/visuals in DiagCategory::Projectiles
   - `init_combat_asset_cache` runs on OnEnter(AppState::InGame)

7. **`artifacts/developer/src/game/combat/turret.rs`** — Turret rotation systems (turret_aiming_system, turret_rotation_system, update_turret_visual_system)

### Design doc verification checklist (combat.md):

| Design Doc Property | Code Location | Status |
|---|---|---|
| AttackType derived CanMiss | shared/types.rs:382 | FullyConnected/HeadDisjointed=false, Tail/Doubly=true ✓ |
| AttackType derived CanTargetGround | shared/types.rs:387 | Same pattern as CanMiss ✓ |
| AttackType RequiresProjectileSpeed | shared/types.rs:392 | HeadDisjointed/DoublyDisjointed=true ✓ |
| Aiming interruptible=true | combat/types.rs:115 | ✓ |
| Firing interruptible=false | combat/types.rs:115 | ✓ |
| Cooldown interruptible=false | combat/types.rs:115 | ✓ |
| Reloading interruptible=true | combat/types.rs:115 | ✓ |
| UnitBase Aiming: move=F, turn=T | combat/types.rs:128 | ✓ |
| UnitBase Firing/Cooldown: both F | combat/types.rs:129 | ✓ |
| UnitBase Reloading: both T | combat/types.rs:130 | ✓ |
| Turret source: always both T | combat/types.rs:122-123 | ✓ |
| AoE damage_share formula | combat/systems/core.rs:520 | damage * (overlap/aoe_area) ✓ |
| AoE effective_armor formula | combat/systems/core.rs:523 | full_armor * (overlap/unit_area) ✓ |
| Directional armor negated attack dir | combat/systems/core.rs:462 | (-attack_dir_2d).dot(facing_2d) ✓ |
| Domain compatibility | combat/utils.rs:262-268 | Ground→Ground+Underground, Air→Air, Universal→all ✓ |

### How to verify:

1. Run `cargo test` from `artifacts/developer/` — expect all tests to pass
2. Read each file listed above and confirm the implementation matches the design doc
3. If any test fails, investigate and fix
4. If any logic diverges from the design doc, fix the code to match
5. Do NOT refactor or restructure — this is verification only

### Existing test coverage (comprehensive):
- `combat/types.rs` tests: interruptibility (5), phase constraints UnitBase (5), phase constraints Turret (5), consistency checks (3), CombatAssetCache (3), explosion animation (4) = ~25 tests
- `combat/utils.rs` tests: AttackTypeEnum derived (12), domain compatibility (9), ValidTarget (7), AttackCapability (1), AttackTarget (2), AttackState (3), AttackSource (1), melee subtype (7), armor (2), silhouette (1), directional armor constants+multiplier (6), DamageEvent (2), SingleTarget damage (4), circle-rect overlap (6), AoE damage (2), attack line scale (2), sphere projectile scale (1), explosion scale (2) = ~70 tests
- `combat/systems/core.rs` tests: can_threaten stub (4), compute_relative_turret_angle (2+) tests

## Dependencies

None — this is a standalone verification task. All combat systems are already implemented and registered in the plugin. No new code needs to be written unless verification reveals a discrepancy.
