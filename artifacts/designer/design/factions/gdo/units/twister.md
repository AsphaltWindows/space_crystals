# Twister
GDO vehicle produced at the Factory. A little slower than the Raptor. Mounts a machine gun on a fully rotating turret for anti-air defense, and lays hidden mines.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Factory
## UnitBase - WheeledVehicle (a little slower than the Raptor)

## TurretAttributes:
- TurnAngle: 360° (full rotation)
- TurnRate: placeholder below

## AttackAttributes:
- AttackType: FullyConnected (Ranged) — machine-gun type weapon
- TargetDomain: **Air** (anti-air only)
- Damage, Range, timings: placeholders below

## Mines
- **Stock:** the Twister carries up to **3 mines** and **starts with 3** when produced. Further mines are **purchased one at a time** (one after another; presses queue up to the free stock, Q0160) (a "Buy Mine" ability, like the Drone Operator's Build Drone). Mine cost and purchase time: placeholders below.
- **Lay Mine (targeted ability):** the player picks a location — **anywhere** (not limited to the GDO build area). The Twister drives there, **stops temporarily** to lay the mine, then is free to move again.
- **Hidden:** mines are **underground and hidden** from enemies (detection mechanics not yet designed).
- **What a mine is** (user, 2026-11-26; Q0160): **closer to a permanently deployed unit** than to a structure. It is **not a structure** (it doesn't count for the Defeat Condition, core/game_rules.md) and it **can be selected** by its owner, like a unit (others only once they can see it; detection TBD). Designer's placeholders: Unit Control 0, Groupable true, no commands in its panel yet.
- **Trigger:** each mine has an **activation radius**. When an enemy unit — **infantry or vehicle** — enters it, the mine **explodes after a short delay**. **Hover units don't set it off** (user, 2026-11-26; Q0160): HoverVehicles and HoverCraft pass over safely, as over sludge; the blast still damages a HoverVehicle caught in it.
- **Destroyed by damage, it doesn't explode** (user, 2026-11-26; Q0160): it is simply removed.

- **Friendly fire:** the explosion **damages friendly units** caught in it, but friendly units **do not trigger** the mine — only enemies do.
- **Air units** neither trigger mines nor take damage from them (the explosion is ground-only).
- **Destroyable:** enemies can attack and kill mines (once they can see them — detection not yet designed).

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 52x36 space units; MaxHP 140; PointArmor 2, FullArmor 2; SightRange 6 grid units; Groupable true; UnitControlCost 2.
- FixedTurnRadiusMovement: MinimumTurnRadius 80 su; ForwardMaxSpeed **8 su/frame**; ForwardAcceleration 1; ReverseMaxSpeed 3; ReverseAcceleration 0.5; Deceleration 1.
- Turret TurnRate 12°/frame. Machine gun: Damage 4; Range 6 grid units; Aim 1, Firing 1, Cooldown 1, Reload 1 frames (16 damage per second before armor).
- Mines: **Buy Mine** 25 Space Crystals, 80 frames (5 s); purchases **queue up to the free stock** (user, 2026-11-26; Q0160): with 0 of 3 carried, three presses buy three mines in 15 s, one after another, each paid at the press; **laying** 16 frames (1 s, stopped); activation radius **32 su (0.5 GU)**; delay **8 frames**; explosion Damage **200** total, AoERadius **64 su**; mine HP 40; **mine SightRange 1 grid unit, PointArmor 12, FullArmor 12** (user, 2026-11-27; Q0182); mine Size: a 16x16 su object, hidden underground.
- Cost: Factory, 125 Space Crystals **+ 1 Supply** (user, 2026-11-26), 320 frames.
- Panel: BasicCombatUnitInterfaceState plus **R: Lay Mine** (targeted) and **F: Buy Mine**.
