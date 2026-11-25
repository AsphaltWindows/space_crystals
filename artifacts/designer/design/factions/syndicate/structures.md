# Syndicate: Structures

## Tech Tree Overview

Complete list of Syndicate structures. "Tier" is the Tier Requirement checked against the Tunnel building the expansion; "Requires" is the TechPrerequisite, satisfied by a fully constructed structure anywhere in the Tunnel Network (see Tunnel Expansions > Expansion Requirements and core/tech_and_research.md > Tech Prerequisites).

| Structure | Type | Built by | Tier | Requires | Role |
|---|---|---|---|---|---|
| Tunnel | Surface | Agent | — (upgradeable T1 → T2 → T3 with Supplies) | — | Network entry/exit, hosts expansions, Tunnel Space |
| Hideout | Expansion | Tunnel | 1 | — | Primary infantry accommodations; produces Agents, Guards |
| Optics Lab | Expansion | Tunnel | 1 | — | Laser weaponry tech |
| Chemical Plant | Expansion | Tunnel | 1 | — | Explosives and propulsion tech |
| Vault | Expansion | Tunnel | 1 | Hideout | Equipment storage for re-arming forces |
| Vehicle Assembly | Expansion | Tunnel | 2 | Chemical Plant | Ground vehicle production |
| Forge | Expansion | Tunnel | 2 | Vehicle Assembly | Advanced alloys and materials |
| Null Chamber | Expansion | Tunnel | 2 | Hideout | Shielded crystal consumption; manned units exiting the Tunnel Network get a temporary buff (detection, sight, damage reduction) |
| Aerospace Assembly | Expansion | Tunnel | 3 | Forge | Aviation and orbital (satellite) production |
| Orbital Command | Expansion | Tunnel | 3 | Aerospace Assembly | Control/coordination of orbital forces |
| Laser Turret | Surface / pop-up | Agent or Tunnel | — | Optics Lab | Defensive structure, attacks ground and air; inside a Tunnel Area it is a hidden pop-up turret |

## Tunnel Expansions

Underground buildings constructed within a Tunnel's Tunnel Area. Expansions are invisible to enemies without detection and can be walked over by surface units. All Syndicate units are produced by Tunnel expansions.

### Expansion Requirements

Each expansion has two independent kinds of requirement:

- **Tier Requirement — checked against the building Tunnel.** A Tunnel can only build expansions inside its own Tunnel Area, and the expansion's Tier Requirement must be met by **that specific Tunnel's** current tier. Other Tunnels' tiers are irrelevant. (E.g. a Tier 2 expansion can only be built by a Tier 2+ Tunnel.)
- **TechPrerequisite — checked across the whole Tunnel Network.** If an expansion requires other expansion type(s), those may be located in **any** Tunnel the player owns, regardless of that Tunnel's tier. (E.g. a Tier 2 expansion requiring a Hideout can be built by a Tier 2 Tunnel whose own Tunnel Area contains no Hideout, as long as a Hideout exists in some other Tunnel — even a Tier 1 one.)

TechPrerequisites follow the shared rules in core/tech_and_research.md > Tech Prerequisites: only fully constructed expansions count, the check happens only when construction is started, and losing a prerequisite never disables existing or in-progress expansions — it only blocks starting new ones.

### Rally Point Behavior

Each production expansion can have a rally point set. This determines what happens when a unit finishes production:

- **Rally point set on the surface**: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point.
- **No rally point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel.

### Hideout

The Hideout (renamed from Headquarters) is the primary accommodations for Syndicate infantry — a Tier 1 Tunnel expansion that produces Agents and Guards. The Syndicate player starts with one pre-built in their starting Tunnel. Additional Hideouts can be constructed in any Tier 1+ Tunnel.

#### Entity Type - Structure Type (Underground)
#### Size - 2x2
#### Tier Requirement - 1
#### TechPrerequisite - None
#### Cost - 200 Space Crystals
#### Build Time - 400 frames (25 seconds)
#### HP - 400
#### PointArmor - 1
#### FullArmor - 4
#### Produces:
- Agent: 100 Space Crystals, 240 frames (15 seconds)
- Guard: 125 Space Crystals, 120 frames (7.5 seconds). Unit Prerequisite: player owns at least one Optics Lab (2026-11-24).
- Grenadier: cost/time TBD. Unit Prerequisite: player owns at least one Chemical Plant (2026-11-25).

#### HideoutInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 20, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

#### ObjectInterfaceState[Hideout]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Agent**: deducts 100 Space Crystals from player, adds Agent to BuildQueue. Only available if BuildQueue has fewer than 20 entries and player has sufficient Space Crystals. Tunnel Space is checked when the Agent is about to start, not when queued (see Tunnel > Tunnel Space > Production and Tunnel Space).
- **W: Build Guard**: deducts 125 Space Crystals from player, adds Guard to BuildQueue. Only available if the player owns at least one Optics Lab, BuildQueue has fewer than 20 entries and player has sufficient Space Crystals. Tunnel Space is checked when the Guard is about to start, not when queued.
- **E: Build Grenadier** (hotkey tentative): deducts its cost (TBD), adds Grenadier to BuildQueue. Only available if the player owns at least one Chemical Plant, BuildQueue has fewer than 20 entries and the player can pay.
- **X: Cancel Production**: removes last entry from BuildQueue, refunds full cost to player. Only available if BuildQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

### Optics Lab

Syndicate tech building enabling the Syndicate's laser weaponry. It is the prerequisite for the **Guard** (laser rifle), the **Laser Turret** (2026-11-24), the **Laser Battery** and the **Phantom** (2026-11-25). Whether it also hosts laser research is TBD.

#### Entity Type - Structure Type (Underground)
#### Tier Requirement - 1
#### TechPrerequisite - None

(Full stats TBD)

### Chemical Plant

Syndicate tech building enabling the Syndicate's explosives and propulsion technologies. Prerequisite for the **Grenadier** (2026-11-25) and the Vehicle Assembly.

#### Entity Type - Structure Type (Underground)
#### Tier Requirement - 1
#### TechPrerequisite - None

(Full stats TBD)

### Vault

Syndicate equipment storage warehouse for re-arming Syndicate forces.

Prerequisite for the **Shadebringer** (2026-11-25). While the player owns a Vault, Shadebringers inside the Tunnel Network recharge their Sludge and Cover charges (factions/syndicate/units.md > Shadebringer > Charges and recharging).

#### Entity Type - Structure Type (Underground)
#### Tier Requirement - 1
#### TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)

(Full stats TBD)

### Vehicle Assembly

Syndicate production building that assembles Syndicate ground vehicles.

Produces (2026-11-25): **Assault Bike**, **Shadebringer** (requires a Vault), **Laser Battery** (requires an Optics Lab), **Subterranean Transport** (requires a Forge) (factions/syndicate/units.md). Costs, times and panel layout: TBD.

#### Entity Type - Structure Type (Underground)
#### Tier Requirement - 2
#### TechPrerequisite - player owns at least one Chemical Plant (anywhere in the Tunnel Network)

(Full stats TBD)

### Forge

Syndicate facility for developing advanced alloys and materials. Prerequisite for the **Subterranean Transport** (2026-11-25).

#### Entity Type - Structure Type (Underground)
#### Tier Requirement - 2
#### TechPrerequisite - player owns at least one Vehicle Assembly (anywhere in the Tunnel Network)

(Full stats TBD)

### Null Chamber

A shielded dome that works like a Faraday cage against imprinting, so crystal-based human enhancement can be done without the risk of an operative's identity being imprinted by consumers nearby (see core/lore.md > Space Crystals > Properties and core/lore.md > Syndicate > Paranoia about imprinting).

Distinct from the Syndicate's physical augmentation: Syndicate infantry are Heavy Infantry because of mechanical exoskeleton-style assistance, which involves no crystals. The Null Chamber is about **consumption**: its effects are subtle and mental (heightened awareness), unlike the Cults' morphological Altar augmentation and unlike Colonists psionics.

#### Null Chamber Exit Buff
While the player owns at least one Null Chamber anywhere in the Tunnel Network, every **manned** Syndicate unit that exits the Tunnel Network through any Tunnel receives a temporary buff that wears off over time.
- **Manned units**: all Syndicate units except satellites (infantry, vehicles, and crewed aircraft).
- **Effects**:
  - **Detection** (stealth/detection mechanic not yet designed — see loose ends)
  - **Increased SightRange**
  - **Percentage damage-taken reduction**, applied **before armor**: damage taken = Attack Damage × (1 − reduction) − armor
- **Refresh**: exiting the Tunnel Network again instantly resets the buff to full duration (no cooldown, for now).
- **Lore**: occupants consume crystals inside the Null Chamber, shielded from imprinting; the effect fades once they are on the surface.

TBD: duration; SightRange bonus; damage-reduction percentage; whether the buff ends abruptly or decays gradually; whether the buff (or which of its effects) must first be unlocked by research at the Null Chamber, and any further Null Chamber upgrades to the buff.

#### Entity Type - Structure Type (Underground)
#### Tier Requirement - 2
#### TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)

(Full stats TBD)

### Aerospace Assembly

Syndicate production building that assembles Syndicate aviation and orbital forces.

"Orbital forces" are primarily satellites, which operate in-game as air units using the HoverCraft unit base (systems/units.md). No separate orbital layer or domain — they are Air-domain units like any other.

#### Entity Type - Structure Type (Underground)
#### Tier Requirement - 3
#### TechPrerequisite - player owns at least one Forge (anywhere in the Tunnel Network)

Produces (2026-11-25): **Phantom** (requires an Optics Lab) (factions/syndicate/units.md).

(Full stats TBD)

### Orbital Command

Syndicate facility allowing for control and coordination of orbital forces.

#### Entity Type - Structure Type (Underground)
#### Tier Requirement - 3
#### TechPrerequisite - player owns at least one Aerospace Assembly (anywhere in the Tunnel Network)

(Full stats TBD)

## Surface Defensive Structures

Surface (above-ground) Syndicate structures constructed by Agents (the Laser Turret can also be built by a Tunnel). Unlike Tunnel expansions, they have no Tier Requirement and are not restricted to Tunnel Areas; when built by an Agent they follow the Worker-Built Structures placement rules (core/entities.md > Placement Validation). A Laser Turret that stands inside one of the player's Tunnel Areas becomes a pop-up turret (see Laser Turret). TechPrerequisites are satisfied by expansions anywhere in the Tunnel Network.

### Laser Turret

Syndicate defensive structure capable of attacking both ground and air units. Its form depends on where it stands: inside one of the player's Tunnel Areas it is a hidden **pop-up turret**; outside every Tunnel Area it is an **upright turret**.

#### Entity Type - Structure Type (Surface / pop-up)
#### Size - 1x1
#### Constructed by - Agent (anywhere) or Tunnel (within its own Tunnel Area, via the ExpandMenu)
#### TechPrerequisite - player owns at least one Optics Lab (anywhere in the Tunnel Network)
#### AttackAttributes.TargetDomain - Universal

#### Upright vs Pop-up (by location)
- **Upright turret** (outside every own Tunnel Area): a normal surface structure, always visible. Costs no Tunnel Space.
- **Pop-up turret** (inside an own Tunnel Area, whoever built it):
  - Occupies 1 grid unit of that Tunnel's tunnel-region, like an expansion, so it reduces Tunnel Space (¼ Tunnel Space, subject to the rounding rule in Tunnel Space).
  - While it has no target it stays **underground**, hidden like an expansion (invisible to enemies without detection).
  - It notices enemies the same way an upright turret does. When an enemy comes into range it rises to the surface **already facing the target** and fires; it retracts once it has no target. Rise and retract timing: TBD.
  - It **can be attacked while underground** (by enemies that can see it), but it has **much higher armor while underground** than when raised. Values TBD.

#### Conversion
The form updates automatically whenever Tunnel Areas change:
- A new Tunnel, or a Tunnel upgrade, whose Tunnel Area now covers an upright turret converts it into a pop-up turret (it starts occupying that Tunnel's tunnel-region).
- When a Tunnel is destroyed, the pop-up turrets in its Tunnel Area become upright turrets (their tunnel-region is lost along with the Tunnel).
- If a conversion pushes Tunnel Space below the player's current usage, the over-capacity rule applies (existing units unaffected; production waits — see Tunnel Space).

#### Construction
- **By an Agent**: built using the same flow as Tunnel construction (see Agent > Building > Tunnel Construction Flow): the Agent enters the partially-built structure and is untargetable for the duration; if the structure is destroyed or cancelled mid-construction the Agent survives and emerges at the location. On successful completion, the Agent emerges on the surface adjacent to the finished structure (unlike Tunnels, where the Agent ends up inside the Tunnel Network). If built inside a Tunnel Area, it is a pop-up turret on completion.
- **By a Tunnel**: from the Tunnel's ExpandMenu, placed within that Tunnel's Tunnel Area like an expansion. Whether it occupies the Tunnel's single operation slot (like an expansion) is TBD.

(Full stats TBD — cost, build time, HP, raised/underground armor, range, damage, rise/retract timing.)
