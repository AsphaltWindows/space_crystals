# Ticket: Syndicate Underground Expansions Must Not Block Surface Movement

## Current State
Syndicate underground expansions (HQ and other Tunnel expansions) mark their tile footprints as impassable on the surface. When an Agent is produced and ejected at the parent Tunnel's Side A, it spawns on or adjacent to tiles occupied by the HQ, trapping it permanently. This blocks all Syndicate gameplay — Agents cannot move, gather, or build.

## Desired State
Underground Syndicate expansions (HQ, Barracks, and any future expansion types) should NOT mark their surface tiles as impassable or occupied. Surface units must be able to walk freely over the footprint of any underground expansion. Only the Tunnel itself (a surface structure) should occupy tiles.

Specifically:
- Underground expansion placement should not alter the traversability of surface tiles
- The Tunnel's Side A spawn point must land on a walkable tile (outside both the Tunnel's footprint and any tile erroneously blocked by underground expansions)
- Existing pathfinding and collision systems should treat underground expansion footprints as transparent on the surface layer

## Justification
The feature spec (`features/syndicate_objects.md`, Tunnel Expansions section) explicitly states: "Expansions are invisible to enemies without detection and **can be walked over by surface units**." The current implementation violates this requirement, making all Syndicate gameplay impossible. Identified during QA session 2026-03-09, reported in forum topic `syndicate_hq_blocks_agent_movement.md`.

## QA Steps
1. [human] Start a game as Syndicate. Locate the starting Tunnel and its underground HQ. Verify that the tiles above the HQ's 2x2 footprint are walkable by selecting any surface unit and right-clicking on those tiles.
2. [human] Produce an Agent from the HQ with a surface rally point. Verify the Agent ejects from Side A and successfully moves to the rally point without getting stuck.
3. [human] Order a unit to pathfind across the HQ's underground footprint (walk from one side to the other). Verify the unit paths through without detour or blockage.
4. [human] Build a second underground expansion (e.g., Barracks) in the Tunnel Area. Verify its footprint tiles are also walkable on the surface.
5. [human] Verify that the Tunnel structure itself (4x4 surface building) still correctly blocks surface movement — only the underground expansions should be walkable.

## Expected Experience
- Step 1: Clicking on tiles above the underground HQ shows them as valid move targets. The unit moves to those tiles without issue.
- Step 2: After production completes, the Agent emerges from Side A of the Tunnel and pathfinds smoothly to the rally point. No stuck/frozen units.
- Step 3: The unit walks in a straight (or reasonable) path across the HQ's underground footprint, treating those tiles as normal terrain.
- Step 4: After the new expansion is built, surface tiles above it remain traversable. No new blocked tiles appear on the surface.
- Step 5: Attempting to move a unit through the Tunnel's own 4x4 footprint causes the unit to path around it, as expected for a surface structure.
