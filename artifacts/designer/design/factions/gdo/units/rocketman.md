# Rocketman
GDO missile infantry, trained at the Barracks once the player owns an Arsenal. Fires missiles at both ground and air targets. Normally its missiles fly to where the target was when it fired, so they can be dodged; its Target Lock ability lets it lock onto a single unit and fire homing missiles that cannot miss.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Barracks
## TechPrerequisite - player owns at least one Arsenal (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

## AttackAttributes (normal attack):
- AttackType: DoublyDisjointed (the missile travels to where the target was at the end of Aiming; targets can dodge; can also be fired at the ground)
- TargetDomain: Universal (ground and air)
- MinRange: yes (user, 2026-11-25): enemies closer than MinRange can't be targeted; value: placeholder below (systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange)
- Damage, Range, ProjectileSpeed, timings: placeholders below

## Target Lock (targeted ability)
The player orders Target Lock on one enemy unit.

**Locking:** the Rocketman aims at the target continuously for LockOnDuration. The target may move; the Rocketman turns to keep aiming at it. Locking succeeds only if the target stays within LockRange and visible for the whole duration.

**Locked:** once locked, the Rocketman fires at the locked target from then on, and every missile it fires at that target is **homing** (HeadDisjointed behavior: tracks the target, cannot miss). Locked missiles can be fired from anywhere within LockRange.

**LockRange** is a bit longer than the normal missile Range — so a lock both reaches further and is harder to shake off than a normal attack.

**No minimum range when locked** (user, 2026-11-25): locked homing missiles can be fired at **any distance** up to LockRange, even inside the normal attack's MinRange.

**The lock (or the locking in progress) breaks when:**
- the target leaves LockRange,
- the Rocketman loses vision of the target,
- the Rocketman is given **any** order (move, stop, attack something else, etc.),
- or the target dies.

**Target Lock can target structures** (user, 2026-11-26) as well as units.

**Placeholder stats** (designer, 2026-11-26, user's standing permission; tune in the numbers pass):
- UnitBase LightInfantry; Silhouette 24x24 space units; MaxHP 45; PointArmor 1, FullArmor 1; SightRange 6 grid units; Groupable true; UnitControlCost 1.
- Movement as the Peacekeeper, but MaxSpeed **3 space units/frame** (heavier kit).
- Normal attack: Damage **20**; Range **6 grid units**; MinRange **2 grid units**; ProjectileSpeed **16 su/frame**; AimDuration 6, FiringDuration 2, CooldownDuration 16, ReloadDuration 8 frames (one missile every 32 frames, 2 s: 10 damage per second before armor).
- Target Lock: **LockOnDuration 48 frames (3 s)**; **LockRange 7 grid units**; locked missiles use the same Damage and timings, homing.
- Cost: Barracks, **100 Space Crystals, 160 frames (10 s)**; hotkey **W** at the Barracks (as listed).
- Panel: BasicCombatUnitInterfaceState, plus **R: Target Lock** (targeted: left-click an enemy unit or structure).
