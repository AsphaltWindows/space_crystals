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

## Armory

Training building where Recruits are converted into combat units. Recruits enter through one side and trained units exit from the opposite side. The Armory maintains an internal pool of Recruits that can be trained into Soldiers or Gunners.

### Faction - TheCults
### Size - 2x3 grid units (2 tall, 3 wide in the default orientation)
### SymmetryType - ABAC (top A, right B, bottom A, left C; core/entities.md > Structure Type > Side lettering convention)
### MaxHP - 300
### PointArmor - 1
### FullArmor - 6
### SightRange - 4 grid units
### Destructible - true
### Groupable - true
### TechPrerequisite - player owns at least one Storage

Space Crystals cost and construction time are listed under the Recruit's Constructs list, not here (see Recruit > Building).

### EntranceSide - one short side (B)
### ExitSide - opposite short side (C)

Recruits enter through the entrance side. Trained units and ejected Recruits exit through the exit side, one at a time through the exit doorway (core/entities.md > Structure Type > Exit Doorway).

### InternalRecruitCapacity - 10

The Armory can hold up to 10 Recruits in its internal pool at a time.

### Training
Training is queueable and serial — orders are processed one at a time, in the order queued. Queuing an order immediately deducts its Space Crystals cost from the player and reserves one Recruit from StoredRecruits (removing it from the internal pool for the duration it sits in the queue). Only the front of the queue actively trains; queued-but-not-yet-started orders wait with their Recruit and Space Crystals already committed. When an order finishes training, its reserved Recruit is consumed and the trained unit exits from the exit side. The trained unit inherits the Recruit's AugmentationState unchanged (see Augmentation) — Augmentation and Armory training can happen in either order with the same result.

Available unit types:
- **Soldier** (SpaceCrystalsCost: 20, TrainingTime: 32 frames)
- **Gunner** (SpaceCrystalsCost: 40, TrainingTime: 32 frames). TechPrerequisite: player owns at least one Gunworks.

### ArmoryInstanceState:
- StoredRecruits: list of Recruit references (max 10)
- TrainingQueue: array of {UnitType: Soldier | Gunner, ReservedRecruit: Recruit reference} (max 20)
- CurrentTrainingProgress: number (frames elapsed for the front entry of TrainingQueue) | None
- RallyPoint: Coordinates | ObjectInstance | None

### Eject All
Immediate command that ejects all stored Recruits from the exit side in a rapid stream, one after another. Ejected Recruits are unchanged — they return to the field as normal Recruits. This only affects StoredRecruits (the unreserved pool); Recruits already reserved by a queued training order are not ejected and remain committed to that order.

### ConstructionHP Rule - applies

### ObjectInterfaceState[Armory]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **A: Eject All**: ejects all stored Recruits from the exit side. Only available if StoredRecruits is not empty.
- **X: Cancel Production**: removes the last entry from TrainingQueue, refunds its full Space Crystals cost to the player, and returns its reserved Recruit to StoredRecruits. Only available if TrainingQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

Production commands (CommandIssuingTransition):
- **Q: Train Soldier**: deducts Soldier's Space Crystals cost from the player, moves one Recruit from StoredRecruits into a new TrainingQueue entry. Only available if StoredRecruits is not empty, TrainingQueue has fewer than 20 entries, and player has sufficient Space Crystals.
- **W: Train Gunner**: deducts Gunner's Space Crystals cost from the player, moves one Recruit from StoredRecruits into a new TrainingQueue entry. Only available if StoredRecruits is not empty, TrainingQueue has fewer than 20 entries, player has sufficient Space Crystals, and the player owns at least one Gunworks (TechPrerequisite).

## Gunworks

Tier 2 support building of the Tech/Weaponry branch. A makeshift arms workshop. Unlocks Gunner training at the Armory and researches weapon upgrades.

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Armory

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, Recruit staffing capacity, research queue size, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### Unlocks
- **Gunner** training at the Armory (Gunner's TechPrerequisite).

### Research
Weapon upgrades (list, costs, BaseResearchTimes TBD). Research follows core/tech_and_research.md > Research, including the Cults exceptions: staffed by Recruits (right-click own Gunworks with Recruits selected → Enter Gunworks), and stat upgrades are not retroactive (only units that finish training after the research completes benefit).

## Garrison

The Cults' defensive structure. Infantry enter the Garrison and fire from inside it: Recruits and Acolytes fire **as if they were Soldiers** (using the Soldier's attack — the weapons come from the Armory, which is why the Garrison requires one), while Soldiers and Gunners fire their own weapons. Without units inside, the Garrison does not attack.

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Armory
### GarrisonCapacity - 4 units

### Garrisoning
- **Who can enter**: Recruits, Soldiers, Gunners, and Acolytes. Units enter with the Enter command (right-click own Garrison with eligible infantry selected), up to 4 units total.
- **Weapons**: a Recruit or Acolyte inside attacks with the Soldier's AttackAttributes (see Soldier); Soldiers and Gunners use their own AttackAttributes. (Whether an Acolyte can also cast spells from inside: TBD.)
- **Range bonus**: **Soldiers and Gunners** inside get **+1 grid unit of Range** on their attack. Recruits and Acolytes (firing with the Soldier's attack) do not get the bonus — they fire at the Soldier's normal Range.
- Units inside are not consumed and still count toward Unit Control.
- Evacuation on Destruction applies: when the Garrison is destroyed, everyone inside comes out alive around its footprint.
- **Protection**: units inside cannot be targeted and take no damage while the Garrison stands.
- Whether the Garrison changes their SightRange: TBD.

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, and ObjectInterfaceState (incl. eject commands): TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Chop Shop

Tier 2 major building of the Tech/Weaponry branch. A scavenger's garage where Recruits are turned into vehicles.

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Armory

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### Production
Produces Cults vehicles (none designed yet). Each vehicle consumes multiple Recruits (number set per vehicle type), and its Unit Control cost equals the number of Recruits consumed. The production flow (e.g. whether it follows the Armory's Recruit pool + serial queue model) is TBD.

## Machine Shop

Tier 2 support building of the Tech/Weaponry branch (sits beside the Chop Shop). Researches upgrades for vehicles and aircraft.

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Chop Shop

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, Recruit staffing capacity, research queue size, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### Research
Vehicle upgrades and air upgrades (list, costs, BaseResearchTimes TBD). Research follows core/tech_and_research.md > Research, including the Cults exceptions: staffed by Recruits (right-click own Machine Shop with Recruits selected → Enter Machine Shop), and stat upgrades are not retroactive.

## Airstrip

Tier 3 major building of the Tech/Weaponry branch. A makeshift runway where Recruits are turned into aircraft.

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Chop Shop

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### Production
Produces Cults aircraft (none designed yet), made from Recruits (number per aircraft TBD). Advanced aircraft — a large paratrooper plane and an airship/zeppelin — are also built here but require the player to own a Scrap Hangar. The production flow is TBD.

## Scrap Hangar

Tier 3 support building of the Tech/Weaponry branch (sits beside the Airstrip). Unlocks production of advanced aircraft at the Airstrip.

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Airstrip

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### Unlocks
Advanced aircraft at the Airstrip (TechPrerequisite on each), not yet designed:
- A large paratrooper plane (carries infantry and drops them — transport/paradrop behavior TBD)
- An airship/zeppelin

## Temple

Tier 2 of the Mystical/Human Enhancement branch. Researches technologies for the Cults. Research follows the shared rules in core/tech_and_research.md > Research, including the Cults exceptions: stat upgrades are not retroactive, and research only progresses while Recruits are staffing the building (speed scales with the number of Recruits inside).

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Altar

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, research queue size, Recruit staffing capacity, BaseResearchTime per technology, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### Research (first set)

Each Augmentation has its own two-level improvement line. Each level raises the effect of that Augmentation for units augmented **after** the level completes (see Augmentation > Level Locking).

| Technology | Requires | Effect | Cost / Time |
|---|---|---|---|
| Enhanced Constitution Level 1 | — | Enhanced Constitution becomes +4 Constitution / −2 Metabolism | TBD |
| Enhanced Constitution Level 2 | Enhanced Constitution Level 1; player owns at least one Cathedral | Enhanced Constitution becomes +6 Constitution / −3 Metabolism | TBD |
| Improved Metabolism Level 1 | — | Improved Metabolism becomes +4 Metabolism / −2 Constitution | TBD |
| Improved Metabolism Level 2 | Improved Metabolism Level 1; player owns at least one Cathedral | Improved Metabolism becomes +6 Metabolism / −3 Constitution | TBD |

Level 2 of both lines is researched at the Temple, but also requires the player to own a Cathedral (counting rule as in core/tech_and_research.md > Tech Prerequisites).

Further Temple research sets: TBD.

### Recruit Staffing
Recruits enter the Temple to staff its research (right-click own Temple with Recruits selected → Enter Temple). Only Recruits can staff research. See core/tech_and_research.md > Research > Faction Exceptions > Cults for the staffing rules (not consumed, linear speed scaling, pauses when empty).

## Cathedral

Tier 3 and last building of the Mystical/Human Enhancement branch. Trains Recruits into Acolytes, the Cults' spell caster, and unlocks the second level of the Temple's Augmentation improvement lines (Enhanced Constitution Level 2 and Improved Metabolism Level 2 require owning a Cathedral).

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Temple

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, entrance/exit sides, internal Recruit capacity, training queue size, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### Training
Works like the Armory's Training: Recruits enter the Cathedral and join an internal pool; training orders are queueable and serial; queuing an order deducts its Space Crystals cost and reserves one Recruit; when the order finishes, the Recruit is consumed and the trained unit exits to the RallyPoint. The trained unit inherits the Recruit's AugmentationState (see Augmentation > Persistence).

Available unit types:
- **Acolyte** (consumes 1 Recruit; SpaceCrystalsCost: TBD, TrainingTime: TBD)

## Shrine

Tier 3 of the Mystical/Human Enhancement branch (sibling of the Cathedral). Recruits gather inside the Shrine and take part in a ritual that charges it with energy. The Shrine spends that energy on area abilities that support Cults infantry.

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Temple

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, Recruit capacity, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### Ritual Participation
Recruits enter the Shrine to take part in the ritual (right-click own Shrine with Recruits selected → Enter Shrine). Only Recruits can participate. Participating Recruits are not consumed. Evacuation on Destruction applies.

### Energy
- **MaxEnergy**: TBD
- **Energy regeneration**: proportional to the number of Recruits inside (rate per Recruit TBD). With no Recruits inside, the Shrine does not regenerate energy.

### Abilities
Both abilities are cast on an area anywhere the player currently has vision, and spend Shrine energy.

The effect is a **buff applied to units at the moment of casting**: qualifying units inside the area when the ability is cast receive the buff for its duration and keep it even if they leave the area. Units that enter the area afterward do not receive it. Only the caster's own units are affected.

| Ability | Affects | Effect | Energy cost | Area | Duration |
|---|---|---|---|---|---|
| Health regeneration (name TBD) | Own Cults infantry | Grants health regeneration (rate TBD) for the duration — Cults infantry have no natural regeneration | TBD | TBD | TBD |
| Energy regeneration (name TBD) | Own Acolytes | Increases Acolyte energy regeneration (amount TBD) for the duration | TBD | TBD | TBD |

Cooldowns (if any): TBD.

### ShrineInstanceState:
- ParticipatingRecruits: list of Recruit references (max TBD)
- Energy: number (0 to MaxEnergy)
