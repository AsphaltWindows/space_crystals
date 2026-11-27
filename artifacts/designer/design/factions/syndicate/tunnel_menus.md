# Syndicate: Tunnel Menus

See also: `factions/syndicate/tunnels.md`, `factions/syndicate/tunnel_menus.md`, `factions/syndicate/tunnel_area_and_space.md` (split from `tunnels.md`, 2026-11-27).

## Tunnel ObjectInterfaceState

**Panel letters (4x3 grid)** (user, 2026-11-25; Q0068, "fine for now"): **Q** Upgrade Tunnel, **W** Expand Tunnel, **E** Eject, **X** Cancel (upgrade, construction or the current operation). In the **ExpandMenu** and **EjectMenu**, entries take the grid slots in order (Q, W, E, R, A, S, …): expansions in the Tech Tree table's order (Hideout, Optics Lab, Chemical Plant, Vault, …), unit types in the order of `units/README.md` (Agent, Guard, Grenadier, …); **Z** goes back; unavailable entries are greyed. These letters replace the older ones listed below where they differ.

### DefaultState commands:
- **A: Upgrade Tunnel** — upgrades to next tier (CommandIssuingTransition). Costs Supplies per upgrade cost formula. Unavailable if already Tier 3 or if currently performing an operation.
- **B: Expand Tunnel** — enters ExpandMenu (StateOnlyTransition). Multi-stage: select an underground expansion to build, then place it within the Tunnel Area.
- **C: Eject** — enters EjectMenu (StateOnlyTransition). Multi-stage: select units from the Tunnel Network to eject from this Tunnel.
- **X: Cancel Upgrade** — cancels the in-progress Tunnel upgrade (CommandIssuingTransition). Supplies refunded at the current top price (factions/syndicate/tunnels.md > Upgrade Costs; Q0085). Only available while an upgrade is in progress.

### EjectMenu:
Displays a grid of unit type tiles representing all units currently in the Tunnel Network. Each tile shows the unit type icon and a count of how many of that type are in the network. Unit types whose base category exceeds this Tunnel's tier are visible but greyed out (disabled).

- Click an enabled unit type tile: ejects one unit of that type from this Tunnel's Side A (CommandIssuingTransition). Ejected units are queued. **Ground units walk out of the exit path single file, at their normal speed and spacing** (user, 2026-11-25; Q0064, mirroring Walking in; replaces the earlier "every 8 frames minimum"): the exit rate is set by unit speed and size. Each unit appears at the exit path's spot just outside Side A. **If that spot is blocked** (by any unit or structure), the next unit waits until it clears; no other spot is tried. With no rally point, a unit walks about 1 grid unit straight out of Side A and idles (a blocked walk follows the normal Failure rule). **Which unit comes out** (user, 2026-11-25; Q0064): of the chosen type, the one that has been **inside the network longest**. (Not "the healthiest": other per-unit factors such as re-arming at a Vault or the Null Chamber's re-buff may matter too.) **Not "the one whose services are done" either** (user, 2026-11-26; Q0158): a unit may be waiting for repair, re-arming, a refill or the buff, and the game doesn't know which of these the player is waiting for, so the rule stays **inside longest**. **The eject queue**: each click reserves a specific unit (the tile's count drops at the click); the queue can't be cancelled; if the Tunnel is destroyed with a queue, the queued units stay in the network (stranded-unit rule applies). **Air units** leave through the Tier 3 **air exit door**, in parallel with ground units; since aircraft don't collide, the door has its own **exit rate: one aircraft per 8 frames (0.5 s), placeholder** (the same rate for the air entry door). The **air doors are on the Tunnel's roof**: the entry door over the entry path's half of the Tunnel, the exit door over the exit path's half (Q0064).
- **The panel stays in the EjectMenu after a click** (user, 2026-11-26; Q0086), so repeated clicks eject more; each tile's count drops per click.
- **Every Syndicate unit type keeps its tile** (user, 2026-11-26; Q0086), in panel order (factions/syndicate/units/README.md > Files): a type with no units in the network shows a count of 0, greyed. Letters never move.
- **Units still in the transit delay** (user, 2026-11-26; Q0077): if every unit of a type is still within its 2-second transit delay, that type's tile is **greyed** and a click is **refused**; nothing is queued. It enables once one of them clears the delay.
- **Ejected units with no rally point** (user, 2026-11-26; Q0077): walk **1 grid unit straight out** of the exit path and go idle there. Following units nudge them aside under the idle-unit rules; to keep the exit clear, set a rally point.
- **Z**: returns to DefaultState (StateOnlyTransition)

### ExpandMenu:
Displays available underground expansion types for this Tunnel's current tier. An expansion is available only if (1) its Tier Requirement is at or below **this** Tunnel's tier, and (2) its TechPrerequisite (if any) is satisfied anywhere in the Tunnel Network (see Tunnel Expansions > Expansion Requirements).

- Click an expansion type: begins construction if the Tunnel is not already performing an operation (enters AwaitingPlacement for that expansion)
- **Fixed slots** (user, 2026-11-26; Q0081): **all nine expansions** are always listed, in the Tech Tree table's order, each holding its slot; ones not available (tier, prerequisite, or not yet in the build) are greyed. The **Laser Turret is always the tenth slot**, so no hotkey ever moves.
- The ExpandMenu also offers the **Laser Turret** (once the player owns an Optics Lab), placed within this Tunnel's Tunnel Area like an expansion; it becomes a pop-up turret (see Laser Turret).
- **Z**: returns to DefaultState (StateOnlyTransition)

### AwaitingPlacement (Expansion):
- A ghost preview of the expansion follows the cursor within the Tunnel Area, snapped to the grid. Tinted green when valid, red when invalid. The expansion must fit entirely within the Tunnel Area. **Every tile under it must be Drillable** (user, 2026-11-25; Q0062: Plane, Rugged and Cliff yes; Mountain and Water no; core/map.md > DefaultTilePresets). Own and neutral underground structures block it; so does the Tunnel's own footprint. **Surface structures block it too** (user, 2026-11-25; Q0062): no expansion can be placed under any surface building, any player's (judged by the truth; the placement is refused).
- **Ground units don't block it** (user, 2026-11-26; Q0080): units standing on the tiles don't matter, since the expansion is underground (core/entities.md > Direct Placement).
- **Lingering Tunnel ghosts can't be in the way** (user, 2026-11-26; Q0080): a Tunnel ghost can never be placed inside another Tunnel's area (factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule; pending ghosts reserve their area), and expansions are placed only inside their own Tunnel's area, so the two never meet.
- R rotates the ghost 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically.
- Left-click valid location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState)

### Expansions under construction
(user, 2026-11-25; Q0065)
- **Paid when the ghost is placed** (the left-click above). **Tunnel Space is used once construction has started.** Today placing the ghost starts construction at once (the Tunnel does one operation at a time, so there's no queue); if expansion queuing is ever added, payment stays at ghost placement and the Tunnel Space only goes when the building starts.
- **HP**: the ConstructionHP Rule applies to every expansion (starts at 10%, grows with progress); damage taken stays (current HP rises by the same amount).
- **Cancelling**: while building, the expansion offers only **X Cancel Construction**, refunding **75%** of its cost (rounded down). The Tunnel's X cancels whatever operation it is running (an upgrade or an expansion).
- **Sight**: none (underground). **Groupable**: true.
- **Its Tunnel destroyed mid-build**: the unfinished expansion is destroyed, no refund.
- **The Tunnel meanwhile** shows the progress; Eject still works (it isn't an operation).
- **Z**: returns to ExpandMenu (StateOnlyTransition)
