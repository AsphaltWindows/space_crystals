# combat/

Combat system including attacks, turrets, projectiles, and combat behaviors.

## Structure

- **types.rs** — AttackCapability, AttackType, AttackState, AttackPhase, AttackTarget, AttackSourceEnum, DamageEvent, Turret, TurretVisual, Projectile, ProjectileVisual, ExplosionEffect, AttackMoveOrigin, PatrolEngaged, leash constants
- **utils.rs** — Turret creation, projectile spawning, explosion effects, AOE damage helpers, targeting validation (is_domain_compatible, is_valid_target)
- **systems/** — Combat system implementations (core attack cycle + combat behaviors)
- **projectile.rs** — Projectile movement, impact detection, and explosion animation
- **turret.rs** — Turret aiming, rotation, and visual synchronization
