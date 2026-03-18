# Ticket: Fix Pathfinding Zigzag (No Diagonal Movement) and Unit Oscillation

## Current State
Two pathfinding bugs exist:

1. **No diagonal movement**: The pathfinding algorithm only considers cardinal neighbors (N/S/E/W), producing zigzag/staircase paths even on open terrain. Units never move diagonally.

2. **Oscillation/fidgeting**: Units sometimes get stuck oscillating back and forth between two positions, never reaching their destination. This occurs on open, flat terrain with no obstacles, suggesting the issue is in waypoint arrival detection or movement dampening rather than obstacle avoidance.

## Desired State
1. **Diagonal movement**: The pathfinding algorithm should consider 8-directional neighbors (including NE/NW/SE/SW diagonals) with appropriate diagonal cost weighting (sqrt(2) for diagonals vs 1.0 for cardinals). Units should take smooth, direct paths across open terrain.

2. **No oscillation**: Units should reliably arrive at waypoints and their final destination without fidgeting. Waypoint arrival detection should use a reasonable threshold to prevent overshoot/undershoot loops. Units should come to rest once they reach their target.

## Justification
Per `features/unit_system.md`, units have 5 movement models with continuous rotation and sophisticated locomotion physics. Cardinal-only pathfinding produces visually unacceptable zigzag paths that contradict the intended movement behavior. The oscillation bug makes units unusable in gameplay. Both issues were identified during QA testing and confirmed as implementation bugs via forum topic `pathfinding_zigzag_and_oscillation.md`.

## QA Steps
1. Start a new game as GDO faction.
2. Spawn a Peacekeeper unit.
3. Right-click to move the unit to a destination that is diagonally offset (e.g., both X and Z differ from the unit's position) on open terrain.
4. Observe the unit's path — verify it moves in a smooth diagonal or near-diagonal line, not a zigzag staircase pattern.
5. Right-click to move the unit to a distant location across open terrain.
6. Observe that the unit reaches the destination and stops cleanly — no fidgeting, oscillation, or back-and-forth movement.
7. Issue several move commands in quick succession to different locations.
8. Verify the unit transitions smoothly between paths without getting stuck.
9. Move a unit to a location very close to its current position (1-2 tiles away).
10. Verify the unit arrives and stops without oscillating.

## Expected Experience
- Units take visually smooth, direct paths across open terrain — no zigzag/staircase patterns.
- Units arrive at their destination and stop cleanly within a reasonable threshold.
- No fidgeting, oscillation, or back-and-forth movement at any point during or after movement.
- Short-distance moves complete correctly without the unit getting stuck.
