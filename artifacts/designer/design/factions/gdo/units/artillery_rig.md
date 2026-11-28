# Artillery Rig
GDO artillery vehicle produced at the Factory; requires Advanced Mechanics. Has two modes: it cannot attack while mobile and must deploy to fire long-range area artillery. (User reference: like the StarCraft 1 Siege Tank, but with no attack in its mobile mode.)

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Factory
## TechPrerequisite - player owns at least one Advanced Mechanics (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)
## UnitBase - TrackedVehicle

## Modes
- **Mobile:** moves normally; **has no attack**.
- **Deployed:** cannot move; fires its artillery attack.
- **Deploy / Undeploy:** switching modes takes time (placeholder: 48 frames each, below); the unit can neither move nor attack while switching.
- **Two unit types**: **Artillery Rig** and **Artillery Rig (Deployed)**, with separate SelectionGroups.
- **Uses Unit Modes, Deploy archetype** (systems/mechanics/modes.md; user, 2026-11-25): an order, queueable, not cancellable, no reversing mid-way, the last order given while switching held. It becomes Artillery Rig (Deployed) when deploying finishes. Undeploy is available.

## AttackAttributes (Deployed only):
- TargetDomain: Ground
- TargetType: AoE — **friendly fire**: the blast damages friendly units too
- MinRange: yes (enemies closer than MinRange cannot be targeted; placeholder below)
- Range: long (placeholder below)
- Vision: like every attacker, it can fire at any enemy visible to its owner (systems/combat.md > ValidTarget) — it does not need its own vision of the target.
- AttackType: **TailDisjointed** — the target location is locked at the end of Aiming; the blast lands there at the end of Firing (no tracking projectile). Units that move out of the locked area in time dodge it. The Artillery Rig stays locked during Firing. Can also be fired at a ground location.
- Damage, AoE radius, timings: placeholders below

## TurretAttributes:
- TurnAngle (Deployed): 360° (full rotation)
- TurnRate: placeholder below

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 64x48 space units; MaxHP 180; PointArmor 3, FullArmor 3; SightRange 6 grid units; Groupable true; UnitControlCost 3.
- SpeedTurnRadiusMovement: MaxSpeed 4 su/frame; Acceleration 0.4; Deceleration 1; TrackWidth 36 su; ReverseMaxSpeed 2 (Q0153; spins about 12.7° a frame).
- **Deploy 48 frames (3 s), Undeploy 48 frames.**
- Deployed attack: Damage **300** total over the circle (AoEDamage); AoERadius **64 su (1 GU)**; Range **12 grid units**; MinRange **3 grid units**; AimDuration 16, FiringDuration **16** (the shell's flight: the gap in which targets can leave the locked area), CooldownDuration 8, ReloadDuration 64 frames (one shell per 104 frames, 6.5 s).
- Turret TurnRate 3°/frame.
- Cost: Factory, 250 Space Crystals **+ 5 Supplies** (user, 2026-11-26; Q0169), 480 frames.
- Panel: BasicCombatUnitInterfaceState plus **R: Deploy** (mobile) / **R: Undeploy** (deployed); the mobile Rig has no attack commands.
