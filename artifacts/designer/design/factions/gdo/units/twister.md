# Twister
GDO vehicle produced at the Factory. A little slower than the Raptor. Mounts a machine gun on a fully rotating turret for anti-air defense, and lays hidden mines.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Factory
## UnitBase - WheeledVehicle (a little slower than the Raptor)

## TurretAttributes:
- TurnAngle: 360° (full rotation)
- TurnRate: TBD

## AttackAttributes:
- AttackType: FullyConnected (Ranged) — machine-gun type weapon
- TargetDomain: **Air** (anti-air only)
- Damage, Range, timings: TBD

## Mines
- **Stock:** the Twister carries up to **3 mines** and **starts with 3** when produced. Further mines are **purchased one at a time** (a "Buy Mine" ability, like the Drone Operator's Build Drone). Mine cost and purchase time TBD.
- **Lay Mine (targeted ability):** the player picks a location — **anywhere** (not limited to the GDO build area). The Twister drives there, **stops temporarily** to lay the mine, then is free to move again.
- **Hidden:** mines are **underground and hidden** from enemies (detection mechanics not yet designed).
- **Trigger:** each mine has an **activation radius**. When an enemy unit — **infantry or vehicle** — enters it, the mine **explodes after a short delay**.

- **Friendly fire:** the explosion **damages friendly units** caught in it, but friendly units **do not trigger** the mine — only enemies do.
- **Air units** neither trigger mines nor take damage from them (the explosion is ground-only).
- **Destroyable:** enemies can attack and kill mines (once they can see them — detection not yet designed).

TBD: mine cost/purchase time, laying time, activation radius, delay, explosion damage and radius, mine HP, stats, cost, build time, ObjectInterfaceState.
