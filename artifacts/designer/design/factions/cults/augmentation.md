# Cults: Augmentation

## Augmentation

A Cults-specific mechanic. At the Altar, Cults infantry consume Space Crystals and fuse them into their flesh, permanently altering their physical capabilities. Augmentations belong to the **individual unit** that receives them — they are not faction-wide.

The two Augmentations are **Enhanced Constitution** and **Improved Metabolism** (working names).

### Increments

Augmentation effects are expressed in two kinds of increments:

- **Constitution increment**: +10 MaxHP; +1 melee Damage (unarmed units only)
- **Metabolism increment**: **+0.5 space units/frame MaxSpeed** (placeholder); for unarmed units also **−1 frame of FiringDuration** per increment (placeholder; FiringDuration never below 2 frames; a negative increment adds 1 frame).

Increments can be negative. A negative increment applies the opposite effect (e.g. −1 Constitution increment = −10 MaxHP, and −1 melee Damage for unarmed units).

**Unarmed units** are infantry whose attack is an unarmed melee attack — currently only the Recruit. Armed infantry (Militant, Gunner) receive only the MaxHP part of Constitution increments and only the movement-speed part of Metabolism increments.

### Augmentation Effects

Each Augmentation grants a positive amount of its own increment and a negative amount of the other. The amounts depend on the Temple research level in effect when the augmentation is applied (see Temple and Level Locking below):

| Augmentation | No research | Level 1 researched | Level 2 researched |
|---|---|---|---|
| Enhanced Constitution | +2 Constitution / −1 Metabolism | +4 Constitution / −2 Metabolism | +6 Constitution / −3 Metabolism |
| Improved Metabolism | +2 Metabolism / −1 Constitution | +4 Metabolism / −2 Constitution | +6 Metabolism / −3 Constitution |

A unit may have neither, either, or both Augmentations. A unit with both sums their effects. Examples:
- Both at no research: +1 Constitution / +1 Metabolism.
- Enhanced Constitution at Level 2 + Improved Metabolism at no research: +5 Constitution / −1 Metabolism.

### Level Locking

A unit receives the research level that is in effect **at the moment its augmentation finishes** at the Altar — not when the order was queued, and not when the unit was trained. The unit keeps that level permanently; research completed later does not change it (Cults stat upgrades are not retroactive — see core/tech_and_research.md > Research > Faction Exceptions).

A unit cannot receive an Augmentation it already has. There is no way to re-augment a unit to a newer research level.

### AugmentationState (per unit instance)
- EnhancedConstitution: None | NoResearch | Level1 | Level2
- ImprovedMetabolism: None | NoResearch | Level1 | Level2

### Eligibility
All four Cults infantry types: Recruit, Militant, Gunner, and Disciple. Only Cults infantry can be augmented.

### Persistence
- **Training** (Armory or Cathedral): a trained unit inherits its Recruit's AugmentationState unchanged. Augmentation and training can happen in either order with the same result.
- **Vehicles**: if augmented Recruits are consumed to produce a vehicle, their Augmentations are lost (vehicles are not infantry).
- **Death**: Augmentations are lost with the unit.

## Altar

Tier 1 of the Mystical/Human Enhancement branch. The place where Cults infantry consume Space Crystals and augment their flesh with them. Any Cults infantry can enter the Altar, where one or both Augmentations can be applied to them (see Augmentation). The Altar works like the Armory: it holds an internal pool of units, processes a serial queue of orders, and sends finished units out to its RallyPoint.

### Faction - TheCults
### Constructed by - Recruit
### TechPrerequisite - player owns at least one Storage

### Size - 2x3 (placeholder)
### SymmetryType - ABAC (placeholder)
### MaxHP - 300 (placeholder)
### PointArmor - 1 (placeholder)
### FullArmor - 6 (placeholder)
### SightRange - 4 grid units (placeholder)
### Destructible - true
### Groupable - true (placeholder)

Space Crystals cost and construction time are listed under the Recruit's Constructs list, not here (see Recruit > Building).

### EntranceSide / ExitSide - B / C (placeholder, like the Armory)
### InternalUnitCapacity - 10 (placeholder)

### Entry
Any Cults infantry (Recruit, Militant, Gunner, Disciple) can enter the Altar to join its StoredUnits pool, up to InternalUnitCapacity. Right-clicking an own Altar with Cults infantry selected issues Enter Altar.

Fully augmented units (both Augmentations already applied) can also enter, but there is nothing to apply to them — they can only leave, via Eject All.

### Augmentation Orders
Order types: **Enhanced Constitution**, **Improved Metabolism**, or **Both** (a single order that applies both). An order can only target a unit that is missing every Augmentation the order applies.

Orders are queueable and serial — processed one at a time, in the order queued. Queuing an order immediately deducts its Space Crystals cost and reserves one unit of the selected type and AugmentationState from StoredUnits (removing it from the pool while it waits). Only the front of the queue actively progresses. When an order finishes, the unit's AugmentationState is updated using the research level in effect at that moment (see Augmentation > Level Locking), and the unit **automatically exits** from the exit side and moves to the RallyPoint.

Cost and time per order type (placeholder, designer 2026-11-26): **Enhanced Constitution** and **Improved Metabolism**: 25 Space Crystals, 160 frames (10 s) each; **Both**: the sum, 50 Space Crystals, 320 frames.

### AltarInstanceState:
- StoredUnits: list of Cults infantry references (max InternalUnitCapacity)
- AugmentationQueue: array of {Order: EnhancedConstitution | ImprovedMetabolism | Both, ReservedUnit: unit reference} (max 20, placeholder)
- CurrentAugmentationProgress: number (frames elapsed for the front entry of AugmentationQueue) | None
- RallyPoint: Coordinates | ObjectInstance | None

### Eject All
Same as the Armory's Eject All: ejects all StoredUnits from the exit side in a rapid stream. Units reserved by a queued order are not ejected. This is the only way for fully augmented units to leave.

### ConstructionHP Rule - applies

### ObjectInterfaceState[Altar]:

Augmenting is a three-stage selection: **unit type → current AugmentationState → Augmentation to apply**. Each click in the third stage queues one more order; there is no separate confirm step. Z returns to the previous stage at every stage. (Slot assignments below are provisional.)

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Unit type tiles (StateOnlyTransition to AugmentMenu[UnitType]) — each tile shows the count of that unit type in StoredUnits; disabled if there are none:
- **Q: Recruit**
- **W: Militant**
- **E: Gunner**
- **R: Disciple**

Immediate commands (CommandIssuingTransition):
- **A: Eject All**: ejects all StoredUnits. Only available if StoredUnits is not empty.
- **X: Cancel Production**: removes the last entry from AugmentationQueue, refunds its full Space Crystals cost, and returns its reserved unit to StoredUnits. Only available if AugmentationQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

#### AugmentMenu[UnitType]:
AugmentationState tiles for the selected unit type (StateOnlyTransition to AugmentMenu[UnitType, State]). Each tile shows the count of unreserved units of that type and state in StoredUnits; disabled if there are none:
- **Q: Unaugmented**
- **W: Enhanced Constitution only**
- **E: Improved Metabolism only**
- **R: Fully augmented** — always disabled (count shown for information; nothing can be applied)
- **Z**: returns to DefaultState (StateOnlyTransition)

#### AugmentMenu[UnitType, State]:
Shows only the Augmentations the selected group is missing. Each click queues one order (CommandIssuingTransition) and stays in this menu so further clicks queue more. Available only if the group still has an unreserved unit, the AugmentationQueue is not full, and the player has sufficient Space Crystals:
- From Unaugmented: **Q: Enhanced Constitution**, **W: Improved Metabolism**, **E: Both**
- From Enhanced Constitution only: **W: Improved Metabolism**
- From Improved Metabolism only: **Q: Enhanced Constitution**
- **Z**: returns to AugmentMenu[UnitType] (StateOnlyTransition)
