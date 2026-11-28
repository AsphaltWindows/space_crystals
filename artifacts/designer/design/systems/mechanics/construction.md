# Mechanic: Construction

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit or faction; gathers rules previously written in the Cults Building Mechanics, the Agent's and the Field Engineer's docs; no rule changed in the move. Placement itself stays in core/entities.md > Placement Validation; the site's HP in core/entities.md > ConstructionHP Rule.)

How a structure gets built. A builder's (or constructor's) doc says which archetype it uses, its parameters, and its exceptions.

## Common rules
- **Payment** (user): the cost is deducted **when the build order is given** (the order needs enough of each resource then), not on arrival. **Refunded in full** if the order ends **before construction starts** (the builder gets another order, is killed, or placement fails on arrival).
- **A site under construction does nothing but Cancel** (no services, no production, no capacity); it **counts as a structure** for the Defeat Condition (core/game_rules.md); its HP follows the ConstructionHP Rule. A site's only command is **X: Cancel Construction**.
- **Cancel refund**: a parameter (e.g. 75% rounded down; the current top price of a Tunnel's Supplies).
- **Builders go on to queued orders** after the site completes or is cancelled; with none, they idle.

## Archetype: Builders outside
(Cults Recruits; user, 2026-11-26) Builders work **from outside** the site and are **not consumed**; they stay on the field, vulnerable, as the site is.
- **Several builders**: the building needs a fixed number of **builder-frames** (BaseConstructionTime with 1 builder). Each frame, every builder **actively building** adds 1 (walking, waiting or anything else adds nothing). Speed scales linearly, with no cap but room around the site. **Assist** joins a site **already under construction** (not a ghost whose order hasn't started).
- **Pulled off**: progress **stalls** at its value (no decay) until builders resume or the site is cancelled. Re-ordering builders only removes them from the site; once started, the site is cancelled only through its own Cancel.
- **Where they work** (Q0097): each walks to the part of the footprint closest to it; if taken, it wall-hugs along the footprint to a free part, and builds within its reach of the edge.
- **Start** (Q0097): the site starts when the **first builder of the order arrives** (placement checked then). **Other units on the footprint** then make **the whole order fail** (refunded once in full; nothing retries). **The order's own builders** on the footprint (including one holding this build queued in a Shift chain) are **waited for** to step off (Q0130/Q0131; fallback if hard to build: the ghost is red while a selected builder stands on the footprint).
- **Under attack**: builders keep building.
- **Group refunds** (Q0098): the cost is refunded only once **no builder of the group still holds the order** (before it starts); **all builders killed** before the start: full refund; in a **Shift chain**, each queued building is refunded when its last holder drops it before it starts.
- **Cancel**: the site is destroyed (cosmetic rubble); builders stay on the field.

## Archetype: Builder inside
(Syndicate Agent, GDO Field Engineer; user, 2026-11-25/26) **One builder per site, no help.** The builder walks to the site; when construction begins, the site appears **with the builder inside it**, **untargetable and unseen**.
- **Completed**: where the builder goes is a parameter (e.g. into the faction's network; out through a door or at an edge).
- **Site destroyed**: the builder survives and comes out at the **centre of the footprint**; the cost is lost.
- **Cancelled**: the builder comes out at the centre of the footprint, with the cancel refund.
- A builder ordered to build while **carrying a load keeps it** (Agents).

## Archetype: Placed by a constructor
A structure placed directly from a menu (core/entities.md > Placement Validation > Direct Placement), by a constructor building or a network (e.g. the GDO Deployment Center's build queue; Syndicate expansions and turrets from a Tunnel's menu). The constructor's doc gives its queue and costs.

## Not these archetypes
Transformations (a Colonists Drone deploying into a Beacon; Warp In; a Prospector deploying into an Outpost) follow their own docs (factions/colonists/economy.md; systems/mechanics/modes.md > Deploy).

## Used by (index only)
Builders outside: Cults Recruits. Builder inside: Syndicate Agent (Tunnels), GDO Field Engineer (Bunker, First Aid Tent). Placed by a constructor: GDO Deployment Center, Syndicate Tunnel menus.
