# Ticket: Tunnel Structure and Network

## Current State
No Syndicate faction structures or mechanics exist. The entity system supports Structure Types and Structure Instances, and the unit system defines unit base categories, but there is no Tunnel or Tunnel Network implementation.

## Desired State
Implement the Tunnel structure type and Tunnel Network system:

### Tunnel (Structure Type)
- Size: 4x4
- SymmetryType: ABCD (each side has a distinct function)
  - Side A: Unit entrance/exit (units enter and emerge from the Tunnel Network)
  - Side B: Crystal drop-off (one Agent at a time)
  - Side C: Supply drop-off (one Agent at a time)
  - Side D: Back wall (no gameplay function)
- Destructible: true
- Groupable: false (Ungroupable — each Tunnel is always its own SelectionGroup, enabling per-Tunnel ObjectInterfaceState)
- SightRange: 5

### Tunnel Tiers
Three upgrade tiers with increasing capabilities:

| | Tier 1 (base) | Tier 2 | Tier 3 |
|---|---|---|---|
| HP | 600 | 800 | 1000 |
| PointArmor | 1 | 1 | 1 |
| FullArmor | 16 | 16 | 16 |
| Tunnel Space | 20 | 30 | 40 |
| Tunnel Area Radius | 3 (10x10) | 4 (12x12) | 5 (14x14) |
| Transit | Infantry | Infantry + Vehicles | Infantry + Vehicles + Air |
| Buildings | T1 expansions | T1 + T2 expansions | T1 + T2 + T3 expansions |

### Tunnel Network
- The Tunnel Network is the collective of all Tunnels owned by a single Syndicate player
- Units inside the network travel freely between any Tunnels in the network
- Entry and exit from the network requires a Tunnel whose tier meets the unit's transit requirement

### Transit Tier Requirements
| Minimum Tier | Unit Bases Allowed |
|-------------|-------------------|
| Tier 1+ | LightInfantry, HeavyInfantry |
| Tier 2+ | WheeledVehicle, TrackedVehicle, DrillUnit, HoverVehicle, Mech |
| Tier 3+ | HoverCraft, Glider |

## Justification
Implements the core Syndicate faction mechanic as specified in `features/syndicate_objects.md`. The Tunnel is the Syndicate's primary surface structure and the anchor for all underground expansion. The Tunnel Network and transit tier mapping are essential for Syndicate unit logistics and differentiate the faction from GDO's surface-based building system.

## QA Steps
1. Create a Syndicate player and verify a Tunnel can be placed as a 4x4 structure on the map
2. Verify the Tunnel has ABCD symmetry — each side is distinct (A: unit entrance/exit, B: crystal drop-off, C: supply drop-off, D: back wall)
3. Verify the Tunnel is destructible and Ungroupable (selecting a Tunnel always creates a single-element SelectionGroup)
4. Verify a newly placed Tunnel starts at Tier 1
5. Upgrade a Tunnel to Tier 2, then to Tier 3 — verify tier is tracked correctly
6. Place two Tunnels owned by the same player — verify they form a single Tunnel Network
7. Attempt to enter a Tier 1 Tunnel with a LightInfantry unit — should succeed
8. Attempt to enter a Tier 1 Tunnel with a WheeledVehicle unit — should be denied
9. Upgrade that Tunnel to Tier 2 and retry with WheeledVehicle — should now succeed
10. Attempt to enter a Tier 2 Tunnel with a Glider — should be denied
11. Upgrade to Tier 3 and retry with Glider — should now succeed
12. Send a unit into the network via one Tunnel and have it exit from a different Tunnel — verify free intra-network travel
13. Send an Agent to Side B (crystal drop-off) — verify it can deliver crystals
14. Send a second Agent to Side B while the first is delivering — verify the second Agent queues (one at a time per side)
15. Send an Agent to Side C (supply drop-off) simultaneously with an Agent on Side B — verify both can deliver at the same time (separate sides)
16. Verify Side D (back wall) has no interactive function

## Expected Experience
- A Tunnel appears as a 4x4 structure on the map surface with 4 distinct sides (A/B/C/D)
- Tier upgrades visually or logically change the Tunnel's tier state; HP increases to 600/800/1000
- Units of the correct base category can enter/exit via Side A; units of too-high a category are blocked with clear feedback
- Crystal drop-offs occur on Side B, supply drop-offs on Side C — one Agent per side at a time, but both sides can be active simultaneously
- Side D has no interactive function
- Units inside the network can freely choose any eligible Tunnel as an exit point
- Multiple Tunnels owned by the same player are part of one collective network
