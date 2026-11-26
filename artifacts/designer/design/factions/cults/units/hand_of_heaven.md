# Hand of Heaven

(Name: user, 2026-11-27: righteous help from the sky.)

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
- MinRange: **yes** (user, 2026-11-27): enemies closer than MinRange can't be targeted (systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange); value: placeholder to follow.

Reveal: the shared rule applies (core/vision.md > Attacks from outside vision): it is revealed to those it damages when the missile lands.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 64x40 space units (length x width); MaxHP 200; PointArmor 2, FullArmor 2; SightRange 7 grid units (it needs spotters for its full range); Groupable true.
- SpeedTurnRadiusMovement: MaxSpeed 5 su/frame; Acceleration 0.5 su/frame²; Deceleration 1 su/frame²; SpeedToTurnRadiusRatio 8 (turns in place when stopped).
- Turret: TurnAngle **20°** total; TurnRate 4°/frame.
- Attack: Damage **600** total over the circle (AoEDamage; a 3x3 building wholly inside takes it all, a lone 24x24 infantry about 12, so it is a siege weapon, poor against spread or moving units); AoERadius **96 su (1.5 GU)**; Range **14 grid units**; MinRange **4 grid units**; ProjectileSpeed **8 su/frame** (a full-range shot flies about 7 s); AimDuration **48** frames (3 s); FiringDuration 2; CooldownDuration **4**; ReloadDuration **96** (6 s): one missile per 150 frames, about 9.4 s.
- Cost: Chop Shop, 3 Recruits, **150 Space Crystals, 480 frames (30 s)**.
- ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **CanTargetGround** (Attack Ground), as every attack that can miss.
