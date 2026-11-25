# GDO: Units

## Peacekeeper
GDO basic infantry unit. A light infantry soldier armed with a fully connected ground attack.

Targets ground only. This is intended: there are no air units in the early game, and GDO's anti-air comes later (Arsenal missile infantry, Missile Turret).

Lore: "Peacekeeper" is the official name for a sanctioned external operator — hired security working under GDO license beyond the borders (see core/lore.md > Factions > GDO).

Look: corporate private security — polished body armor, branded, standardized gear.

Role intent: in the early game the Peacekeeper should feel like a great all-rounder. Later in the game it may become slightly redundant as specialized units take over.

### Faction - GlobalDefenseOrganization
### Silhouette - 24x24 space units (square)
### MaxHP - 50
### PointArmor - 1
### FullArmor - 1
### SightRange - 5 grid units
### Destructible - true
### Groupable - true
### UnitBase - LightInfantry
### UnitControlCost - 1

### UnitBaseAttributes[LightInfantry]:
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 4 space units/frame
- RuggedTerrainDefenseBonus: 50%

### TurretAttributes - None

### AttackAttributes:
- AttackType: FullyConnected
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 10
- Range: 4 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 1 frame
- CooldownDuration: 2 frames
- ReloadDuration: 12 frames

### Riot Shield (toggled ability)
The Peacekeeper carries a riot shield that can be raised or lowered (toggle).

**Two unit types** (user, 2026-11-24): shield down and shield up are **two unit types**, **Peacekeeper** and **Peacekeeper (Shielded)**, like the Sabre and Rifle Legionnaire (factions/colonists/units.md > Weapon Conversion). They share everything except what the shield changes (below). Raising or lowering the shield switches the unit's type.
- In a selection, the two types form **separate SelectionGroups**; the shield command goes to the ActiveGroup. Example: 6 shielded + 4 unshielded selected → two groups; Tab to the 4, raise → all 10 are one type and form one group.
- The Peacekeeper's panel shows **Raise Shield**; Peacekeeper (Shielded)'s shows **Lower Shield** and no Attack.
- **A toggle sets a target state** (user, 2026-11-25; Q0049/Q0053, same rule as factions/colonists/units.md > Weapon Conversion > A toggle sets a target form): pressing R flips the Peacekeeper's target (shield up or down), and the unit works to reach it. The type, and so the SelectionGroup, switches **at the press**, including when the raise is deferred behind a shot. The pressed units join the other type's group, and **that merged group becomes the ActiveGroup**, so a second press reaches all of them and everyone ends in the same state.
- The Barracks produces the unshielded Peacekeeper. Cost and Unit Control are the same for both types.

While the shield is **up**:
- MaxSpeed is halved (4 → 2 space units/frame).
- The Peacekeeper **cannot attack**.
- Incoming damage is halved — a 50% **pre-armor** reduction (step 1 of systems/combat.md > DamageModifierOrder), multiplying with other pre-armor reductions. The reduction applies to damage from **all directions**.
- Attack orders are not possible: the player must lower the shield before ordering an attack. The Peacekeeper does not auto-target enemies while the shield is up.
- **Explicit Attack (A + click) in a mixed selection** (user, 2026-11-24): Peacekeepers whose shield is up are **left out** of the order and keep their current orders; the units that can attack carry it out. This follows the CommandPanel rule that a command goes only to objects that can carry it out (systems/controls/selection_and_hud.md > CommandPanel). It applies per unit, so in a Peacekeeper group with some shields up and some down, only the lowered ones attack. Note the difference from right-click (below), where shielded Peacekeepers follow. A + click on **ground** (AttackMove) is different: shielded Peacekeepers take part and simply move, never stopping to fight (systems/controls/commands.md > AttackMove > Units that can't attack).

While the shield is **down**, the Peacekeeper behaves normally.

**Toggle time:** raising and lowering the shield takes a short time, so it cannot be flicked on and off between enemy shots. **6 frames (0.375 s), placeholder** (user, 2026-11-24; should stay under half a second).

**During the toggle** (user, 2026-11-24):
- While raising **or** lowering, the Peacekeeper moves at half speed (as with the shield up) and **cannot attack**.
- The 50% damage reduction applies **only once the shield is fully up**; a half-raised shield gives none.
- Pressing the toggle again mid-way **reverses from the current point** (halfway up → lowering takes only the half it already moved).
- **Pressed mid-attack** (user, 2026-11-24): pressed during the Firing or Cooldown phase (not interruptible), the raise is **deferred**: the shot finishes, then the shield starts rising immediately. Same as the Legionnaires' Convert (factions/colonists/units.md > Weapon Conversion > Eligibility), so one press raises a whole fighting group.
- **Lower pressed while a raise is still deferred** (user, 2026-11-25; Q0053): the target flips back to shield down, so the waiting raise is dropped; the shield never starts rising.

**Right-click on an enemy while the shield is up** (user, 2026-11-24): the Peacekeeper can't attack, so it does what any unit that can't carry out a right-click does (systems/controls/selection_and_hud.md > RightClickResolution): it **follows** an enemy unit (Move to that unit) and moves to an enemy structure. In a mixed group, the other units attack while the shielded Peacekeepers stay with them.

**Orders are kept** (user, 2026-11-24): raising or lowering the shield is a mode switch, not an order; the Peacekeeper's orders stay. An order it can't carry out while shielded is **on hold**, not dropped:
- an explicit **Attack** order: while the shield is up it follows the target (as with a right-click while shielded); when the shield is lowered it **resumes attacking** that target;
- an **AttackMove** or Patrol: while the shield is up it only moves (systems/controls/commands.md > AttackMove > Units that can't attack); once lowered it engages along the way again.
Same principle as the Colonists' Weapon Conversion (factions/colonists/units.md > Weapon Conversion > Orders are kept).

**Not queueable** (user, 2026-11-24): the toggle always happens immediately; holding Shift changes nothing.

**Hotkey: R** (Raise Shield on Peacekeeper, Lower Shield on Peacekeeper (Shielded)), placeholder (user, 2026-11-24).

### ObjectInterfaceState: BasicCombatUnitInterfaceState

## Rocketman
GDO missile infantry, trained at the Barracks once the player owns an Arsenal. Fires missiles at both ground and air targets. Normally its missiles fly to where the target was when it fired, so they can be dodged; its Target Lock ability lets it lock onto a single unit and fire homing missiles that cannot miss.

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Barracks
### TechPrerequisite - player owns at least one Arsenal (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

### AttackAttributes (normal attack):
- AttackType: DoublyDisjointed (the missile travels to where the target was at the end of Aiming; targets can dodge; can also be fired at the ground)
- TargetDomain: Universal (ground and air)
- Damage, Range, ProjectileSpeed, timings: TBD

### Target Lock (targeted ability)
The player orders Target Lock on one enemy unit.

**Locking:** the Rocketman aims at the target continuously for LockOnDuration. The target may move; the Rocketman turns to keep aiming at it. Locking succeeds only if the target stays within LockRange and visible for the whole duration.

**Locked:** once locked, the Rocketman fires at the locked target from then on, and every missile it fires at that target is **homing** (HeadDisjointed behavior: tracks the target, cannot miss). Locked missiles can be fired from anywhere within LockRange.

**LockRange** is a bit longer than the normal missile Range — so a lock both reaches further and is harder to shake off than a normal attack.

**The lock (or the locking in progress) breaks when:**
- the target leaves LockRange,
- the Rocketman loses vision of the target,
- the Rocketman is given **any** order (move, stop, attack something else, etc.),
- or the target dies.

TBD: LockOnDuration, LockRange, hotkey, whether Target Lock can target structures, stats (Silhouette, HP, armor, speed, UnitBase, cost, build time, Unit Control cost), ObjectInterfaceState.

## Drone Operator
GDO specialist infantry, trained at the Barracks once the player owns a Career Campus. Carries explosive suicide drones, launches them one at a time and pilots them into enemies. (User reference: like the Reaver and its Scarabs.)

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Barracks
### TechPrerequisite - player owns at least one Career Campus (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

### Drone Stock
- Starts with **1** drone when trained; holds a **maximum of 1** (changed from 3).
- **Build Drone** is one of the Drone Operator's abilities: once its drone has been used, it purchases/builds a replacement (cost, build time, hotkey TBD).

### Launching and piloting
- The Drone Operator launches a drone from its stock. The drone is a separate flying unit **controlled directly by the player**.
- Only **one drone can be active at a time** per operator.
- The Drone Operator **cannot move** while it has a drone active.
- The drone **can be shot down** — but only by attacks that can hit air (Air or Universal TargetDomain). It is an Air-domain unit.
- **Detonation:** the drone behaves like a combat unit with a **melee-range attack that it can use only once** — the attack is its explosion. It takes normal Attack orders and attacks like any melee unit; using its attack destroys it.
- **Explosion targets ground units only** (TargetDomain: Ground) — the drone cannot attack air units.
- **No recall:** once launched, a drone is spent. It cannot return to the operator's stock.
- The Drone Operator has **no weapon** of its own; its only offense is its drones.

### Drone limits — how a drone ends
| Event | Drone falls | Explodes |
|---|---|---|
| Detonates on enemies (its purpose) | — | Yes |
| Leaves the **leash range** around its operator | Yes | **Yes** (falls and explodes) |
| Runs out of **battery** (time limit) | Yes | No |
| Its **operator dies** | Yes | No |

TBD: whether the explosion is AoE or single-target; leash range, battery time, explosion damage/radius, drone HP/speed; all operator stats, cost, build time; hotkeys; ObjectInterfaceState.

## Field Engineer
GDO specialist infantry, trained at the Barracks once the player owns a Career Campus.

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Barracks
### TechPrerequisite - player owns at least one Career Campus (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)

### Abilities
- **Repair**: repairs buildings and vehicles. Repair costs Space Crystals (not free).
- **Build Bunker**: constructs a Bunker (see factions/gdo/structures.md > Bunker). Costs Space Crystals.
- **Set up First Aid Tent**: sets up a First Aid Tent (see factions/gdo/structures.md > First Aid Tent). Costs Space Crystals.

The Field Engineer carries **no weapon**.

TBD: repair rate and how repair cost is calculated, stats, cost, build time, hotkeys, ObjectInterfaceState.

## Raptor
GDO light missile vehicle produced at the Factory — a Humvee-like wheeled vehicle with a missile launcher turret. Like a Rocketman whose missiles are always homing. Does not carry infantry.

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Factory
### UnitBase - WheeledVehicle (fast, fixed turn radius, can reverse, directional armor)

### TurretAttributes:
- TurnAngle: **60°** full arc (±30° from the vehicle's facing) — placeholder. Deliberately narrow: an intentional exception, since WheeledVehicles typically have wide to full turret arcs.
- TurnRate: TBD

### AttackAttributes:
- AttackType: HeadDisjointed (homing missiles — track the target, cannot miss)
- TargetDomain: Universal (ground and air)
- Damage, Range, ProjectileSpeed, timings: TBD

### Transport - none (does not carry infantry)

Role vs Rocketman: both fire missiles at ground and air; the main difference is mobility — the Rocketman is slow infantry, the Raptor is **very fast**.

TBD: Silhouette, MaxHP, armor, SightRange, MaxSpeed/turn radius, cost, build time, Unit Control cost, ObjectInterfaceState.

## Twister
GDO vehicle produced at the Factory. A little slower than the Raptor. Mounts a machine gun on a fully rotating turret for anti-air defense, and lays hidden mines.

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Factory
### UnitBase - WheeledVehicle (a little slower than the Raptor)

### TurretAttributes:
- TurnAngle: 360° (full rotation)
- TurnRate: TBD

### AttackAttributes:
- AttackType: FullyConnected (Ranged) — machine-gun type weapon
- TargetDomain: **Air** (anti-air only)
- Damage, Range, timings: TBD

### Mines
- **Stock:** the Twister carries up to **3 mines** and **starts with 3** when produced. Further mines are **purchased one at a time** (a "Buy Mine" ability, like the Drone Operator's Build Drone). Mine cost and purchase time TBD.
- **Lay Mine (targeted ability):** the player picks a location — **anywhere** (not limited to the GDO build area). The Twister drives there, **stops temporarily** to lay the mine, then is free to move again.
- **Hidden:** mines are **underground and hidden** from enemies (detection mechanics not yet designed).
- **Trigger:** each mine has an **activation radius**. When an enemy unit — **infantry or vehicle** — enters it, the mine **explodes after a short delay**.

- **Friendly fire:** the explosion **damages friendly units** caught in it, but friendly units **do not trigger** the mine — only enemies do.
- **Air units** neither trigger mines nor take damage from them (the explosion is ground-only).
- **Destroyable:** enemies can attack and kill mines (once they can see them — detection not yet designed).

TBD: mine cost/purchase time, laying time, activation radius, delay, explosion damage and radius, mine HP, stats, cost, build time, ObjectInterfaceState.

## Artillery Rig
GDO artillery vehicle produced at the Factory; requires Advanced Mechanics. Has two modes: it cannot attack while mobile and must deploy to fire long-range area artillery. (User reference: like the StarCraft 1 Siege Tank, but with no attack in its mobile mode.)

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Factory
### TechPrerequisite - player owns at least one Advanced Mechanics (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)
### UnitBase - TrackedVehicle

### Modes
- **Mobile:** moves normally; **has no attack**.
- **Deployed:** cannot move; fires its artillery attack.
- **Deploy / Undeploy:** switching modes takes time (values TBD); the unit can neither move nor attack while switching.

### AttackAttributes (Deployed only):
- TargetDomain: Ground
- TargetType: AoE — **friendly fire**: the blast damages friendly units too
- MinRange: yes (enemies closer than MinRange cannot be targeted; value TBD)
- Range: long (value TBD)
- Vision: like every attacker, it can fire at any enemy visible to its owner (systems/combat.md > ValidTarget) — it does not need its own vision of the target.
- AttackType: **TailDisjointed** — the target location is locked at the end of Aiming; the blast lands there at the end of Firing (no tracking projectile). Units that move out of the locked area in time dodge it. The Artillery Rig stays locked during Firing. Can also be fired at a ground location.
- Damage, AoE radius, timings: TBD

### TurretAttributes:
- TurnAngle (Deployed): 360° (full rotation)
- TurnRate: TBD

TBD: Silhouette, MaxHP, armor, SightRange, speed, cost, build time, Unit Control cost, hotkeys, ObjectInterfaceState.

## Prospector
GDO expansion vehicle produced at the Factory once the player owns Advanced Mechanics. A tracked vehicle that deploys into an **Outpost** (see factions/gdo/structures.md > Outpost), which extends the GDO build area so buildings can be placed far from the main base.

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Factory
### TechPrerequisite - player owns at least one Advanced Mechanics (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)
### UnitBase - TrackedVehicle

### Deploy into Outpost (ability)
- Can deploy on **any Buildable terrain** (not limited to the existing GDO build area). Placement follows the Worker-Built Structures rules (core/entities.md > Placement Validation > Worker-Built Structures): the order can target a location, the Prospector drives there, and placement is validated on arrival.
- Deploying is **permanent** — an Outpost cannot pack back up into a Prospector.

TBD: weapon (assumed none), deploy time, whether deployment can be cancelled mid-way, stats, cost, build time, Unit Control cost, hotkeys, ObjectInterfaceState.

## Hornet
GDO air-to-ground attack aircraft produced at the Airfield. A HoverCraft (a chopper) that fires **air-to-ground missiles**. Carries **limited ammo** and rearms at its Airfield (factions/gdo/structures.md > Airfield > Aircraft Support).

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Airfield
### TechPrerequisite - none beyond owning the Airfield that produces it
### UnitBase - HoverCraft (can hover in place, turns in place)

### AttackAttributes:
- Weapon: air-to-ground missiles
- TargetDomain: Ground
- Ammo: **5 missiles**, then it must reload at its assigned Airfield
- RearmTimePerAmmo: TBD (time to reload one missile at the Airfield; differs per aircraft type)
- AttackType, Damage, Range, timings: TBD

TBD: Silhouette, MaxHP, armor, SightRange, movement values, turret, cost, build time, Unit Control cost, ObjectInterfaceState.

## Dragonfly
GDO air-to-air fighter produced at the Airfield once the player owns Propulsion Labs. Fires **air-to-air missiles**. Carries **limited ammo** and rearms at its Airfield (factions/gdo/structures.md > Airfield > Aircraft Support).

### Entity Type - Unit
### Faction - GlobalDefenseOrganization
### Produced at - Airfield
### TechPrerequisite - player owns at least one Propulsion Labs (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)
### UnitBase - Glider (must keep moving; circles when idle; wider turns at higher speed)

### AttackAttributes:
- Weapon: air-to-air missiles
- TargetDomain: Air
- Ammo: **1 missile**, then it must reload at its assigned Airfield
- RearmTimePerAmmo: TBD (time to reload one missile at the Airfield; differs per aircraft type)
- AttackType, Damage, Range, timings: TBD

TBD: Silhouette, MaxHP, armor, SightRange, movement values, turret, cost, build time, Unit Control cost, ObjectInterfaceState.

## SupplyChopper
GDO supply transport air unit. An unarmed hovercraft that collects supplies from Supply Delivery Stations and delivers them to Supply Towers. Automatically picks up all available supplies when landing on a Supply Delivery Station.

### Cargo
A Supply Chopper carries one of the following at a time:
- Up to 4 infantry units
- A single vehicle
- Supplies

Unit transport mechanics (Load / Unload commands, landing to load, what counts as infantry vs vehicle for capacity, what happens to carried units if the chopper is destroyed) are TBD — deferred until unit design resumes.

### Faction - GlobalDefenseOrganization
### Silhouette - 60x60 space units (square)
### MaxHP - 150
### PointArmor - 1
### FullArmor - 1
### SightRange - 5 grid units
### Destructible - true
### Groupable - true
### UnitBase - HoverCraft
### AttackAttributes - None
### TurretAttributes - None

### UnitBaseAttributes[HoverCraft]:
- ForwardAcceleration: 0.9 space units/frame squared
- OmniDirectionalAcceleration: 0.1 space units/frame squared
- DragRatio: 0.1 per frame
- TurnRate: 10 degrees/frame

### SupplyChopperInstanceState:
- CarriedSupplies: number (supplies currently being transported)
- AttachedTower: ObjectInstance | None (the Supply Tower this chopper is attached to)

### Supply Pickup:
- When a Supply Chopper lands on a Supply Delivery Station, it automatically picks up all CurrentSupplies from the station.

### ObjectInterfaceState[SupplyChopper]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues Move command to that location
- Right-click SupplyDeliveryStation: issues PickUpSupplies command (fly to SDS, land, pick up all available supplies). Only if chopper is not carrying units.
- Right-click own SupplyTower (when carrying supplies): issues DropOffSupplies command (fly to tower, land, drop off all carried supplies, immediately lift off — unless this is the chopper's attached tower)
- Right-click own SupplyTower (when not carrying supplies): issues AttachToTower command (fly to tower, land, attach). Only if chopper is not carrying units.
- Right-click other friendly/neutral Object: issues Move command to that object's location

Immediate commands (CommandIssuingTransition):
- Stop: issues Stop command
- Hold Position: issues HoldPosition command

Target commands (StateOnlyTransition):
- Move: enters AwaitingTarget[Move]
- Pick Up Supplies: enters AwaitingTarget[PickUpSupplies]. Only available if chopper is not carrying units.
- Attach to Tower: enters AwaitingTarget[AttachToTower]. Only available if chopper is not carrying units.
- Drop Off Supplies: enters AwaitingTarget[DropOffSupplies]. Only available if chopper is carrying supplies.

### AwaitingTarget[PickUpSupplies] resolution:
- Left-click SupplyDeliveryStation: issues PickUpSupplies command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

### AwaitingTarget[AttachToTower] resolution:
- Left-click own SupplyTower with no attached chopper: issues AttachToTower command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

### AwaitingTarget[DropOffSupplies] resolution:
- Left-click own SupplyTower with no attached chopper: issues DropOffSupplies command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

### AwaitingTarget[Move] resolution:
- Left-click ground: issues Move command to that location (CommandIssuingTransition, returns to DefaultState)
- Left-click object: issues Move command to that object's location (CommandIssuingTransition, returns to DefaultState)
- Escape/right-click: returns to DefaultState (StateOnlyTransition)
