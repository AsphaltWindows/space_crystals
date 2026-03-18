# Ticket: GDO Power Plant and Barracks

## Current State
No GDO power generation or infantry production structures exist. There is no build queue, rally point, or power grid mechanic.

## Desired State
Implement the Power Plant and Barracks structures:

**Power Plant** (Structure):
- Faction: GlobalDefenseOrdinance
- Size: 2x2, Symmetry: AAAA, MaxHP: 350, PointArmor: 1, FullArmor: 4, SightRange: 3
- BuildRadiusExtension: 1, Power: +20
- ObjectInterfaceState: None (info display only)
- Built by Deployment Center (150 SC, 160 frames)

**Barracks** (Structure):
- Faction: GlobalDefenseOrdinance
- Size: 3x2, Symmetry: ABAC, MaxHP: 300, PointArmor: 1, FullArmor: 6, SightRange: 4
- BuildRadiusExtension: 2, Power: -30
- Units exit from B side
- BarracksInstanceState: RallyPoint (Coordinates|ObjectInstance|None), BuildQueue (array of ObjectEnum, max 5), CurrentBuild (ObjectEnum|None), CurrentBuildProgress (frames|None)
- Produces: Peacekeeper (50 SC, 80 frames / 5 seconds)
- Build queue: max 5 entries. Cancel removes last entry and refunds full cost.
- RallyPoint behavior: when a unit spawns, if RallyPoint is set, it receives the default right-click command resolved against the RallyPoint target (Ground->Move, Enemy->Attack, Friendly/Neutral->Move to object). If RallyPoint references a destroyed object, it resets to None.

**Barracks Interface**:
- Right-click Ground: SetRallyPoint to location
- Right-click Object: SetRallyPoint to object
- Build Peacekeeper (CommandIssuingTransition): deducts 50 SC, adds to BuildQueue. Available only if queue < 5 and player has sufficient SC and Unit Control.
- Cancel Production (CommandIssuingTransition): removes last queue entry, refunds full cost. Available only if queue not empty.

## Justification
Defined in `features/gdo_objects.md` (PowerPlant and Barracks sections). Power Plant is the prerequisite for Supply Tower and provides the power resource. Barracks is the primary infantry production facility with standard RTS build queue and rally point mechanics.

## QA Steps
1. Place a Power Plant via Deployment Center. Verify it provides +20 power to the GDO player's power grid.
2. Verify Power Plant has no interactive commands — only an info display.
3. Place a Barracks via Deployment Center. Verify it consumes -30 power.
4. Select the Barracks. Verify "Build Peacekeeper" and "Cancel Production" commands are available.
5. Click Build Peacekeeper with sufficient SC (50) and Unit Control. Verify SC is deducted and a Peacekeeper enters the BuildQueue.
6. Queue 5 Peacekeepers. Verify "Build Peacekeeper" becomes unavailable at max queue.
7. Click Cancel Production. Verify the last queue entry is removed and 50 SC is refunded.
8. Wait for a Peacekeeper to finish production. Verify it spawns exiting from the B side of the Barracks.
9. With no RallyPoint set, verify the spawned unit has no command (idle at spawn point).
10. Right-click empty ground while Barracks is selected. Verify RallyPoint is set to that location.
11. Produce another Peacekeeper. Verify it spawns and immediately receives a Move command to the rally location.
12. Right-click an enemy unit to set RallyPoint. Produce a Peacekeeper. Verify it spawns with an Attack command targeting that enemy.
13. Right-click a friendly structure to set RallyPoint. Produce a Peacekeeper. Verify it spawns with a Move command to that structure.
14. Set RallyPoint to a friendly unit. Destroy that unit. Verify RallyPoint resets to None.
15. Attempt to build a Peacekeeper with insufficient SC. Verify the command is unavailable.

## Expected Experience
The Power Plant should appear as a passive structure with a clear info display showing its +20 power contribution. The Barracks build queue should feel like a standard RTS production queue: click to queue, cancel removes last entry, units pop out from the correct side when done. Rally point should be set via right-click with a visible rally indicator on the map. Spawned units should immediately obey the rally command without visible delay.
