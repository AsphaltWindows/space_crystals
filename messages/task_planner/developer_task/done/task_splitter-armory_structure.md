# armory_structure

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-add_cults_armory.md

## Task

Add the Cults Armory structure to the game.

### ObjectEnum
Add `ObjectEnum::CultsArmory` variant.

### ObjectType
- Faction: TheCults
- Size: 3x2 grid units
- SymmetryType: ABCB (entrance side A, matching long sides B, exit side C)
- MaxHP: TBD (use placeholder, e.g. 300)
- Destructible: true
- SightRange: TBD (use placeholder, e.g. 3)
- Groupable: true

### StructureType
- PointArmor: TBD (use placeholder, e.g. 1)
- FullArmor: TBD (use placeholder, e.g. 4)
- Symmetry: ABCB

### ArmoryState component
Define `ArmoryState` with:
- `stored_recruits: Vec<Entity>` (max 10, the internal Recruit pool)
- `training_queue: Option<ObjectEnum>` (current training order — use simple one-at-a-time for now since queue design is TBD)
- `training_progress: u32` (frames elapsed)
- `rally_point: Option<RallyPointTarget>` (reuse existing rally point pattern from BarracksState/HeadquartersState)

### spawn_cults_armory function
Follow existing spawn patterns (spawn_barracks, spawn_headquarters). Spawn with:
- ObjectInstance::destructible (or under_construction with ConstructionHP rule)
- StructureInstance
- ArmoryState (default empty)
- All standard structure components (Owner, FactionEnum, GridPosition, etc.)

### Constants
Define placeholder constants:
- ARMORY_MAX_HP
- ARMORY_POINT_ARMOR, ARMORY_FULL_ARMOR
- ARMORY_SC_COST (TBD placeholder)
- ARMORY_INTERNAL_RECRUIT_CAPACITY = 10
- SOLDIER_TRAINING_COST (TBD placeholder), SOLDIER_TRAINING_FRAMES (TBD placeholder)
- GUNNER_TRAINING_COST (TBD placeholder), GUNNER_TRAINING_FRAMES (TBD placeholder)

### Tests
- ObjectType returns correct size/symmetry/faction
- StructureType returns correct armor values
- ArmoryState default is empty (no recruits, no training, no rally)
- spawn function creates entity with all expected components
