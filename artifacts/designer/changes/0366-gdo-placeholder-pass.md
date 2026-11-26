# 0366 — GDO placeholder pass: every designed GDO building and unit gets numbers

- **Date**: 2026-11-27T02:30:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/gdo/structures/{deployment_center,factory,airfield,advanced_mechanics,propulsion_labs,industrial_powerplant,radar_station,defense_system,missile_turret,outpost,bunker,first_aid_tent,barracks}.md`;
  - `factions/gdo/units/{artillery_rig,raptor,twister,hornet,dragonfly,drone_operator,prospector,rocketman}.md`.

## What changed and why
Under the user's standing permission, every designed GDO object has placeholder numbers, marked as such. The only GDO TBD left is the **Supply Chopper's transport mechanics**, a design question.

**Buildings** (cost / time at the Deployment Center; Power):

| Building | Cost / time | Power | Other |
|---|---|---|---|
| Factory | 250 / 480 | −30 | 3x3 |
| Airfield | 250 / 480 | −30 | landing and take-off 16 f |
| Advanced Mechanics | 200 / 400 | −20 | |
| Propulsion Labs | 200 / 400 | −20 | |
| Industrial Powerplant | 250 / 480 | **+60** | |
| Radar Station | 150 / 320 | −20 | Scan Q: radius 6, 8 s, cooldown 60 s |
| Defense System | 150 / 320 | −20 | |
| Supply Tower | 150 / 320 | — | |
| Missile Turret (from the Defense System) | 100 / 240 | −10 | 1x1, homing Universal, Damage 18, range 6 |
| Outpost | — | −10 | 3x3, BuildRadiusExtension 6 |
| Bunker (Field Engineer) | 100 / 320 | — | 2x2, 2/8, one door, A Eject All |
| First Aid Tent (Field Engineer) | 75 / 240 | — | radius 3, +2 HP/s, no stacking |

**Units** (cost / time; Unit Control):

| Unit | Cost / time | UC | Key numbers |
|---|---|---|---|
| Raptor | 100 / 240 | 2 | very fast at 10 su/f; homing, 14 dmg every 32 f, range 6 |
| Twister | 125 / 320 | 2 | 8 su/f; AA machine gun 4 dmg every 4 f |
| Prospector | 300 / 480 | 2 | deploys in 64 f |
| Artillery Rig | 250 / 480 | 3 | deploy and undeploy 48 f; 300 AoE over 1 GU; range 12, MinRange 3; Firing 16 f (the dodge window) |
| Hornet | 150 / 320 | 2 | 5 homing missiles, 30 dmg; rearm 32 f each |
| Dragonfly | 175 / 320 | 2 | 1 missile, 90 dmg; rearm 64 f |
| Drone Operator | 100 / 160 | — | drone 50 SC / 160 f; leash 8; battery 20 s; 150 AoE |

**Twister mines:** 25 SC / 80 f each, laid in 16 f; trigger radius 32 su, delay 8 f; 200 AoE over 64 su; mine HP 40.

**Hotkeys** (placeholders):
- R for deploy and launch abilities;
- F for Return to Base and the buy or build abilities;
- the Factory panel: Q, W, E, R.

**Designer's readings that are really rules**, flagged in the docs "to confirm" and being asked:
- the Missile Turret, Hornet and Dragonfly missiles **home** (can't miss);
- the drone's explosion is **AoE**;
- the Prospector follows the **vehicle deploy model** (can't cancel);
- units in a Bunker see with its sight;
- the First Aid Tent heals **own GDO infantry only**.

## Implementation impact
- Everything listed can now be built on placeholders.
- None of it is in the M5 scope unless you choose.

## Verification
—
