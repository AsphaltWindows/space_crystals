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

### AttackAttributes:
- AttackType: DoublyDisjointed (slow projectile to the target's position at the end of Aiming; can be dodged)
- TargetDomain: Ground
- TargetType: AoE (moderate-high damage)
- CanTargetGround: true (derived)

All numbers (HP, armor, sight, speed, TunnelSpaceCost, damage, AoE radius, range, projectile speed, attack timings, cost, build time): TBD.

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
- **Refract** (ability, deployed only): to be designed together with satellites (user, 2026-11-25).
- **Switching modes works like the GDO Artillery Rig** (user, 2026-11-25; factions/gdo/units.md > Artillery Rig > Modes): deploying and undeploying take time, during which it can **neither move nor attack**.
- **Two unit types**: **Laser Battery** and **Laser Battery (Deployed)**, with separate SelectionGroups.
- **Deploy is an order, not a mode switch** (user, 2026-11-25; unlike Convert and the Riot Shield, which follow the toggle rule):
  - **The difference** (user, 2026-11-25): with a mode switch the unit's type changes **instantly** and the unit catches up in its own time. With deployment, **the deploying process is what actually changes the unit**: it stays the mobile type until the process completes.
  - **Queueable**: Shift-Deploy waits until the earlier orders are done (e.g. move here, then deploy). Without Shift it replaces the current orders and starts at once.
  - **No reversing mid-way**: **Undeploy is only available once deployment is complete**. The unit becomes Laser Battery (Deployed) (type and SelectionGroup) when deploying finishes; until then it is still a Laser Battery. (Undeploying is assumed symmetric: Deploy becomes available again only once undeploying is complete.)
  - **Not cancellable** (user, 2026-11-25; replaces the earlier wind-back on Stop): once deploying has started it always runs to completion; Stop doesn't interrupt it. (Assumed symmetric: undeploying also always completes.)
  - **Orders given while deploying** (user, 2026-11-25): the **last order given** (without Shift; each new one replaces the previous) is **held**, and it is the first thing the unit tries once deployment finishes. Shift-queued orders line up behind it as usual. At that point each order is checked against the deployed unit: one it can't carry out (e.g. **Move**, since a deployed unit can't move) is **dropped** (with the usual unavailable-command feedback, assumed), and the next one is tried.
  - **Not the same as the Colonists Drone's Deploy** (user, 2026-11-25): a Drone becomes a Beacon **immediately** and the transformation **can be cancelled** (factions/colonists/economy.md > Beacon Deployment).
  - When the switch makes a group disappear, the group the units joined becomes active (systems/controls/selection_and_hud.md > ActiveGroup > When the ActiveGroup disappears).
- The battery keeps charging in every mode (Battery: always charging).
- TBD: deploy/undeploy time.

All numbers (HP, armor, sight, speed, TunnelSpaceCost, battery, damage, range, timings, cost, build time): TBD.

## Subterranean Transport
(user, 2026-11-25; name tentative, the user picks) Syndicate **infantry transport** that travels **underground**. It carries **infantry only**.

### Entity Type - Unit
### Faction - TheSyndicate
### UnitBase - DrillUnit (assumed; exits only through Tier 2+ Tunnels like other vehicles)
### Produced at - Vehicle Assembly
### TechPrerequisite - TBD
### AttackAttributes - none (assumed)

### Above-ground mode
- When built, it **drives out of the Tunnel in its above-ground mode**, behaving as a tracked vehicle.
- In this mode, **infantry can be loaded** into it, and it can **unload** them.

### Burrow (ability)
Makes it **go underground**. Underground it can travel **under buildings and obstacles such as cliffs**: across **all drillable terrain**.

### Underground: hidden unless detected
(user, 2026-11-25; the rule for every DrillUnit, systems/units.md > DrillUnit) While underground it is **hidden** from other players (its owner sees it). It becomes visible, and **targetable**, only while **revealed by a detector**. Undetected, it can't be **targeted**, but **area (AoE) damage still affects it** (user, 2026-11-25): a blast landing over it damages it like any unit in the area, e.g. a Grenadier's Attack Ground on a spot where it is suspected. Detection itself is not designed yet (detectors: which units/structures, range; to be designed, likely together with stealth such as the Colonists Scout's).

### Surface (ability)
Brings it back to its above-ground mode. **It can't surface on top of a building, or on ground that tracked vehicles can't traverse** (user, 2026-11-25). The check is the tile itself (it must be passable for a tracked vehicle and free of buildings), **not** whether a tracked vehicle could drive there: so it **can** surface on a plateau or behind cliffs that ground vehicles can't reach. Once surfaced it can unload and load again.

TBD: capacity (and whether it counts each infantry unit's TunnelSpaceCost); burrow/surface time and whether they follow the deploy model (not cancellable, factions/gdo/units.md > Artillery Rig > Modes) or the mode toggle model; what happens to passengers if it is destroyed; whether it can enter the Tunnel Network carrying passengers; load/unload commands and the panel; all numbers.

