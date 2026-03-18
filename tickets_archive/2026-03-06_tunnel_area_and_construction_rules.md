# Ticket: Tunnel Area and Construction Rules

## Current State
No Tunnel Area or underground build zone system exists. GDO has a surface-based BuildArea system, but there is no underground equivalent for Syndicate.

## Desired State
Implement the Tunnel Area system and construction/upgrade rules:

### Tunnel Area
- Each Tunnel has a square underground build zone centered on its 4x4 footprint
- The area extends outward by the tier's radius in each direction, forming a square of `(radius + 4 + radius)` per side:
  - Tier 1: radius 3 = 10x10 area
  - Tier 2: radius 4 = 12x12 area
  - Tier 3: radius 5 = 14x14 area
- Underground expansions are placed spatially within the Tunnel Area on the grid, occupying cells like surface buildings
- Placement mechanics should be consistent with GDO's surface building system (spatial grid placement)

### Non-Overlap Rule
- Non-overlap check uses the Tunnel's **current tier's area**, not its maximum potential area
- A Tunnel upgrade is **blocked** if the enlarged area would overlap with another Tunnel's current area
- Strategic implication: Tier 1 Tunnels placed close together for early aggression sacrifice upgrade potential; wider spacing preserves late-game upgrade paths

### Construction and Upgrade Rules
- A Tunnel can perform only **one operation at a time**:
  - Constructing an underground expansion, **OR**
  - Upgrading to the next tier
- Cannot do both simultaneously
- Upgrading locks the Tunnel out of expansion construction for the upgrade duration

### Cost Scaling
All costs are in Supplies.

#### New Tunnel Construction
Cost = current number of Tunnels owned (all tiers).
- 1st Tunnel: 0 Supplies (player starts with this)
- 2nd Tunnel: 1 Supply
- 3rd Tunnel: 2 Supplies
- nth Tunnel: (n-1) Supplies

#### Upgrade to Tier 2
Cost = 2 + 2 x (number of T2+ Tunnels currently owned). Higher-tier Tunnels count toward lower-tier scaling.
- 1st T2: 2 Supplies
- 2nd T2: 4 Supplies
- 3rd T2: 6 Supplies

#### Upgrade to Tier 3
Cost = 3 + 3 x (number of T3 Tunnels currently owned).
- 1st T3: 3 Supplies
- 2nd T3: 6 Supplies
- 3rd T3: 9 Supplies

### Construction Time
480 frames (30 seconds). An Agent must be present for the full duration.

## Justification
Implements the Syndicate's underground building mechanic as specified in `features/syndicate_objects.md`. The Tunnel Area system provides physical limits on expansions per Tunnel, enables meaningful scouting via detection, and mirrors GDO's surface placement mechanics for consistency. The non-overlap rule and construction mutex create strategic depth in Tunnel placement and upgrade timing.

## QA Steps
1. Place a Tier 1 Tunnel — verify a 10x10 underground build zone appears centered on the 4x4 footprint
2. Verify underground expansion buildings can be placed within the Tunnel Area on the spatial grid
3. Attempt to place an expansion outside the Tunnel Area boundary — should be rejected
4. Upgrade the Tunnel to Tier 2 — verify the area expands to 12x12
5. Upgrade to Tier 3 — verify the area expands to 14x14
6. Place two Tier 1 Tunnels with their 10x10 areas NOT overlapping — both should be valid
7. Place two Tier 1 Tunnels close together (areas would overlap if upgraded) — placement should succeed since current Tier 1 areas don't overlap
8. Attempt to upgrade one of the close Tunnels to Tier 2 — should be blocked because the enlarged 12x12 area would overlap the other Tunnel's 10x10 area
9. Place a Tunnel far enough away that upgrade to Tier 3 is possible — upgrade should succeed
10. Start constructing an underground expansion in a Tunnel — verify the Tunnel cannot simultaneously begin an upgrade
11. Start a Tunnel upgrade — verify no expansion construction can begin until the upgrade completes
12. After upgrade completes — verify expansion construction can resume
13. Build a first Tunnel — verify it costs 0 Supplies (player starts with one; 2nd Tunnel costs 1 Supply)
14. Build a third Tunnel — verify it costs 2 Supplies
15. Upgrade first Tunnel to T2 — verify cost is 2 Supplies (2 + 2x0)
16. Upgrade second Tunnel to T2 — verify cost is 4 Supplies (2 + 2x1, since one T2+ already exists)
17. Upgrade first Tunnel to T3 — verify cost is 3 Supplies (3 + 3x0)
18. Upgrade second Tunnel to T3 — verify cost is 6 Supplies (3 + 3x1)
19. Begin constructing a Tunnel — verify the Agent must remain present for 480 frames (30 seconds) for construction to complete
20. Remove the Agent mid-construction — verify construction halts or fails

## Expected Experience
- Underground build zones are clearly bounded areas tied to each Tunnel
- Expansion buildings snap to grid cells within the zone, consistent with surface building placement
- Attempting to place outside the zone or overlap another Tunnel's zone gives clear rejection feedback
- Upgrade attempts that would cause overlap are blocked with an explanation
- Only one operation (build or upgrade) proceeds at a time per Tunnel
- Tunnel and upgrade costs visibly increase as the player builds more Tunnels
