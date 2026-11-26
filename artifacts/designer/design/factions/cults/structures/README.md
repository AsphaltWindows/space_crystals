# Cults: Structures

## Tech Tree Overview

Complete list of Cults structures designed so far. Every Cults structure is built by Recruits (see Cults Building Mechanics). "Requires" is the TechPrerequisite (see core/tech_and_research.md > Tech Prerequisites).

The Cults tech tree splits into two branches above a shared base. A player may build from both branches in the same game — the branches are simply two independent prerequisite chains, with no lock-out and no extra cost for pursuing both.

- **Tech/Weaponry branch**: weapons, combat training, weapon upgrades, vehicles, aircraft, and vehicle/air upgrades. Buildings use scavenged/makeshift industrial names.
- **Mystical/Human Enhancement branch**: crystal-based augmentation of infantry flesh, research that improves augmentation, and the Disciple (spell caster).

"Tier" is an informal, descriptive label — not a game rule; all gating is done through TechPrerequisites. Each tier is anchored by its major building, the one that produces units; support buildings (upgrades, unlocks) share the tier of the producer they sit beside, even when they require it.

| Structure | Branch | Tier | Requires | Role |
|---|---|---|---|---|
| RecruitmentHall | Shared base | — | — | Starting structure; produces Recruits from Recruitable terrain; provides Unit Control |
| Storage | Shared base | — | — | Space Crystals drop-off |
| Armory | Tech/Weaponry | 1 | Storage | Trains Recruits into Militants, and Gunners (Gunner requires a Gunworks) |
| Gunworks | Tech/Weaponry | 2 | Armory | Unlocks Gunner training at the Armory; researches weapon upgrades (staffed by Recruits) |
| Garrison | Tech/Weaponry (defense) | 2 | Armory | Defensive structure; up to 4 Recruits/Militants/Gunners/Disciples garrison inside; Recruits and Disciples fire as Militants; Militants and Gunners get +1 range |
| Chop Shop | Tech/Weaponry | 2 | Armory | Vehicle production (vehicles consume multiple Recruits) |
| Machine Shop | Tech/Weaponry | 2 | Chop Shop | Researches vehicle and air upgrades (staffed by Recruits) |
| Airstrip | Tech/Weaponry | 3 | Chop Shop | Aircraft production (from Recruits) |
| Scrap Hangar | Tech/Weaponry | 3 | Airstrip | Unlocks advanced aircraft at the Airstrip (a large paratrooper plane, an airship/zeppelin) |
| Altar | Mystical/Human Enhancement | 1 | Storage | Applies Augmentations (Enhanced Constitution, Improved Metabolism) to individual infantry |
| Temple | Mystical/Human Enhancement | 2 | Altar | Researches technologies (staffed by Recruits), including the Augmentation improvement lines |
| Cathedral | Mystical/Human Enhancement | 3 | Temple | Trains Recruits into Disciples (spell caster); enables Level 2 of the Augmentation improvement lines |
| Shrine | Mystical/Human Enhancement | 3 | Temple | Ritual building powered by Recruits inside; casts area abilities that grant infantry health regeneration and boost Disciple energy regeneration |

### Cults Infantry

The Cults have four infantry unit types: Recruit, Militant, Gunner, and Disciple (spell caster, trained at the Cathedral). All four can receive Augmentations at the Altar.

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

Every Cults unit, including vehicles and aircraft, is made from Recruits. Vehicles (the Seeker so far) are produced at the Chop Shop and consume multiple Recruits each. Aircraft (the Sparrow so far) are produced at the Airstrip; the number of Recruits per aircraft is TBD per aircraft type. Because a trained unit's Unit Control cost equals the number of Recruits consumed (see factions/cults/structures/recruitment_hall.md > Unit Control Tracking), vehicles cost one Unit Control per Recruit consumed. The exact number of Recruits per vehicle is set per vehicle type.

## Choosing Recruits, Release and Eject All
(user, 2026-11-26) Buildings that hold Cults units let the player choose **which kind** of unit to use, by its augmentation. The four **kinds** are: **Plain** (unaugmented), **Constitution** (Enhanced Constitution only), **Metabolism** (Improved Metabolism only) and **Both**. Each kind's button shows the unit with an **icon for its augments** and the count inside (unreserved); it is greyed if there are none.

**Pool buildings** (Armory, Cathedral, Chop Shop, Airstrip; the Altar does the same through its own menus, factions/cults/augmentation.md):
- DefaultState: **Q** Plain, **W** Constitution, **E** Metabolism, **R** Both (each enters KindMenu[kind]); **A** Eject All; **C** Set Rally Point; **X** Cancel Production.
- **KindMenu[kind]**: the building's production options, using Recruits of that kind (e.g. Armory: **Q** Train Militant, **W** Train Gunner; a Seeker reserves 2 Recruits of the kind), plus **A: Release**; **Z** back. Clicks stay in the menu so they can be repeated. **You pick a kind at the Chop Shop and Airstrip too**, even though vehicles lose augments, since you may want to release your augmented Recruits ("maybe they're just hiding out").
- **Release** (user, 2026-11-26): a **no-op order**: **free**, very fast (**4 frames**, placeholder), queued like any order; when it completes the reserved unit walks out of the exit **unchanged** and goes to the RallyPoint. Press it n times to get n units of that kind out. Within a kind, the unit that entered first is used first (0332).

**Tech buildings** (Temple, Gunworks, Machine Shop, Shrine): DefaultState has **one button per kind**, **A** Plain, **S** Constitution, **D** Metabolism, **F** Both: pressing one sends **one** Recruit of that kind out of the exit at once (no queue). Research and abilities keep the top row; **V** Eject All; **X** Cancel (letters: designer placeholder).

**Entering pool buildings** (user, 2026-11-26; Q0104):
- Units go in at the **entrance doorway** (middle of the entrance side; core/entities.md > Entrance and exit).
- **Full building**: a unit arriving when the building is full has its Enter **end** and idles beside the building (the normal Failure rule).
- **Loads**: a Recruit carrying Space Crystals **keeps its load** inside and comes out with it; a Recruit consumed by training loses it.
- **Under construction**: a building under construction **can't be entered**; a right-click on it Assists.
- **Capacity counts everyone inside**, reserved units included: the queue can never hold more orders than the building's capacity allows (e.g. the Armory: 10 Recruits inside in all, so at most 10 queued orders). Capacities are placeholders and may be raised in the numbers pass.

**Leaving** (user, 2026-11-26; Q0105): ejected and released units walk **to the RallyPoint**, like trained units; units sharing the exit doorway come out **in the order they are ready**.

**Not groupable** (user, 2026-11-26; Q0105): Cults production buildings (Armory, Altar, Cathedral, Chop Shop, Airstrip) are Ungroupable: a command goes to one building.

**Eject All still exists** everywhere (user, 2026-11-26): every unreserved unit leaves, one at a time through the exit (the Garrison: all at once through its door).

## Files

- `recruitment_hall.md` — **RecruitmentHall**: Starting structure; Recruits, Recruitment Area, Unit Control
- `storage.md` — **Storage**: Space Crystals drop-off
- `armory.md` — **Armory**: Trains Recruits into combat units
- `gunworks.md` — **Gunworks**: Tech/Weaponry T2 support
- `garrison.md` — **Garrison**: Defensive structure
- `chop_shop.md` — **Chop Shop**: Tech/Weaponry T2 vehicle production
- `machine_shop.md` — **Machine Shop**: Tech/Weaponry T2 support
- `airstrip.md` — **Airstrip**: Tech/Weaponry T3 aircraft production
- `scrap_hangar.md` — **Scrap Hangar**: Tech/Weaponry T3 support
- `temple.md` — **Temple**: Mystical T2; research, Recruit staffing
- `cathedral.md` — **Cathedral**: Mystical T3; trains Disciples
- `shrine.md` — **Shrine**: Mystical T3; rituals, energy, abilities
