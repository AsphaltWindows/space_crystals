# Syndicate: Units

## Guard

The Guard is the Syndicate's basic combat infantry. A heavy infantry unit with a rapid-fire fully connected attack, tougher than the GDO Peacekeeper but with shorter range.

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - HeavyInfantry

### Silhouette - 36x36
### MaxHP - 80
### PointArmor - 1
### FullArmor - 1
### SightRange - 5
### TunnelSpaceCost - 1
### Groupable - true

### UnitBaseAttributes[HeavyInfantry]:
- MaxSpeed: 5 space units/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 degrees/frame

### TurretAttributes - None

### AttackAttributes:
- AttackType: FullyConnected
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 6
- Range: 3 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 1 frame
- CooldownDuration: 1 frame
- ReloadDuration: 4 frames

### ObjectInterfaceState: BasicCombatUnitInterfaceState

## Agent

The Agent is the Syndicate's cyborg worker unit. Agents construct Tunnels and defensive structures on the surface, gather Space Crystals and Supplies, and deliver them to Tunnels. Agents have a melee attack for basic self-defense.

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - HeavyInfantry

### Silhouette - 36x36
### MaxHP - 75
### PointArmor - 1
### FullArmor - 1
### SightRange - 5
### TunnelSpaceCost - 1
### Groupable - false

### UnitBaseAttributes[HeavyInfantry]:
- MaxSpeed: 6 space units/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 degrees/frame

### TurretAttributes - None

### AttackAttributes:
- AttackType: FullyConnected (Melee)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 6
- Range: melee (adjacent contact)
- AimDuration: 2 frames
- FiringDuration: 4 frames
- CooldownDuration: 1 frame
- ReloadDuration: 9 frames

### Gathering

Agents gather resources and deliver them to Tunnels. Crystal and Supply drop-offs use separate Tunnel sides (B and C respectively), so they do not block each other.

#### Space Crystals
- CarryCapacity: 50 Space Crystals per load
- MiningDuration: 80 frames (5 seconds)
- DropOffDuration: 64 frames (4 seconds)
- Drop-off side: Tunnel Side B
- MiningReach: 8 space units — an Agent can mine once its Silhouette is within 8 su of the patch's edge (placeholder)
- DropOffReach: 8 space units — within 8 su of Side B (placeholder)
- Patch sharing: any number of Agents may mine the same Space Crystals patch at the same time (unlike Drones, see Colonists > Drone > Patch Occupancy). The only queue in an Agent's trip is at the Tunnel's Side B.

Because a Tunnel's crystal side accepts one drop-off at a time, a Tunnel takes in at most 50 Space Crystals per 4 seconds = **12.5 Space Crystals per second**. Saturation by distance from Side B (Agent speed 1.5 grid units per second; trip = 5 s mining + 4 s drop-off + walking both ways):

| Distance to Side B (grid units) | Trip time | Per Agent (SC/s) | Payback (100 SC) | Agents to saturate | Income with 2 Agents | 3 Agents | 4 Agents |
|---|---|---|---|---|---|---|---|
| 0 | 9.0 s | 5.6 | 18 s | 2.25 | 11.1 | 12.5 | 12.5 |
| 1 | 10.3 s | 4.8 | 21 s | 2.58 | 9.7 | 12.5 | 12.5 |
| 2 | 11.7 s | 4.3 | 23 s | 2.92 | 8.6 | 12.5 | 12.5 |
| 3 | 13.0 s | 3.8 | 26 s | 3.25 | 7.7 | 11.5 | 12.5 |
| 4 | 14.3 s | 3.5 | 29 s | 3.58 | 7.0 | 10.5 | 12.5 |
| 5 | 15.7 s | 3.2 | 31 s | 3.92 | 6.4 | 9.6 | 12.5 |
| 6 | 17.0 s | 2.9 | 34 s | 4.25 | 5.9 | 8.8 | 11.8 |

Rationale: the drop-off time (not carry or mining time) sets the ceiling, so lengthening it lowers the per-Tunnel ceiling while keeping the number of Agents (and Tunnel Space) needed to saturate a Tunnel low. The 5-second mining time deliberately makes distance matter: 3 Agents saturate a Tunnel only when the crystals are within ~2 grid units of Side B, and 2 Agents never fully saturate. This incentivizes placing Tunnels as close to the crystals as possible (with Side B facing them). The Agent's payback is faster than the Extraction Plate's (22.5 s) only within ~1 grid unit, and slower beyond — tight Tunnel placement is what makes Agents efficient. Agents take 15 seconds to build at the Hideout, so the Syndicate's early ramp is not instant even though its ceiling is low.

#### Supplies
- CarryCapacity: 1 Supply per trip
- PickUpDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side C

### Agent ObjectInterfaceState

#### DefaultState commands:
- **A: Build Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90° clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **B: Drop Off Resources** — targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side C for supplies). Always visible, greyed out when Agent is not carrying resources.
- **Build Laser Turret** (hotkey TBD) — enters AwaitingPlacement for a Laser Turret (StateOnlyTransition), same placement flow as Build Tunnel. Left-click a valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Visible but greyed out until the player owns an Optics Lab.

#### Unit Commands:
- **Move** — right-click ground
- **Stop** — panel button
- **Attack** — right-click enemy unit/building (melee)
- **Enter** — right-click own Tunnel (when not carrying resources)
- **Gather** — right-click Space Crystals patch or Supply Delivery Station
- **Drop Off Resources** — right-click own Tunnel (when carrying resources), or via panel button B
- **Build Tunnel** — via panel button A

#### Right-Click Resolution:
- Space Crystals patch → Gather crystals
- Supply Delivery Station → Gather supplies
- Own Tunnel (carrying resources) → Drop off resources (auto-routes to correct side)
- Own Tunnel (not carrying resources) → Enter
- Enemy unit/building → Attack (melee)
- Ground → Move

Multiple Agents can be selected simultaneously. Right-click commands are issued to all selected Agents even though the Agent is ungroupable (no shared control panel — the panel displays one Agent's interface).

### Building

Agents construct Tunnels and surface defensive structures (see Surface Defensive Structures) on the surface. Defensive structures use the same construction flow as Tunnels (below). Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction.

#### Tunnel Construction Flow
1. Agent receives build command and walks to the target location
2. Construction begins — the partially-built Tunnel appears at the location with the Agent inside it. The Tunnel starts at 10% HP (ConstructionHP Rule). The Agent is untargetable for the duration of construction.
3. **If construction completes**: The Tunnel becomes operational and the Agent is inside the Tunnel Network, available for redeployment from any Tunnel.
4. **If the partially-built Tunnel is destroyed**: The Agent survives and emerges at the location. The Tunnel is lost and any Supplies spent are lost.
5. **If construction is cancelled by the player**: The Agent survives and emerges at the location, same as destruction. A rubble animation plays (cosmetic only, no gameplay effect). 100% of the Supplies cost is refunded.

#### Cancel Construction
While a Tunnel is under initial construction (Agent inside, not yet operational), the owning player may cancel it:
- **X: Cancel Construction**: cancels the in-progress initial build (CommandIssuingTransition). The Agent survives and emerges at the location. Full refund of Supplies cost. Only available while the Tunnel is under initial construction — mutually exclusive with the post-operational **X: Cancel Upgrade** command, since a Tunnel is never in both states at once.
