# Ticket: Object and Structure Types and Instances

## Current State
No object type or structure type definitions exist. There is no way to define game objects (units, buildings) or place instances of them on the map.

## Desired State
Define Object Type, Structure Type, Object Instance, and Structure Instance as Bevy ECS types:

**Object Type** (a Visible Entity with Selectable=true):
- `ObjectEnum` value (enum identifying the object type)
- `Name` (string)
- `Size` (Height x Width, in GridUnits from `simulation_core`)
- `Destructible` (boolean)
- `SightRange` (number — provides vision to owning player)
- `Groupable` (boolean — if false, instance is always in its own SelectionGroup)
- `InfoPanel` definition: a function of `(owner: PlayerNumber | None, selector: Player) -> Display` (model as a trait or callback type; concrete implementations come later per object type)

**Structure Type** (extends Object Type):
- `SymmetryType` enum: `AAAA | AAAB | AABB | ABAB | AABC | ABAC | ABCD`
- 4 rotation positions (0, 90, 180, 270 degrees)
- Height and Width may differ only for symmetry types `ABAB`, `ABAC`, `ABCD`

**Object Instance** (runtime instance on the map):
- `Type` (ObjectEnum)
- `Location` (Coordinates)
- `Owner` (PlayerNumber | None)
- `HP` (present only for Destructible types)

**Structure Instance** (extends Object Instance):
- `Rotation`: 0 | 90 | 180 | 270
- `StructureState`: struct (placeholder, to be defined per structure)

## Justification
Required by `features/entity_system.md` (Object Type Properties, Structure Type, Object Instance, Structure Instance sections). These types are the foundation for all selectable game objects — units, buildings, and resources.

## QA Steps
1. Verify `ObjectEnum` exists as an enum type.
2. Verify `SymmetryType` enum has all 7 variants: AAAA, AAAB, AABB, ABAB, AABC, ABAC, ABCD.
3. Verify an Object Type definition can be created with all required fields (ObjectEnum, Name, Size, Destructible, SightRange, Groupable).
4. Verify a Structure Type definition can be created with a SymmetryType.
5. Verify an Object Instance can be spawned with Type, Location, Owner, and optionally HP.
6. Verify a Structure Instance can be spawned with Rotation and StructureState.
7. Write a unit test that creates a Structure Type with SymmetryType::AAAA and verifies that Height must equal Width.
8. Write a unit test that creates a Structure Type with SymmetryType::ABAB and verifies that Height and Width can differ.
9. Write a unit test that spawns an Object Instance of a Destructible type and confirms HP is present, then spawns one of an indestructible type and confirms HP is absent or zero.

## Expected Experience
All unit tests pass. Object and Structure types compile with their full property sets. Instances can be spawned and queried. The symmetry/size constraint is enforced (AAAA requires equal H/W; ABAB/ABAC/ABCD allow unequal).
