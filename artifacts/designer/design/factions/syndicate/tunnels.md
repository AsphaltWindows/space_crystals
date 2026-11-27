# Syndicate: Tunnels

See also: `factions/syndicate/tunnels.md`, `factions/syndicate/tunnel_menus.md`, `factions/syndicate/tunnel_area_and_space.md` (split from `tunnels.md`, 2026-11-27).

## Tunnel Network

The Tunnel Network is the collective of all Tunnels owned by a Syndicate player. Units inside the Tunnel Network can travel between Tunnels freely, but can only enter or exit the network through a Tunnel whose tier is sufficient for their unit base category.

**Regeneration inside** (user, 2026-11-25; Q0063): **every unit inside the Tunnel Network slowly regenerates HP**: **1% of its MaxHP per second, placeholder**. This is the Syndicate's only healing (the other factions have shields, Shrines or Field Engineers). Units carried inside a Subterranean Transport don't count as being in the network themselves (user, 2026-11-26; Q0166; the transport itself regenerates).

**Transit delay** (user, 2026-11-25; Q0063): after a unit **enters** the network, there is a **small delay before it can exit**: **2 seconds (32 frames), placeholder**. After that it can exit from **any** Tunnel whose tier suits it, however far away: the delay is flat, not by distance. Units **produced** inside the network (by expansions) have **no delay**: they can exit as soon as they are produced (user, 2026-11-25). This includes produced units **rallied into the Tunnel Network** (rally point on a Tunnel, so they stay inside): they can be ejected at once. **An Agent that finishes building a Tunnel** is inside the network with **no delay** either: it can be ejected the moment the Tunnel completes (user, 2026-11-26; Q0087).

**Walking in** (user, 2026-11-25; Q0063): units enter through the entry path by **walking into it at their normal speed, one behind another with normal spacing**, like walking through any one-unit-wide passage; there is no fixed hold time per unit. A unit leaves the map once it has walked into the Tunnel. So an army files in about as fast as it can walk in single file, without big traffic jams.
- **On entering** (user, 2026-11-25; Q0063): the unit's orders are **cleared**, queued ones included; it is idle inside. It **stays in its control groups** (a recall selects only the members on the map; recalling a group whose members are **all** inside does **nothing**: selection, panel and camera unchanged, like recalling an empty group; user, 2026-11-26; Q0086). An Agent **keeps its load**.
- **Upgrading Tunnels** (user, 2026-11-25; Q0063): until the upgrade has fully finished, a Tunnel only accepts, and only lets out, units supported by the tier it had **before** the upgrade began.
- **Target Tunnel gone or unsuitable**: if the Tunnel a unit is heading for is destroyed, or its tier doesn't suit the unit, the Enter order ends and the unit idles (Q0063, as proposed).
- **No special waiting rule** (user, 2026-11-25; Q0063): a unit heading for the entry path moves like any unit. In a moving line it keeps making progress; if the way is genuinely blocked, the normal Failure rule applies (systems/controls/command_behaviors.md > MovingToLocation > Failure): it gives up after about 7 s and idles. An occasional idle unit is acceptable.

### Transit Tier Requirements
- Tier 1+: Infantry (Light Infantry, Heavy Infantry)
- Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)
- Tier 3+: Air units (Hover Craft, Glider)

### Losing Tunnels
- **Expansions become neutral**: when a Tunnel is destroyed, every expansion in its Tunnel Area becomes neutral — no longer owned by the Syndicate player.
  - Neutral expansions are **no longer hidden** (visible to everyone, no detection needed). This holds for **every expansion building of the destroyed Tunnel**, its lowered pop-up turrets included, which become visible at once (user, 2026-11-26; Q0091).
  - They **slowly lose HP** until destroyed. **Decay is a percentage of MaxHP per second** (user, 2026-11-25; Q0069): **1% per second, placeholder** (a Hideout lasts 100 s); the number is to be tuned.
  - **Attacking them** (Q0069): only with an **explicit Attack** order; right-click never attacks a neutral object and auto-targeting ignores them.
  - **Reclaiming happens** the moment the covering Tunnel, or its upgrade, **finishes**; a Tunnel under construction reclaims nothing (Q0069).
  - **Partly inside an own Tunnel Area** (user, 2026-11-25; Q0069): until reclaimed, a neutral expansion still **blocks its cells**: nothing can be built there, and they **count as occupied** for that Tunnel's Tunnel Space.
  - **Reclaiming**: if a Tunnel is built whose Tunnel Area fully covers a neutral expansion, and that Tunnel's tier meets the expansion's Tier Requirement, the expansion becomes owned by that Tunnel's owner — the original Syndicate player or an enemy Syndicate player — and stops decaying. It occupies that Tunnel's tunnel-region like any expansion (reducing Tunnel Space). An expansion only partly covered by the new Tunnel Area, or whose Tier Requirement is higher than the new Tunnel's tier, stays neutral (and keeps decaying) — until that Tunnel is upgraded to a tier where its Tunnel Area fully covers the expansion and meets its Tier Requirement, at which point the Tunnel takes ownership of it.
  - **In-progress work is destroyed**: any production or research in progress in the expansion when its Tunnel is destroyed is lost.
  - **Queued work is refunded**: units or research queued in the expansion but not yet started are removed and their full cost is refunded to the player.
- **Laser Turrets go neutral too** (user, 2026-11-26): a destroyed Tunnel's turrets behave like its other expansions (neutral, visible, high armor, decaying, reclaimable; Laser Turret > When its Tunnel is lost).
- **Defeat**: the shared rule applies (a player who owns no structures is defeated — see core/game_rules.md > Defeat Condition). Since turrets and expansions go neutral with their Tunnel, **losing its last Tunnel defeats the Syndicate** unless it owns some other structure. The 0-Supplies first Tunnel (Cost Scaling) still applies to a player who survives that way.
- **Stranded units**: whenever a Tunnel is destroyed, any unit inside the Tunnel Network that can no longer exit through any remaining Tunnel (no remaining Tunnel's tier is sufficient for its unit base category) is **destroyed**. Example: if the last Tier 2+ Tunnel is destroyed, every vehicle inside the network is destroyed, while infantry survive (they can still exit through Tier 1 Tunnels). If the last Tunnel of all is destroyed and the player is not defeated (only possible in a custom scenario: Laser Turrets don't keep a Syndicate player alive, 0360), every unit inside the network is destroyed.
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

Starting Space Crystals: **100** (same as GDO and the Colonists). With 100, the second Agent can start immediately and a starting Tunnel has its **third Agent at work** (Q0089: close to, not necessarily at, 12.5 Space Crystals per second; factions/syndicate/units/agent.md > The table is an ideal) around 0:40.

### Entity Type - Structure Type
### Constructed by - Agent
### TechPrerequisite - None

### Size - 4x4
### SymmetryType - ABCD

Each side of the Tunnel has a distinct function:
- **Side A**: Unit entrance/exit (units enter and emerge from the Tunnel Network here). It has **two paths: one for entering, one for exiting** (user, 2026-11-25). **Each path is one unit wide** and fixes where units go in and come out: entering and exiting units never block each other, and an army enters or leaves single file, each unit walking through at its normal speed with normal spacing (Tunnel Network > Walking in). (The Tier 3 air doors have their own rate: one aircraft per 8 frames, placeholder; EjectMenu.)
- **Side B**: Crystal drop-off, a **hatch** for Space Crystals (Agents deliver Space Crystals here). Each hatch is **1 grid square** long, near the middle of its side (user, 2026-11-25): of the side's two middle squares, the one **farther from Side A**, keeping Agents off the entry and exit paths (user, 2026-11-26; Q0076); it turns and flips with the Tunnel
- **Side C** (opposite Side A): Back wall (no gameplay function)
- **Side D**: Supply drop-off, a **hatch** for Supplies (Agents deliver Supplies here)

(User, 2026-11-25: corrected from the earlier C = Supplies, D = back wall. The two hatches are the sides **next to** the entrance, and the back wall is **opposite** it. Clockwise from the top in the default orientation: A entrance/exit, B crystal hatch, C back wall, D Supplies hatch.)

**Which half of Side A** (user, 2026-11-25; Q0063): the **exit path** is the half of Side A **nearer the crystal hatch** (Side B), the **entry path** the half **nearer the Supplies hatch** (Side D). Both turn and flip with the Tunnel.

Each resource must be delivered to **its own hatch**: crystals only at Side B, Supplies only at Side D (user, 2026-11-25; unchanged rule, now stated explicitly).

**Look by tier** (user, 2026-11-25):
- **Tier 1**: the Side A paths are **narrow and shaggy-looking**, supporting only infantry.
- **Tier 2**: the paths get **wider and well paved**, supporting vehicles.
- **Tier 3**: adds **two horizontal circular sliding doors**, **one for flying units entering and one for flying units exiting**. Aircraft use these doors, not the Side A paths.

Only one Agent may drop off resources at a side at a time. Because crystal and supply drop-offs are on separate sides, one crystal delivery and one supply delivery can occur simultaneously.

### Destructible - true
### Groupable - false
### SightRange - 5

### Tiers

Tunnels have 3 upgrade tiers. Each tier increases HP, Tunnel Area radius, and unlocks higher-tier buildings and unit transit. Tunnel Space is no longer a flat value per tier — see factions/syndicate/tunnel_area_and_space.md > Tunnel Space; it is derived from unused Tunnel Area instead.

| | Tier 1 (base) | Tier 2 | Tier 3 |
|---|---|---|---|
| HP | 600 | 800 | 1000 |
| PointArmor | 1 | 1 | 1 |
| FullArmor | 16 | 16 | 16 |
| Tunnel Area Radius | 2 (8x8) | 3 (10x10) | 4 (12x12) |
| Transit | Infantry | Infantry + Vehicles | Infantry + Vehicles + Air |
| Buildings | T1 expansions | T1 + T2 expansions | T1 + T2 + T3 expansions |

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

**Agent killed before construction starts** (user, 2026-11-26; Q0079): the order is **refunded at the current top price**, like an order that is cancelled or fails.
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

**The future area is reserved** (user, 2026-11-25; Q0060): from the start of the upgrade, the larger Tunnel Area of the target tier counts against other placements (factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule), so nothing placed during the upgrade can make the finished area overlap.

#### Construction Time
480 frames (30 seconds). Agent must be present for the duration.
