# colonists_beacon_stats_and_warp_in_mechanic

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Fully defined the Colonists Beacon structure and formalized the Warp In mechanic (how every other Colonists building comes into existence), and renamed the Colonists worker unit from "Prospector" to "Drone" throughout the design docs. Changed `artifacts/designer/design/colonists_objects.md`, `artifacts/designer/design/entities.md`, and `artifacts/designer/design/factions.md`.

Note: two earlier pending feature_requests (`colonists_prospector_full_definition` and `colonists_shields_and_prospector_beacon_deployment`, still in `messages/task_splitter/feature_request/pending/` at time of writing) have already been edited in place to use "Drone" instead of "Prospector" — no separate rename task is needed for that unit's own definition. This message only covers the NEW Beacon/Warp In content below.

## Rename: Prospector -> Drone
Every design-doc reference to the Colonists worker unit now reads "Drone". This affects the unit's own definition (in the two pending messages above) as well as its mentions in `factions.md` (DisplayHud/resource descriptions) and throughout `colonists_objects.md`.

## Beacon — full stats (colonists_objects.md)
The Beacon (Colonists' foundational structure, created by a Drone deploying into it) now has complete stats, previously all TBD:
- Size: 2x2 grid units
- MaxHP: 120
- MaxShields: 10
- PointArmor: 0
- FullArmor: 0
- SightRange: 4 grid units
- Destructible: true
- Groupable: true
- BeaconCapacity provided: 5
- SymmetryType: AAAA

Still open (not in this change): Beacon's full ObjectInterfaceState (presumably exposes the Warp In option as a panel command).

## New: Warp In mechanic (colonists_objects.md, new section; entities.md, new Placement Validation category)
This is how every Colonists building other than the Beacon itself gets constructed — there is no separate "worker builds from outside" flow like Cults Recruits or Syndicate Agents. Instead:
- The player selects an established Beacon, selects Warp In, and picks a target building from the Beacon's Constructs list (same pattern as GDO's DeploymentCenter Constructs list and Cults' Recruit Constructs list — each buildable structure has its own resource cost and WarpInTime, defined per-building, NOT derived from cost via any formula).
- The building's footprint is shown as a ghost overlay centered on the Beacon's current location. The player can rotate it in 90-degree increments and flip it horizontally/vertically before confirming (standard Structure Type placement controls, see entities.md > Structure Type).
- All footprint tiles must be clear (Buildable, unoccupied) EXCEPT the Beacon's own footprint, which will be consumed by the transformation. Ghost is tinted red if blocked elsewhere, green if clear — same visual convention as other placement ghosts in the game.
- Left-click confirms and starts the transformation. The Beacon itself transforms in place into the target building (same in-place-transformation pattern as the existing Drone -> Beacon deployment, generalized to Beacon -> any buildable structure).
- HP/Shields interpolate linearly from the Beacon's MaxHP/MaxShields to the target building's MaxHP/MaxShields over the building's WarpInTime (same interpolation formula already used for Drone -> Beacon deployment, just parameterized by WarpInTime instead of a fixed 48 frames).
- The transforming structure is vulnerable throughout. If destroyed mid-Warp-In, the Beacon is destroyed along with it (no partial building, no refund).
- Cancellable at any point before completion: cancelling refunds 75% of the building's resource cost (rounded down — same refund rate as Cults' Cancel Construction) and the structure reverts to being a normal, undamaged Beacon.
- The resulting structure continues to provide the Beacon's BeaconCapacity (5) in addition to its new function — Warping In a building does not lose that capacity contribution.

`entities.md`'s "Placement Validation" section gained a third category, "Warp In (Colonists Beacon transformation)", alongside the existing "Direct Placement" and "Worker-Built Structures" categories, cross-referencing colonists_objects.md for the full mechanic.

## Still open (not in this change)
- Beacon's full ObjectInterfaceState.
- Egress Node's own stats, size, and placement mechanics (still TBD, and still distinct from the Beacon/Warp In flow — Space Crystals are dropped off there, not at Beacons; it's unclear yet whether the Egress Node itself is created via Warp In or some other method).
- Colonists combat units — none defined yet.
- Shield regen upgrades — still a placeholder for future discussion.

## QA Instructions

**Beacon stats:**
1. Deploy a Drone into a Beacon (existing mechanic) and let it complete. Select the resulting Beacon and confirm: Size 2x2 grid units, MaxHP 120, MaxShields 10, PointArmor 0, FullArmor 0, SightRange 4 grid units.
2. Box-select two Beacons together — confirm they group into a shared selection (Groupable: true).
3. Confirm the Beacon contributes 5 to the Colonists' Beacon Capacity HUD total (visible in the faction HUD display, capped at 200 total).

**Warp In:**
1. Select an established Beacon, choose Warp In, and pick a buildable structure. Confirm a ghost overlay of that building's footprint appears centered on the Beacon's location, and that it can be rotated (90-degree increments) and flipped before confirming.
2. Try to Warp In at a location where the building's footprint (excluding the Beacon's own 2x2) overlaps another structure or unbuildable terrain — confirm the ghost shows red and placement cannot be confirmed. Move to a clear location and confirm the ghost turns green and placement can be confirmed.
3. Confirm clicking to confirm consumes the Beacon and begins an in-place transformation, during which HP and Shields visibly interpolate from the Beacon's stats (120 HP / 10 Shields) toward the target building's stats over that building's WarpInTime (not an instant swap).
4. Attack the transforming structure mid-Warp-In and destroy it — confirm the Beacon is destroyed entirely (no partial building survives, no refund).
5. Start a Warp In and cancel it before completion — confirm you receive 75% of the building's resource cost back (rounded down) and the structure reverts to being a normal, undamaged Beacon.
6. Once a Warp In completes, confirm the resulting building still contributes 5 to the Colonists' Beacon Capacity total, on top of whatever its normal function is (e.g., production, resource storage).

**Rename check:**
7. Confirm the Colonists worker unit is named "Drone" everywhere in the UI (unit portrait/tooltip, selection panel, production menus) — there should be no remaining references to "Prospector".
