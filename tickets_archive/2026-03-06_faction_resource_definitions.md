# Ticket: Faction Resource Definitions

## Current State
The entity system defines FactionEnum and Faction as abstract concepts. No concrete faction variants are populated and no per-faction resource type definitions exist.

## Desired State
Populate FactionEnum with the 4 concrete variants and define per-faction resource structs.

**FactionEnum variants**:
- `GlobalDefenseOrdinance` (display name: "Global Defense Ordinance")
- `TheSyndicate` (display name: "The Syndicate")
- `TheCults` (display name: "The Cults")
- `Colonists` (display name: "Colonists")

**Per-faction resource structs** (attached as components to the faction's Player entity):

**GDO**: SpaceCrystals (stockpile), Supplies (stockpile), Power (capacity — net sum), UnitControl (cap: fixed 200)
**Syndicate**: SpaceCrystals (stockpile), Supplies (stockpile), TunnelSpace (cap: 200, provided by Tunnels based on upgrade level)
**Cults**: SpaceCrystals (stockpile), UnitControl (cap: uncapped, provided by RecruitmentCenters proportional to Recruitable tiles)
**Colonists**: SpaceCrystals (stockpile), Alloys (stockpile, refined from SpaceCrystals), Essence (stockpile, refined from SpaceCrystals), Conduits (stockpile, refined from Alloys + Essence), BeaconCapacity (cap: 200, provided by Beacons)

Each resource should be categorized as either:
- **Stockpile**: a gathered/refined amount that increases and decreases
- **Capacity**: a current/available value with a cap (fixed or dynamic)

A per-player resource state struct should hold the current values for all resources belonging to that player's faction.

## Justification
Required by `features/factions_and_resources.md`. Each faction has a unique economy that drives all construction, unit production, and tech progression. These definitions are foundational data structures needed before any economy logic can be implemented.

## QA Steps
1. Verify FactionEnum has exactly 4 variants: GlobalDefenseOrdinance, TheSyndicate, TheCults, Colonists, each with correct display name.
2. Verify that each of the 4 factions has a distinct resource set defined in code.
2. Verify GDO has exactly 4 resources: SpaceCrystals, Supplies, Power, UnitControl.
3. Verify Syndicate has exactly 3 resources: SpaceCrystals, Supplies, TunnelSpace.
4. Verify Cults has exactly 2 resources: SpaceCrystals, UnitControl.
5. Verify Colonists has exactly 5 resources: SpaceCrystals, Alloys, Essence, Conduits, BeaconCapacity.
6. Verify that stockpile resources are represented as simple numeric values.
7. Verify that capacity resources track both "used" and "available" values.
8. Verify GDO UnitControl has a fixed available value of 200.
9. Verify Cults UnitControl has no hard cap (available is dynamically determined).
10. Verify Syndicate TunnelSpace and Colonists BeaconCapacity cap at 200.

## Expected Experience
Inspecting the resource definitions in code should show clearly differentiated resource sets per faction with appropriate typing. Unit tests should confirm that stockpile resources can be incremented/decremented and capacity resources track used vs. available correctly, with caps enforced where specified.
