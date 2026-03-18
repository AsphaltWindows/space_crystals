# Syndicate Objects

## Tunnel Network

The Tunnel Network is the collective of all Tunnels owned by a Syndicate player. Units inside the Tunnel Network can travel between Tunnels freely, but can only enter or exit the network through a Tunnel whose tier is sufficient for their unit base category.

### Transit Tier Requirements
- Tier 1+: Infantry (Heavy Infantry)
- Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)
- Tier 3+: Air units (Hover Craft, Glider)

## Tunnel

The Tunnel is the Syndicate's core surface structure. It serves as the entry/exit point to the Tunnel Network and defines an underground Tunnel Area where expansions (underground buildings) can be constructed. Tunnels are visible to enemies on the surface, but underground expansions within the Tunnel Area are invisible without detection.

The Syndicate player starts with one Tier 1 Tunnel and one pre-built Headquarters expansion.

### Entity Type - Structure Type

### Size - 4x4
### SymmetryType - ABCD

Each side of the Tunnel has a distinct function:
- **Side A**: Unit entrance/exit (units enter and emerge from the Tunnel Network here)
- **Side B**: Crystal drop-off (Agents deliver Space Crystals here)
- **Side C**: Supply drop-off (Agents deliver Supplies here)
- **Side D**: Back wall (no gameplay function)

Only one Agent may drop off resources at a side at a time. Because crystal and supply drop-offs are on separate sides, one crystal delivery and one supply delivery can occur simultaneously.

### Destructible - true
### Groupable - false
### SightRange - 5

### Tiers

Tunnels have 3 upgrade tiers. Each tier increases HP, Tunnel Area radius, Tunnel Space provided, and unlocks higher-tier buildings and unit transit.

| | Tier 1 (base) | Tier 2 | Tier 3 |
|---|---|---|---|
| HP | 600 | 800 | 1000 |
| PointArmor | 1 | 1 | 1 |
| FullArmor | 16 | 16 | 16 |
| Tunnel Space | 20 | 30 | 40 |
| Tunnel Area Radius | 3 (10x10) | 4 (12x12) | 5 (14x14) |
| Transit | Infantry | Infantry + Vehicles | Infantry + Vehicles + Air |
| Buildings | T1 expansions | T1 + T2 expansions | T1 + T2 + T3 expansions |

### Tunnel ObjectInterfaceState

#### DefaultState commands:
- **A: Upgrade Tunnel** — upgrades to next tier (CommandIssuingTransition). Costs Supplies per upgrade cost formula. Unavailable if already Tier 3 or if currently performing an operation.
- **B: Expand Tunnel** — enters ExpandMenu (StateOnlyTransition). Multi-stage: select an underground expansion to build, then place it within the Tunnel Area.
- **C: Eject** — enters EjectMenu (StateOnlyTransition). Multi-stage: select units from the Tunnel Network to eject from this Tunnel.
- **X: Cancel Upgrade** — cancels the in-progress Tunnel upgrade (CommandIssuingTransition). Full refund of Supplies cost. Only available while an upgrade is in progress.

#### EjectMenu:
Displays a grid of unit type tiles representing all units currently in the Tunnel Network. Each tile shows the unit type icon and a count of how many of that type are in the network. Unit types whose base category exceeds this Tunnel's tier are visible but greyed out (disabled).

- Click an enabled unit type tile: ejects one unit of that type from this Tunnel's Side A (CommandIssuingTransition). Ejected units are queued — a new unit begins ejecting every 8 frames minimum, but actual throughput is limited by unit speed and collision at Side A. Standard movement and collision mechanics apply as units emerge.
- **Z**: returns to DefaultState (StateOnlyTransition)

#### ExpandMenu:
Displays available underground expansion types for this Tunnel's current tier. Only expansions at or below the Tunnel's tier are available.

- Click an expansion type: begins construction if the Tunnel is not already performing an operation (enters AwaitingPlacement for that expansion)
- **Z**: returns to DefaultState (StateOnlyTransition)

#### AwaitingPlacement (Expansion):
- A ghost preview of the expansion follows the cursor within the Tunnel Area, snapped to the grid. Tinted green when valid, red when invalid. The expansion must fit entirely within the Tunnel Area.
- R rotates the ghost 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically.
- Left-click valid location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState)
- **Z**: returns to ExpandMenu (StateOnlyTransition)

### Tunnel Area

The Tunnel Area is a square underground build zone centered on the Tunnel. The radius extends outward from the Tunnel's 4x4 footprint in each direction, forming a square of (radius + 4 + radius) per side.

Underground expansions are placed spatially within the Tunnel Area on the grid, occupying cells just like surface buildings. This provides:
- A physical limit on the number of expansions per Tunnel
- Meaningful scouting with detection (enemies can see what was built and where)
- Consistent placement mechanics with GDO's surface building system

### Tunnel Area Non-Overlap Rule

Tunnel Areas cannot overlap with each other. The non-overlap check uses the **current tier's area**, not the maximum potential area. However, upgrading a Tunnel is blocked if the radius increase would cause its Tunnel Area to overlap with another Tunnel's current area.

This means players can place Tier 1 Tunnels closer together for early aggression, but sacrifice the ability to upgrade them later. Spacing Tunnels further apart preserves upgrade potential for the late game.

### Construction and Upgrade Rules

A Tunnel can only perform one operation at a time:
- Constructing an underground expansion, OR
- Upgrading to the next tier

It cannot do both simultaneously. Upgrading temporarily locks the Tunnel out of expansion construction for the upgrade duration.

### Cost Scaling

Tunnel construction and upgrades cost Supplies. Costs scale based on existing infrastructure.

#### Construction Cost
Cost = current Tunnels owned (all tiers count), in Supplies.
- 1st Tunnel: 0 Supplies (player starts with this Tunnel)
- 2nd Tunnel: 1 Supply
- 3rd Tunnel: 2 Supplies
- 4th Tunnel: 3 Supplies
- 5th Tunnel: 4 Supplies

#### Upgrade Costs
Higher-tier Tunnels count toward lower-tier cost calculations (a T3 Tunnel counts for both T2 and T1 cost scaling).

**Upgrade to Tier 2**: 2 + 2 x (number of T2+ Tunnels owned), in Supplies.
- 1st T2: 2 Supplies
- 2nd T2: 4 Supplies
- 3rd T2: 6 Supplies

**Upgrade to Tier 3**: 3 + 3 x (number of T3 Tunnels owned), in Supplies.
- 1st T3: 3 Supplies
- 2nd T3: 6 Supplies
- 3rd T3: 9 Supplies

#### Construction Time
480 frames (30 seconds). Agent must be present for the duration.

## Tunnel Expansions

Underground buildings constructed within a Tunnel's Tunnel Area. Expansions are invisible to enemies without detection and can be walked over by surface units. All Syndicate units are produced by Tunnel expansions.

### Rally Point Behavior

Each production expansion can have a rally point set. This determines what happens when a unit finishes production:

- **Rally point set on the surface**: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point.
- **No rally point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel.

### Headquarters

The Headquarters is a Tier 1 Tunnel expansion that produces Agents and Guards. The Syndicate player starts with one pre-built in their starting Tunnel. Additional Headquarters can be constructed in any Tier 1+ Tunnel.

#### Entity Type - Structure Type (Underground)
#### Size - 2x2
#### Tier Requirement - 1
#### Cost - 200 Space Crystals
#### Build Time - 400 frames (25 seconds)
#### HP - 400
#### PointArmor - 1
#### FullArmor - 4
#### Produces:
- Agent: 100 Space Crystals, 160 frames (10 seconds)
- Guard: 125 Space Crystals, 120 frames (7.5 seconds)

#### HeadquartersInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 5)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

#### ObjectInterfaceState[Headquarters]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Agent**: deducts 100 Space Crystals from player, adds Agent to BuildQueue. Only available if BuildQueue has fewer than 5 entries and player has sufficient Space Crystals.
- **W: Build Guard**: deducts 125 Space Crystals from player, adds Guard to BuildQueue. Only available if BuildQueue has fewer than 5 entries and player has sufficient Space Crystals.
- **X: Cancel Production**: removes last entry from BuildQueue, refunds full cost to player. Only available if BuildQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

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
### TunnelSpaceCost - 2
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
### TunnelSpaceCost - 2
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
- MiningDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side B

#### Supplies
- CarryCapacity: 1 Supply per trip
- PickUpDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side C

### Agent ObjectInterfaceState

#### DefaultState commands:
- **A: Build Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90° clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **B: Drop Off Resources** — targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side C for supplies). Always visible, greyed out when Agent is not carrying resources.

#### Unit Commands:
- **Move** — right-click ground
- **Stop** — panel button
- **Attack** — right-click enemy unit/building (melee)
- **Enter** — right-click own Tunnel (when not carrying resources)
- **Gather** — right-click crystal field or supply source
- **Drop Off Resources** — right-click own Tunnel (when carrying resources), or via panel button B
- **Build Tunnel** — via panel button A

#### Right-Click Resolution:
- Crystal field → Gather crystals
- Supply source → Gather supplies
- Own Tunnel (carrying resources) → Drop off resources (auto-routes to correct side)
- Own Tunnel (not carrying resources) → Enter
- Enemy unit/building → Attack (melee)
- Ground → Move

Multiple Agents can be selected simultaneously. Right-click commands are issued to all selected Agents even though the Agent is ungroupable (no shared control panel — the panel displays one Agent's interface).

### Building

Agents construct Tunnels on the surface. Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction.

#### Tunnel Construction Flow
1. Agent receives build command and walks to the target location
2. Construction begins — the partially-built Tunnel appears at the location with the Agent inside it. The Tunnel starts at 10% HP (ConstructionHP Rule). The Agent is untargetable for the duration of construction.
3. **If construction completes**: The Tunnel becomes operational and the Agent is inside the Tunnel Network, available for redeployment from any Tunnel.
4. **If the partially-built Tunnel is destroyed**: The Agent survives and emerges at the location. The Tunnel is lost and any Supplies spent are lost.
