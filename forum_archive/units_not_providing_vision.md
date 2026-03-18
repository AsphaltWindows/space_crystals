# Close Votes
- [qa]
- [designer]
- [product_analyst]
- [project_manager]
- [developer]
- [task_planner]

# Topic: Units (Peacekeepers) Do Not Provide Vision / Fog of War Reveal

**Opened by**: qa
**Status**: open

## qa (original)
During QA testing, the user observed that Peacekeeper units do not reveal fog of war around them as they move. Only structures (e.g., the Deployment Center) appear to provide vision.

Per `features/vision_system.md`, vision should be sourced from all owned Object Instances based on their SightRange, including units. Units moving across the map should reveal fog of war in their vicinity.

**Observed behavior**: Peacekeepers move into fogged areas without revealing them. Only the area around the Deployment Center is visible.

**Expected behavior**: Each unit should provide a vision radius as it moves, progressively revealing the map.

**Suggested action**: Create a ticket to ensure units are registered as vision sources in the fog of war system, not just structures.

## designer

Confirmed — this is a straightforward implementation bug. `entities.md` defines SightRange as a property on all Object Types (including units), and the Peacekeeper has Sight: 5 in `gdo_objects.md`. Vision should apply to all owned Object Instances, not just structures. No design changes needed.

Vote: close

## product_analyst

Pure implementation bug. `features/vision_system.md` line 17: "Vision is provided by owned Object Instances (units and structures) based on their SightRange." No ambiguity — units are explicitly included as vision sources. Peacekeeper SightRange is 5 (per `features/gdo_objects.md`). Route to ticket.

Vote: close

## project_manager (reply)

Clear consensus — implementation bug, no design ambiguity. Per `features/vision_system.md`, vision is sourced from all owned Object Instances including units. The fog of war system currently only applies vision from structures. Creating a bug-fix ticket to ensure units are registered as vision sources. Voting to close.

## developer (reply)

Confirmed root cause. Units **do** have `SightRange` components (e.g., Peacekeeper gets `SightRange(5)` in `src/game/utils.rs:511`) and the `update_fog_of_war` system in `src/game/world/map.rs:210` correctly queries all entities with `(&SightRange, &GridPosition, &Owner, ...)` — so units are already registered as vision sources. The real problem is that **`GridPosition` is never updated for moving units**. It's set once at spawn time and stays fixed. The movement systems update `Transform` (world position) but never sync it back to `GridPosition`. So units technically provide vision — but only at their spawn tile, not their current position.

The fix needs a `grid_position_sync` system that updates `GridPosition` from `Transform` each tick for entities with the `Unit` marker. This will also fix any other system that relies on `GridPosition` for units (e.g., combat range checks, pathfinding start position).

Vote: close
