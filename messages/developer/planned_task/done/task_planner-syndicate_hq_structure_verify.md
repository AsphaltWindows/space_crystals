# syndicate-hq-structure-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-syndicate-headquarters-structure.md

## Task

**Verification-only task**: The Headquarters underground expansion structure is already fully implemented. All stats, construction details, production catalog, and starting condition match the design spec. Verify the following are correct and aligned with `artifacts/designer/design/syndicate_objects.md`:

1. **ObjectEnum::Headquarters** in `objects.rs`: size (2,2), destructible, groupable=false
2. **Constants** in `structures.rs` (syndicate_structure_stats): HQ_MAX_HP=400, HQ_POINT_ARMOR=1, HQ_FULL_ARMOR=4, HQ_SC_COST=200, HQ_BUILD_FRAMES=400
3. **spawn_headquarters()** in `utils.rs`: ObjectInstance::destructible with HQ_MAX_HP, DomainEnum::Underground, HeadquartersState::default(), TunnelExpansionMarker
4. **HeadquartersState** in `structures.rs`: rally_point, build_queue (max 5), current_build, current_build_progress, production_cost() for Agent (100 SC/160f) and Guard (125 SC/120f)
5. **Starting condition** in `faction.rs`: spawn_headquarters called during Syndicate setup (pre-built in starting tunnel)
6. **Tunnel ExpandMenu integration**: HQ appears in expand menu, costs 200 SC, builds in 400 frames
7. **Tests**: All existing tests pass (HQ stats, production costs, spawn)

If any discrepancies are found, fix them. Otherwise confirm all is correct.

## Technical Context

This is a verification task. I have investigated every item — all are already correctly implemented. Here is the verification checklist with file references:

### 1. ObjectEnum::Headquarters in objects.rs
- **File**: `artifacts/developer/src/game/types/objects.rs` line 295
- size: (2, 2) ✓, destructible: true ✓, groupable: false ✓
- sight_range: 0 (underground, no surface sight) — matches underground nature

### 2. Constants in structures.rs
- **File**: `artifacts/developer/src/game/types/structures.rs` lines 455-459
- HQ_MAX_HP=400.0 ✓, HQ_POINT_ARMOR=1 ✓, HQ_FULL_ARMOR=4 ✓, HQ_SC_COST=200 ✓, HQ_BUILD_FRAMES=400 ✓

### 3. spawn_headquarters() in utils.rs
- **File**: `artifacts/developer/src/game/utils.rs` lines 777-814
- Uses `ObjectInstance::destructible(ObjectEnum::Headquarters, HQ_MAX_HP)` ✓
- Inserts `DomainEnum::Underground` ✓
- Inserts `HeadquartersState::default()` ✓
- Inserts `TunnelExpansionMarker { parent_tunnel }` ✓
- Also: StructureInstance, Selectable, SelectionBounds, GridPosition, SightRange(3)

### 4. HeadquartersState in structures.rs
- **File**: `artifacts/developer/src/game/types/structures.rs` lines 252-280
- rally_point: Option<RallyTarget> ✓
- build_queue: Vec<ObjectEnum> ✓, MAX_QUEUE_SIZE=5 ✓
- current_build: Option<ObjectEnum> ✓
- current_build_progress: Option<f32> ✓
- production_cost(SyndicateAgent) = 100 SC / 160 frames ✓
- production_cost(SyndicateGuard) = 125 SC / 120 frames ✓

### 5. Starting condition in faction.rs
- **File**: `artifacts/developer/src/game/world/faction.rs` lines 94-122
- `setup_syndicate_game_start()` calls `spawn_headquarters()` pre-built in starting tunnel ✓

### 6. Tunnel ExpandMenu integration
- **File**: `artifacts/developer/src/game/world/faction.rs` line 1465 — cost check uses HQ_SC_COST (200) ✓
- **File**: `artifacts/developer/src/game/world/faction.rs` line 1804 — build time uses HQ_BUILD_FRAMES (400) ✓
- **File**: `artifacts/developer/src/ui/command_panel.rs` lines 649, 1935, 2305 — expand menu references ✓

### 7. Tests
- **File**: `artifacts/developer/src/game/types/structures.rs` lines 1802-1888
- Tests cover: HQ stat constants, object type properties (size, destructible, groupable, sight), production costs for Agent and Guard, queue operations (try_queue, cancel_last, max capacity, mixed queue), destructible instance creation
- **File**: `artifacts/developer/src/game/world/faction.rs` lines 1997-2095
- Tests cover: rally point behavior (none, location, parent tunnel clear, other entity)

### Action Required
Run `cargo test` to confirm all tests pass. No code changes should be needed — this is a confirmation task. If tests pass, mark complete.

### Running Tests
```bash
cd artifacts/developer && cargo test 2>&1 | tail -20
```

Focus on these test modules:
- `game::types::structures::tests` (HQ constants, production costs, queue ops)
- `game::types::objects::tests` (HQ object type properties)
- `game::world::faction::tests` (rally point behavior)

## Dependencies

None — this is a standalone verification task with no dependencies on other planned_tasks.
