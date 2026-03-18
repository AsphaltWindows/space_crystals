# Ticket: Air Units Blocked by Impassable Terrain

## Current State
Air units (Supply Chopper / HoverCraft domain) cannot fly over impassable terrain (mountains, etc.). The pathfinding system at `src/game/units/pathfinding.rs:186-194` runs the `tile_map.is_traversible()` check unconditionally for all units, regardless of domain. Additionally, combat behavior systems in `src/game/combat/systems/behaviors.rs` call `find_path()` directly (ground-only) instead of `find_path_for_domain()`, and rally point pathfinding in `src/game/world/faction.rs` hardcodes `UnitBaseEnum::LightInfantry`. A test (`find_path_air_still_respects_terrain`) incorrectly asserts the broken behavior.

## Desired State
Air units (Domain=Air) should ignore tile traversability checks entirely when pathfinding. They should only respect grid bounds. All pathfinding call sites should use `find_path_for_domain()` with the actual unit's domain. The incorrect test should be inverted to assert air units CAN cross impassable terrain.

Specific fixes:
1. In `find_path_inner`: when `skip_occupancy` is true (air mode), skip `is_traversible` checks — only enforce grid bounds
2. Update 5 `find_path()` call sites in `behaviors.rs` to `find_path_for_domain()`
3. Update rally pathfinding in `faction.rs` to pass the actual unit's `UnitBaseEnum` and use `find_path_for_domain()`
4. Fix the `find_path_air_still_respects_terrain` test to assert air units CAN cross impassable terrain

## Justification
The specs are unambiguous across three feature files:
- `features/tile_system.md`: `Traversible` is defined as "boolean (ground units can cross)" — explicitly scoped to ground units
- `features/unit_system.md`: HoverCraft has Domain=Air, and air units "do not collide with ground units or structures"
- `features/vision_system.md`: Air units are exempt from elevation modifiers, reinforcing that air units ignore ground terrain properties

Bug confirmed in forum topic `supply_chopper_impassable_terrain.md` (archived, 6/6 close votes).

## QA Steps
1. [human] Start a GDO game and build a Supply Tower to produce Supply Choppers
2. [human] Identify an area of impassable terrain (mountains) on the map
3. [human] Select a Supply Chopper and right-click on the far side of the impassable terrain
4. [human] Verify the Supply Chopper flies directly over the impassable terrain without being blocked or rerouting
5. [auto] Run pathfinding unit tests — the updated `find_path_air_still_respects_terrain` test should pass asserting air units traverse impassable tiles
6. [human] Select a ground unit (e.g., Peacekeeper) and right-click on the far side of the same impassable terrain — verify ground units are still blocked and route around it

## Expected Experience
Supply Choppers (and any future air units) should fly freely over mountains and other impassable terrain, pathing in a straight line or near-straight line to their destination. Ground units should continue to be blocked by impassable terrain as before. The pathfinding behavior should cleanly differentiate between air and ground domains.
