# Colonists: Economy

## Beacon Deployment

Colonists build their foundational structure, the Beacon, by having a Drone deploy directly into it — the Drone unit itself transforms into the Beacon in place, rather than constructing a separate structure from outside (contrast with Cults Recruits or Syndicate Agents, which remain independent units after building something). Every other Colonists building is warped in onto an already-established Beacon (see Warp In below).

### Deploy Command
- Select a Drone
- Select the Deploy option
- Left-click a ground location
- The Drone walks to the location, then begins deploying in place

**Where the Drone starts deploying** (user, 2026-11-24): the Drone walks to the **centre of the Beacon's footprint** and starts deploying when it gets there. Safety net: if it is stopped by a blocker on the way while already overlapping the footprint, it starts deploying from where it stands. If it stops without overlapping the footprint, the order fails (the Drone stops and idles). The Beacon always appears on the footprint the ghost showed, wherever the Drone ended up. (In practice a blocker on the footprint makes the site invalid anyway; the safety net only matters for a blocker just outside it.)

### Placement
Beacon deployment follows the shared Worker-Built Structures placement rules (core/entities.md > Placement Validation > Worker-Built Structures): the Beacon's 2x2 footprint must be on Buildable, unoccupied tiles. The Deploy order may target any location, including territory hidden by fog of war, like any other building order; the Drone walks there (revealing the area on its way) and validates the site on arrival. If the site is invalid on arrival, the order is cancelled and the Drone stops and idles. There is no minimum spacing from other Beacons and no range limit from other Colonists buildings.

### Deployment Duration
48 frames (3 seconds).

### A Beacon from the first frame
(user, 2026-11-24) The moment deployment begins, the Drone **becomes a Beacon under deployment**:
- It is a **structure** on its 2x2 footprint: the footprint is claimed immediately, units can't walk onto it, it can't move, and every rule that treats structures and units differently (targeting, auto-targeting, placement) treats it as a structure.
- It is the **same object** as the Drone, so it stays selected (and stays in any control group the Drone was in).
- On **cancel**, it turns back into the Drone, standing at the centre of the footprint.

### Beacon Capacity during deployment
(user, 2026-11-24) The Drone's 1 Beacon Capacity is **freed as soon as deployment starts** (it is already a Beacon, see A Beacon from the first frame). The Beacon's 5 is provided only on **completion**. Cancelling takes the 1 back, even if that puts the player over capacity (factions/colonists/overview.md > Beacon Capacity > Over capacity). Example: at 10/10 a Drone deploys (9/10), a new Drone starts production (10/10), then the deploy is cancelled → 11/10.

### HP/Shields Interpolation
During deployment, the transforming unit's HP and Shields each linearly interpolate from the Drone's own MaxHP/MaxShields toward the Beacon's MaxHP/MaxShields as deployment progresses:
- HP = Drone MaxHP + (Beacon MaxHP - Drone MaxHP) x progress
- Shields = Drone MaxShields + (Beacon MaxShields - Drone MaxShields) x progress

Where progress is frames elapsed / 48, from 0.0 to 1.0.

**Damage stays taken** (user, 2026-11-24): the formulas give the **maximum** HP and Shields. Each frame the current value rises by the same amount as the maximum does, so damage taken before or during deployment is carried through unchanged. Example: halfway (max 70 HP) it takes 30 damage, leaving 40/70; it completes at 90/120. A Drone that starts deploying at 10/20 HP completes at 110/120. Shields regenerate normally, up to the current maximum.

### Vulnerability
The transforming unit is vulnerable to attack for the entire deployment duration. If destroyed mid-deployment, both the Drone and the in-progress Beacon are lost.

### Cancel Deployment
The player may cancel the deployment at any point before it completes. The transformation reverses immediately and the unit reverts to being a normal, fully-functional Drone (no HP/Shields loss).

**HP and Shields on cancel** (user, 2026-11-24): the Drone returns with its own maxima and the **same fraction** of HP and Shields the Beacon had at the moment of cancelling. Example: cancelled at 90/120 HP (75%) → Drone at 15/20. Cancelling never kills: the result is at least 1 HP. Rejected: restoring the values from when deployment started (deploying would be a free damage sponge, since cancelling costs nothing), and subtracting what the interpolation added (could kill on cancel).

### Completion
Once the 48 frames elapse, the transformation is final and one-way. The resulting Beacon can never revert back into a Drone.

### While transforming: only Cancel
(user, 2026-11-24) A Beacon under deployment and a building under Warp In show **only the Cancel command** in their CommandPanel (no rally point, no other commands). None of the building's functions (production, accepting deliveries, research, abilities) work until the transformation completes. Exception already covered elsewhere: a building under Warp In keeps providing its 5 Beacon Capacity (Warp In > Beacon Capacity Retention). Cancel's hotkey: **X** (Cancel Deployment / Cancel Warp In; user, 2026-11-24).

### Deploying with a load
(user, 2026-11-24) A Drone carrying a load deploys without delivering it first. The load stays with the deploying object and is **lost when deployment completes**; on **cancel** the Drone returns still carrying it.

### Placement
Beacons may be placed adjacent to or near other Beacons — there is no minimum spacing requirement between them.

## Beacon

The Colonists' foundational structure, created when a Drone deploys into it (see Beacon Deployment above). Provides Beacon Capacity. Every other Colonists building is warped in onto an established Beacon.

### Entity Type - Structure Type
### Faction - Colonists
### Size - 2x2 grid units
### MaxHP - 120
### MaxShields - 10
### PointArmor - 0
### FullArmor - 0
### SightRange - 4 grid units
### Destructible - true
### Groupable - false
(user, 2026-11-24) A Beacon is used up by a Warp In, like a worker used up by a build, so each Beacon is its own SelectionGroup (systems/controls/selection_and_hud.md > SelectionGroup). With several Beacons selected, Warp In applies to the Beacon the player has Tabbed to.
### BeaconCapacity provided - 5
### SymmetryType - AAAA

### Beacon ObjectInterfaceState
(user, 2026-11-24) Warp In has more buildings than fit one menu (14 buildings, 11 usable slots), so the Beacon has **two Warp In menus**, like the Deployment Center's two build menus. The split and the menu names are **provisional** (user: easy to reshuffle later).

DefaultState:
```
[Q] Warp In (Basic)   [W] Warp In (Advanced)   [E] —   [R] —
[A] —                 [S] —                    [D] —   [F] —
[Z] —                 [X] —                    [C] —   [V] —
```
Warp In (Basic): economy and production
```
[Q] Egress Node     [W] Robo Portal       [E] Bio Portal      [R] —
[A] Alloy Furnace   [S] Essence Refinery  [D] Flux Reactor    [F] —
[Z] Back            [X] —                 [C] —               [V] —
```
Warp In (Advanced): tech and defense (each defense next to the building it requires)
```
[Q] Exophysics Engine   [W] Skylance            [E] Fusion Core          [R] Plasma Turret
[A] Relay Array         [S] Navigation Spire    [D] Psionic Amplifier    [F] Structural Analyzer
[Z] Back                [X] —                   [C] —                    [V] —
```
Choosing a building enters placement (Warp In > Warp In Command). Buildings whose costs are still TBD are shown but unavailable until defined.

## Warp In

Colonists build every structure other than the Beacon itself by warping it in directly onto an already-established Beacon. (Exception: the Heavy Portal and Star Portal are not Warped In from a Beacon — they are upgrade stages of a Bio Portal, see Portals.) The Beacon transforms in place into the target structure — the same in-place-transformation pattern as Beacon Deployment (Drone -> Beacon), generalized to Beacon -> any buildable structure.

### Warp In Command
- Select an established Beacon
- Select the Warp In option
- Select the desired building from the Beacon's Constructs list
- The building's footprint is shown as a ghost overlay centered on the Beacon's location. The player may rotate the ghost in 90-degree increments and flip it horizontally/vertically before confirming (see core/entities.md > Structure Type)
- **Odd-sized buildings** (user, 2026-11-24): a building with an odd width or height (e.g. the 3x3 Bio Portal) can't be exactly centred on the 2x2 Beacon; it covers the Beacon and sticks out one extra tile on one side. The ghost starts in a **fixed default position: the extra tiles to the bottom-right**. Rotating and flipping the ghost transforms it **around the Beacon's centre**, so the extra tiles move with it: a horizontal flip moves them to the left/right, a vertical flip to the top/bottom, a 90-degree rotation to the next corner. Every corner can be reached with either orientation of the building (rotation and flip combined), so the player chooses both the corner and the building's facing with the same keys.
- All footprint tiles must be clear (Buildable, unoccupied), with the exception of the Beacon's own footprint, which will be consumed by the transformation. If the footprint is blocked elsewhere, the ghost is tinted red and placement cannot be confirmed; if clear, it is tinted green
- Left-click confirms the footprint and begins the transformation. The Beacon is consumed and begins transforming in place into the selected building

### Constructs List
Each buildable structure has a Space Crystals (or other resource) cost and a WarpInTime, defined per-building on the Beacon's Constructs list — the same pattern used for GDO's DeploymentCenter and Cults' Recruit. WarpInTime is set per-building by design; it is not derived from cost.

| Building | Cost | WarpInTime |
|----------|------|------------|
| Egress Node | 75 Space Crystals | 320 frames (20 seconds) |
| Robo Portal | 250 Space Crystals (was 400; user, 2026-11-24) | 640 frames (40 seconds) |
| Bio Portal | 150 Space Crystals (kept; user, 2026-11-24: with the Beacon's Drone, ≈ 200 effective, level with GDO's Barracks) | 480 frames (30 seconds) |
| Alloy Furnace | TBD | TBD |
| Essence Refinery | TBD | TBD |
| Flux Reactor | TBD | TBD |
| Exophysics Engine | TBD | TBD |
| Skylance | TBD (requires Exophysics Engine) | TBD |
| Fusion Core | TBD | TBD |
| Plasma Turret | TBD (requires Fusion Core) | TBD |
| Relay Array | TBD | TBD |
| Navigation Spire | TBD | TBD |
| Psionic Amplifier | TBD | TBD |
| Structural Analyzer | TBD | TBD |

### HP/Shields Interpolation
During the Warp In, the transforming structure's HP and Shields each linearly interpolate from the Beacon's own MaxHP/MaxShields toward the target building's MaxHP/MaxShields as the transformation progresses:
- HP = Beacon MaxHP + (Building MaxHP - Beacon MaxHP) x progress
- Shields = Beacon MaxShields + (Building MaxShields - Beacon MaxShields) x progress

Where progress is frames elapsed / WarpInTime, from 0.0 to 1.0. (Same formula pattern as Beacon Deployment.)

Damage stays taken, exactly as in Beacon Deployment > HP/Shields Interpolation: the formulas give the maximum, and the current value rises by the same amount each frame (user, 2026-11-24).

### The target building from the first frame
(user, 2026-11-24) Same as Beacon Deployment: when the Warp In begins, the object is immediately the **target building under Warp In**, and its full footprint is claimed at once. It is the same object as the Beacon (stays selected). On cancel it shrinks back to the Beacon on the Beacon's original footprint.

### Vulnerability
The transforming structure is vulnerable to attack for the entire Warp In duration. If destroyed mid-Warp-In, the Beacon is destroyed along with it, and there is **no refund** (user, 2026-11-24): the whole investment is lost. Unlike GDO producers (factions/gdo/structures.md > Barracks > Destroyed while producing), the unfinished building is itself on the map and is what the attacker killed; catching a Warp In is meant to be a real reward.

### Cancel Warp In
The player may cancel the Warp In at any point before it completes. Cancelling refunds 75% of the building's resource cost (rounded down), and the structure reverts to being a normal, fully-functional Beacon (no HP/Shields loss).

HP and Shields on cancel work as in Beacon Deployment > Cancel Deployment: the Beacon returns with the same fraction of HP and Shields the building had, at least 1 HP (user, 2026-11-24). Example: cancelled at 50% HP → Beacon at 60/120.

### Beacon Capacity Retention
Every Colonists building provides 5 BeaconCapacity — the Beacon itself, and every structure Warped In from one, since they all originate from a Beacon and retain its BeaconCapacity contribution in addition to their own new function/production capabilities.

The 5 is provided **throughout a Warp In** as well: a Beacon being warped in keeps providing it the whole time (user, 2026-11-24).

## Starting Conditions
Where each starting object is placed is defined by the map's Starting Layout for this faction (core/map_editor.md > Starting Layouts).

The Colonists player starts with one pre-built Egress Node, one pre-built Robo Portal (see below for both), and **four Drones**. Both structures are already-Warped-In — each still provides 5 BeaconCapacity per the Beacon Capacity Retention rule (see Warp In above), same as any other Colonists building.

Starting resources: **100 Space Crystals** (same as GDO and the Syndicate). Starting Alloys, Essence and Flux: **0**.

## Egress Node

Drop-off point where Drones deliver Space Crystals to be transferred back to the Colonists' homeworld. Distinct from the Beacon — Space Crystals are NOT dropped off at Beacons. Built via Warp In (see Warp In above), same as every other Colonists building.

### Entity Type - Structure Type
### Faction - Colonists
### Size - 2x2 grid units
### MaxHP - 200
### MaxShields - 10
### PointArmor - 0
### FullArmor - 0
### SightRange - 3 grid units
### Destructible - true
### Groupable - false
### SymmetryType - AAAA

### Drop Off
Multiple Drones may drop off resources at the same Egress Node simultaneously. The Egress Node accepts every resource type (Space Crystals, Alloys, Essence, Flux) and is the only way resources enter the player's resource pool.

### Egress Node ObjectInterfaceState

#### DefaultState commands:
- **F: Select Returning Drones** — Immediate command (StateOnlyTransition). Replaces the current Selection with all Drones whose AssignedDropOff is this Egress Node and which are currently executing a Gather command — whether mining, walking to the patch, carrying resources back, or dropping off. Only available if at least one such Drone exists.

## Refining

Shared rules for the three Colonists refineries (Alloy Furnace, Essence Refinery, Flux Reactor).

- **Physical logistics**: Drones carry input resources into a refinery and carry output resources out of it (see Drone > Gathering). Refineries never draw from, or deposit into, the player's resource pool directly. Since resources in the pool cannot be withdrawn, the Flux Reactor's Alloys and Essence must come straight from Alloy Furnace / Essence Refinery output.
- **Input and output sides**: each refinery has specific sides where input is delivered and output is picked up (exact sides set per building with its Size and SymmetryType — TBD).
- **Storage**: each refinery has a MaxInputStorage and a MaxOutputStorage (values TBD per building).
- **Automatic conversion**: a refinery converts continuously whenever it has input and room in its output storage. Conversion rate: TBD per building.
- **Output full**: if output storage reaches its maximum, the refinery pauses conversion; delivered input keeps stacking up in input storage.
- **Input full**: if input storage reaches its maximum, the refinery accepts no more input; arriving feeder Drones wait with their load and a notification is raised (see Drone > Gathering > Waiting at a Full Refinery).
- **Conversion ratios preserve quantity**: the cost of refining is entirely the Drone time and the refinery buildings — no resources are lost.

## Alloy Furnace

Colonists refinery. Converts Space Crystals into Alloys.

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Conversion - 1 Space Crystal → 1 Alloy

Size, SymmetryType, input/output sides, MaxHP, MaxShields, armor, SightRange, Groupable, MaxInputStorage, MaxOutputStorage, conversion rate, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.

## Essence Refinery

Colonists refinery. Converts Space Crystals into Essence.

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Conversion - 1 Space Crystal → 1 Essence

Size, SymmetryType, input/output sides, MaxHP, MaxShields, armor, SightRange, Groupable, MaxInputStorage, MaxOutputStorage, conversion rate, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.

## Flux Reactor

Colonists high-tech refinery. Combines Alloys and Essence into Flux.

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Conversion - 1 Alloy + 1 Essence → 2 Flux

### Separate Inputs
The Flux Reactor has **separate entrances and separate input storage** for Alloys and Essence (each with its own maximum), so an oversupply of one input cannot block delivery of the other. Flux is picked up from its output side.

Size, SymmetryType, input/output sides, MaxHP, MaxShields, armor, SightRange, Groupable, MaxInputStorage (per input), MaxOutputStorage, conversion rate, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.
