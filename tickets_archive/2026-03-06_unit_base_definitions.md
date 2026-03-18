# Ticket: UnitBase Enum and Definitions

## Current State
No UnitBase types exist. The entity system provides Object Type and Object Instance but nothing specific to mobile units.

## Desired State
A `UnitBaseEnum` with 9 variants and a `UnitBase` struct defining the boolean property matrix for each base. Each of the 9 bases must be concretely defined with its fixed property values.

### UnitBaseEnum variants
LightInfantry, HeavyInfantry, WheeledVehicle, TrackedVehicle, DrillUnit, HoverVehicle, Mech, HoverCraft, Glider

### UnitBase struct fields
- Domain: Ground | Air | Underground
- HasTurret: bool
- DirectionalArmor: bool
- RuggedTerrain: bool
- Crushable: bool
- CanTurnInPlace: bool
- CanReverse: bool
- MovementModel: MovementModelEnum

### 9 concrete base definitions (property matrix)

| Base | Domain | Turret | DirArmor | Rugged | Crushable | TurnInPlace | Reverse | Movement |
|------|--------|--------|----------|--------|-----------|-------------|---------|----------|
| LightInfantry | Ground | No | No | Yes | Yes | Yes | No | TurnRate |
| HeavyInfantry | Ground | No | No | Yes | No | Yes | No | TurnRate |
| WheeledVehicle | Ground | Yes | Yes | No | No | No | Yes | FixedTurnRadius |
| TrackedVehicle | Ground | Yes | Yes | No | No | Yes | Yes | SpeedTurnRadius |
| DrillUnit | Underground | Yes | Yes | No | No | Yes | Yes | SpeedTurnRadius |
| HoverVehicle | Ground | Yes | Yes | No | No | Yes | No | Drag |
| Mech | Ground | Yes | Yes | Yes | No | Yes | No | TurnRate |
| HoverCraft | Air | Yes | No | No | No | Yes | No | Drag |
| Glider | Air | Yes | No | No | No | No | No | Glider |

### Special properties
- LightInfantry: has a unique `RuggedTerrainDefenseBonus` (number) field
- TrackedVehicle and Mech: crush enemy LightInfantry (design note, not a field on UnitBase itself)
- DrillUnit: visible while underground, can traverse Drillable tiles underground, cannot fire underground, has above-ground mode (stationary or tracked behavior)

## Justification
Defined in `features/unit_system.md`. UnitBase is the core classification that determines a unit's movement, terrain interaction, and physical characteristics. All other unit system components depend on knowing the UnitBase.

## QA Steps
1. Verify `UnitBaseEnum` exists with exactly 9 variants.
2. Verify `UnitBase` struct has all 8 boolean/enum fields listed above.
3. For each of the 9 bases, verify the concrete property values match the table above exactly.
4. Verify LightInfantry has the additional `RuggedTerrainDefenseBonus` field.
5. Verify a `DomainEnum` (or equivalent) exists with Ground, Air, Underground variants.
6. Verify `MovementModelEnum` exists with TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider variants (this enum is shared with the movement models ticket).

## Expected Experience
Inspecting the code shows a clean enum with 9 variants, a struct with the 8 property fields, and 9 concrete definitions whose values match the property matrix. Each base's properties are easily readable and match the table row-for-row. The LightInfantry definition has an extra numeric field for rugged terrain defense bonus.
