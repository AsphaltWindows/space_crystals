# Ticket: Syndicate Agent Unit

## Current State
The Syndicate faction has Tunnel structures and a Headquarters expansion defined, but no unit types are implemented. The Headquarters is specified to produce Agents, but the Agent unit itself does not exist.

## Desired State
Implement the Syndicate Agent unit — a HeavyInfantry cyborg worker with combat, gathering, and building capabilities.

### Unit Definition
- **Faction**: TheSyndicate
- **UnitBase**: HeavyInfantry
- **Silhouette**: 36x36 space units
- **MaxHP**: 75
- **PointArmor**: 1, **FullArmor**: 1
- **SightRange**: 5
- **TunnelSpaceCost**: 2
- **Groupable**: true

### Movement (TurnRateMovement)
- MaxSpeed: 6 su/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 deg/frame

### Turret
None (HeavyInfantry base, no turret).

### Attack (FullyConnected Melee)
- AttackType: FullyConnected (Melee subtype)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 6
- Range: melee (adjacent contact — short fixed range per Melee subtype rules)
- AimDuration: 2 frames
- FiringDuration: 4 frames
- CooldownDuration: 1 frame
- ReloadDuration: 9 frames
- No ElevationModifier on range (Melee subtype exemption)

### Gathering
Agents gather resources and deliver them to Tunnels. Crystal and Supply drop-offs use separate Tunnel sides (B and C), allowing simultaneous deliveries by different Agents.

**Space Crystals**:
- CarryCapacity: 50 Space Crystals per load
- MiningDuration: 48 frames (3 seconds) at Space Crystal Patch
- DropOffDuration: 48 frames (3 seconds) at Tunnel Side B
- One Agent at a time at the drop-off side

**Supplies**:
- CarryCapacity: 1 Supply per trip
- PickUpDuration: 48 frames (3 seconds) at Supply Delivery Station
- DropOffDuration: 48 frames (3 seconds) at Tunnel Side C
- One Agent at a time at the drop-off side

### Building — Tunnel Construction Flow
Agents construct Tunnels and defensive structures on the surface. The Agent must remain present at the construction site for the full build duration (480 frames for Tunnels).

Detailed Tunnel construction sequence:
1. Agent receives build command and walks to the target location
2. Construction begins — the partially-built Tunnel appears. The Tunnel starts at **10% HP** (ConstructionHP Rule). The Agent embeds inside the Tunnel and becomes **untargetable** for the duration.
3. HP increases linearly during construction: `HP = MaxHP x (10% + 90% x construction_progress)`
4. **If construction completes**: The Tunnel becomes operational. The Agent is placed inside the Tunnel Network, available for redeployment from any Tunnel.
5. **If the partially-built Tunnel is destroyed**: The Agent survives and emerges at the Tunnel's location. The Tunnel is lost and any Supplies spent are lost.

### Production
- Produced by: Headquarters (T1 expansion)
- Cost: 100 Space Crystals
- Build time: 160 frames (10 seconds)

## Justification
The Agent is the Syndicate's foundational unit, required for resource gathering, base construction, and early-game self-defense. Without it, the faction cannot function — Headquarters produces Agents, and Agents are the only unit that can construct new Tunnels and gather resources. Specified in `features/syndicate_objects.md` (Agent section, lines 119-167). Depends on `fullyconnected_melee_subtype` ticket for the Melee attack implementation.

## QA Steps
1. Produce an Agent from a Headquarters — verify it costs 100 SC and takes 160 frames (10 seconds)
2. Verify the Agent spawns with correct stats: 75 HP, 1/1 armor, 36x36 silhouette, SightRange 5
3. Verify the Agent uses TurnRateMovement: issue a move command, confirm MaxSpeed 6 su/frame and 180 deg/frame turn rate
4. Verify the Agent's HeavyInfantry base properties apply (e.g., Tunnel transit at Tier 1+)
5. Order the Agent to attack a ground enemy unit — verify melee attack engages at adjacent contact range with 6 damage, correct phase durations (2f aim, 4f fire, 1f cooldown, 9f reload)
6. Verify the Agent cannot attack air units (TargetDomain: Ground)
7. Send the Agent to a Space Crystal Patch — verify it mines for 48 frames, picks up 50 SC, then returns to Tunnel Side B to drop off in 48 frames
8. Verify only one Agent can drop off crystals at Side B at a time (second Agent waits)
9. Send the Agent to a Supply Delivery Station — verify it picks up 1 Supply in 48 frames, then returns to Tunnel Side C to drop off in 48 frames
10. Verify only one Agent can drop off supplies at Side C at a time
11. Order the Agent to construct a Tunnel — verify:
    a. The partially-built Tunnel appears and starts at 10% of MaxHP (60 HP for T1)
    b. The Agent embeds inside the Tunnel and becomes untargetable
    c. HP increases linearly over 480 frames toward full MaxHP
12. Allow construction to complete — verify the Agent is placed inside the Tunnel Network (not on the surface)
13. Start another Tunnel construction, then destroy the partially-built Tunnel — verify the Agent survives and emerges at the Tunnel's location
14. Verify TunnelSpaceCost is 2 (Agent occupies 2 Tunnel Space when inside the network)
15. Verify Agent is Groupable (can be added to control groups and multi-selected with other Agents)

## Expected Experience
- Agents emerge from Tunnels (Side A) after being produced by a Headquarters
- They move at moderate infantry speed with instant acceleration and fast turning
- When ordered to attack, they close to melee range and strike with a quick attack cycle
- Gathering follows a clear loop: travel to resource, work for 3 seconds, carry resources back to Tunnel, drop off for 3 seconds, repeat
- Crystal and Supply gathering use different Tunnel sides, so both can happen at the same Tunnel simultaneously (one Agent on Side B, one on Side C)
- Construction requires the Agent to stay on-site — pulling it away interrupts the build
