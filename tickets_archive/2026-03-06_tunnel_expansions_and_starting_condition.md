# Ticket: Tunnel Expansions and Starting Condition

## Current State
The Tunnel structure and Tunnel Area systems are defined (see `tunnel_structure_and_network` and `tunnel_area_and_construction_rules` tickets), but there is no implementation of underground expansion buildings that occupy Tunnel Area space, nor a starting condition for Syndicate players.

## Desired State
Implement the Tunnel Expansion system and the Syndicate starting condition:

### Tunnel Expansions (General)
- Underground buildings constructed within a Tunnel's Tunnel Area
- Placed spatially on the underground grid, occupying cells like surface buildings
- Invisible to enemies without detection mechanics
- Surface units can walk over expansion locations (no surface collision)
- All Syndicate units are produced by Tunnel expansions (not by Tunnels themselves)
- Each expansion has a rally point that determines whether produced units emerge from the parent Tunnel or remain in the Tunnel Network

### Headquarters (T1 Expansion)
- Entity Type: Structure Type (Underground)
- Tier Requirement: 1 (can be built in any Tier 1+ Tunnel)
- Produces: Agent (100 SC, 160 frames / 10 seconds)
- Unique: No (multiple can be built)
- Size: TBD (open question in feature spec)
- HP/Armor/Cost: TBD (open question in feature spec)

> **Note**: Guard was removed from HQ production. Guard will be produced by a separate T1 underground expansion (TBD).

### Starting Condition
- Syndicate player starts with:
  - 1 Tier 1 Tunnel (placed on map)
  - 1 pre-built Headquarters expansion inside that Tunnel's area

## Justification
Implements the Syndicate unit production pipeline as specified in `features/syndicate_objects.md`. Without expansions, Tunnels are purely transit structures with no production capability. The Headquarters is the minimum viable expansion needed for the faction to function — it produces the Agent (worker unit) that forms the early-game roster. Guard production will be handled by a separate TBD expansion. The starting condition ensures Syndicate players begin with a functional base.

## QA Steps
1. Place a Tier 1 Tunnel — verify the Tunnel Area can accept underground expansion buildings
2. Construct a Headquarters inside the Tunnel Area — verify it occupies grid cells within the area
3. Verify the Headquarters is invisible to an enemy player without detection
4. Move a surface unit over the Headquarters location — verify no collision (unit walks over it)
5. Set the Headquarters rally point to emerge from the parent Tunnel — produce an Agent and verify it appears at the Tunnel's Side A on the surface
6. Set the rally point to remain in the Tunnel Network — produce a unit and verify it stays inside the network
7. Build a second Headquarters in the same or different Tunnel — verify it is allowed (not unique)
8. Attempt to build a Headquarters in a Tunnel that has no remaining Tunnel Area space — verify placement is rejected
9. Start a new Syndicate game — verify the player begins with 1 Tier 1 Tunnel and 1 pre-built Headquarters inside it
10. Verify the starting Headquarters is immediately functional (can produce Agents from game start)

## Expected Experience
- Underground expansions appear in the Tunnel Area view, occupying grid cells
- Headquarters produces Agent units; produced units either emerge from the parent Tunnel's entrance or stay in the network based on rally point
- The starting condition gives the Syndicate player an immediately functional base with one Tunnel and one Headquarters
- Enemies cannot see expansions without detection — scouting the surface above reveals nothing
- Surface units move freely over underground expansion locations
