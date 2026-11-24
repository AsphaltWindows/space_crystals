# Colonists: Structures

## Tech Model

The Colonists have **no building tech tree**. Colonists buildings have no TechPrerequisites — any building can be Warped In at any time (given a Beacon and the resources to pay for it).

The one structural exception is the **portal upgrade line**: a Heavy Portal and a Star Portal are later construction stages of a Bio Portal, reached by upgrading it in place rather than by a separate Warp In (see Portals). They are stages of one building, not buildings with prerequisites.

**Defensive structures are an exception** to the no-prerequisites rule: they may have TechPrerequisites. Since a Drone can deploy a Beacon anywhere (no spacing requirement) and any building can be Warped In onto it, prerequisite-free defenses would allow abusive tower rushes. Examples: the Skylance requires an Exophysics Engine; the Plasma Turret requires a Fusion Core.

Progression is gated by the **economy** instead:
- Advanced buildings cost advanced **refined resources** (Alloys, Essence, Flux). Obtaining those requires the right refineries to be built and more Drone working time to be committed than a lower-tech building of the same total resource cost.
- Because only one Drone can mine a patch at a time (see Drone > Gathering > Patch Occupancy), each patch saturates after a few Drones. As the economy grows, extra Drones add little to crystal income, so the opportunity cost of assigning Drones to refining work falls.

**Units** may still have Unit Prerequisites (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites): some units require additional buildings beyond their production building to be Warped In before they can be produced.

### Operability Principle

Every Colonists unit is built on the homeworld and transported in through a portal, so any unit *could* always be produced. A Colonists tech building is required not because a unit cannot be produced without it, but because without that building's support on the surface the unit **would not be operable** once it arrives on this world. Each tech building provides the surface infrastructure its units or defenses depend on:

| Tech building | Surface support it provides | Unlocks |
|---|---|---|
| Exophysics Engine | Computes this planet's surface physics (gravity, atmosphere, terrain) needed to guide antimatter missiles | Lancer, Skylance |
| Fusion Core | Generates the fusion power its energy weapons draw on | Plasma Turret |
| Relay Array | Surface-to-surface communications, carrying control and data across the surface | Scout |
| Navigation Spire | Lighthouse-like navigation guidance for large aircraft | Large flying units |
| Psionic Amplifier | Amplifies psionic power so psionic units can function on this world's surface | Psionic unit (Bio Portal); possibly a psionic air unit (Star Portal) |
| Structural Analyzer | Analyzes local materials and conditions so the homeworld can fabricate armor and structures adapted to this world | Treaded vehicles (Heavy Portal) |

#### Rationale
This keeps the Colonists' fiction consistent: they are colonizing an alien world (Earth, from their point of view) with everything shipped in from home, so the limit on what they can field is what the surface can support, not what the homeworld can build.

### Upgrade Layout

All Colonists research is split by weapon/defense technology. These are **incremental stat upgrades**: each level adds a flat bonus (e.g. +1 damage or +1 armor) to every eligible unit, in multi-level lines (core/tech_and_research.md > Research, rule 5). As the Colonists have no Research exception, they apply to existing and future units alike (rule 1). Number of levels and per-level values TBD.

| Building | Researches |
|---|---|
| Fusion Core | Shield regeneration upgrades; energy weapon upgrades (beam/particle weapons, both Legionnaire weapons) |
| Exophysics Engine | Antimatter missile upgrades |
| Structural Analyzer | All armor upgrades (infantry, vehicle, flying) |

Refineries are purely economic and research nothing (see Structural Analyzer > Rationale).

### Refining Chain

```
Space Crystals ─┬→ Alloy Furnace ────→ Alloys ──┬→ Flux Reactor → Flux
                └→ Essence Refinery ─→ Essence ─┘
```

Every arrow is physical: Drones carry each input to its refinery and carry each output away (see Refining). Resources only enter the player's resource pool when delivered to an Egress Node.

## Portals

The Colonists' production structures are portals: gateways through which units are transported in from the homeworld rather than built on site. Each portal is defined by what it can carry.

| Portal | Carries | How it is obtained |
|---|---|---|
| Robo Portal | Machines (Drones, Scout). The cheaper kind of portal. | Warped In from a Beacon |
| Bio Portal | Living beings (infantry). A more expensive portal, able to support the transport of living beings. | Warped In from a Beacon |
| Heavy Portal | Vehicles | Upgrade of a Bio Portal |
| Star Portal | Air units | Upgrade of a Heavy Portal |

### Portal Upgrade Line

```
Robo Portal                                   (standalone)

Bio Portal → Heavy Portal → Star Portal       (one building, built in stages)
```

Bio Portal → Heavy Portal → Star Portal is a multi-stage construction: one building upgraded in place, not three buildings with prerequisites.

- A Heavy Portal can only be obtained by upgrading a Bio Portal in place, and a Star Portal only by upgrading a Heavy Portal in place. Neither can be Warped In directly from a Beacon.
- Each stage **replaces** the previous stage's production. An upgraded portal can no longer bring in what the lower stage brought in: a Heavy Portal does not produce infantry, and a Star Portal produces neither infantry nor vehicles.
- Upgrade cost, upgrade time, and behavior during an upgrade (production, cancellation/refund, HP/Shields): TBD.

### The Robo Portal Is Standalone

The Robo Portal is not part of the upgrade line. It is Warped In from a Beacon, only ever brings in Drones, and cannot be upgraded into a Bio Portal.

#### Rationale: why the Bio Portal is not an upgrade of the Robo Portal

Making the Bio Portal an upgrade of the Robo Portal was considered and rejected because of how it would play in a Colonists-vs-Colonists game.

**Core problem: no one would build a Bio Portal, and the early game would be skipped.** With nothing able to threaten them, both players would just keep adding Robo Portals. Drone production feeds more Beacons, which feed more portals, so the economy would grow exponentially and the low-economy early stage of the game would effectively disappear.

Why no one would commit to a Bio Portal:

1. **Nothing can attack before the first Bio Portal exists.** The Drone is unarmed (the only unarmed worker in the game), and the Colonists' first combat units come from the Bio Portal. So in a mirror match each player's scouting Drone can stay in the other's base indefinitely — nothing can remove it.
2. **The upgrade would be a single, fully visible, all-or-nothing event.** It would happen on a building whose location the opponent knows from the start (each player starts with one Robo Portal), and it would end Drone production at that portal. Because every Beacon consumes a Drone, it would also stop Beacon Capacity growth: the starting 10 Beacon Capacity supports at most 5 Sabre Legionnaires.
3. **The second player to upgrade would act with perfect information.** Seeing the opponent's upgrade start, they could keep producing Drones for roughly as long as the opposing army needs to cross the map, then upgrade, and meet the attack with the same number of Legionnaires and a larger economy.

Result: upgrading first is the worse move, so both players wait, and the mirror match turns into a standoff over one building. Against other factions the standoff does not arise, because their units can attack from the start and force the timing.

With the Bio Portal as its own Warp In, building one does not stop Drone production, army can be added alongside the economy in any amount, and the first Legionnaires can drive scouts out, which restores hidden information. The later upgrades in the line do not have this problem: both sides have combat units by then, and those upgrades do not touch Drone production.

### Open Questions

- **Footprint of the upgraded stages**: whether the Heavy Portal and Star Portal keep the Bio Portal's 3x3 footprint or grow. To weigh: if a stage needs ground the previous stage did not cover, that ground must be clear at upgrade time, and in a Colonists-vs-Colonists game an enemy Drone can deploy a Beacon on it (3 seconds, no spacing requirement). Before a player has combat units, nothing can remove that Beacon.
- **Bio Portal cost**: the Bio Portal is described as the more expensive portal, but its current Warp In cost (150 Space Crystals) is lower than the Robo Portal's (400). To be revisited together with the other Colonists costs.

## Robo Portal

The Colonists' Drone production structure. Produces Drones from a build queue, same standard production pattern as GDO's Barracks or Cults' Armory — a finished Drone "warps in" (materializes) at the Robo Portal rather than walking out from a fixed exit side, consistent with the Colonists' overall warping theme. Built via Warp In (see Warp In above). Standalone: not part of the portal upgrade line, and cannot be upgraded (see Portals).

### Entity Type - Structure Type
### Faction - Colonists
### Size - 4x4 grid units
### MaxHP - 600
### MaxShields - 20
### PointArmor - 1
### FullArmor - 16
### SightRange - 4 grid units
### Destructible - true
### Groupable - true
### SymmetryType - AAAA

Since the Robo Portal is fully symmetric (AAAA), it has no fixed exit side — a warped-in Drone may emerge from any of the 4 sides.

**Which side** (user, 2026-11-24): the same rule as the Bio Portal, over all four sides: the side **closest to the RallyPoint**; with no RallyPoint, or if that side is blocked, a random unblocked side (deterministic random). "Closest" = straight-line distance from the middle of each side to the rally point (an object's centre for object rallies); ties go top, right, bottom, left (in the building's default orientation). The same measure applies to the Bio Portal's A sides.

### RoboPortalInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 20, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

### RallyPoint behavior:
When a unit (Drone or Scout) finishes production and spawns, if RallyPoint is set, it is issued its own default right-click command resolved against the RallyPoint target (systems/controls/selection_and_hud.md > What a rally point makes a new unit do). E.g. a Drone rallied on a patch Gathers it. If RallyPoint is None, the unit spawns with no command. If the RallyPoint references an ObjectInstance that no longer exists, the RallyPoint is reset to None. Rally points on enemy objects turn into ground rally points when the enemy leaves sight (systems/controls/selection_and_hud.md > Rally points on enemy objects). (Same pattern as GDO's Barracks.)

### Produces:
- Drone: 50 Space Crystals, 160 frames (10 seconds)
- Scout: cost/time TBD. Unit Prerequisite: player owns at least one Relay Array.

### Robo Portal ObjectInterfaceState
(user, 2026-11-24) Standard production panel, same conventions as GDO's Barracks:
```
[Q] Drone     [W] Scout   [E] —                  [R] —
[A] —         [S] —       [D] —                  [F] —
[Z] —         [X] Cancel Production   [C] Set Rally Point   [V] —
```

## Bio Portal

The Colonists' basic military production structure. Produces basic combat units from a build queue, same standard production pattern as GDO's Barracks, Cults' Armory, or the Colonists' own Robo Portal. Built via Warp In (see Warp In above). First stage of the portal upgrade line: can be upgraded in place into a Heavy Portal, after which it no longer produces infantry (see Portals).

### Entity Type - Structure Type
### Faction - Colonists
### Size - 3x3 grid units
### MaxHP - 250
### MaxShields - 15
### PointArmor - 1
### FullArmor - 9
### SightRange - 3 grid units
### Destructible - true
### Groupable - true
### SymmetryType - AABB

### Exit Behavior
The Bio Portal has two exit-eligible sides (the "A" sides, per its AABB symmetry). A produced unit exits from whichever eligible A side is closest to the RallyPoint. If no RallyPoint is set, or the closest side is blocked (obstructed, not currently traversable), the unit exits from whichever eligible A side is unblocked, chosen at random among unblocked candidates.

### BioPortalInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 20, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

### RallyPoint behavior:
When a unit finishes production and spawns, if RallyPoint is set, it is issued the default right-click command resolved against the RallyPoint target (same resolution rules as the produced unit's own Right-Click Resolution). If RallyPoint is None, the unit spawns with no command. If the RallyPoint references an ObjectInstance that no longer exists, the RallyPoint is reset to None. Rally points on enemy objects turn into ground rally points when the enemy leaves sight (systems/controls/selection_and_hud.md > Rally points on enemy objects). (Same pattern as GDO's Barracks and the Colonists' own Robo Portal.)

### Produces:
- Sabre Legionnaire: 100 Space Crystals, 160 frames (10 seconds)
- Lancer: cost/time TBD. Unit Prerequisite: player owns at least one Exophysics Engine.
- Psionic unit (unnamed, not yet designed): cost/time TBD. Unit Prerequisite: player owns at least one Psionic Amplifier.

Rifle Legionnaire is not directly produced — it is reached only by converting an existing Sabre Legionnaire (see Weapon Conversion below).

### Bio Portal ObjectInterfaceState
(user, 2026-11-24) Standard production panel, same conventions as GDO's Barracks:
```
[Q] Sabre Legionnaire   [W] Lancer              [E] Psionic unit (TBD)   [R] —
[A] —                   [S] —                   [D] —                    [F] —
[Z] —                   [X] Cancel Production   [C] Set Rally Point      [V] —
```

## Heavy Portal

Second stage of the portal upgrade line (see Portals). A portal able to transport vehicles. Colonists vehicles use the HoverVehicle base; any heavy vehicle uses the TrackedVehicle base.

### Entity Type - Structure Type
### Faction - Colonists
### Obtained by - upgrading a Bio Portal in place (cannot be Warped In from a Beacon)

### Produces:
Vehicles (none designed yet). Does not produce infantry. Hover vehicles require no building beyond the Heavy Portal; treaded (TrackedVehicle) vehicles require a Structural Analyzer (Unit Prerequisite).

Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, Groupable, exit behavior, upgrade cost/time, and ObjectInterfaceState: TBD.

## Star Portal

Third stage of the portal upgrade line (see Portals). A portal able to transport air units.

### Entity Type - Structure Type
### Faction - Colonists
### Obtained by - upgrading a Heavy Portal in place (cannot be Warped In from a Beacon)

### Produces:
Air units (none designed yet). Does not produce infantry or vehicles. Large flying units additionally require a Navigation Spire (Unit Prerequisite; assumed to be produced here — to be confirmed). A psionic air unit requiring a Psionic Amplifier may also be produced here (tentative). The Scout is the one air unit not produced here: as a machine it comes from the Robo Portal.

Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, Groupable, exit behavior, upgrade cost/time, and ObjectInterfaceState: TBD.

## Exophysics Engine

A computing facility that models the physics of this planet's surface — gravity, atmosphere, terrain — for the Colonists, to whom Earth is the alien world. Researches upgrades and unlocks the Colonists' antimatter-missile technology.

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Unlocks
- **Lancer** production at the Bio Portal (Unit Prerequisite)
- **Skylance** Warp In (TechPrerequisite of a defensive structure)

### Research
**Missile upgrades** — improve antimatter-missile weapons (Lancer, Skylance). List, levels, costs, and times TBD.

Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.

## Skylance

Colonists anti-air defensive structure. Fires salvos of antimatter missiles at air targets.

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - player owns at least one Exophysics Engine (defensive structures are the exception to the Colonists' no-prerequisites rule — see Tech Model)

TargetDomain (presumably Air), salvo details (missiles per attack, damage, range, timing), Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, and ObjectInterfaceState: TBD. "Salvo" — several projectiles per attack — is not yet defined in systems/combat.md. Warp In cost/time: see Warp In > Constructs List.

## Fusion Core

A fusion power-generation facility. Unlocks the Colonists' anti-ground energy-beam defense.

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Unlocks
- **Plasma Turret** Warp In (TechPrerequisite of a defensive structure)
- Units: TBD (not yet decided whether the Fusion Core gates any unit)

### Research
- **Shield upgrades** — each level increases ShieldRegenRate (base 4 Shields/second, systems/combat.md > Shields). Does not change MaxShields. Applies to all Colonists units **and buildings** (the same objects that share the base rate). Per-level increase TBD.
- **Energy weapon upgrades** — improve the Colonists' energy weapons: beam/particle weapons such as the Plasma Turret's beam, and both Legionnaire weapons (the Sabre Legionnaire's melee weapon and the Rifle Legionnaire's ranged weapon). Antimatter missiles are excluded (upgraded at the Exophysics Engine).

Levels, costs, and times TBD.

Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.

## Plasma Turret

Colonists anti-ground defensive structure. Fires an energy beam at ground targets.

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - player owns at least one Fusion Core (defensive structures are the exception to the Colonists' no-prerequisites rule — see Tech Model)

Beam behavior is TBD — options under consideration: (a) continuous beam dealing damage over time while held on a target, (b) instant-hit beam per shot, (c) traveling plasma bolts with a beam visual. TargetDomain (presumably Ground), damage, range, timing, Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.

## Relay Array

A surface-to-surface communications facility. Carries control signals and data between Colonists assets across the surface — without it, a remotely operated machine like the Scout could neither be steered nor report what it sees (see Tech Model > Operability Principle).

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Unlocks
- **Scout** production at the Robo Portal (Unit Prerequisite)

### Research
Whether the Relay Array has upgrades: TBD.

Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.

## Navigation Spire

A tall, lighthouse-like communications structure that provides navigation guidance for large aircraft in this world's skies (see Tech Model > Operability Principle).

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Unlocks
- **Large flying units** (none designed yet; Unit Prerequisite). Assumed produced at the Star Portal — to be confirmed.

### Research
Whether the Navigation Spire has upgrades: TBD.

Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.

## Psionic Amplifier

A facility that amplifies psionic power, allowing the Colonists' psionic units to function on this world's surface (see Tech Model > Operability Principle).

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Unlocks
- **Psionic unit** at the Bio Portal (Unit Prerequisite). Unnamed, not yet designed.
- **Psionic air unit** at the Star Portal (Unit Prerequisite) — tentative; not yet decided whether this unit exists.

### Research
Whether the Psionic Amplifier has upgrades: TBD.

Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.

## Structural Analyzer

A facility that analyzes this world's local materials and conditions and transmits the results home, so the homeworld can fabricate armor and structures optimized for the surface. Everything is still built off-world; the Structural Analyzer supplies the local knowledge (see Tech Model > Operability Principle).

### Entity Type - Structure Type
### Faction - Colonists
### TechPrerequisite - None (Colonists buildings have no TechPrerequisites)

### Unlocks
- **Treaded vehicles** (TrackedVehicle base) at the Heavy Portal (Unit Prerequisite) — their treads and armor must be adapted to this world's terrain. None designed yet.

### Research
- **Armor upgrades** — all Colonists armor upgrades are researched here: separate lines for **infantry**, **vehicle**, and **flying** unit armor. Buildings get no armor upgrades (their upgradeable protection is shield regeneration, via the Fusion Core). Levels, costs, and times TBD.

#### Rationale: why armor is not researched at the Alloy Furnace
The refineries stay purely economic buildings. Refinery placement is driven by Drone logistics (near patches and Egress Nodes), while research buildings want safe placement; a refinery that also researched would force a bad trade-off, and a raid on it would cost both economy and research (Research rule 7). Every player builds an Alloy Furnace anyway, so armor there would come free with the economy — whereas every other Colonists upgrade or unlock is a commitment to a dedicated building.

Size, SymmetryType, MaxHP, MaxShields, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Warp In cost/time: see Warp In > Constructs List.
