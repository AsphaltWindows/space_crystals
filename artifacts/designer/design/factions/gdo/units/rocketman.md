# Rocketman
GDO missile infantry, trained at the Barracks once the player owns an Arsenal. Fires missiles at both ground and air targets. Normally its missiles fly to where the target was when it fired, so they can be dodged; its Target Lock ability lets it lock onto a single unit and fire homing missiles that cannot miss.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Barracks
## TechPrerequisite - player owns at least one Arsenal (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

## AttackAttributes (normal attack):
- AttackType: DoublyDisjointed (the missile travels to where the target was at the end of Aiming; targets can dodge; can also be fired at the ground)
- TargetDomain: Universal (ground and air)
- MinRange: yes (user, 2026-11-25): enemies closer than MinRange can't be targeted; value TBD (systems/controls/behaviors.md > AttackingObject > Target inside MinRange)
- Damage, Range, ProjectileSpeed, timings: TBD

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

TBD: LockOnDuration, LockRange, hotkey, whether Target Lock can target structures, stats (Silhouette, HP, armor, speed, UnitBase, cost, build time, Unit Control cost), ObjectInterfaceState.
