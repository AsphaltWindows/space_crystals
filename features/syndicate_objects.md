# Feature: Syndicate Objects

## Overview
Syndicate faction structures and mechanics, centered on the Tunnel Network system — visible surface entry points with invisible underground expansion zones.

## Design Sources
- `design/syndicate_objects.md`
- `design/factions.md` (Syndicate resource system)

## Specifications

### Tunnel Network
The Tunnel Network is the collective of all Tunnels owned by a Syndicate player. Units inside the network travel between Tunnels freely. Entry/exit requires a Tunnel whose tier meets the unit's transit requirement.

#### Transit Tier Requirements
| Tier | Unit Bases Allowed |
|------|-------------------|
| Tier 1+ | LightInfantry, HeavyInfantry |
| Tier 2+ | WheeledVehicle, TrackedVehicle, DrillUnit, HoverVehicle, Mech |
| Tier 3+ | HoverCraft, Glider |

> **Note**: Design source lists Tier 1 as "Infantry (Heavy Infantry)" — LightInfantry is inferred from the category label and design update summary ("Infantry transit"). All other tiers exhaustively list their bases.

---

### Tunnel (Structure)
The Syndicate's core surface structure. Entry/exit point to the Tunnel Network and anchor for underground Tunnel Area. Each side has a distinct function.

- **Size**: 4x4
- **SymmetryType**: ABCD
- **Destructible**: true
- **Groupable**: false (Ungroupable — each Tunnel is always its own SelectionGroup)
- **SightRange**: 5

#### Side Functions
- **Side A**: Unit entrance/exit (units enter and emerge from the Tunnel Network)
- **Side B**: Crystal drop-off (one Agent at a time)
- **Side C**: Supply drop-off (one Agent at a time)
- **Side D**: Back wall (no gameplay function)

Crystal and supply drop-offs are on separate sides, so one crystal delivery and one supply delivery can occur simultaneously.

### Tunnel Tiers
Tunnels have 3 upgrade tiers. Each tier increases HP, Tunnel Area radius, Tunnel Space, and unlocks higher-tier buildings and unit transit.

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
Since Tunnel is Ungroupable, the SelectionGroup always contains exactly one Tunnel instance, so the ObjectInterfaceState can read that specific Tunnel's game state (tier, current operation, etc.).

#### DefaultState commands:
- **A: Upgrade Tunnel** — Upgrades to next tier (CommandIssuingTransition). Costs Supplies per upgrade cost formula. Unavailable if already Tier 3 or if currently performing an operation.
- **B: Expand Tunnel** — Enters ExpandMenu (StateOnlyTransition). Multi-stage: select an underground expansion type, then place it within the Tunnel Area.
- **C: Eject** — Enters EjectMenu (StateOnlyTransition). Multi-stage: select units from the Tunnel Network to eject from this Tunnel.
- **X: Cancel Upgrade** — Cancels in-progress Tunnel tier upgrade (CommandIssuingTransition). Full refund of Supplies cost. Only available while an upgrade is in progress.

#### EjectMenu:
Displays a grid of unit type tiles representing all units currently in the **Tunnel Network** (not just this Tunnel). Each tile shows the unit type icon and a count of that type in the network. Unit types whose base category exceeds this Tunnel's tier are visible but greyed out (disabled).

- Click an enabled unit type tile: ejects one unit of that type from this Tunnel's Side A (CommandIssuingTransition). Ejected units are queued — a new unit begins ejecting every **8 frames minimum** (0.5 seconds), but actual throughput is limited by unit speed and collision at Side A. Standard movement and collision mechanics apply as units emerge.
- **Z**: returns to DefaultState (StateOnlyTransition)
- Escape/right-click: also returns to DefaultState (StateOnlyTransition)

#### ExpandMenu:
Displays available underground expansion types for this Tunnel's current tier. Only expansions at or below the Tunnel's tier are available. Click only works if the Tunnel is not already performing an operation (no concurrent construction/upgrade).

- Click an expansion type: enters AwaitingPlacement for that expansion
- **Z**: returns to DefaultState (StateOnlyTransition)
- Escape/right-click: also returns to DefaultState (StateOnlyTransition)

#### AwaitingPlacement (Expansion):
- Ghost preview of the expansion follows cursor within the Tunnel Area, snapped to grid. Tinted green when valid, red when invalid. Expansion must fit entirely within the Tunnel Area.
- R rotates 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically.
- Left-click valid location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState)
- **Z**: returns to ExpandMenu (StateOnlyTransition)
- Escape/right-click: also returns to ExpandMenu (StateOnlyTransition)

---

### Tunnel Area
A square underground build zone centered on the Tunnel. The radius extends outward from the 4x4 footprint in each direction, forming a square of `(radius + 4 + radius)` per side.

Underground expansions are placed spatially within the Tunnel Area on the grid, occupying cells like surface buildings. This provides:
- Physical limit on expansions per Tunnel (based on area size and expansion footprints)
- Meaningful scouting with detection (enemies can discover layout)
- Consistent placement mechanics with GDO's surface building system

### Tunnel Area Non-Overlap Rule
- Non-overlap check uses the **current tier's area**, not maximum potential
- Upgrading is blocked if the enlarged area would overlap another Tunnel's current area
- Strategic implication: Tier 1 Tunnels can be placed closer together for early aggression at the cost of upgrade potential; wider spacing preserves late-game upgrades

### Construction and Upgrade Rules
A Tunnel can perform only one operation at a time:
- Constructing an underground expansion, **OR**
- Upgrading to the next tier

Cannot do both simultaneously. Upgrading locks the Tunnel out of expansion construction for the upgrade duration.

### Construction Time
480 frames (30 seconds). An Agent must be present for the duration.

### Starting Condition
The Syndicate player starts with 1 Tier 1 Tunnel and 1 pre-built Headquarters expansion inside it.

### Cost Scaling
All Tunnel construction and upgrades cost Supplies. Costs scale based on existing infrastructure.

#### Construction Cost (New Tunnel)
Cost = current number of Tunnels owned (all tiers), in Supplies.
- 1st Tunnel: 0 Supplies (player starts with this Tunnel)
- 2nd Tunnel: 1 Supply
- 3rd Tunnel: 2 Supplies
- 4th Tunnel: 3 Supplies
- 5th Tunnel: 4 Supplies

#### Upgrade to Tier 2
Cost = 2 + 2 x (number of T2+ Tunnels owned), in Supplies. Higher-tier Tunnels count toward lower-tier cost scaling.
- 1st T2: 2 Supplies
- 2nd T2: 4 Supplies
- 3rd T2: 6 Supplies

#### Upgrade to Tier 3
Cost = 3 + 3 x (number of T3 Tunnels owned), in Supplies.
- 1st T3: 3 Supplies
- 2nd T3: 6 Supplies
- 3rd T3: 9 Supplies

---

### Tunnel Expansions
Underground buildings constructed within a Tunnel's Tunnel Area. Expansions are invisible to enemies without detection and can be walked over by surface units. All Syndicate units are produced by Tunnel expansions. Produced units either emerge from the parent Tunnel or remain in the Tunnel Network depending on the expansion's rally point.

#### Rally Point Behavior
Each production expansion can have a rally point set. This determines what happens when a unit finishes production:

- **Rally point set on the surface**: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point.
- **No rally point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel.

#### Headquarters
- **Entity Type**: Structure Type (Underground)
- **Size**: 2x2
- **Tier Requirement**: 1 (can be built in any Tier 1+ Tunnel)
- **Cost**: 200 Space Crystals
- **Build Time**: 400 frames (25 seconds)
- **HP**: 400
- **PointArmor**: 1
- **FullArmor**: 4
- **Produces**:
  - Agent: 100 Space Crystals, 160 frames (10 seconds)
  - Guard: 125 Space Crystals, 120 frames (7.5 seconds)
- **Unique**: No (can build multiples)
- Player starts with one pre-built in their starting Tunnel

**HeadquartersInstanceState**:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 5)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

**ObjectInterfaceState[Headquarters]**:
- Right-click Ground/Object: SetRallyPoint
- **Q: Build Agent** (CommandIssuingTransition): deducts 100 SC, adds Agent to BuildQueue. Requires queue < 5, sufficient SC.
- **W: Build Guard** (CommandIssuingTransition): deducts 125 SC, adds Guard to BuildQueue. Requires queue < 5, sufficient SC.
- **X: Cancel Production** (CommandIssuingTransition): removes last BuildQueue entry, full refund. Only if queue non-empty.
- **C: Set Rally Point** (StateOnlyTransition -> AwaitingTarget[SetRallyPoint]): left-click ground/object sets rally (CommandIssuingTransition, returns to DefaultState).

---

### Agent (Unit - HeavyInfantry)
The Syndicate's cyborg worker unit. Agents construct Tunnels and defensive structures on the surface, gather Space Crystals and Supplies, and deliver them to Tunnels. Agents have a melee attack for basic self-defense.

- **Faction**: TheSyndicate
- **UnitBase**: HeavyInfantry
- **Silhouette**: 36x36 space units
- **MaxHP**: 75, **PointArmor**: 1, **FullArmor**: 1
- **SightRange**: 5
- **TunnelSpaceCost**: 2
- **Groupable**: false (Ungroupable — each Agent is its own SelectionGroup, but right-click commands are issued to all selected Agents simultaneously)

#### Movement (TurnRateMovement)
- MaxSpeed: 6 su/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 deg/frame

#### TurretAttributes
None (HeavyInfantry, no turret)

#### AttackAttributes
- AttackType: FullyConnected (Melee)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 6
- Range: melee (adjacent contact)
- AimDuration: 2 frames
- FiringDuration: 4 frames
- CooldownDuration: 1 frame
- ReloadDuration: 9 frames

#### Gathering
Agents gather resources and deliver them to Tunnels. Crystal and Supply drop-offs use separate Tunnel sides (B and C respectively), allowing simultaneous deliveries.

**Space Crystals**:
- CarryCapacity: 50 Space Crystals per load
- MiningDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side B

**Supplies**:
- CarryCapacity: 1 Supply per trip
- PickUpDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side C

#### Agent ObjectInterfaceState
Since Agent is Ungroupable, the SelectionGroup always contains one Agent instance, so the panel displays that Agent's interface. However, right-click commands are issued to **all selected Agents** simultaneously despite the ungroupable status.

**DefaultState commands:**
- **A: Build Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **B: Drop Off Resources** — targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side C for supplies). Always visible, greyed out when Agent is not carrying resources.

**Unit Commands:**
- Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel

**Right-Click Resolution:**
| Right-Click Target | Command Issued |
|---|---|
| Crystal field | Gather crystals |
| Supply source | Gather supplies |
| Own Tunnel (carrying resources) | Drop off resources (auto-routes to correct side) |
| Own Tunnel (not carrying resources) | Enter |
| Enemy unit/building | Attack (melee) |
| Ground | Move |

> **Note**: The Agent's right-click resolution is more complex than BasicCombatUnitInterfaceState — it adds resource-context-sensitive Tunnel interaction (carry state determines Enter vs Drop Off) and Gather for resource targets. The Agent does NOT use BasicCombatUnitInterfaceState.

#### Building
Agents construct Tunnels and defensive structures on the surface. Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction. The Agent must remain present at the construction site for the full build duration.

**Tunnel Construction Flow:**
1. Agent receives build command and walks to the target location
2. Construction begins — the partially-built Tunnel appears at the location. The Tunnel starts at **10% HP** (ConstructionHP Rule). The Agent embeds inside the Tunnel and becomes **untargetable** for the duration of construction.
3. HP increases linearly during construction: `HP = MaxHP x (10% + 90% x construction_progress)`
4. **If construction completes**: The Tunnel becomes operational. The Agent is placed inside the Tunnel Network, available for redeployment from any Tunnel.
5. **If the partially-built Tunnel is destroyed**: The Agent survives and emerges at the Tunnel's location. The Tunnel is lost and any Supplies spent are lost.

---

### Guard (Unit - HeavyInfantry)
The Syndicate's basic combat infantry. A heavy infantry unit with a rapid-fire fully connected ranged attack, tougher than the GDO Peacekeeper but with shorter range.

- **Faction**: TheSyndicate
- **UnitBase**: HeavyInfantry
- **Silhouette**: 36x36 space units
- **MaxHP**: 80, **PointArmor**: 1, **FullArmor**: 1
- **SightRange**: 5
- **TunnelSpaceCost**: 2
- **Groupable**: true

#### Movement (TurnRateMovement)
- MaxSpeed: 5 su/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 deg/frame

#### TurretAttributes
None (HeavyInfantry, no turret)

#### AttackAttributes
- AttackType: FullyConnected (Ranged)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 6
- Range: 3 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 1 frame
- CooldownDuration: 1 frame
- ReloadDuration: 4 frames

#### ObjectInterfaceState
BasicCombatUnitInterfaceState

> **Note**: MaxSpeed is a UnitBaseAttribute (HeavyInfantry defines that it has one), but the value is set per unit type. Agent has MaxSpeed 6, Guard has MaxSpeed 5.

---

## Production Chain Summary
```
Headquarters --> Agent
Headquarters --> Guard
```

## Dependencies
- `factions_and_resources` (Syndicate resource definitions: Supplies, Tunnel Space, Space Crystals)
- `entity_system` (Structure Type, Structure Instance, structure placement, FlipHorizontal/FlipVertical)
- `unit_system` (HeavyInfantry base, unit base categories for transit tier mapping)
- `combat_system` (Agent's FullyConnected Melee attack)

## Open Questions
- Full expansion building roster beyond Headquarters (higher-tier expansions TBD)
- Syndicate defensive structures (Agent can build them, but none specified yet)
- Detection mechanics for underground buildings (how enemies reveal Tunnel Areas)
- Can a Tunnel be destroyed while units are inside the Network? What happens to them? (Note: during construction, the embedded Agent survives destruction — but what about units in an operational Tunnel's network?)
- Tunnel upgrade duration (construction time is 480 frames, but upgrade time not specified)
- CommandIndicators for Agent-specific commands (Gather, Drop Off Resources, Build Tunnel) — colors/types not specified
- Gathering behaviors: how do GatherCrystals/GatherSupplies behaviors work? (approach resource, mining/pickup duration, return to Tunnel side, drop-off duration — implied by stats but no formal behavior algorithm)
- Drop Off Resources behavior: does the Agent auto-return to gathering after drop-off? (standard RTS convention but not specified)
- Eject queue behavior when Side A is physically blocked (units pile up? queue stalls? overflow handling?)
