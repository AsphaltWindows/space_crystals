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
- **Hatch jams: the plan** (user, 2026-11-26; Q0111): the 1-square hatch stays for now. The fix tried first is the new **90° body-width sidestep** (systems/units.md > Ground Collision; 0323). If Agents still lock up, the next try is a **2-square hatch** (both middle squares of Side B), still with **one Agent delivering at a time**. After that comes the backstop, **stuck gathering Agents make way like idle units**.
- **Waiting at the hatch** (user, 2026-11-26; Q0109; replaces "within reach or as close as it can get"): an Agent **waits only once it is within reach** of the hatch, while another drops off; the first to arrive within reach delivers next. **Until it is within reach it is never "arrived"**: it keeps actively moving toward the hatch, steering around whatever is in its way (systems/units.md > Ground Collision, moving units steer around blockers), and is exempt from MovingToLocation's "blocked near the target by standing units counts as arrived" rule (systems/controls/behaviors.md). **It waits as long as it takes; it never gives up** (unlike the Failure rule for other orders), so a worker never silently stops with a load.
- **Search radius** (user, 2026-11-26; Q0076): when its patch runs out (or can't be reached), an Agent looks for another known patch within **6 grid units** of itself, the Drone's value (placeholder).
- **Patches under a GDO Extraction Plate** can't be mined (as for Drones). **Agents of different players** may mine the same patch.

### Space Crystals
- CarryCapacity: 50 Space Crystals per load
- MiningDuration: 64 frames (4 seconds) (user, 2026-11-26; Q0076/Q0088; was 5 s): equal to DropOffDuration
- DropOffDuration: 64 frames (4 seconds)
- Drop-off side: Tunnel Side B
- MiningReach: 8 space units — an Agent can mine once its Silhouette is within 8 su of the patch's edge (placeholder)
- DropOffReach: 8 space units — within 8 su of the **crystal hatch** on Side B (placeholder)
- Patch sharing: any number of Agents may mine the same Space Crystals patch at the same time (unlike Drones, see Colonists > Drone > Patch Occupancy). The only queue in an Agent's trip is at the Tunnel's Side B.

Because a Tunnel's crystal side accepts one drop-off at a time, a Tunnel takes in at most 50 Space Crystals per 4 seconds = **12.5 Space Crystals per second**. Saturation by distance from Side B (Agent speed 1.5 grid units per second; trip = 4 s mining + 4 s drop-off + walking both ways; the walk is counted edge to edge, so real trips are a little shorter):

| Distance to Side B (grid units) | Trip time | Per Agent (SC/s) | Payback (100 SC) | Agents to saturate | Income with 2 Agents | 3 Agents | 4 Agents |
|---|---|---|---|---|---|---|---|
| 0 | 8.0 s | 6.25 | 16 s | 2.00 | 12.5 | 12.5 | 12.5 |
| 1 | 9.3 s | 5.36 | 19 s | 2.33 | 10.7 | 12.5 | 12.5 |
| 2 | 10.7 s | 4.69 | 21 s | 2.67 | 9.4 | 12.5 | 12.5 |
| 3 | 12.0 s | 4.17 | 24 s | 3.00 | 8.3 | 12.5 | 12.5 |
| 4 | 13.3 s | 3.75 | 27 s | 3.33 | 7.5 | 11.3 | 12.5 |
| 5 | 14.7 s | 3.41 | 29 s | 3.67 | 6.8 | 10.2 | 12.5 |
| 6 | 16.0 s | 3.12 | 32 s | 4.00 | 6.2 | 9.4 | 12.5 |

**Syndicate economy check** (user, 2026-11-25; Q0070):
- **Distance** in the table is measured **edge to edge**, from the patch's edge to the Tunnel's **crystal hatch** (the 1-square hatch on Side B), as for Drones (0107).
- **Tolerance**: ±15% on every figure, except the **12.5 Space Crystals per second ceiling per Tunnel**, which must hold exactly (hard rule: one drop-off per 4 s).
- **Targets**: the opening (the **third Agent is at work** around 0:40, i.e. 0:34–0:46, with the patch **3 GU** from the hatch; user, 2026-11-26, moved from 1 GU; Q0089: the Tunnel needn't actually reach 12.5 then); the table **from 3 GU outward** (closer rows are theoretical: with collision, Agents may need the player's help there, see Close patches below); stacked Tunnels double a field's intake; the Syndicate's one-base saturation income is the lowest of the measured factions.
- **Reference layout**: one Tunnel with its crystal hatch facing a single patch at each distance (Agents share patches, so the patch count doesn't matter). For the **stacked** check, the two Tunnels stand on **opposite sides of the field**, each hatch facing it, so every Agent's nearest Tunnel is its own (Agents deliver to the nearest Tunnel, Gathering rules).
- **Stacking is not something maps should encourage** (user, 2026-11-25): map makers shouldn't lay out fields that invite two Tunnels on one patch group. It stays allowed, with its existing costs (Tunnel Area Non-Overlap Rule > Crystal intake and stacked Tunnels).

Rationale (user, 2026-11-26; replaces the 5 s mining rationale): the drop-off time (not carry or mining time) sets the ceiling, so lengthening it lowers the per-Tunnel ceiling while keeping the number of Agents (and Tunnel Space) needed to saturate a Tunnel low. **Mining equals drop-off (4 s / 4 s)**: 2 Agents could saturate only with no walking at all (a patch touching the hatch), and a patch lasts only a couple of minutes at full intake (1500 SC default = 30 loads), so in practice the **third Agent is always needed** once the nearest patch runs out. 3 Agents saturate out to about 3 grid units, 4 Agents to about 6. Agents take 15 seconds to build at the Hideout, so the Syndicate's early ramp is not instant even though its ceiling is low.

**The table is an ideal, not a promise** (user, 2026-11-26; Q0089): with collision, real income at a given distance varies a little below it (e.g. 3 Agents at 3 GU measure about 10.7 SC/s, 4 Agents about 11.8; from about 5 GU 4 Agents reach 12.5), and depends on how the player spreads Agents across patches. That variation is wanted; exactly hitting the 12.5 ceiling is not a goal. The economy check passes within ±15%.

**Close patches and traffic** (user, 2026-11-26; Q0076/Q0088): Agents **keep normal collision** while gathering (no pass-through). Very close to the hatch (0–2 grid units) they can get in each other's way and income drops. **How far a starting Tunnel sits from its crystals is up to the map maker**, and where players build later Tunnels is their choice. A little player intervention to get Agents mining well (moving the Tunnel back, re-ordering an Agent to another patch) is acceptable, in line with the Core principle (systems/controls/behaviors.md > Core principle: predictable, not optimal).

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

## Repair
(user, 2026-11-26) Agents can **repair** own **Tunnels** and **Laser Turrets**, but **not expansions**, which regenerate instead (structures/README.md > Tunnel Expansions) (right-click a damaged Laser Turret; a Tunnel's right-click stays **Enter**, so repair it with **D: Repair**, user, 2026-11-26). **Laser Turrets cost Space Crystals to repair; Tunnels are repaired for free** (user, 2026-11-26). **Repair rules shared by every repairer** (user, 2026-11-26): several repairers on one target repair **faster, scaling linearly** (2 repairers = twice the rate), as construction does. **Placeholder rate and cost** (designer, 2026-11-26): repairing from 0 to full HP takes **half the target's build time** with one repairer, and costs **half its price** in the resource it was bought with, paid gradually as HP is restored (except Tunnels, which are free; a Tunnel's rate uses its construction time, 480 frames, at every Tier, Q0124). How an Agent reaches a pop-up turret or a Tunnel (side, reach) follows the Recruit/Field Engineer repair flow; details: Director's discretion. **When repair ends** (user, 2026-11-26; Q0124): repairers **stop once the target is at full HP**; if the player **runs out of Space Crystals** mid-repair, the order **ends** (they stop; a free Tunnel repair never runs out). **Gliders can't be repaired** (they can't stop; user, 2026-11-26).

## Agent ObjectInterfaceState

**Panel letters (4x3 grid)** (user, 2026-11-25; Q0068, "fine for now"): **Q** Move, **A** Construct (a menu: Tunnel or Laser Turret; user, 2026-11-26), **S** Drop Off Resources, **D** Repair (2026-11-26, matching the Recruit's panel), **X** Stop. No Attack, Hold Position, Patrol or AttackMove buttons; the Agent attacks by right-click. These letters replace the older ones listed below where they differ.

### DefaultState commands:
- **A: Construct** — enters ConstructMenu (StateOnlyTransition; user, 2026-11-26, a separate construct menu as for Recruits):
  - **Q: Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90° clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
  - **W: Laser Turret** — enters AwaitingPlacement for a Laser Turret (StateOnlyTransition), same placement flow as Tunnel. Left-click a valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Visible but greyed out until the player owns an Optics Lab. Greyed until available.
  - **Z**: back to DefaultState.
- **S: Drop Off Resources** (letters per Panel letters above) — targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side D for supplies). Always visible, greyed out when Agent is not carrying resources.
- **D: Repair** — targeted command (StateOnlyTransition). Left-click a damaged own Tunnel or Laser Turret to repair it (see Repair).

### Unit Commands:
- **Move** — right-click ground
- **Stop** — panel button
- **Attack** — right-click enemy unit/building (pistol, ground only)
- **Enter** — right-click own Tunnel (when not carrying resources)
- **Gather** — right-click Space Crystals patch or Supply Delivery Station
- **Drop Off Resources** — right-click own Tunnel (when carrying resources), or via panel button S
- **Build Tunnel / Build Laser Turret** — via panel button A (Construct), then Q / W

### Right-Click Resolution:
- Space Crystals patch → Gather crystals
- Supply Delivery Station → Gather supplies
- Own Tunnel (carrying resources) → Drop off resources (auto-routes to correct side)
- Own Tunnel (not carrying resources) → Enter (also when damaged: Enter wins over Repair)
- Damaged own Laser Turret → Repair
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
