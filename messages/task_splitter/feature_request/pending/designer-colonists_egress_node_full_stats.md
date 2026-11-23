# colonists_egress_node_full_stats

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Fully defined the Colonists Egress Node structure, previously a stub with all stats TBD. Changed `artifacts/designer/design/colonists_objects.md`.

## Egress Node — full stats
The Egress Node (drop-off point where Drones deliver Space Crystals for transfer back to the Colonists' homeworld — distinct from the Beacon, which does NOT accept Space Crystal drop-offs) now has:
- Entity Type: Structure Type
- Size: 2x2 grid units
- MaxHP: 200
- MaxShields: 10
- PointArmor: 0
- FullArmor: 0
- SightRange: 3 grid units
- Destructible: true
- Groupable: false
- SymmetryType: AAAA

## Build method confirmed: Warp In
The Egress Node is built the same way as every other non-Beacon Colonists building — Warped In from an established Beacon (see the `colonists_beacon_stats_and_warp_in_mechanic` feature_request for the full Warp In mechanic: footprint ghost centered on the Beacon, rotate/flip, footprint must be clear except the Beacon's own tile, HP/Shields interpolate over WarpInTime, cancellable for 75% refund, destroyed if killed mid-transformation, retains the Beacon's BeaconCapacity contribution afterward).

## Drop Off
Multiple Drones may drop off Space Crystals at the same Egress Node simultaneously (no queuing/exclusivity, unlike Syndicate's Tunnel drop-off which is limited to one Agent at a time).

## Still open (not in this change)
- Egress Node's full ObjectInterfaceState — likely just an info display with no active commands (comparable to Cults' Storage), but not yet formalized.
- Colonists combat units — none defined yet.
- Shield regen upgrades — still a placeholder for future discussion.

## QA Instructions

1. Warp In an Egress Node from an established Beacon (using the Warp In flow). Confirm the resulting structure has Size 2x2 grid units, MaxHP 200, MaxShields 10, PointArmor 0, FullArmor 0, SightRange 3 grid units.
2. Box-select an Egress Node alongside another Egress Node (or any other structure) — confirm they do NOT group together into a shared control panel (Groupable: false).
3. Send multiple Drones carrying Space Crystals to the same Egress Node at the same time — confirm all of them can drop off simultaneously with no waiting/queue (contrast with Syndicate Tunnels, which only allow one Agent to drop off at a time).
4. Confirm right-clicking an Egress Node while carrying Space Crystals resolves to Drop Off Resources (per the existing Drone ObjectInterfaceState right-click resolution), and confirm right-clicking a Beacon while carrying crystals does NOT trigger a drop-off.
