# Colonists: Units

## Drone

The Colonists' worker unit. Gathers Space Crystals and deploys into Beacons (see Beacon Deployment above). Unarmed.

### Entity Type - Unit
### Faction - Colonists
### UnitBase - HoverVehicle

### Silhouette - 36x36 space units (square)
### MaxHP - 20
### MaxShields - 10
### PointArmor - 0
### FullArmor - 0
### SightRange - 5 grid units
### Destructible - true
### Groupable - false
### BeaconCapacityCost - 1

With 5 BeaconCapacity per Beacon and the Drone's own 1 freed when it starts deploying, each Beacon is a net +6: beyond the starting 10 BeaconCapacity, roughly every 6th Drone must deploy as a Beacon to keep Drone production going.

### UnitBaseAttributes[HoverVehicle]:
- TurnRate: 45 degrees/frame
- ForwardAcceleration: 0.6 space units/frame squared
- OmniDirectionalAcceleration: 0.2 space units/frame squared
- DragRatio: 0.1 per frame

MaxSpeed equilibrium = (ForwardAcceleration + OmniDirectionalAcceleration) / DragRatio = (0.6 + 0.2) / 0.1 = 8 space units/frame.

### TurretAttributes - None
### AttackAttributes - None (unarmed)

### Gathering

Every Drone job is **pick up at X, deliver to Y**. The pick-up point is set by the Gather order; the delivery point is the Drone's AssignedDropOff. Drones never deliver to a Beacon.

| Job | Pick up at (Gather target) | Deliver to (AssignedDropOff) |
|---|---|---|
| Mining | Space Crystals patch | Egress Node |
| Feeding a crystal refinery | Space Crystals patch | Alloy Furnace or Essence Refinery |
| Hauling refined output | Refinery (its output storage) | Egress Node |
| Feeding the Flux Reactor | Alloy Furnace or Essence Refinery (output) | Flux Reactor |

Feeding and hauling are separate jobs — the player assigns different Drones to each. Resources only enter the player's resource pool when delivered to an Egress Node.

- CarryCapacity: 8 per load, for every resource type (Space Crystals, Alloys, Essence, Flux)
- MiningDuration: 32 frames
- PickUpDuration (from a refinery's output): TBD
- DropOffDuration: 2 frames
- MiningReach: 8 space units — a Drone can mine once its Silhouette is within 8 su of the patch's edge (placeholder)
- DropOffReach: 8 space units — within 8 su of the drop-off point's edge (placeholder)

#### Gathering Drones pass through each other
(user, 2026-11-24) A Drone **on a Gather job** does not collide with any other Drone on a Gather job, **of any player**. "On a Gather job" covers every step of the jobs in the table above: travelling to the pick-up point, mining or picking up, waiting at an occupied patch, carrying the load and delivering it.
- They still collide with everything else: other units, Drones that are not on a Gather job (idle, moving, deploying), structures, patches and terrain.
- **Drones only.** Other factions' workers (e.g. Cults Recruits) keep normal collision while mining.
- Reason: at saturation about 24 Drones work the narrow strip between a patch row and its Egress Node, and the returning Drones converge on one node. With hard collision they would queue and jam, and the Colonists' income targets assume patch occupancy is the only limit (factions/gdo/economy.md > Economy Targets (GDO vs Colonists)).
- A Drone that leaves its Gather job while overlapping another unit follows the overlap rule in systems/units.md > UnitCollision > Units that already overlap.

#### Patch Occupancy (one Drone per patch)
A Space Crystals patch can be mined by **only one Drone at a time**, from **any player** — an enemy Drone mining a patch blocks your Drones too. A patch is occupied while a Drone is mining it (Drones travelling to it do not count).

This rule applies to Drones only. Other factions' workers are not limited by it: they ignore Drones on a patch, and Drones ignore them.

When a Drone arrives at an occupied patch, it goes to the **nearest unoccupied patch to itself** instead. If no patch is unoccupied, it **waits** at the current patch and mines as soon as it frees up.

**Search radius** (user, 2026-11-24): a Drone only looks for another patch **within a radius around itself**. This radius is an internal per-unit stat of the Drone (stat name TBD). Value: **6 grid units (placeholder)**. Which patches count within it (e.g. only ones the player knows): TBD (Q0037 point 5).

#### When a Gather ends
(user, 2026-11-24) A Gather order normally never ends: the Drone repeats its trip. It **ends on its own** only when the Drone runs out of crystals to mine: its patch is gone and no patch is left within its search radius. It first **delivers any load** it carries, then becomes idle (or starts its next queued order).

**Orders Shift-queued behind a Gather** wait until the Gather has ended this way, e.g. "mine this field out, then move to X". A Drone waiting at an occupied patch is still gathering. A Shift-queued Deploy behind a Gather therefore means "deploy once the field is mined out"; the user expects players rarely to do this. (Rejected: starting the queued order after the next drop-off, or after the current mining/delivery half of the trip, which could send the Drone off holding a load.)

Consequence: one patch yields at most 8 crystals per 32 frames (4 Space Crystals per second) from Drones. A patch saturates at roughly (round-trip time ÷ 2 seconds) Drones, since each Drone occupies it for 2 of its trip's seconds: about 1 Drone for a patch adjacent to the drop-off, about 2 at 2 grid units, about 2.5 at 3 grid units (ignoring acceleration; slightly more once Drone acceleration is accounted for). Example: 8 patches in a row with the Egress Node 2 grid units from the row (edge to edge, centred on the long side; factions/gdo/economy.md > Economy Targets (GDO vs Colonists)) saturate at roughly 18–22 working Drones.

#### Waiting at a Full Refinery
A Drone delivering input to a refinery whose input storage for that resource is full **waits** at the refinery with its load until there is room. This raises a **notification** for the player (notification system not yet designed).

### Assigned Drop-off
Each Drone tracks an AssignedDropOff — the Egress Node or refinery it delivers its current and future loads to.

- **Auto-assignment**: When a Drone with no AssignedDropOff is ordered to Gather, it automatically assigns the nearest Egress Node as its AssignedDropOff.
- **Set Drop-off Location command**: The player may explicitly assign or reassign a Drone's AssignedDropOff to an Egress Node or a refinery via the Set Drop-off Location command (see ObjectInterfaceState below). This does not interrupt the Drone's current action (mining, walking, carrying, etc.) — it only updates AssignedDropOff, which determines the destination of the Drone's return trip (including redirecting a trip already in progress, since the return trip's destination is read from AssignedDropOff).
- A refinery is only a valid AssignedDropOff for resources it accepts as input (Space Crystals for the Alloy Furnace and Essence Refinery; Alloys or Essence for the Flux Reactor).

### Deploying

Drones can deploy into a Beacon (see Beacon Deployment above). This transformation is one-way and consumes the Drone entirely once complete.

### Drone ObjectInterfaceState

#### DefaultState commands:
(Layout confirmed by user, 2026-11-24. No Hold Position, Patrol or Attack.)
```
[Q] Move     [W] —                 [E] —   [R] —
[A] Deploy   [S] Set Drop-off      [D] —   [F] —
[Z] —        [X] Stop              [C] —   [V] —
```
- **Q: Move** — as in BasicCombatUnitInterfaceState.
- **X: Stop** — as in BasicCombatUnitInterfaceState.
- **A: Deploy** — enters AwaitingPlacement for a Beacon (StateOnlyTransition). Ghost preview follows cursor, snapped to grid, tinted green when valid / red when invalid (no minimum spacing from other Beacons). Left-click valid location confirms placement and dispatches the Drone (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **S: Set Drop-off Location** — enters AwaitingTarget[SetDropOffLocation] (StateOnlyTransition). Left-click an Egress Node or own refinery sets it as the selected Drone(s)' AssignedDropOff (CommandIssuingTransition, returns to DefaultState). Does not interrupt current mining/movement — only updates AssignedDropOff (see Assigned Drop-off above). Escape/right-click cancels back to DefaultState.

#### Unit Commands:
- **Move** — right-click ground
- **Stop** — panel button
- **Gather** — right-click Space Crystal patch, or right-click own refinery to haul from its output
- **Drop Off Resources** — right-click own Egress Node, or own refinery that accepts the carried resource (when carrying resources)
- **Deploy** — via panel button A
- **Set Drop-off Location** — via panel button S, then left-click an Egress Node or own refinery

#### Right-Click Resolution:
- Space Crystal patch → Gather
- Own Egress Node (carrying resources) → Drop off resources
- Own refinery (carrying a resource it accepts as input) → Drop off resources
- Own refinery (not carrying) → Gather (haul from its output)
- Enemy or other object → Move to that object's location (Drone is unarmed and cannot attack)
- Ground → Move

Multiple Drones can be selected simultaneously. Right-click commands are issued to all selected Drones even though the Drone is ungroupable (no shared control panel — the panel displays one Drone's interface).

## Scout

Colonists flying scout machine produced at the Robo Portal. Unarmed, with a large sight range. Placeholder name — final name TBD.

### Entity Type - Unit
### Faction - Colonists
### Produced at - Robo Portal
### Unit Prerequisite - player owns at least one Relay Array
### Domain - Air

The Colonists' only air unit not produced at the Star Portal: the Robo Portal carries machines, and the Scout is a machine. (The Star Portal is for air combat units.)

Stealth/detection: TBD. The design currently has no stealth or detection mechanic; whether the Scout is invisible without detection and/or acts as a detector would introduce one for all factions.

UnitBase, Silhouette, SeparationRadius, MaxHP, MaxShields, armor, SightRange, speed, BeaconCapacityCost, cost/time, and ObjectInterfaceState: TBD.

## Lancer

Colonists infantry produced at the Bio Portal. Fires a salvo of antimatter missiles that can hit both ground and air targets (TargetDomain: Universal).

### Entity Type - Unit
### Faction - Colonists
### Produced at - Bio Portal
### Unit Prerequisite - player owns at least one Exophysics Engine

Salvo details (missiles per attack, tracking vs locked-position projectiles, damage, range, timing), UnitBase, Silhouette, MaxHP, MaxShields, armor, SightRange, speed, BeaconCapacityCost, cost/time, and ObjectInterfaceState: TBD. "Salvo" — several projectiles per attack — is a new combat concept not yet defined in systems/combat.md.

## Sabre Legionnaire

The Colonists' basic melee combat infantry, produced at the Bio Portal. Shares an identical physical profile with the Rifle Legionnaire and may convert into one at will (see Weapon Conversion below).

### Entity Type - Unit
### Faction - Colonists
### UnitBase - HeavyInfantry

### Silhouette - 36x36 space units (square)
### MaxHP - 70
### MaxShields - 10
### PointArmor - 0
### FullArmor - 0
### SightRange - 5 grid units
### Destructible - true
### Groupable - true
### BeaconCapacityCost - 2

### UnitBaseAttributes[HeavyInfantry]:
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 5 space units/frame

### TurretAttributes - None

### AttackAttributes:
- AttackType: FullyConnected (Melee)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 15
- Range: melee (adjacent contact)
- AimDuration: 3 frames
- FiringDuration: 3 frames
- CooldownDuration: 1 frame
- ReloadDuration: 5 frames

### ObjectInterfaceState: BasicCombatUnitInterfaceState, plus:
- **R: Convert to Rifle Legionnaire** — Immediate command (CommandIssuingTransition). See Weapon Conversion below for eligibility and duration.

## Rifle Legionnaire

The Colonists' basic ranged combat infantry. Shares an identical physical profile with the Sabre Legionnaire and may convert into one at will (see Weapon Conversion below). Not directly produced — reached only by converting a Sabre Legionnaire.

### Entity Type - Unit
### Faction - Colonists
### UnitBase - HeavyInfantry

### Silhouette - 36x36 space units (square)
### MaxHP - 70
### MaxShields - 10
### PointArmor - 0
### FullArmor - 0
### SightRange - 5 grid units
### Destructible - true
### Groupable - true
### BeaconCapacityCost - 2

### UnitBaseAttributes[HeavyInfantry]:
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 5 space units/frame

### TurretAttributes - None

### AttackAttributes:
- AttackType: FullyConnected (Ranged)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 10
- Range: 4 grid units
- MinRange: 0
- AimDuration: 3 frames
- FiringDuration: 2 frames
- CooldownDuration: 7 frames
- ReloadDuration: 4 frames

### ObjectInterfaceState: BasicCombatUnitInterfaceState, plus:
- **R: Convert to Sabre Legionnaire** — Immediate command (CommandIssuingTransition). See Weapon Conversion below for eligibility and duration.

## Weapon Conversion

Sabre Legionnaire and Rifle Legionnaire are two forms of the same underlying unit, sharing an identical physical profile (Silhouette, MaxHP, MaxShields, PointArmor, FullArmor, SightRange, UnitBase, UnitBaseAttributes, BeaconCapacityCost) and differing only in AttackAttributes. Either form may convert into the other at any time via the Convert command.

### Convert Command
- Select Sabre Legionnaire(s) or Rifle Legionnaire(s) (units may be mixed in selection; each converts to its respective other form)
- Issue the Convert command (panel button R)
- The unit converts in place over 4 frames

### Cost
No resource cost.

### HP/Shields
No interpolation — since both forms share identical MaxHP/MaxShields, current HP/Shields values carry over unchanged.

### Vulnerability
The unit remains fully attackable throughout the 4-frame conversion — no invulnerability window.

### Eligibility
The Convert command follows the same interruption rules as any other command relative to an in-progress attack sequence (see systems/combat.md > AttackPhase): it may be issued freely while the unit is idle, moving, or in the Aiming or Reloading phases (both Interruptible = true), interrupting that phase immediately. It cannot be issued while the unit is in the Firing or Cooldown phases (both Interruptible = false) — the command is rejected during those phases.

### Completion
Once initiated, the conversion runs to completion over its 4 frames and cannot be cancelled.
