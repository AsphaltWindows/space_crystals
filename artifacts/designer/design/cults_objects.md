# Cults Objects

## Cults Building Mechanics

Cults buildings are constructed by Recruits. Recruits assigned to build a structure enter it and are consumed when construction completes. If the building is cancelled before completion, all assigned Recruits are returned. Multiple Recruits can be assigned to a single building to speed up construction proportionally. Recruits cannot be selectively removed from a building in progress — only the entire building can be cancelled.

### Build Command
- Select one or more Recruits
- Select the Construct option
- Select the desired building
- Left Click on the ground to place it
- All selected Recruits walk to the location and enter the building to construct it

### Assist Construction Command
- Select one or more Recruits
- Select the Assist Construction option
- Left Click on an in-progress building
- Once the selected Recruits reach the building they enter it and increase the building speed

## RecruitmentCenter

The Cults' starting structure. Automatically produces Recruits from the surrounding population of Recruitable terrain. Each Recruitment Center operates independently with its own local capacity and production rate.

### Faction - TheCults
### Size - 4x4 grid units
### SymmetryType - AAAA
### MaxHP - TBD
### PointArmor - TBD
### FullArmor - TBD
### SightRange - TBD
### Destructible - true
### Groupable - false

### RecruitmentArea
A 10x10 grid unit area centered on the 4x4 building.

### TileClaiming
A Recruitment Center claims Recruitable tiles within its Recruitment Area that are not already claimed by another active Recruitment Center. Tiles are claimed on a first-built basis — the center that was constructed first has priority. This priority also applies when tiles are freed by the destruction of a competing center; if multiple centers contest a newly freed tile, the one that was built first claims it.

### Effectiveness
Effectiveness = (number of claimed Recruitable tiles in Recruitment Area) / (total number of tiles in Recruitment Area).

Effectiveness scales both the center's maximum Unit Control capacity and its Recruit production speed linearly.

### BaseUnitControlCapacity - 20
### BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit

At reduced effectiveness, capacity and production time scale linearly. Example: at 50% effectiveness, capacity = 10, production time = 24 seconds per Recruit.

### RecruitmentCenterInstanceState:
- Effectiveness: number (0.0 to 1.0)
- ClaimedTiles: set of Coordinates
- LocalUnitControlUsage: number (current usage against this center)
- LocalUnitControlCapacity: number (BaseUnitControlCapacity * Effectiveness)
- CurrentProductionTime: number (BaseRecruitProductionTime / Effectiveness)
- ProductionProgress: number (frames elapsed) | None
- RallyPoint: Coordinates | ObjectInstance | None

### Production Behavior
The Recruitment Center automatically and continuously produces Recruits at no resource cost until LocalUnitControlUsage reaches LocalUnitControlCapacity. Produced Recruits spawn at the center and move to its rally point. Production logic is entirely local — global Unit Control state does not affect production decisions.

### Unit Control Tracking
Each Recruit tracks which Recruitment Center produced it. This lineage persists when a Recruit is trained into another unit. A trained unit's Unit Control cost equals the number of Recruits consumed to produce it, with each point of cost attributed to the originating center of the respective Recruit. When a unit is destroyed, LocalUnitControlUsage decreases at the originating center(s).

### Cults Unit Control HUD Display
Displays: (sum of all LocalUnitControlUsage across all active Recruitment Centers and orphaned units) / (sum of all LocalUnitControlCapacity across all active Recruitment Centers). This is a global aggregation for display purposes only.

### On Destruction:
- LocalUnitControlCapacity is removed from the global sum
- Surviving units produced by this center remain on the field and continue to count toward the global usage sum
- Claimed tiles are freed and may be reclaimed by other active Recruitment Centers based on build-order priority

### ConstructionHP Rule - applies

### ObjectInterfaceState[RecruitmentCenter]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **X: Cancel Production**: cancels the current Recruit in production, resetting ProductionProgress. Only available if ProductionProgress is not None.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

## Storage

Drop-off point for Space Crystals collected by Recruits. Multiple Recruits can drop off Space Crystals simultaneously.

### Faction - TheCults
### Size - 3x2 grid units
### SymmetryType - ABAB
### MaxHP - TBD
### PointArmor - TBD
### FullArmor - TBD
### SightRange - TBD
### Destructible - true
### Groupable - true
### SpaceCrystalsCost - TBD

### ConstructionHP Rule - applies

### ObjectInterfaceState: None (info display only)

## Armory

Training building where Recruits are converted into combat units. Recruits enter through one side and trained units exit from the opposite side. The Armory maintains an internal pool of Recruits that can be trained into Soldiers or Gunners.

### Faction - TheCults
### Size - 3x2 grid units
### SymmetryType - ABCB
### MaxHP - TBD
### PointArmor - TBD
### FullArmor - TBD
### SightRange - TBD
### Destructible - true
### Groupable - true
### SpaceCrystalsCost - TBD

### EntranceSide - one short side (A)
### ExitSide - opposite short side (C)

Recruits enter through the entrance side. Trained units and ejected Recruits exit through the exit side.

### InternalRecruitCapacity - 10

The Armory can hold up to 10 Recruits in its internal pool at a time.

### Training
Training consumes one Recruit from the internal pool and costs Space Crystals. The Armory produces the selected unit type after a training period, and the trained unit exits from the exit side.

Available unit types:
- **Soldier** (SpaceCrystalsCost: TBD, TrainingTime: TBD)
- **Gunner** (SpaceCrystalsCost: TBD, TrainingTime: TBD)

### Open Questions
- Can the Armory queue multiple training orders, or one at a time?
- Can training be cancelled mid-production? If so, is the Recruit returned, the Space Crystals refunded, or both?
- Can the Armory train one unit at a time or multiple in parallel?

### ArmoryInstanceState:
- StoredRecruits: list of Recruit references (max 10)
- TrainingQueue: TBD (pending queue design)
- RallyPoint: Coordinates | ObjectInstance | None

### Eject All
Immediate command that ejects all stored Recruits from the exit side in a rapid stream, one after another. Ejected Recruits are unchanged — they return to the field as normal Recruits.

### ConstructionHP Rule - applies

### ObjectInterfaceState[Armory]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **E: Eject All**: ejects all stored Recruits from the exit side. Only available if StoredRecruits is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

Production commands (CommandIssuingTransition):
- **Q: Train Soldier**: begins training a Soldier. Only available if StoredRecruits is not empty and player has sufficient Space Crystals.
- **W: Train Gunner**: begins training a Gunner. Only available if StoredRecruits is not empty and player has sufficient Space Crystals.
