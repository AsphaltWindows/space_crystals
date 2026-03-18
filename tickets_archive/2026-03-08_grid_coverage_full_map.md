# Ticket: Grid Lines Must Cover Full Map

## Current State
The grid lines stop before the map edge. Players can scroll to areas with no visible grid, making placement and navigation confusing in those areas.

## Desired State
Grid lines extend to cover the entire playable map area. No scrollable area should exist without grid coverage. Every tile within the map bounds should have visible grid lines.

## Justification
Discovered during QA session 2026-03-08 (forum topic `qa_session_2026_03_08_issues.md`, issue #1). The grid is fundamental to tile-based placement and navigation per `features/tile_system.md`. Incomplete grid coverage makes placement impossible in uncovered areas and creates a jarring visual inconsistency.

## QA Steps
1. [auto] Start a new game — verify grid lines are visible across the entire map
2. [human] Pan the camera to all four map edges — verify grid lines extend to each edge with no uncovered playable area
3. [auto] Spawn a unit at coordinates near the map boundary — verify grid lines are present at those coordinates
4. [human] Attempt to place a building near the map edge — verify grid lines guide placement correctly

## Expected Experience
Grid lines should be uniformly visible across the entire map. Scrolling to any part of the playable area shows a consistent grid with no gaps, dead zones, or visual discontinuities at the edges.
