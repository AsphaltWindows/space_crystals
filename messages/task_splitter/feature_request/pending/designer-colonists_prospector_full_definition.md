# colonists_prospector_full_definition

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added the full unit definition for the Colonists' Drone (worker unit), and stubbed a new structure it depends on: the Egress Node. Changed `artifacts/designer/design/colonists_objects.md`.

Note: this unit was originally named "Prospector" and was renamed to "Drone" in a later session, before this message was ever picked up by task_splitter. All references below already use the final name — implement as "Drone".

## Drone — full definition
- **UnitBase**: HoverVehicle (first unit in the game to use this base — Ground domain, DragMovement model, directional armor, turret-capable per the base but this unit has none since it's unarmed).
- **Silhouette**: 36x36 space units. **MaxHP**: 20. **MaxShields**: 10 (uses the new Colonists Shields mechanic from the prior `colonists_shields_and_prospector_beacon_deployment` feature_request). **PointArmor**: 0. **FullArmor**: 0. **SightRange**: 5 grid units. **Groupable**: false (matches the ungroupable-worker pattern used by Cults Recruit and Syndicate Agent).
- **Movement** (UnitBaseAttributes[HoverVehicle]): TurnRate 45 degrees/frame, ForwardAcceleration 0.6 space units/frame², OmniDirectionalAcceleration 0.2 space units/frame², DragRatio 0.1 per frame. This is the first unit using DragMovement's equilibrium formula for ground vehicles: MaxSpeed = (ForwardAcceleration + OmniDirectionalAcceleration) / DragRatio = (0.6 + 0.2) / 0.1 = 8 space units/frame. NOTE: this formula (MaxSpeed = (ForwardAcceleration + OmniDirectionalAcceleration) / DragRatio) had not been formally written down anywhere before this session — it was derived and confirmed with the user specifically for this unit. It should also apply to the existing GDO SupplyChopper (also DragMovement) for consistency, though that wasn't re-verified/changed this session — worth a sanity check if it comes up.
- **TurretAttributes**: None. **AttackAttributes**: None — the Drone is entirely unarmed, unlike every other faction's worker (Recruit and Agent both have a weak melee self-defense attack).
- **Gathering**: CarryCapacity 8 Space Crystals/load, MiningDuration 32 frames, DropOffDuration 2 frames. Delivers to the new Egress Node structure (see below) — NOT to the Beacon.
- **Deploying**: References the existing Beacon Deployment mechanic (Drone transforms into a Beacon) from the prior feature_request.
- **Full ObjectInterfaceState** defined: **A: Deploy** (opens AwaitingPlacement for a Beacon, same ghost-preview flow as other factions' placement commands, no spacing restriction). Unit commands: Move, Stop, Gather, Drop Off Resources, Deploy. Right-click resolution: Space Crystal patch → Gather; own Egress Node while carrying → Drop off; ground → Move; enemy/other object → Move to that location (since it can't attack, unlike armed workers which resolve enemy right-clicks to Attack). Multiple Drones can be multi-selected and commanded together despite being ungroupable, same convention as Recruit/Agent.

## New: Egress Node (stub, structure)
A brand new Colonists structure — the drop-off point where Drones deliver Space Crystals for transfer back to the Colonists' homeworld. This is explicitly separate from the Beacon (Space Crystals are NOT dropped off at Beacons). Currently only stubbed: Entity Type, Faction, Destructible. MaxHP/MaxShields/PointArmor/FullArmor/SightRange/Groupable/Size/SymmetryType and whether multiple Drones can drop off simultaneously are all still TBD — left open for a future session, same pattern as the Beacon stub.

## Still open (not in this change)
- Beacon's own stats (MaxHP/MaxShields/PointArmor/FullArmor/SightRange/BeaconCapacity/Size/SymmetryType) — still TBD.
- Egress Node's own stats and placement mechanics — still TBD.
- Warp In mechanic (how non-Beacon buildings, including presumably the Egress Node, materialize onto a Beacon) — still unformalized.

## QA Instructions

1. Select a Drone. Confirm it shows Silhouette 36x36, MaxHP 20, MaxShields 10, PointArmor 0, FullArmor 0, SightRange 5 grid units, and does NOT group with other selected Drones into a shared control panel (ungroupable).
2. Move a Drone and confirm it accelerates/decelerates with visible momentum (drag-based movement, not instant start/stop like infantry) and reaches a top speed of 8 space units/frame. Confirm it turns at 45 degrees/frame.
3. Order a Drone to attack an enemy — confirm there is no Attack command available at all (fully unarmed), and right-clicking an enemy simply moves the Drone toward that location instead.
4. Order a Drone to gather from a Space Crystal Patch — confirm it mines for 32 frames, then carries exactly 8 Space Crystals.
5. Right-click an Egress Node while carrying Space Crystals — confirm it resolves to Drop Off Resources (taking 2 frames), and confirm right-clicking a Beacon while carrying crystals does NOT trigger a drop-off (Beacons are not a valid delivery point).
6. Confirm the Drone's Deploy command (panel button A) opens a ghost-preview placement flow for a Beacon, consistent with the previously-implemented Beacon Deployment mechanic (48-frame transformation, cancellable, vulnerable, no spacing restriction between Beacons).
