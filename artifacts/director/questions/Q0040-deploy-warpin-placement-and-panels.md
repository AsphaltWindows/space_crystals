# Q0040: Deploy and Warp In placement, and the Colonists panels

- **Date**: 2026-11-24T05:02:31-05:00
- **Blocking**: no. F, E, J1 and J2 build the proposals.
- **Docs**: `core/entities.md > Placement Validation > Worker-Built Structures`, `> Warp In`, `> Direct Placement`; `factions/colonists/economy.md > Warp In > Warp In Command`, `> Beacon`; `units.md > Drone ObjectInterfaceState`; `structures.md > Robo Portal`, `> Bio Portal` ("Full ObjectInterfaceState ... still TBD"); `systems/controls/selection_and_hud.md > No smart-casting`, `> Shift while placing`.

## Question
1. **Deploy: where must the Drone be to start?** "The Drone walks to the location, then begins deploying in place." **Proposal:** it walks to the centre of the footprint. It deploys when it arrives there, or when it stops, blocked, with its box overlapping the footprint. Otherwise the order fails.
2. **"Unoccupied" for Worker-Built and Warp In: do ground units count?** Direct Placement says they do (0094); the other two methods just say "unoccupied". **Proposal:** yes. Every ground unit counts except the deploying Drone; air and underground units don't.
3. **Warp In and visibility.** Direct Placement requires Visible tiles; Warp In lists no such rule. **Proposal:** no visibility rule. Every tile lies within about 3 GU of the Beacon's centre and its sight is 4, so it rarely matters. Where a tile isn't visible, the ghost could reveal a hidden unit.
4. **The Deploy ghost over fog.** The ghost is "tinted green when valid / red when invalid", but the order may target fog. **Proposal:** the ghost and the command check what the player knows (never-explored tiles pass), and the arrival checks the truth.
5. **Centring a 3x3 ghost (Bio Portal) on a 2x2 Beacon.** No placement is exactly centred. The 4x4 Robo Portal and the 2x2 Egress Node centre exactly. **Proposal:** of the four 3x3 placements that cover the Beacon, the one nearest the cursor.
6. **Several Beacons selected.** Beacons are Groupable, and with no smart-casting a command goes to the whole ActiveGroup, but a Warp In ghost belongs to one Beacon. **Proposal:** Warp In is available only when the ActiveGroup holds one Beacon. The alternative is to make Beacons Ungroupable, like the Drone and the Extraction Facility.
7. **Shift with Deploy.** "A Shift-click sends it and ends placement like a plain click." **Proposal:** the Deploy is queued behind the Drone's orders; behind a Gather, it starts after the next drop-off (Q0037 point 12).
8. **Panels** (the docs say TBD). **Proposal:**
   - Beacon: Q Warp In, which opens Q Egress Node, W Robo Portal, E Bio Portal and Z Back. X Cancel Warp In while warping.
   - A Beacon under deployment: X Cancel Deployment.
   - Robo Portal: Q Drone, X Cancel Production, C Set Rally Point.
   - Bio Portal: Q Sabre Legionnaire, X, C.
   - Drone: A Deploy, S Set Drop-off, Stop in its BasicCombatUnit slot, Move in Q.
   - Egress Node: F, as written.
9. **Set Drop-off with several Drones selected.** The docs say "the selected Drone(s)", but Drones are Ungroupable and the panel shows one Drone. **Proposal:** every selected Drone.

## Context
Worker-Built and Warp In are stated as lists of rules; these are the gaps between them and the interface. None blocks the sim work.

## Options
- **Point 6:** A, one Beacon only, as proposed. B, make Beacons Ungroupable. C, a Warp In per Beacon from one ghost (not possible with centred footprints).
- **Point 9:** A, every selected Drone, as proposed. B, only the Drone the panel shows.
- **Other points:** the proposal, or a rule you give.
