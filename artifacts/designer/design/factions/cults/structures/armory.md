# Armory

Training building where Recruits are converted into combat units. Recruits enter through one side and trained units exit from the opposite side. The Armory maintains an internal pool of Recruits that can be trained into Militants or Gunners.

## Faction - TheCults
## Size - 2x3 grid units (2 tall, 3 wide in the default orientation)
## SymmetryType - ABAC (top A, right B, bottom A, left C; core/entities.md > Structure Type > Side lettering convention)
## MaxHP - 300
## PointArmor - 1
## FullArmor - 6
## SightRange - 4 grid units
## Destructible - true
## Groupable - false (user, 2026-11-26; Q0105: Cults production buildings aren't groupable)
## TechPrerequisite - player owns at least one Storage

Space Crystals cost and construction time are listed under the Recruit's Constructs list, not here (see Recruit > Building).

## EntranceSide - one short side (B)
## ExitSide - opposite short side (C)

Recruits enter through the entrance side. Trained units and ejected Recruits exit through the exit side, one at a time through the exit doorway (core/entities.md > Structure Type > Exit Doorway).

## InternalRecruitCapacity - 10

The Armory can hold up to 10 Recruits in its internal pool at a time.

## Training
Uses **Unit Production, Pool producer** (systems/mechanics/production.md). Its rules in full:
Training is queueable and serial — orders are processed one at a time, in the order queued. Queuing an order immediately deducts its Space Crystals cost from the player and reserves one Recruit from StoredRecruits (removing it from the internal pool for the duration it sits in the queue). Only the front of the queue actively trains; queued-but-not-yet-started orders wait with their Recruit and Space Crystals already committed. When an order finishes training, its reserved Recruit is consumed and the trained unit exits from the exit side. The trained unit inherits the Recruit's AugmentationState unchanged (see Augmentation) — Augmentation and Armory training can happen in either order with the same result. **Which unit** (user, 2026-11-26; Q0105/Q0114/Q0121): an order reserves the matching unit(s) that **entered the building first** (first in, first trained). **HP**: infantry trained from a Recruit **keep the Recruit's damage** (the missing HP carries over); vehicles and aircraft come out at full HP.

Available unit types:
- **Militant** (SpaceCrystalsCost: 20, TrainingTime: 32 frames)
- **Gunner** (SpaceCrystalsCost: 40, TrainingTime: 32 frames). TechPrerequisite: player owns at least one Gunworks.

## ArmoryInstanceState:
- StoredRecruits: list of Recruit references (max 10)
- TrainingQueue: array of {UnitType: Militant | Gunner, ReservedRecruit: Recruit reference} (max 20, including the order currently training; user, 2026-11-24; **in practice bounded by the Armory's capacity of 10 Recruits, reserved ones included**, user, 2026-11-26, Q0104)
- CurrentTrainingProgress: number (frames elapsed for the front entry of TrainingQueue) | None
- RallyPoint: Coordinates | ObjectInstance | None

## Eject All
Immediate command that ejects all stored Recruits from the exit side in a rapid stream, one after another. Ejected Recruits are unchanged — they return to the field as normal Recruits. This only affects StoredRecruits (the unreserved pool); Recruits already reserved by a queued training order are not ejected and remain committed to that order.

## ConstructionHP Rule - applies

## ObjectInterfaceState[Armory]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **A: Eject All**: ejects all stored Recruits from the exit side. Only available if StoredRecruits is not empty.
- **X: Cancel Production**: removes the last entry from TrainingQueue, refunds its full Space Crystals cost to the player, and returns its reserved Recruit to StoredRecruits. Only available if TrainingQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

Kinds and Release (user, 2026-11-26): **Q/W/E/R choose a kind of Recruit first** (Plain, Constitution, Metabolism, Both), then **Q: Train Militant**, **W: Train Gunner** or **A: Release** (factions/cults/structures/README.md > Choosing Recruits, Release and Eject All). The commands below act on Recruits of the chosen kind.

Production commands (CommandIssuingTransition, in KindMenu[kind]):
- **Q: Train Militant**: deducts Militant's Space Crystals cost from the player, moves one Recruit from StoredRecruits into a new TrainingQueue entry. Only available if StoredRecruits is not empty, TrainingQueue has fewer than 20 entries, and player has sufficient Space Crystals.
- **W: Train Gunner**: deducts Gunner's Space Crystals cost from the player, moves one Recruit from StoredRecruits into a new TrainingQueue entry. Only available if StoredRecruits is not empty, TrainingQueue has fewer than 20 entries, player has sufficient Space Crystals, and the player owns at least one Gunworks (TechPrerequisite).
