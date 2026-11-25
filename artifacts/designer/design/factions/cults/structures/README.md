# Cults: Structures

## Tech Tree Overview

Complete list of Cults structures designed so far. Every Cults structure is built by Recruits (see Cults Building Mechanics). "Requires" is the TechPrerequisite (see core/tech_and_research.md > Tech Prerequisites).

The Cults tech tree splits into two branches above a shared base. A player may build from both branches in the same game — the branches are simply two independent prerequisite chains, with no lock-out and no extra cost for pursuing both.

- **Tech/Weaponry branch**: weapons, combat training, weapon upgrades, vehicles, aircraft, and vehicle/air upgrades. Buildings use scavenged/makeshift industrial names.
- **Mystical/Human Enhancement branch**: crystal-based augmentation of infantry flesh, research that improves augmentation, and the Acolyte (spell caster).

"Tier" is an informal, descriptive label — not a game rule; all gating is done through TechPrerequisites. Each tier is anchored by its major building, the one that produces units; support buildings (upgrades, unlocks) share the tier of the producer they sit beside, even when they require it.

| Structure | Branch | Tier | Requires | Role |
|---|---|---|---|---|
| RecruitmentHall | Shared base | — | — | Starting structure; produces Recruits from Recruitable terrain; provides Unit Control |
| Storage | Shared base | — | — | Space Crystals drop-off |
| Armory | Tech/Weaponry | 1 | Storage | Trains Recruits into Soldiers, and Gunners (Gunner requires a Gunworks) |
| Gunworks | Tech/Weaponry | 2 | Armory | Unlocks Gunner training at the Armory; researches weapon upgrades (staffed by Recruits) |
| Garrison | Tech/Weaponry (defense) | 2 | Armory | Defensive structure; up to 4 Recruits/Soldiers/Gunners/Acolytes garrison inside; Recruits and Acolytes fire as Soldiers; Soldiers and Gunners get +1 range |
| Chop Shop | Tech/Weaponry | 2 | Armory | Vehicle production (vehicles consume multiple Recruits) |
| Machine Shop | Tech/Weaponry | 2 | Chop Shop | Researches vehicle and air upgrades (staffed by Recruits) |
| Airstrip | Tech/Weaponry | 3 | Chop Shop | Aircraft production (from Recruits) |
| Scrap Hangar | Tech/Weaponry | 3 | Airstrip | Unlocks advanced aircraft at the Airstrip (a large paratrooper plane, an airship/zeppelin) |
| Altar | Mystical/Human Enhancement | 1 | Storage | Applies Augmentations (Enhanced Constitution, Improved Metabolism) to individual infantry |
| Temple | Mystical/Human Enhancement | 2 | Altar | Researches technologies (staffed by Recruits), including the Augmentation improvement lines |
| Cathedral | Mystical/Human Enhancement | 3 | Temple | Trains Recruits into Acolytes (spell caster); enables Level 2 of the Augmentation improvement lines |
| Shrine | Mystical/Human Enhancement | 3 | Temple | Ritual building powered by Recruits inside; casts area abilities that grant infantry health regeneration and boost Acolyte energy regeneration |

### Cults Infantry

The Cults have four infantry unit types: Recruit, Soldier, Gunner, and Acolyte (spell caster, trained at the Cathedral). All four can receive Augmentations at the Altar.

Cults infantry have no natural health regeneration. Their only source of healing is the Shrine's health regeneration ability.

The Mystical/Human Enhancement branch forks after the Temple: the Cathedral and the Shrine both require the Temple and are both Tier 3.

```
Storage → Armory ─┬→ Gunworks
                  ├→ Garrison
                  └→ Chop Shop ─┬→ Machine Shop
                                └→ Airstrip → Scrap Hangar

Storage → Altar → Temple ─┬→ Cathedral
                          └→ Shrine
```

### Cults Vehicles

Every Cults unit, including vehicles and aircraft, is made from Recruits. Vehicles (not yet designed) are produced at the Chop Shop and consume multiple Recruits each. Aircraft (not yet designed) are produced at the Airstrip; the number of Recruits per aircraft is TBD per aircraft type. Because a trained unit's Unit Control cost equals the number of Recruits consumed (see RecruitmentHall > Unit Control Tracking), vehicles cost one Unit Control per Recruit consumed. The exact number of Recruits per vehicle is set per vehicle type.

## Files

- `armory.md` — **Armory**: Trains Recruits into combat units
- `gunworks.md` — **Gunworks**: Tech/Weaponry T2 support
- `garrison.md` — **Garrison**: Defensive structure
- `chop_shop.md` — **Chop Shop**: Tech/Weaponry T2 vehicle production
- `machine_shop.md` — **Machine Shop**: Tech/Weaponry T2 support
- `airstrip.md` — **Airstrip**: Tech/Weaponry T3 aircraft production
- `scrap_hangar.md` — **Scrap Hangar**: Tech/Weaponry T3 support
- `temple.md` — **Temple**: Mystical T2; research, Recruit staffing
- `cathedral.md` — **Cathedral**: Mystical T3; trains Acolytes
- `shrine.md` — **Shrine**: Mystical T3; rituals, energy, abilities
