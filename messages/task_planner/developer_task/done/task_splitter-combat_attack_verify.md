# combat-attack-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-combat-attack-system.md

## Task

**Verification-only task** — the combat and attack system is already fully implemented. Verify that all components, systems, and calculations match the design spec in `artifacts/designer/design/combat.md` and ensure tests pass.

### What already exists (do NOT re-implement):

1. **AttackAttributes** (`combat/types.rs`): `AttackCapability` component with all fields (damage, range, min_range, aim_time, fire_time, cooldown_time, reload_time, attack_type, target_domain, target_type, aoe_radius)
2. **AttackType variants** (`combat/types.rs`): FullyConnected (Ranged/Melee subtypes), HeadDisjointed, TailDisjointed, DoublyDisjointed — with projectile_speed, effect_radius, projectile_visual
3. **AttackPhase** (`combat/types.rs`): None/Aiming/Firing/Cooldown/Reloading with is_interruptible() and base_action_constraints()
4. **AttackTarget** (`combat/types.rs`): UnitTarget(Entity) / LocationTarget(Vec3)
5. **AttackState** (`combat/types.rs`): phase + time_in_phase + current_target tracking
6. **Armor** (`combat/types.rs`): point_armor, full_armor, directional_armor
7. **Silhouette** (`combat/types.rs`): width, height for AoE overlap
8. **attack_command_system** (`combat/systems/core.rs`): Accepts new targets respecting interruptibility
9. **attack_phase_system** (`combat/systems/core.rs`): Full Aiming→Firing→Cooldown→Reloading cycle with all 4 attack type variants (FullyConnected instant hit, TailDisjointed projectile, HeadDisjointed AoE, DoublyDisjointed projectile+AoE), elevation-adjusted range, melee exemption
10. **apply_damage_system** (`combat/systems/core.rs`): SingleTarget (point armor) and AoE (overlap-based with circle_rect_overlap_area) damage calculation with directional armor
11. **directional_armor_multiplier** (`combat/systems/core.rs`): Front/side/rear multipliers
12. **Domain compatibility** (`combat/utils.rs`): is_domain_compatible(), is_valid_target()
13. **Projectile systems** (`combat/systems/core.rs`): projectile_movement_system, projectile_impact_system
14. **Visual effects**: attack_line_decay_system, explosion_effect_system, target_highlight_decay_system, CombatAssetCache
15. **Auto-targeting**: turret_autonomous_scanning_system, base_auto_target_system, idle_leash_system

### Verification steps:

1. Run `cargo test` — all combat tests should pass
2. Confirm AttackType enum derived properties match design doc (CanMiss, CanTargetGround, RequiresProjectileSpeed)
3. Confirm phase constraints match design doc tables (Aiming: move=false turn=true for UnitBase; Firing/Cooldown: both false; Reloading: both true; Turret source: always both true)
4. Confirm AoE damage formula: damage_share = Damage x (overlap/AoE_area), effective_armor = FullArmor x (overlap/unit_area)
5. Confirm directional armor uses dot product of negated attack direction vs facing
