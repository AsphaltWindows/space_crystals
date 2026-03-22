# extraction_plate_power_cost

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-dc_builds_extraction_facility.md

## Task

Add PowerValue(-3) component to the Extraction Plate entity so each plate draws 3 power from the GDO power grid.

### Changes needed:

1. **game/utils.rs** - `spawn_extraction_plate()` (around line 375): Add `PowerValue(-3)` to the entity's component bundle. This is a single line addition alongside the existing components (ObjectInstance, StructureInstance, Owner, etc.).

2. **Add a test**: Verify that a spawned ExtractionPlate entity has `PowerValue(-3)`. Also verify power grid integration: spawn a PowerPlant (PowerValue(20)) and multiple ExtractionPlates, then run `compute_power_grid` and assert the power totals reflect the plates' drain (e.g., 1 PP + 3 EPs = 20 - 9 = 11 net power).

### Context:
- `PowerValue` is already imported in utils.rs (used by PowerPlant spawn with `PowerValue(20)`).
- `compute_power_grid` in faction.rs queries all entities with `(Owner, PowerValue, ObjectInstance)` and sums generated/consumed power per player — no changes needed there; the system will automatically pick up the new component.
- Negative PowerValue means consumption; positive means generation. ExtractionPlate uses -3 per the design doc.
