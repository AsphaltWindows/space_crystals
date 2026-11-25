# Agent

The Agent is the Syndicate's cyborg worker unit. Agents construct Tunnels and defensive structures on the surface, gather Space Crystals and Supplies, and deliver them to Tunnels. Agents carry a **pistol** for basic self-defense: short range, low-moderate damage, moderate reload, ground only, overall weak (user, 2026-11-24; replaces the melee attack).

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - HeavyInfantry

## Silhouette - 36x36
## MaxHP - 75
## PointArmor - 1
## FullArmor - 1
## SightRange - 5
## TunnelSpaceCost - 1
## Groupable - false

## UnitBaseAttributes[HeavyInfantry]:
- MaxSpeed: 6 space units/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 degrees/frame

## TurretAttributes - None

## AttackAttributes:
(Pistol; user, 2026-11-24, replacing the melee attack. Numbers are placeholders.)
- AttackType: FullyConnected (Ranged; instant hit)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 5
- Range: 2 grid units
- MinRange: 0
- AimDuration: 3 frames
- FiringDuration: 1 frame
- CooldownDuration: 2 frames
- ReloadDuration: 14 frames

One shot every 20 frames (1.25 s) = 4 damage per second, a third of a Guard's.

## Gathering

Agents gather resources and deliver them to Tunnels. Crystal and Supply drop-offs use separate Tunnel sides (B and D respectively), so they do not block each other.

### Gathering rules (user, 2026-11-25; Q0066)
- **The Drone's job rules apply to Agents** (factions/colonists/units/drone.md > Gathering: Last load of a patch, Interrupted mining, Other orders keep the load, Gather while carrying a load, Drop Off order, When a Gather ends), with two exceptions: Agents **share patches** (no occupancy rule), and there is **no Set Drop-off** command.
- **Which Tunnel**: the **nearest** own operational Tunnel (straight line to its crystal hatch), **regardless of how many other Agents already use it**; no load balancing. It is chosen when the Gather's first return starts and kept for that Gather (a new Gather picks again; a lost Tunnel means picking the nearest again). A Drop Off order delivers to the Tunnel clicked, once.
- **The hatch** is **1 grid square** long, near the middle of its side (Tunnel > SymmetryType - ABCD). An Agent must be within DropOffReach of the hatch to deliver.
- **Waiting at the hatch**: an Agent waits within reach (or as close as it can get) while another drops off, and the first to arrive delivers next. **It waits as long as it takes; it never gives up** (unlike the Failure rule for other orders), so a worker never silently stops with a load.
- **Patches under a GDO Extraction Plate** can't be mined (as for Drones). **Agents of different players** may mine the same patch.

### Space Crystals
- CarryCapacity: 50 Space Crystals per load
- MiningDuration: 80 frames (5 seconds)
- DropOffDuration: 64 frames (4 seconds)
- Drop-off side: Tunnel Side B
- MiningReach: 8 space units — an Agent can mine once its Silhouette is within 8 su of the patch's edge (placeholder)
- DropOffReach: 8 space units — within 8 su of the **crystal hatch** on Side B (placeholder)
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

**Syndicate economy check** (user, 2026-11-25; Q0070):
- **Distance** in the table is measured **edge to edge**, from the patch's edge to the Tunnel's **crystal hatch** (the 1-square hatch on Side B), as for Drones (0107).
- **Tolerance**: ±15% on every figure, except the **12.5 Space Crystals per second ceiling per Tunnel**, which must hold exactly (hard rule: one drop-off per 4 s).
- **Targets**: the opening (3 Agents saturate a Tunnel around 0:40, i.e. 0:34–0:46, with the patch 1 GU from the hatch); the table row by row; stacked Tunnels double a field's intake; the Syndicate's one-base saturation income is the lowest of the measured factions.
- **Reference layout**: one Tunnel with its crystal hatch facing a single patch at each distance (Agents share patches, so the patch count doesn't matter). For the **stacked** check, the two Tunnels stand on **opposite sides of the field**, each hatch facing it, so every Agent's nearest Tunnel is its own (Agents deliver to the nearest Tunnel, Gathering rules).
- **Stacking is not something maps should encourage** (user, 2026-11-25): map makers shouldn't lay out fields that invite two Tunnels on one patch group. It stays allowed, with its existing costs (Tunnel Area Non-Overlap Rule > Crystal intake and stacked Tunnels).

Rationale: the drop-off time (not carry or mining time) sets the ceiling, so lengthening it lowers the per-Tunnel ceiling while keeping the number of Agents (and Tunnel Space) needed to saturate a Tunnel low. The 5-second mining time deliberately makes distance matter: 3 Agents saturate a Tunnel only when the crystals are within ~2 grid units of Side B, and 2 Agents never fully saturate. This incentivizes placing Tunnels as close to the crystals as possible (with Side B facing them). The Agent's payback is faster than the Extraction Plate's (22.5 s) only within ~1 grid unit, and slower beyond — tight Tunnel placement is what makes Agents efficient. Agents take 15 seconds to build at the Hideout, so the Syndicate's early ramp is not instant even though its ceiling is low.

### Supplies
- CarryCapacity: 1 Supply per trip
- PickUpDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side D
- PickUpReach: 8 space units from the Supply Delivery Station's edge; DropOffReach: 8 space units from the **Supplies hatch** on Side D (user, 2026-11-26; Q0082; placeholders, as for crystals)
- **Which Tunnel**: the nearest own operational Tunnel measured to its **crystal hatch**, the same measure as for crystals (user, 2026-11-26; Q0082), even though Supplies are delivered at Side D.

(user, 2026-11-25; Q0067)
- **Several Agents at one station**: any number pick up at once, as at patches.
- **When a Supply leaves the station**: when the pick-up ends. A pick-up starts only if a Supply remains that no running pick-up has claimed.
- **An empty station**: the owner gets a **notification** (notifications not designed yet), and every Agent picking up there **stands next to it indefinitely**, keeping its job, waiting for the next supply delivery.
- **Switching resources**: an Agent carrying Space Crystals that is ordered to gather Supplies goes **straight to the station**; its crystals are **lost when it picks up the Supply**: at the **end** of the pick-up, when the new load arrives. An Agent interrupted mid-pick-up keeps its crystals. The same holds the other way: a carried Supply is lost when the Agent **finishes** mining a crystal load (user, 2026-11-26; Q0082).

## Agent ObjectInterfaceState

**Panel letters (4x3 grid)** (user, 2026-11-25; Q0068, "fine for now"): **Q** Move, **A** Build Tunnel, **S** Drop Off Resources, **D** Build Laser Turret (greyed until available), **X** Stop. No Attack, Hold Position, Patrol or AttackMove buttons; the Agent attacks by right-click. These letters replace the older ones listed below where they differ.

### DefaultState commands:
- **A: Build Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90° clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **S: Drop Off Resources** (letters per Panel letters above) — targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side D for supplies). Always visible, greyed out when Agent is not carrying resources.
- **D: Build Laser Turret** — enters AwaitingPlacement for a Laser Turret (StateOnlyTransition), same placement flow as Build Tunnel. Left-click a valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Visible but greyed out until the player owns an Optics Lab.

### Unit Commands:
- **Move** — right-click ground
- **Stop** — panel button
- **Attack** — right-click enemy unit/building (pistol, ground only)
- **Enter** — right-click own Tunnel (when not carrying resources)
- **Gather** — right-click Space Crystals patch or Supply Delivery Station
- **Drop Off Resources** — right-click own Tunnel (when carrying resources), or via panel button S
- **Build Tunnel** — via panel button A

### Right-Click Resolution:
- Space Crystals patch → Gather crystals
- Supply Delivery Station → Gather supplies
- Own Tunnel (carrying resources) → Drop off resources (auto-routes to correct side)
- Own Tunnel (not carrying resources) → Enter
- Enemy unit/building → Attack (pistol)
- Ground → Move

Multiple Agents can be selected simultaneously. Right-click commands are issued to all selected Agents even though the Agent is ungroupable (no shared control panel — the panel displays one Agent's interface).

## Building

Agents construct Tunnels and surface defensive structures (see Surface Defensive Structures) on the surface. Defensive structures use the same construction flow as Tunnels (below). Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction.

### Tunnel Construction Flow
1. Agent receives build command and walks to the target location
2. Construction begins — the partially-built Tunnel appears at the location with the Agent inside it. The Tunnel starts at 10% HP (ConstructionHP Rule). The Agent is untargetable for the duration of construction.
3. **If construction completes**: The Tunnel becomes operational and the Agent is inside the Tunnel Network, available for redeployment from any Tunnel.
4. **If the partially-built Tunnel is destroyed**: The Agent survives and emerges at the location. The Tunnel is lost and any Supplies spent are lost.
5. **If construction is cancelled by the player**: The Agent survives and emerges at the location, same as destruction. A rubble animation plays (cosmetic only, no gameplay effect). The Supplies are refunded at the **current top price** (factions/syndicate/tunnels.md > Upgrade Costs; Q0085).

### While the Tunnel is under construction
(user, 2026-11-25; Q0061, the Director's proposals)
- **It does nothing but Cancel**: no Tunnel Space, no drop-offs, no Enter, no Expand, no Eject; the only command is X Cancel Construction (as for a Colonists building being transformed).
- **HP**: the ConstructionHP formula gives the **maximum** HP; damage taken stays (current HP rises by the same amount each frame, like the Colonists' Warp In).
- **Where the Agent comes out** on cancel or destruction: at the **centre of the footprint**, which is now free; it then idles.
- **Defeat**: the site counts as a structure (core/game_rules.md > Defeat Condition). Cancelling it when it is the player's only structure defeats them in that tick.
- **A carried load**: an Agent ordered to build while carrying crystals or Supplies **keeps its load**, in the site and in the network.
- **What enemies see**: the site, like any surface structure, but not the Agent inside it.

### Cancel Construction
While a Tunnel is under initial construction (Agent inside, not yet operational), the owning player may cancel it:
- **X: Cancel Construction**: cancels the in-progress initial build (CommandIssuingTransition). The Agent survives and emerges at the location. Supplies refunded at the current top price (factions/syndicate/tunnels.md > Upgrade Costs; Q0085). Only available while the Tunnel is under initial construction — mutually exclusive with the post-operational **X: Cancel Upgrade** command, since a Tunnel is never in both states at once.
