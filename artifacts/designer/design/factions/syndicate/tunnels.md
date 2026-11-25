# Syndicate: Tunnels

## Tunnel Network

The Tunnel Network is the collective of all Tunnels owned by a Syndicate player. Units inside the Tunnel Network can travel between Tunnels freely, but can only enter or exit the network through a Tunnel whose tier is sufficient for their unit base category.

### Transit Tier Requirements
- Tier 1+: Infantry (Light Infantry, Heavy Infantry)
- Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)
- Tier 3+: Air units (Hover Craft, Glider)

### Losing Tunnels
- **Expansions become neutral**: when a Tunnel is destroyed, every expansion in its Tunnel Area becomes neutral — no longer owned by the Syndicate player.
  - Neutral expansions are **no longer hidden** (visible to everyone, no detection needed).
  - They **slowly lose HP** until destroyed. Decay rate: TBD.
  - **Reclaiming**: if a Tunnel is built whose Tunnel Area fully covers a neutral expansion, and that Tunnel's tier meets the expansion's Tier Requirement, the expansion becomes owned by that Tunnel's owner — the original Syndicate player or an enemy Syndicate player — and stops decaying. It occupies that Tunnel's tunnel-region like any expansion (reducing Tunnel Space). An expansion only partly covered by the new Tunnel Area, or whose Tier Requirement is higher than the new Tunnel's tier, stays neutral (and keeps decaying) — until that Tunnel is upgraded to a tier where its Tunnel Area fully covers the expansion and meets its Tier Requirement, at which point the Tunnel takes ownership of it.
  - **In-progress work is destroyed**: any production or research in progress in the expansion when its Tunnel is destroyed is lost.
  - **Queued work is refunded**: units or research queued in the expansion but not yet started are removed and their full cost is refunded to the player.
- **Laser Turrets stay owned**: a destroyed Tunnel's pop-up Laser Turrets become upright turrets (see Laser Turret > Conversion) and remain the Syndicate player's.
- **Defeat**: the shared rule applies (a player who owns no structures is defeated — see core/game_rules.md > Defeat Condition). Losing the last Tunnel therefore defeats the Syndicate only if they own no Laser Turrets. A Syndicate player who survives this way can rebuild with a surviving Agent: with no Tunnels owned, a new Tunnel costs 0 Supplies (see Cost Scaling).
- **Stranded units**: whenever a Tunnel is destroyed, any unit inside the Tunnel Network that can no longer exit through any remaining Tunnel (no remaining Tunnel's tier is sufficient for its unit base category) is **destroyed**. Example: if the last Tier 2+ Tunnel is destroyed, every vehicle inside the network is destroyed, while infantry survive (they can still exit through Tier 1 Tunnels). If the last Tunnel of all is destroyed and the player is not defeated (they still own Laser Turrets, or a custom scenario), every unit inside the network is destroyed.
- A Tunnel that is still under construction does not count as an exit, and a Tunnel that is mid-upgrade counts at its current (pre-upgrade) tier.

## Tunnel

The Tunnel is the Syndicate's core surface structure. It serves as the entry/exit point to the Tunnel Network and defines an underground Tunnel Area where expansions (underground buildings) can be constructed. Tunnels are visible to enemies on the surface, but underground expansions within the Tunnel Area are invisible without detection.

The Syndicate player starts with one Tier 1 Tunnel and one pre-built Hideout expansion.

### Starting Conditions
Where each starting object is placed is defined by the map's Starting Layout for this faction (core/map_editor.md > Starting Layouts).

The Syndicate player starts with:
- One pre-built **Tier 1 Tunnel** containing one pre-built **Hideout**
- One **Agent**
- **1 Supply** (enough for the 2nd Tunnel, which costs 1 Supply)

Starting Space Crystals: **100** (same as GDO and the Colonists). With 100, the second Agent can start immediately and a starting Tunnel is saturated (3 Agents, 12.5 Space Crystals per second) around 0:40.

### Entity Type - Structure Type
### Constructed by - Agent
### TechPrerequisite - None

### Size - 4x4
### SymmetryType - ABCD

Each side of the Tunnel has a distinct function:
- **Side A**: Unit entrance/exit (units enter and emerge from the Tunnel Network here). It has **two paths: one for entering, one for exiting** (user, 2026-11-25). **Each path takes one unit at a time** and fixes where units go in and come out: entering and exiting units never block each other, but an army enters or leaves single file. (Whether the Tier 3 air doors also take one aircraft at a time: TBD.)
- **Side B**: Crystal drop-off, a **hatch** for Space Crystals (Agents deliver Space Crystals here)
- **Side C**: Supply drop-off, a **hatch** for Supplies (Agents deliver Supplies here)
- **Side D**: Back wall (no gameplay function)

Each resource must be delivered to **its own hatch**: crystals only at Side B, Supplies only at Side C (user, 2026-11-25; unchanged rule, now stated explicitly).

**Look by tier** (user, 2026-11-25):
- **Tier 1**: the Side A paths are **narrow and shaggy-looking**, supporting only infantry.
- **Tier 2**: the paths get **wider and well paved**, supporting vehicles.
- **Tier 3**: adds **two horizontal circular sliding doors**, **one for flying units entering and one for flying units exiting**. Aircraft use these doors, not the Side A paths.

Only one Agent may drop off resources at a side at a time. Because crystal and supply drop-offs are on separate sides, one crystal delivery and one supply delivery can occur simultaneously.

### Destructible - true
### Groupable - false
### SightRange - 5

### Tiers

Tunnels have 3 upgrade tiers. Each tier increases HP, Tunnel Area radius, and unlocks higher-tier buildings and unit transit. Tunnel Space is no longer a flat value per tier — see Tunnel Space below; it is derived from unused Tunnel Area instead.

| | Tier 1 (base) | Tier 2 | Tier 3 |
|---|---|---|---|
| HP | 600 | 800 | 1000 |
| PointArmor | 1 | 1 | 1 |
| FullArmor | 16 | 16 | 16 |
| Tunnel Area Radius | 2 (8x8) | 3 (10x10) | 4 (12x12) |
| Transit | Infantry | Infantry + Vehicles | Infantry + Vehicles + Air |
| Buildings | T1 expansions | T1 + T2 expansions | T1 + T2 + T3 expansions |

### Tunnel ObjectInterfaceState

#### DefaultState commands:
- **A: Upgrade Tunnel** — upgrades to next tier (CommandIssuingTransition). Costs Supplies per upgrade cost formula. Unavailable if already Tier 3 or if currently performing an operation.
- **B: Expand Tunnel** — enters ExpandMenu (StateOnlyTransition). Multi-stage: select an underground expansion to build, then place it within the Tunnel Area.
- **C: Eject** — enters EjectMenu (StateOnlyTransition). Multi-stage: select units from the Tunnel Network to eject from this Tunnel.
- **X: Cancel Upgrade** — cancels the in-progress Tunnel upgrade (CommandIssuingTransition). Full refund of Supplies cost. Only available while an upgrade is in progress.

#### EjectMenu:
Displays a grid of unit type tiles representing all units currently in the Tunnel Network. Each tile shows the unit type icon and a count of how many of that type are in the network. Unit types whose base category exceeds this Tunnel's tier are visible but greyed out (disabled).

- Click an enabled unit type tile: ejects one unit of that type from this Tunnel's Side A (CommandIssuingTransition). Ejected units are queued — a new unit begins ejecting every 8 frames minimum, but actual throughput is limited by unit speed and collision at Side A. Standard movement and collision mechanics apply as units emerge.
- **Z**: returns to DefaultState (StateOnlyTransition)

#### ExpandMenu:
Displays available underground expansion types for this Tunnel's current tier. An expansion is available only if (1) its Tier Requirement is at or below **this** Tunnel's tier, and (2) its TechPrerequisite (if any) is satisfied anywhere in the Tunnel Network (see Tunnel Expansions > Expansion Requirements).

- Click an expansion type: begins construction if the Tunnel is not already performing an operation (enters AwaitingPlacement for that expansion)
- The ExpandMenu also offers the **Laser Turret** (once the player owns an Optics Lab), placed within this Tunnel's Tunnel Area like an expansion; it becomes a pop-up turret (see Laser Turret).
- **Z**: returns to DefaultState (StateOnlyTransition)

#### AwaitingPlacement (Expansion):
- A ghost preview of the expansion follows the cursor within the Tunnel Area, snapped to the grid. Tinted green when valid, red when invalid. The expansion must fit entirely within the Tunnel Area. **Every tile under it must be Drillable** (user, 2026-11-25; Q0062: Plane, Rugged and Cliff yes; Mountain and Water no; core/map.md > DefaultTilePresets). Own and neutral underground structures block it; so does the Tunnel's own footprint. **Surface structures block it too** (user, 2026-11-25; Q0062): no expansion can be placed under any surface building, any player's (judged by the truth; the placement is refused).
- R rotates the ghost 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically.
- Left-click valid location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState)
- **Z**: returns to ExpandMenu (StateOnlyTransition)

### Tunnel Area

The Tunnel Area is a square underground build zone centered on the Tunnel. The radius extends outward from the Tunnel's 4x4 footprint in each direction, forming a square of (radius + 4 + radius) per side.

Underground expansions are placed spatially within the Tunnel Area on the grid, occupying cells just like surface buildings. This provides:
- A physical limit on the number of expansions per Tunnel
- Meaningful scouting with detection (enemies can see what was built and where)
- Consistent placement mechanics with GDO's surface building system

### Tunnel Area Non-Overlap Rule

Tunnel Areas cannot overlap with each other. The non-overlap check uses the **current tier's area**, not the maximum potential area. However, upgrading a Tunnel is blocked if the radius increase would cause its Tunnel Area to overlap with another Tunnel's current area.

This means players can place Tier 1 Tunnels closer together for early aggression, but sacrifice the ability to upgrade them later. Spacing Tunnels further apart preserves upgrade potential for the late game.

**Every player's Tunnels count** (user, 2026-11-25; Q0062): the rule applies between **all** Tunnels, not just a player's own. In Syndicate-vs-Syndicate games a player can **crowd or block** an enemy's Tunnels: a Tunnel placed near an enemy one stops the enemy from building where the areas would overlap, and can stop the enemy Tunnel's upgrade.
- **The ghost** judges enemy Tunnels by what the player knows: their remembered position and their tier as last seen (core/entities.md > Worker-Built Structures > The ghost judges by what the player knows). The real check on arrival uses the truth, so a build can fail on arrival because an enemy Tunnel was upgraded unseen.
- **Upgrades** are blocked by any Tunnel's area, own or enemy (the upgrade check uses the truth; the upgrading Tunnel's owner learns that something blocks it).

**Crystal intake and stacked Tunnels.** A Tunnel's crystal side accepts one drop-off at a time (4 seconds per 50 Space Crystals), so one Tunnel takes in at most 12.5 Space Crystals per second however many patches surround it — reached with only 2–3 Agents (see Agent > Gathering). This is intended: the Syndicate has the lowest one-base saturation income of any faction, reaches it fastest, and grows by adding Tunnels (paid in Supplies) rather than by saturating one field. Two Tier 1 Tunnels may share a rich crystal field (doubling its intake), and this is allowed: such Tunnels can never be upgraded, and clustering puts a lot of Supplies' worth of Tunnels in one place where a single attack can take them all. The Syndicate is meant to benefit from spreading its Tunnels across the map.

### Tunnel Space

Tunnel Space is derived from unused underground space rather than provided as a flat value per Tunnel.

- **Tunnel-region**: a Tunnel's Tunnel Area, excluding the 4x4 grid units occupied by the Tunnel's own footprint. (E.g. a Tier 1 Tunnel has an 8x8 = 64 grid unit Tunnel Area, minus its own 16 grid unit footprint = 48 grid units of tunnel-region.)
- **Unused tunnel-region**: the portion of a Tunnel's tunnel-region not currently occupied by a placed expansion.
- **Cells off the map count** (user, 2026-11-25; Q0062): a Tunnel near the map edge keeps its whole square for Tunnel Space, including the part past the edge. No expansion can be built there, and an edge Tunnel can't be crowded from that side; both are judged minor.
- **Tunnel Space = floor((sum of unused tunnel-region grid units across all owned Tunnels) / 4)** — rounded down to a whole number — capped at a hard maximum of 200 (see factions/syndicate/overview.md > SyndicateResources). Because of the rounding, structures smaller than 2x2 (e.g. a 1x1 pop-up Laser Turret, ¼ of a Tunnel Space) only lower Tunnel Space when they push the unused total below the next multiple of 4.

Building an expansion permanently converts part of a Tunnel's unused tunnel-region into occupied space, directly reducing total Tunnel Space. The only way to grow both production capacity and army capacity simultaneously is building more Tunnels or upgrading existing ones to increase their Tunnel Area (and therefore tunnel-region).

If placing a new expansion (or losing tunnel-region some other way) drops total Tunnel Space below the Syndicate player's currently-committed Tunnel Space usage, existing units are unaffected — usage is simply displayed over capacity (e.g. 12/10) and no further units can be produced until usage drops back within capacity.

#### Production and Tunnel Space
Applies to every Syndicate production building (Hideout, Vehicle Assembly, Aerospace Assembly, and any later ones):

- **Queuing** a unit requires only its Space Crystals cost (paid when queued) and a free queue slot. Queuing does **not** check or reserve Tunnel Space.
- **Starting production**: when a queued unit reaches the front of the queue and is about to start, it requires free Tunnel Space (available minus used) at least equal to its TunnelSpaceCost. If there is enough, the unit's Tunnel Space counts as used from that moment — through production and for as long as the unit lives.
- **Blocked**: if there is not enough free Tunnel Space, the building **waits** — the unit stays at the front of the queue, production is paused, and it starts automatically as soon as enough Tunnel Space frees up (a unit dies, a Tunnel is built or upgraded). The player can still cancel it for a full refund. A blocked queue raises a **notification** for the player (notification system not yet designed).
- Shared with the other factions' capacities (factions/gdo/overview.md > Unit Control, 2026-11-24): several waiting buildings are served in a fixed internal order; a unit with TunnelSpaceCost 0 always passes.

### Construction and Upgrade Rules

A Tunnel can only perform one operation at a time:
- Constructing an underground expansion, OR
- Upgrading to the next tier

It cannot do both simultaneously. Upgrading temporarily locks the Tunnel out of expansion construction for the upgrade duration.

### Cost Scaling

Tunnel construction and upgrades cost Supplies. Costs scale based on existing infrastructure.

#### Construction Cost
Cost = current Tunnels owned (all tiers count), in Supplies.

**Paid when the ghost is placed** (user, 2026-11-25; Q0061): an Agent's build order pays the Supplies **when it is given**, i.e. when the Tunnel's ghost is placed (core/entities.md > Placement Validation > Lingering build ghosts), not on arrival. **Lingering Tunnel ghosts count as Tunnels owned** for the price of further orders, so two orders given in a row cost 1 and 2 (likewise a Tunnel under construction counts). The order is refused, with feedback, if the player can't pay. If the order is cancelled, replaced, or fails before construction starts, it is refunded at the current top price (as for cancelling, Upgrade Costs below).
- 1st Tunnel: 0 Supplies (player starts with this Tunnel)
- 2nd Tunnel: 1 Supply
- 3rd Tunnel: 2 Supplies
- 4th Tunnel: 3 Supplies
- 5th Tunnel: 4 Supplies

#### Upgrade Costs
Higher-tier Tunnels count toward lower-tier cost calculations (a T3 Tunnel counts for both T2 and T1 cost scaling).

**Upgrade duration** (placeholder, designer, 2026-11-25, Q0060): **480 frames (30 seconds)** to Tier 2 and to Tier 3, the same as construction.

**Paid when the upgrade is ordered** (Q0060; follows from the full refund on Cancel Upgrade).

**Upgrade to Tier 2**: 2 + 2 x (number of T2+ Tunnels owned), in Supplies.
- 1st T2: 2 Supplies
- 2nd T2: 4 Supplies
- 3rd T2: 6 Supplies

**Upgrade to Tier 3**: 3 + 3 x (number of T3 Tunnels owned), in Supplies.
- 1st T3: 3 Supplies
- 2nd T3: 6 Supplies
- 3rd T3: 9 Supplies

**Upgrades in progress count** (user, 2026-11-25; Q0060): a Tunnel upgrading to Tier 2 already counts as a T2+ Tunnel (and one upgrading to Tier 3 as a T3 Tunnel) for the cost of further upgrades. Two upgrades started together cost 2 and 4, not 2 each.

**Refunds are the current top price** (user, 2026-11-25): Cancel Upgrade refunds **the price that upgrade would cost if ordered right now** (not counting itself), whichever Tunnel is cancelled: 2 + 2 x (other T2+ Tunnels owned) for Tier 2, 3 + 3 x (other T3 Tunnels owned) for Tier 3. Example: A and B upgrade to Tier 2 together (paying 2 and 4). Cancelling either one first refunds **4**; cancelling the other afterwards refunds **2**. The Supplies paid for what the player still has always equal the formula's total, so the order of starting and cancelling can't gain or lose Supplies. **Tunnel construction refunds the same way** (user, 2026-11-25): the current construction price, not counting itself.

**HP on completion** (user, 2026-11-25; Q0060): max HP rises (600 → 800 → 1000) and current HP rises by the same amount, so damage taken stays (like the Colonists' Warp In).

**Destroyed mid-upgrade** (user, 2026-11-25; Q0060): the Supplies paid for the upgrade are lost.

**The future area is reserved** (user, 2026-11-25; Q0060): from the start of the upgrade, the larger Tunnel Area of the target tier counts against other placements (Tunnel Area Non-Overlap Rule), so nothing placed during the upgrade can make the finished area overlap.

#### Construction Time
480 frames (30 seconds). Agent must be present for the duration.
