# Syndicate: Units

**Syndicate lasers are red** (user, 2026-11-25): every Syndicate laser-shooting unit and structure (Guard, Laser Battery, Laser Turret) fires a **red** laser.

## Guard

The Guard is the Syndicate's basic combat infantry. A heavy infantry unit armed with a **laser rifle** that hits **ground and air** and **cannot miss** (instant hit), firing at a measured pace rather than rapidly (user, 2026-11-24). Tougher than the GDO Peacekeeper but with shorter range.

### Produced at - Hideout
### TechPrerequisite - player owns at least one **Optics Lab** (user, 2026-11-24; core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites). So the Optics Lab is the Syndicate's gate to an army; before it, only Agents (pistol) can fight.

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
- AttackType: FullyConnected (laser; instant hit, cannot miss)
- TargetDomain: Universal (ground and air)
- TargetType: SingleTarget
- Damage: 18 (placeholder)
- Range: 3 grid units
- MinRange: 0
- AimDuration: 4 frames (placeholder)
- FiringDuration: 1 frame (placeholder)
- CooldownDuration: 3 frames (placeholder)
- ReloadDuration: 16 frames (placeholder)

One shot every 24 frames (1.5 s) ≈ 12 damage per second, the same output as the earlier rapid-fire version (6 damage every 0.5 s); fewer, bigger hits make armor matter less against it.

### ObjectInterfaceState: BasicCombatUnitInterfaceState

## Grenadier
(user, 2026-11-25) Syndicate explosives infantry. Fires an **explosive grenade**: a **slow projectile** that flies to where the target was when the Grenadier finished aiming and explodes there, dealing **moderate-high area damage**. **Ground only.** Moving units can dodge it; it punishes units standing still or clumped together. Like every attack that can miss, it can also **Attack Ground** (systems/combat.md > AttackType > CanTargetGround).

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - HeavyInfantry (assumed, like all Syndicate infantry)
### Produced at - Hideout
### TechPrerequisite - player owns at least one **Chemical Plant** (explosives)
### Groupable - true (assumed)

**Placeholder stats** (designer, 2026-11-25, Q0071; tune later):
### Silhouette - 36x36
### MaxHP - 70
### PointArmor - 1
### FullArmor - 1
### SightRange - 5
### TunnelSpaceCost - 1

### UnitBaseAttributes[HeavyInfantry]:
- MaxSpeed: 5 space units/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 degrees/frame

### AttackAttributes:
- AttackType: DoublyDisjointed (slow projectile to the target's position at the end of Aiming; can be dodged)
- TargetDomain: Ground
- TargetType: AoE (moderate-high damage)
- CanTargetGround: true (derived)
- Damage: 100 (total over the circle; systems/combat.md > AoEDamage: a 36x36 infantry fully inside takes about 26 before armor)
- AoE radius: 40 space units (0.625 grid units)
- Range: 4 grid units
- MinRange: 0
- ProjectileSpeed: 8 space units/frame (a 4 GU throw takes 2 s, so moving units can step out of the blast)
- AimDuration: 6 frames; FiringDuration: 2 frames; CooldownDuration: 4 frames; ReloadDuration: 28 frames (one grenade per 40 frames, 2.5 s)

### Cost - 150 Space Crystals; Build Time - 160 frames (10 seconds)

### ObjectInterfaceState: BasicCombatUnitInterfaceState

## Agent

The Agent is the Syndicate's cyborg worker unit. Agents construct Tunnels and defensive structures on the surface, gather Space Crystals and Supplies, and deliver them to Tunnels. Agents carry a **pistol** for basic self-defense: short range, low-moderate damage, moderate reload, ground only, overall weak (user, 2026-11-24; replaces the melee attack).

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

### Gathering

Agents gather resources and deliver them to Tunnels. Crystal and Supply drop-offs use separate Tunnel sides (B and D respectively), so they do not block each other.

#### Gathering rules (user, 2026-11-25; Q0066)
- **The Drone's job rules apply to Agents** (factions/colonists/units.md > Drone > Gathering: Last load of a patch, Interrupted mining, Other orders keep the load, Gather while carrying a load, Drop Off order, When a Gather ends), with two exceptions: Agents **share patches** (no occupancy rule), and there is **no Set Drop-off** command.
- **Which Tunnel**: the **nearest** own operational Tunnel (straight line to its crystal hatch), **regardless of how many other Agents already use it**; no load balancing. It is chosen when the Gather's first return starts and kept for that Gather (a new Gather picks again; a lost Tunnel means picking the nearest again). A Drop Off order delivers to the Tunnel clicked, once.
- **The hatch** is **1 grid square** long, near the middle of its side (Tunnel > SymmetryType - ABCD). An Agent must be within DropOffReach of the hatch to deliver.
- **Waiting at the hatch**: an Agent waits within reach (or as close as it can get) while another drops off, and the first to arrive delivers next. **It waits as long as it takes; it never gives up** (unlike the Failure rule for other orders), so a worker never silently stops with a load.
- **Patches under a GDO Extraction Plate** can't be mined (as for Drones). **Agents of different players** may mine the same patch.

#### Space Crystals
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

#### Supplies
- CarryCapacity: 1 Supply per trip
- PickUpDuration: 48 frames (3 seconds)
- DropOffDuration: 48 frames (3 seconds)
- Drop-off side: Tunnel Side D

(user, 2026-11-25; Q0067)
- **Several Agents at one station**: any number pick up at once, as at patches.
- **When a Supply leaves the station**: when the pick-up ends. A pick-up starts only if a Supply remains that no running pick-up has claimed.
- **An empty station**: the owner gets a **notification** (notifications not designed yet), and every Agent picking up there **stands next to it indefinitely**, keeping its job, waiting for the next supply delivery.
- **Switching resources**: an Agent carrying Space Crystals that is ordered to gather Supplies goes **straight to the station**; its crystals are **lost when it picks up the Supply**. (Assumed the same the other way: a carried Supply is lost when it mines crystals.)

### Agent ObjectInterfaceState

**Panel letters (4x3 grid)** (user, 2026-11-25; Q0068, "fine for now"): **Q** Move, **A** Build Tunnel, **S** Drop Off Resources, **D** Build Laser Turret (greyed until available), **X** Stop. No Attack, Hold Position, Patrol or AttackMove buttons; the Agent attacks by right-click. These letters replace the older ones listed below where they differ.

#### DefaultState commands:
- **A: Build Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90° clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **B: Drop Off Resources** — targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side D for supplies). Always visible, greyed out when Agent is not carrying resources.
- **Build Laser Turret** (hotkey TBD) — enters AwaitingPlacement for a Laser Turret (StateOnlyTransition), same placement flow as Build Tunnel. Left-click a valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to DefaultState). Visible but greyed out until the player owns an Optics Lab.

#### Unit Commands:
- **Move** — right-click ground
- **Stop** — panel button
- **Attack** — right-click enemy unit/building (pistol, ground only)
- **Enter** — right-click own Tunnel (when not carrying resources)
- **Gather** — right-click Space Crystals patch or Supply Delivery Station
- **Drop Off Resources** — right-click own Tunnel (when carrying resources), or via panel button B
- **Build Tunnel** — via panel button A

#### Right-Click Resolution:
- Space Crystals patch → Gather crystals
- Supply Delivery Station → Gather supplies
- Own Tunnel (carrying resources) → Drop off resources (auto-routes to correct side)
- Own Tunnel (not carrying resources) → Enter
- Enemy unit/building → Attack (pistol)
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

#### While the Tunnel is under construction
(user, 2026-11-25; Q0061, the Director's proposals)
- **It does nothing but Cancel**: no Tunnel Space, no drop-offs, no Enter, no Expand, no Eject; the only command is X Cancel Construction (as for a Colonists building being transformed).
- **HP**: the ConstructionHP formula gives the **maximum** HP; damage taken stays (current HP rises by the same amount each frame, like the Colonists' Warp In).
- **Where the Agent comes out** on cancel or destruction: at the **centre of the footprint**, which is now free; it then idles.
- **Defeat**: the site counts as a structure (core/game_rules.md > Defeat Condition). Cancelling it when it is the player's only structure defeats them in that tick.
- **A carried load**: an Agent ordered to build while carrying crystals or Supplies **keeps its load**, in the site and in the network.
- **What enemies see**: the site, like any surface structure, but not the Agent inside it.

#### Cancel Construction
While a Tunnel is under initial construction (Agent inside, not yet operational), the owning player may cancel it:
- **X: Cancel Construction**: cancels the in-progress initial build (CommandIssuingTransition). The Agent survives and emerges at the location. Full refund of Supplies cost. Only available while the Tunnel is under initial construction — mutually exclusive with the post-operational **X: Cancel Upgrade** command, since a Tunnel is never in both states at once.

## Assault Bike
(user, 2026-11-25) Syndicate fast attack vehicle. The **fastest wheeled vehicle in the game**, with a **relatively wide turning radius** for a fast vehicle (and, being wheeled, it can't turn on the spot). Armed with a **mortar**: like the Grenadier's grenade (slow, dodgeable explosive projectile with area damage) but with **slightly longer range**. The mortar sits on a **turret with a narrow turn angle**: it can aim a little to either side of the bike's facing, but the bike must still roughly point toward its target. Ground only (assumed, like the Grenadier); can Attack Ground.

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - WheeledVehicle (exits only through Tier 2+ Tunnels)
### Produced at - Vehicle Assembly
### Groupable - true (assumed)

### TurretAttributes:
- TurnAngle: narrow (value TBD)

### AttackAttributes:
- AttackType: DoublyDisjointed (mortar; slow projectile to the target's position at the end of Aiming; can be dodged)
- TargetDomain: Ground (assumed)
- TargetType: AoE
- Range: slightly longer than the Grenadier's
- CanTargetGround: true (derived)

All numbers (HP, armor, sight, speed, turn radius, turret turn angle and rate, TunnelSpaceCost, damage, AoE radius, range, projectile speed, timings, cost, build time): TBD.

### ObjectInterfaceState: BasicCombatUnitInterfaceState

## Shadebringer
(user, 2026-11-25) Syndicate **support vehicle**. Wheeled. **No attack and no turret.** Has two **toggleable abilities**, Sludge and Cover, each switched on and off by the player; while one is on, the Shadebringer leaves its effect behind it as it drives.

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - WheeledVehicle (exits only through Tier 2+ Tunnels)
### Produced at - Vehicle Assembly
### TechPrerequisite - player owns at least one **Vault**
### Groupable - true (assumed)
### AttackAttributes - none

### Sludge (toggle)
While on, the Shadebringer leaves a **trail of sludge** on the ground in its wake. Each patch of sludge lasts **X seconds** and then disappears.
- **Infantry and non-hover vehicles** that step into sludge are **slowed by Y%** for **Z seconds**.
- **Mechs** are slowed like vehicles (user, 2026-11-25).
- **Drill Units** are slowed while on the surface, and unaffected while underground (user, 2026-11-25).
- Not affected: hover units (HoverVehicle, HoverCraft) and air units.
- **Affects everyone** (user, 2026-11-25): enemy, allied and the player's own units alike. The Shadebringer has to be driven so it doesn't trap its own army.
- **The Shadebringer itself** (user, 2026-11-25): the sludge is spread **behind** it, so laying the trail doesn't slow it. But it is **not immune**: if it drives back over sludge (its own or another Shadebringer's), it is slowed like any other vehicle.
- TBD: X, Y, Z; trail width; whether the slow refreshes while a unit stays in the sludge.

### Cover (toggle)
While on, the Shadebringer leaves a **stack/wall of smoke** in its wake. Each piece of smoke **lasts a set number of seconds** (TBD) and then clears. The smoke **blocks line of sight into it and behind it** for every unit except **satellites**: it's not possible to see into the smoke, or past it, from the other side.
- This is the **first sight-blocking mechanic** in the game: elsewhere vision is a plain radius (core/vision.md).
- **Units inside the smoke can't see anything** (user, 2026-11-25): a unit standing in smoke provides no vision at all, not even of the smoke around it.
- **Structures too** (user, 2026-11-25): the smoke blocks **buildings'** sight the same way as units'. Only satellites see through it.
- TBD: the smoke's duration and width/height.

### Both at once
(user, 2026-11-25) Sludge and Cover are independent toggles and **can both be on at the same time**: one pass then leaves a trail that both slows and blinds, draining both tanks.

### Charges and recharging
(user, 2026-11-25) Sludge and Cover each have their **own, separate charge**. The charges are **recharged only inside the Tunnel Network**, and only while the player owns a **Vault** (this is the Vault's "re-arming" role). Away from the tunnels, a Shadebringer that has used up a charge can't use that ability until it goes back underground.
- **Each charge is a tank** (user, 2026-11-25) that **drains while its toggle is on**. Using an ability for a short while costs only a little. When a tank runs **empty**, that ability switches off and can't be switched back on until the tank has some charge again.
- Inside the Tunnel Network (with a Vault), the tanks **refill gradually**, partly used ones included.
- TBD: tank size, drain rate, refill rate (numbers); whether a nearly empty tank can be switched on (minimum charge).

All numbers (HP, armor, sight, speed, turn radius, TunnelSpaceCost, X, Y, Z, smoke duration, cost, build time) and hotkeys: TBD.

### ObjectInterfaceState
Move-type commands like a combat unit without an attack, plus the two toggles (hotkeys TBD). Whether toggles are queueable and whether they keep orders: TBD (the Riot Shield/Convert rule, mode switches keep orders and are not queueable, is the likely model).

## Laser Battery
(user, 2026-11-25) Syndicate laser vehicle running on a **battery**. Treaded. Fires a **red laser** (FullyConnected: instant, cannot miss) at **fairly long range**, **ground only** while mobile. Its turret has **relatively little mobility**. It can **deploy**, becoming an immobile **anti-air** emplacement with a full-rotation turret.

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - TrackedVehicle (exits only through Tier 2+ Tunnels)
### Produced at - Vehicle Assembly
### TechPrerequisite - player owns at least one **Optics Lab** (user, 2026-11-25)
### Groupable - true (assumed)

### Battery
- The Laser Battery has a **battery charge** that is **always charging** at its charge rate, attacking or not (user, 2026-11-25).
- **A normal surge draws twice what charges in one surge interval**, so while attacking with charge left the battery drains at a net **1× the charge rate**. Resting refills it.
- **Empty battery** (user, 2026-11-25): it keeps firing **just as often**, but each surge uses only what charged since the last one (half a normal surge's charge), so by the ^(2/3) rule it deals about **63%** ((½)^(2/3)) of a normal surge's damage. An exhausted Laser Battery still fights at about two-thirds strength.
- More generally, a surge uses the normal surge charge if the battery holds it, otherwise whatever is left; its damage follows the ^(2/3) rule.
- TBD: battery size, charge rate (numbers); whether it charges while switching modes (assumed yes: always charging).

### Attack: constant beam with surges
- **Animation**: while attacking, a **constant thin red laser** connects the Laser Battery to its target. Periodic **surges** along the beam are what **actually do the damage**.
- **Damage per surge** is proportional to the **charge used^(2/3)** (the square of the cube root of the charge spent on that surge). Spending charge in many small surges deals more total damage than spending the same charge in one big surge, so big surges trade efficiency for burst.
- AttackType: FullyConnected (cannot miss)
- TargetDomain: Ground (mobile); Air (deployed, below)
- TargetType: SingleTarget (assumed)
- Range: fairly long
- TBD: surge interval, charge used per normal surge, damage constant, range, aiming time.

### Amplify (ability)
Converts the **entire remaining battery** into the **next surge**. A single big hit for burst damage, at the cost of efficiency (per the ^(2/3) rule) and of an empty battery afterwards.

**The trade is a tuning dial** (user, 2026-11-25: a reasonable trade; its size depends on the numbers). If a full battery holds N normal surges, an Amplified full battery deals N^(2/3) normal surges' worth of damage instead of N, i.e. N^(−1/3) of the total: N = 4 → ~63%, N = 8 → 50%, N = 27 → ~33%. So the share of the battery a normal surge uses sets how costly Amplify is; choose it in the numbers pass.
- TBD: hotkey; whether it can be used while deployed; whether it is queueable or a mode-like toggle.

### TurretAttributes (mobile):
- TurnAngle: **narrow arc** (user, 2026-11-25): it can only aim within a cone ahead of the hull, so the tank must roughly face its target (like the Assault Bike). Value TBD.

### Deploy (ability)
- **Deployed**: **cannot move**. The turret gets **360° rotation**. It can **only target air units** (no ground targets while deployed).
- **Relay** (ability, deployed only; user, 2026-11-25; renamed from "Refract"): the deployed Laser Battery reflects its beam through the player's **Satellites** to hit far-off **ground or air** targets. See **Relay** below.
- **Switching modes works like the GDO Artillery Rig** (user, 2026-11-25; factions/gdo/units.md > Artillery Rig > Modes): deploying and undeploying take time, during which it can **neither move nor attack**.
- **Two unit types**: **Laser Battery** and **Laser Battery (Deployed)**, with separate SelectionGroups.
- **Deploy is an order, not a mode switch** (user, 2026-11-25; unlike Convert and the Riot Shield, which follow the toggle rule):
  - **The difference** (user, 2026-11-25): with a mode switch the unit's type changes **instantly** and the unit catches up in its own time. With deployment, **the deploying process is what actually changes the unit**: it stays the mobile type until the process completes.
  - **Queueable**: Shift-Deploy waits until the earlier orders are done (e.g. move here, then deploy). Without Shift it replaces the current orders and starts at once.
  - **No reversing mid-way**: **Undeploy is only available once deployment is complete**. The unit becomes Laser Battery (Deployed) (type and SelectionGroup) when deploying finishes; until then it is still a Laser Battery. (Undeploying is assumed symmetric: Deploy becomes available again only once undeploying is complete.)
  - **Not cancellable** (user, 2026-11-25; replaces the earlier wind-back on Stop): once deploying has started it always runs to completion; Stop doesn't interrupt it. Undeploying likewise always completes, and Deploy is available again only once it has (user, 2026-11-25; Q0071).
  - **Orders given while deploying** (user, 2026-11-25): the **last order given** (without Shift; each new one replaces the previous) is **held**, and it is the first thing the unit tries once deployment finishes. Shift-queued orders line up behind it as usual. At that point each order is checked against the deployed unit: one it can't carry out (e.g. **Move**, since a deployed unit can't move) is **dropped** with the usual unavailable-command feedback (user, 2026-11-25; Q0071), and the next one is tried.
  - **Not the same as the Colonists Drone's Deploy** (user, 2026-11-25): a Drone becomes a Beacon **immediately** and the transformation **can be cancelled** (factions/colonists/economy.md > Beacon Deployment).
  - When the switch makes a group disappear, the group the units joined becomes active (systems/controls/selection_and_hud.md > ActiveGroup > When the ActiveGroup disappears).
- The battery keeps charging in every mode (Battery: always charging).
- TBD: deploy/undeploy time.

### Relay (ability, deployed only)
(user, 2026-11-25) Only a **deployed** Laser Battery can relay; no other laser uses Satellites.

**Giving the order**
1. With deployed Laser Batteries selected, choose **Relay**. This **highlights every Satellite they can relay to**: each Satellite **in range of the Batteries themselves**, plus every Satellite connected to one of those by a **contiguous chain** of Satellites.
2. Click one of the highlighted Satellites: the **target Satellite**.

**The relay**
- Each selected Battery **locks onto its nearest Satellite** with its normal thin beam.
- The beams are **reflected as one beam** along the chain of Satellites to the target Satellite.
- The target Satellite becomes **armed**. It **targets enemies in its own range at will**, ground or air, like an attacking unit.
- **Several Batteries** can feed the same relay.

**Firing**
- The armed Satellite attacks at roughly the **Batteries' own attack frequency**. Each surge **discharges the Batteries as normal** (Battery).
- **Animation**, at every link:
  - surges do the damage;
  - between surges, the thin, harmless (0 damage) beam stays visible along the whole path: Battery → Satellite(s) → the armed Satellite → its current target.

**While relaying**
- The Batteries are **locked in**: they do nothing else.
- Their commands are:
  - **Stop Relaying**;
  - **Relay**, which picks another target Satellite;
  - **Amplify**: available while relaying **if the target Satellite has an active target**. Every selected Battery spends its **entire battery** on the next surge (Amplify).

**Every selected Battery must reach the chain** (user, 2026-11-25): the selected deployed Batteries act as one group, so **each** of them must have at least one Satellite in its own range. If any doesn't, **no relay targets are highlighted**. Highlighted targets are the Satellites reachable from **every** selected Battery (designer's reading).

**Sight** (user, 2026-11-25): the armed Satellite can attack **anything the player sees** within its range (any own vision, not only its own).

**Directing the armed Satellite** (user, 2026-11-25): besides auto-targeting, the player can **select the armed Satellite and give it a specific target** (Attack).

**Several Batteries add up** (user, 2026-11-25): each surge of the armed Satellite deals the **sum** of each relaying Battery's own surge damage (each computed from its own charge by the ^(2/3) rule). Four Batteries deal four times one Battery's damage.

**Chain breaks** (user, 2026-11-25): if the chain breaks (a Satellite on it is destroyed or a link goes out of range), the Batteries **stop relaying automatically** and go back to ordinary deployed behavior.

TBD:
- link range between Satellites, and Battery-to-Satellite range;

All numbers (HP, armor, sight, speed, TunnelSpaceCost, battery, damage, range, timings, cost, build time): TBD.

## Subterranean Transport
(user, 2026-11-25; name tentative, the user picks) Syndicate **infantry transport** that travels **underground**. It carries **infantry only**.

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - DrillUnit (confirmed, Q0071; exits only through Tier 2+ Tunnels like other vehicles)
### Produced at - Vehicle Assembly
### TechPrerequisite - player owns at least one **Forge** (user, 2026-11-25)
### AttackAttributes - none (confirmed, Q0071)

### Above-ground mode
- When built, it **drives out of the Tunnel in its above-ground mode**, behaving as a tracked vehicle.
- In this mode, **infantry can be loaded** into it, and it can **unload** them.

### Burrow (ability)
Makes it **go underground**. Underground it can travel **under buildings and obstacles such as cliffs**: across **all drillable terrain**.

### Underground: hidden unless detected
(user, 2026-11-25; the rule for every DrillUnit, systems/units.md > DrillUnit) While underground it is **hidden** from other players (its owner sees it). It becomes visible, and **targetable**, only while **revealed by a detector**. Undetected, it can't be **targeted**, but **area (AoE) damage still affects it** (user, 2026-11-25; confirmed Q0071): a blast landing over it damages it like any unit in the area, e.g. a Grenadier's Attack Ground on a spot where it is suspected. Detection itself is not designed yet (detectors: which units/structures, range; to be designed, likely together with stealth such as the Colonists Scout's).

### Underground movement
(user, 2026-11-25; Q0071) A burrowed transport **passes through every underground structure** (expansions of any player, a retracted pop-up Laser Turret, GDO mines), as underground units already pass through each other (systems/units.md > Underground Collision). Nothing hidden can block it, so moving never reveals anything.

### Surface (ability)
Brings it back to its above-ground mode. **It can't surface on top of a building, or on ground that tracked vehicles can't traverse** (user, 2026-11-25). The check is the tile itself (it must be passable for a tracked vehicle and free of buildings), **not** whether a tracked vehicle could drive there: so it **can** surface on a plateau or behind cliffs that ground vehicles can't reach. It **can surface above an expansion** (units stand over expansions anyway). The general surfacing rules also apply (systems/units.md > Underground Collision > Surfacing, > Reserved while surfacing): it can't surface where it would overlap an above-ground unit, and the spot is reserved once surfacing starts (user, 2026-11-25; Q0071). Once surfaced it can unload and load again.

### Burrowing and surfacing work like vehicle deploy
(user, 2026-11-25) Burrow and Surface follow the **vehicle deploy model** (factions/gdo/units.md > Artillery Rig > Modes): each takes time, is a **queueable order**, **can't be cancelled** once started, and the unit changes mode only when the process completes. The **last order given meanwhile is held** and tried first once it's done; orders the unit can't carry out in its new mode are dropped.

### Entering the Tunnel Network loaded
(user, 2026-11-25) It **can drive into a Tunnel with infantry inside**; they travel along inside it and stay aboard. Inside the network the passengers are **still locked inside the transport**: they only come out when the transport unloads (on the surface), and can't be ejected from a Tunnel on their own. **They always keep counting against the Tunnel Space cap** (user, 2026-11-25; Q0071: Tunnel Space is an army limit, factions/syndicate/tunnels.md > Production and Tunnel Space), aboard or not, inside the network or on the surface.

### Destroyed with passengers
(user, 2026-11-25) If it is destroyed, **all infantry inside die** with it (above ground or underground).

TBD: capacity (and whether it counts each infantry unit's TunnelSpaceCost); burrow/surface time; load/unload commands and the panel; all numbers.

## Phantom
(user, 2026-11-25) Syndicate **hovering aircraft**. It **rises vertically out of a Tier 3 Tunnel's air door** (factions/syndicate/tunnels.md > Tunnel > Look by tier). Fires a **red laser** that hits **both air and ground**. **Relatively manoeuvrable.**

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - HoverCraft (enters and exits only through Tier 3 Tunnels' air doors)
### Produced at - Aerospace Assembly
### TechPrerequisite - player owns at least one **Optics Lab** (user, 2026-11-25)
### Groupable - true (confirmed, Q0071)

### AttackAttributes:
- AttackType: FullyConnected (laser like other Syndicate lasers: instant, cannot miss; confirmed, Q0071)
- TargetDomain: Universal (ground and air)
- TargetType: SingleTarget (confirmed, Q0071)

### Role
(user, 2026-11-25) **Comparable to the Guard** (a laser all-rounder against air and ground), but **somewhat squishier for its cost**, and **faster**, and of course it **flies** (over cliffs, buildings and ground units like any air unit): the airborne, mobile counterpart of the Guard.

### TurretAttributes:
- TurnAngle: **moderate, 60–90°** (user, 2026-11-25: the **total** cone, i.e. 30–45° to each side of its facing; exact value TBD)

TBD: all numbers (HP, armor, sight, speed and handling, TunnelSpaceCost, damage, range, timings, cost, build time); hotkeys.

## Specter
(user, 2026-11-25; named by the user) Syndicate **bomber** carrying a **single payload**. It flies as a **Glider** (must keep moving), but **lifts off and lands vertically** through a Tier 3 Tunnel's air doors (factions/syndicate/tunnels.md > Tunnel > Look by tier).

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - Glider (with vertical lift-off and landing at the Tunnel air doors)
### Produced at - Aerospace Assembly
### TechPrerequisite - player owns at least one **Vault** and at least one **Chemical Plant** (user, 2026-11-25)

### Attack
- **A single payload**: one bomb per sortie. **Re-arming** (user, 2026-11-25, designer's reading of "requires a Vault"): it flies back and lands in any own Tunnel it can land in, i.e. a **Tier 3** Tunnel with air doors (not one mid-upgrade to Tier 3); inside the network it is re-armed, as long as the player owns a Vault **anywhere** in the network (Vault > Network-wide).
- **AoE damage**.
- On a **turret with very low range**: it must be nearly over its target.
- **Can miss**: the bomb falls to a locked location, so targets can dodge (DoublyDisjointed, assumed). The **projectile travel time is relatively high**, almost all of it the **vertical fall**.
- **Out of ammo** (user, 2026-11-25): once its bomb is dropped, its default behavior is to **fly back to the nearest own Tier 3 Tunnel**, land through its air door and re-arm inside, **even if the player owns no Vault** (it then waits inside, unarmed, until a Vault exists). If there is **no Tier 3 Tunnel**, it **circles** where it is (the Glider idle behavior).
- **Attack orders and ammo** (user, 2026-11-25):
  - **Ordered to attack while unarmed**: it can't bomb, so carrying out the Attack means re-arming first. It **flies to the nearest own Tier 3 Tunnel** to re-arm.
  - **On entering the Tunnel its orders are wiped**: the Attack and anything queued after it are dropped, and it **stays inside**. Re-arming never resumes the attack on its own.
  - **After it bombs**: if the target is **destroyed**, the Attack is done and it goes on to its next queued order. With nothing queued it is idle, and an unarmed idle Specter flies home (Out of ammo, above). If the target **survives**, the Attack order still stands. Since it is now unarmed, that means going to re-arm, so it **ignores the orders queued after the Attack** and flies home, where they are wiped.
  - Example: Attack a target, then shift-queue a Move. If the bomb kills the target, it flies the Move (unarmed; confirmed by the user), then home. If the target survives, it never flies the Move: it goes home and stays inside.
  - With **no Tier 3 Tunnel**, an unarmed Specter **ignores Attack orders** (user, 2026-11-25): it has no ammo and no way to re-arm, so the Attack is dropped and it goes on to its next queued order. With nothing queued it circles (Out of ammo, above).
- Like every attack that can miss, it can **Attack Ground** (systems/combat.md > AttackType > CanTargetGround).
- TargetDomain: **Ground only** (user, 2026-11-25).

TBD: re-arming time and the Vault's throughput limit; the lift-off/landing transition; all numbers.

## Satellite (name TBD)
(user, 2026-11-25) An unmanned Syndicate orbital unit whose function, **for now, is only laser relay**: deployed Laser Batteries reflect their beams through it (one or more Batteries; Satellites can be chained), and the target Satellite of a relay becomes **armed**, attacking ground or air enemies in its range (Laser Battery > Relay).

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - HoverCraft (Air domain; factions/syndicate/structures.md > Aerospace Assembly)
### Produced at - Aerospace Assembly
### TechPrerequisite - player owns at least one **Orbital Command** (user, 2026-11-25)
### Manned - no (no Null Chamber buff)

### Launch
It leaves the network through a Tier 3 Tunnel's **air exit**, **launched like a rocket straight up**, and **transforms in the air into its orbital form** (user, 2026-11-25).

### One way only
It is the **only unit that can't get back into the Tunnel Network** (user, 2026-11-25). Nothing recharges it inside, Vault or otherwise.

### Movement
- **Very slow** by default.
- **Directional thrust**: a few **charges**, each giving a quick burst of displacement in a chosen direction (user, 2026-11-25).
- **Charges never come back** (user, 2026-11-25): once they are spent it only drifts slowly.

### Sight
It sees through Shadebringer smoke (Shadebringer > Cover).

TBD: name; the open relay rules (Laser Battery > Relay); launch and transform timing (and whether it can be hit meanwhile); thrust charge count and distance; all numbers.

