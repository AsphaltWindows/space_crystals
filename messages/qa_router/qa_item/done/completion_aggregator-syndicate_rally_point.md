# syndicate_rally_point

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# syndicate-rally-point

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement rally point behavior for Syndicate production expansions as defined in `artifacts/designer/design/syndicate_objects.md` under 'Tunnel Expansions > Rally Point Behavior'.

Each Syndicate production expansion (e.g., Headquarters) can have a rally point set. The rally point determines what happens when a unit finishes production:

**Rally point set on the surface:**
- Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point

**No rally point, or rally point set on the parent Tunnel:**
- Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel

This creates a meaningful choice: set a rally point to get units on the battlefield automatically, or leave it unset to pool units in the network for strategic deployment later.

The HeadquartersInstanceState includes:
- RallyPoint: Coordinates | ObjectInstance | None

Rally point is set via:
- Right-click ground/object from Headquarters DefaultState → SetRallyPoint
- C (Set Rally Point) → AwaitingTarget[SetRallyPoint] → left-click

## QA Instructions

1. Build a Headquarters in a Tunnel. Produce a unit with NO rally point set.
2. Verify the unit stays in the Tunnel Network after production completes (does not auto-eject).
3. Set a rally point on the surface (right-click ground while HQ selected).
4. Produce another unit. Verify it auto-ejects from the parent Tunnel's Side A and moves to the rally point.
5. Set the rally point ON the parent Tunnel itself.
6. Produce a unit. Verify it stays in the Tunnel Network (same as no rally point).
7. Set a rally point on a distant surface location. Produce a unit. Verify it ejects and pathfinds to the rally point.
8. Set a rally point on an enemy unit. Produce a unit. Verify it ejects and attack-moves toward the enemy.
