# Missile platform (name TBD)

(user, 2026-11-27) The Cults' **long-range, mobile, semi-ballistic missile platform**, built at the Chop Shop. It is a **shoot-and-scoot** unit: a **long aim**, a **long missile flight**, a **short cooldown** after launch (so it can drive off at once) and a **long reload** (spent moving). Unlike the GDO's Artillery Rig it **never deploys**.

## Entity Type - Unit
## Faction - TheCults
## UnitBase - TrackedVehicle (turns in place)
## Produced at - Chop Shop (consumes **3 Recruits**; Unit Control cost **3**)

## TurretAttributes:
- TurnAngle: **minimal** (user, 2026-11-27): the launcher barely turns, so to aim at most targets the vehicle must **stop and turn its hull** to face them. Placeholder: **20° total**.

## AttackAttributes:
- AttackType: **DoublyDisjointed** (user, 2026-11-27: **no homing**): the missile flies to where the target was at the end of Aiming, so targets can dodge; it can also be fired at the ground (CanTargetGround, Attack Ground).
- TargetType: **area damage** (AoE, systems/combat.md > AoEDamage; friendly fire applies, as for every area hit).
- **Can't be shot down** in flight (user, 2026-11-27; unless something to shoot down projectiles is introduced later).
- Phase profile (user): **long AimDuration**, **slow missile (long flight)**, **short CooldownDuration**, **long ReloadDuration**.
- TargetDomain: **Ground** only (user, 2026-11-27).
- MinRange: **yes** (user, 2026-11-27): enemies closer than MinRange can't be targeted (systems/controls/behaviors.md > AttackingObject > Target inside MinRange); value: placeholder to follow.

TBD: name (user is choosing; theme: righteous help from the sky); whether it reveals itself on launch beyond the shared reveal-on-damage rule; numbers (placeholders to follow).
