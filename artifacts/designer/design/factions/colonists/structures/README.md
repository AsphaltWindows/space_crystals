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
| Robo Portal | Machines (Drones, Scout). | Warped In from a Beacon |
| Bio Portal | Living beings (infantry): able to support the transport of living beings. | Warped In from a Beacon |
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

### Portal Exits (energy doorway)
(user, 2026-11-24) At the centre of every portal is an energy doorway from which units emerge. Portals don't use exit sides (core/entities.md > Structure Type > Exit Doorway); instead:
1. **Materializing**: a produced unit appears at the portal's **centre** and moves straight out until it is clear of the footprint (it passes through its own portal's footprint; nothing else can).
2. **Open directions**: it may leave in **any direction that isn't impeded by the building's structure**, i.e. any direction from the centre that passes out through an **open side** (the A sides). Robo Portal (AAAA): every direction, 360°. Bio Portal (AABB: top and right open in the default orientation, rotating with the building): every direction that leaves through the top or right side, a half-circle from pointing at the top-left corner round to pointing at the bottom-right corner.
3. **Direction chosen**: straight **toward the RallyPoint** if that direction is open. If it isn't, or the way out in that direction is blocked, the unit **tries to go around**: the nearest open direction in which it can get out. With no RallyPoint: a random open, unblocked direction (deterministic random).
4. **Can't get out**: if no direction is free, the unit stays half out at the edge of the footprint. It is a normal unit there (selectable, attackable, able to attack).
5. **Next unit**: production of the next unit **continues up to 100%**, but that unit **doesn't come out until the previous units have vacated** the way out; it waits fully inside, as in the Exit Doorway rule (a waiting unit is refunded if the portal is destroyed, like any producer's).

Heavy Portal and Star Portal: assumed to work the same way; their open sides are TBD with their sizes.

## Files

- `robo_portal.md` — **Robo Portal**: Drone production (standalone)
- `bio_portal.md` — **Bio Portal**: Basic military production
- `heavy_portal.md` — **Heavy Portal**: Portal upgrade stage 2
- `star_portal.md` — **Star Portal**: Portal upgrade stage 3
- `exophysics_engine.md` — **Exophysics Engine**: Tech building; unlocks Skylance
- `skylance.md` — **Skylance**: Anti-air defense
- `fusion_core.md` — **Fusion Core**: Tech building; unlocks Plasma Turret
- `plasma_turret.md` — **Plasma Turret**: Anti-ground defense
- `relay_array.md` — **Relay Array**: Communications tech building
- `navigation_spire.md` — **Navigation Spire**: Large-aircraft navigation tech building
- `psionic_amplifier.md` — **Psionic Amplifier**: Psionic tech building
- `structural_analyzer.md` — **Structural Analyzer**: Materials-analysis tech building
