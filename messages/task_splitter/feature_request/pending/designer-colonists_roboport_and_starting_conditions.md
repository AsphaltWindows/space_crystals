# colonists_roboport_and_starting_conditions

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added the Colonists' Drone-production structure (Roboport), defined the faction's starting conditions, and generalized the Beacon Capacity Retention rule to a flat per-building constant. Changed `artifacts/designer/design/colonists_objects.md`.

## New: Roboport
The Colonists' Drone production structure — functionally the same pattern as GDO's Barracks or Cults' Armory (a build queue producing one unit type from a rally-pointed structure), themed as Drones "warping in" (materializing) rather than walking out an exit door.

- Entity Type: Structure Type. Size: 4x4 grid units. SymmetryType: AAAA — since it's fully symmetric, it has NO fixed exit side; a warped-in Drone may emerge from any of the 4 sides (contrast with GDO Barracks' fixed exit on the B side).
- MaxHP: 600. MaxShields: 20. PointArmor: 1. FullArmor: 16. SightRange: 4 grid units. Destructible: true. Groupable: true.
- RoboportInstanceState: RallyPoint (Coordinates | ObjectInstance | None), BuildQueue (array of ObjectEnum, max 20), CurrentBuild (ObjectEnum | None), CurrentBuildProgress (frames elapsed | None) — identical shape to BarracksInstanceState.
- RallyPoint behavior: same pattern as Barracks. On spawn, if RallyPoint is set, the new Drone is issued the default right-click command resolved against the RallyPoint target (rally on ground -> Move; rally on enemy -> Move to that location, since Drone is unarmed and cannot attack; rally on friendly/neutral object -> Move to object). If RallyPoint references a destroyed object, it resets to None.
- Produces: Drone — 50 Space Crystals, 160 frames (10 seconds). (Matches the Syndicate Agent's exact production time of 160 frames from the Hideout.)
- Built via Warp In from a Beacon, same as every other non-Beacon Colonists building.
- Full ObjectInterfaceState (production command panel layout — which button queues a Drone, etc.) still TBD.

## New: Starting Conditions
The Colonists player starts with one pre-built Egress Node and one pre-built Roboport, both already-Warped-In structures (analogous to how the Syndicate starts with a pre-built Tunnel + Hideout, and Cults starts with one RecruitmentCenter).

## Changed: Beacon Capacity Retention generalized
Previously worded as "the resulting structure continues to provide the Beacon's BeaconCapacity" (implying it might vary per building). Clarified as a flat rule: EVERY Colonists building provides 5 BeaconCapacity — the Beacon itself, and every structure Warped In from one — since they all originate from a Beacon. This applies to the two starting buildings too (Egress Node and Roboport each contribute 5 BeaconCapacity from game start).

## Still open (not in this change)
- Roboport's full ObjectInterfaceState (production command panel layout).
- Beacon's own ObjectInterfaceState (should expose Warp In as a command).
- Colonists combat units — none defined yet.
- Shield regen upgrades — still a placeholder for future discussion.

## QA Instructions

1. Start a new game as Colonists. Confirm the player begins with exactly one Egress Node and one Roboport already placed and fully built (not under construction), and no separate starting Beacon.
2. Check the Colonists Beacon Capacity HUD total at game start — confirm it reads 10 used... actually confirm the AVAILABLE Beacon Capacity total is 10 (5 from the Egress Node + 5 from the Roboport), since BeaconCapacity is a capacity provided by buildings, not consumed at start.
3. Select the Roboport and confirm its stats: Size 4x4, MaxHP 600, MaxShields 20, PointArmor 1, FullArmor 16, SightRange 4 grid units, Groupable true (box-selecting two Roboports groups them together).
4. Queue a Drone for production at the Roboport. Confirm it costs 50 Space Crystals, takes 160 frames (10 seconds) to complete, and the finished Drone appears ("warps in") at any of the Roboport's 4 sides rather than a single fixed exit — try queuing multiple Drones in a row and confirm they don't all emerge from the same spot every time (or at minimum, confirm there's no hardcoded single exit side, unlike GDO's Barracks).
5. Set a Rally Point on the Roboport (to a ground location, then separately to a friendly object, then to an enemy unit) and confirm newly-produced Drones automatically move toward the rally target each time — including confirming that rallying on an enemy results in the Drone moving toward that location rather than attacking (Drones are unarmed).
6. Warp In a new Egress Node or Roboport from a Beacon elsewhere on the map (using the existing Warp In flow) and confirm it also contributes 5 BeaconCapacity to the player's total once complete.


> Note (designer, 2026-10-18): the Syndicate "Headquarters" expansion has been renamed to "Hideout". All references in this message were updated accordingly — implement under the name Hideout.
