# Syndicate: Units

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
- Not affected: hover units (HoverVehicle, HoverCraft) and air units.
- **Affects everyone** (user, 2026-11-25): enemy, allied and the player's own units alike. The Shadebringer has to be driven so it doesn't trap its own army.
- TBD: X, Y, Z; trail width; whether the slow refreshes while a unit stays in the sludge; Mechs and Drill Units; whether the Shadebringer itself is slowed by its own trail.

### Cover (toggle)
While on, the Shadebringer leaves a **stack/wall of smoke** in its wake. Each piece of smoke **lasts a set number of seconds** (TBD) and then clears. The smoke **blocks line of sight into it and behind it** for every unit except **satellites**: it's not possible to see into the smoke, or past it, from the other side.
- This is the **first sight-blocking mechanic** in the game: elsewhere vision is a plain radius (core/vision.md).
- **Units inside the smoke can't see anything** (user, 2026-11-25): a unit standing in smoke provides no vision at all, not even of the smoke around it.
- TBD: the smoke's duration and width/height; whether structures' sight is blocked too; whether the two toggles can be on at the same time.

### Charges and recharging
(user, 2026-11-25) Sludge and Cover each have their **own, separate charge**. The charges are **recharged only inside the Tunnel Network**, and only while the player owns a **Vault** (this is the Vault's "re-arming" role). Away from the tunnels, a Shadebringer that has used up a charge can't use that ability until it goes back underground.
- **Each charge is a tank** (user, 2026-11-25) that **drains while its toggle is on**. Using an ability for a short while costs only a little. When a tank runs **empty**, that ability switches off and can't be switched back on until the tank has some charge again.
- Inside the Tunnel Network (with a Vault), the tanks **refill gradually**, partly used ones included.
- TBD: tank size, drain rate, refill rate (numbers); whether a nearly empty tank can be switched on (minimum charge).

All numbers (HP, armor, sight, speed, turn radius, TunnelSpaceCost, X, Y, Z, smoke duration, cost, build time) and hotkeys: TBD.

### ObjectInterfaceState
Move-type commands like a combat unit without an attack, plus the two toggles (hotkeys TBD). Whether toggles are queueable and whether they keep orders: TBD (the Riot Shield/Convert rule, mode switches keep orders and are not queueable, is the likely model).

