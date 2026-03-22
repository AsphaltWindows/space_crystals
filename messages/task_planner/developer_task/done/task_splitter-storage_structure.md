# storage_structure

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-cults_recruitment_center_and_storage.md

## Task

Add the Storage structure to the Cults faction.

### ObjectEnum
Add `ObjectEnum::CultsStorage` variant to `shared/types.rs`. Use 'CultsStorage' to avoid ambiguity with any generic 'Storage' concept.

### ObjectType
Implement `object_type()` for CultsStorage:
- Size: 3x2
- SymmetryType: ABAB
- Destructible: true
- ConstructionHP rule: applies
- SightRange: 3 (small utility building, matches PowerPlant)
- Groupable: true (multiple can be selected together)
- Faction: TheCults

Define constants: STORAGE_MAX_HP (200 — small building), STORAGE_POINT_ARMOR (1), STORAGE_FULL_ARMOR (4).

### StorageState Component (optional, minimal)
No complex state needed for now. Storage is a passive drop-off point. If desired, add a marker component `CultsStorageMarker` to distinguish from other structures for drop-off targeting.

### spawn_cults_storage Function
In `game/utils.rs`, add `spawn_cults_storage()` following the pattern of `spawn_barracks()` (3x2 building):
- 3x2 cuboid mesh, dark purple material
- Components: ObjectInstance::destructible, StructureInstance, Owner, Selectable, SelectionBounds, GridPosition, SightRange
- ABAB side labels (like Barracks uses ABAC — Storage uses ABAB per design)
- Label: "Storage"

### Simultaneous Drop-Off
The design specifies multiple Recruits can drop off Space Crystals simultaneously. This is primarily a behavior system concern (the gathering/drop-off behaviors for Recruits, which will come in a separate feature). For now, just note in a code comment that Storage supports concurrent drop-off (no queuing/blocking needed at the structure level).

### Tests
- CultsStorage object_type() returns correct size (3x2), symmetry (ABAB), destructible
- spawn_cults_storage creates entity with all expected components
- CultsStorage is groupable
