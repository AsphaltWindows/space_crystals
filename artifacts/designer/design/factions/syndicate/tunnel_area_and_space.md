# Syndicate: Tunnel Area and Tunnel Space

See also: `factions/syndicate/tunnels.md`, `factions/syndicate/tunnel_menus.md`, `factions/syndicate/tunnel_area_and_space.md` (split from `tunnels.md`, 2026-11-27).

## Tunnel Area

The Tunnel Area is a square underground build zone centered on the Tunnel. The radius extends outward from the Tunnel's 4x4 footprint in each direction, forming a square of (radius + 4 + radius) per side.

Underground expansions are placed spatially within the Tunnel Area on the grid, occupying cells just like surface buildings. This provides:
- A physical limit on the number of expansions per Tunnel
- Meaningful scouting with detection (enemies can see what was built and where)
- Consistent placement mechanics with GDO's surface building system

## Tunnel Area display
(user, 2026-11-26; Q0086)
- **Selecting one of your Tunnels** shows its Tunnel Area on the map.
- **Placing a Tunnel** shows the ghost's Tier 1 area **and the would-be Tier 2 and Tier 3 areas**. Cells that would stop a later upgrade are highlighted **red**: non-Drillable cells, off-map cells, and cells overlapping the areas of **your own** other Tunnels (user, 2026-11-26; Q0086), since all of these block upgrades (Non-Overlap Rule). Enemy areas stay hidden, so an unseen enemy Tunnel can still block an upgrade. Players can see before placing whether the Tunnel can be fully upgraded there.
- **Placing an expansion** (from a selected Tunnel) shows that Tunnel's area too (user, 2026-11-26; Q0086).
- **Enemy Tunnel Areas are never shown.**
- View modes with toggles (e.g. show all own areas): maybe later.

## Tunnel Area Non-Overlap Rule

Tunnel Areas cannot overlap with each other. The non-overlap check uses the **current tier's area**, not the maximum potential area. However, upgrading a Tunnel is blocked if the radius increase would cause its Tunnel Area to overlap with another Tunnel's current area.

This means players can place Tier 1 Tunnels closer together for early aggression, but sacrifice the ability to upgrade them later. Spacing Tunnels further apart preserves upgrade potential for the late game.

**Drillable area** (user, 2026-11-25): a Tunnel can only be **placed** if every cell of its Tunnel Area is **on the map and Drillable** (no Water or Mountain; core/map.md > DefaultTilePresets). So **no Tunnel near the map edge** (closer than its area reaches). An **upgrade** is likewise blocked if the larger area would include a non-Drillable or off-map cell, so a Tier 1 Tunnel by a lake or a mountain may never be upgradable. The ghost shows this like any other invalid placement (terrain is always known to the player as far as explored; never-explored tiles pass on the ghost, and the real check happens on arrival).

**Enemies see a Tunnel's tier** (user, 2026-11-25; Q0069): it is a surface structure and each tier looks different (Look by tier).

**Pending ghosts and sites count** (user, 2026-11-26; Q0079):
- An Agent's **pending Tunnel ghost** (ordered, not yet started) reserves its **whole Tier 1 Tunnel Area** against its **owner's** further Tunnel placements: a new ghost overlapping it shows red. Other players don't know about it, so it doesn't block them; they are stopped once construction starts.
- A Tunnel **under construction** (a site) counts with its whole area for **every player**, from the moment the site appears.

**Every player's Tunnels count** (user, 2026-11-25; Q0062): the rule applies between **all** Tunnels, not just a player's own. In Syndicate-vs-Syndicate games a player can **crowd or block** an enemy's Tunnels: a Tunnel placed near an enemy one stops the enemy from building where the areas would overlap, and can stop the enemy Tunnel's upgrade.
- **The ghost** judges enemy Tunnels by what the player knows: their remembered position and their tier as last seen (core/entities.md > Worker-Built Structures > The ghost judges by what the player knows). The real check on arrival uses the truth, so a build can fail on arrival because an enemy Tunnel was upgraded unseen.
- **Upgrades** are blocked by any Tunnel's area, own or enemy (the upgrade check uses the truth; the upgrading Tunnel's owner learns that something blocks it).

**Crystal intake and stacked Tunnels.** A Tunnel's crystal side accepts one drop-off at a time (4 seconds per 50 Space Crystals), so one Tunnel takes in at most 12.5 Space Crystals per second however many patches surround it — reached with only 2–3 Agents (see Agent > Gathering). This is intended: the Syndicate has the lowest one-base saturation income of any faction, reaches it fastest, and grows by adding Tunnels (paid in Supplies) rather than by saturating one field. Two Tier 1 Tunnels may share a rich crystal field (doubling its intake), and this is allowed: such Tunnels can never be upgraded, and clustering puts a lot of Supplies' worth of Tunnels in one place where a single attack can take them all. The Syndicate is meant to benefit from spreading its Tunnels across the map.

## Tunnel Space

Tunnel Space is derived from unused underground space rather than provided as a flat value per Tunnel.

- **Tunnel-region**: a Tunnel's Tunnel Area, excluding the 4x4 grid units occupied by the Tunnel's own footprint. (E.g. a Tier 1 Tunnel has an 8x8 = 64 grid unit Tunnel Area, minus its own 16 grid unit footprint = 48 grid units of tunnel-region.)
- **Unused tunnel-region**: the portion of a Tunnel's tunnel-region not currently occupied by a placed expansion.
- **The whole area is always drillable** (user, 2026-11-25; replaces the earlier "off-map cells count" and "non-Drillable cells don't count"): a Tunnel can only be placed, and only be upgraded, if its whole Tunnel Area (at the new tier, for an upgrade) is on the map and Drillable (see Tunnel Area Non-Overlap Rule > Drillable area). So every tunnel-region cell is usable.
- **Tunnel Space = floor((sum of unused tunnel-region grid units across all owned Tunnels) / 4)** — rounded down to a whole number — capped at a hard maximum of 200 (see factions/syndicate/overview.md > SyndicateResources). Because of the rounding, structures smaller than 2x2 (e.g. a 1x1 pop-up Laser Turret, ¼ of a Tunnel Space) only lower Tunnel Space when they push the unused total below the next multiple of 4.

Building an expansion permanently converts part of a Tunnel's unused tunnel-region into occupied space, directly reducing total Tunnel Space. The only way to grow both production capacity and army capacity simultaneously is building more Tunnels or upgrading existing ones to increase their Tunnel Area (and therefore tunnel-region).

If placing a new expansion (or losing tunnel-region some other way) drops total Tunnel Space below the Syndicate player's currently-committed Tunnel Space usage, existing units are unaffected — usage is simply displayed over capacity (e.g. 12/10) and no further units can be produced until usage drops back within capacity.

### Production and Tunnel Space
Applies to every Syndicate production building (Hideout, Vehicle Assembly, Aerospace Assembly, and any later ones):

- **Queuing** a unit requires only its Space Crystals cost (paid when queued) and a free queue slot. Queuing does **not** check or reserve Tunnel Space.
- **Starting production**: when a queued unit reaches the front of the queue and is about to start, it requires free Tunnel Space (available minus used) at least equal to its TunnelSpaceCost. If there is enough, the unit's Tunnel Space counts as used from that moment — through production and for as long as the unit lives.
- **Blocked**: if there is not enough free Tunnel Space, the building **waits** — the unit stays at the front of the queue, production is paused, and it starts automatically as soon as enough Tunnel Space frees up (a unit dies, a Tunnel is built or upgraded). The player can still cancel it for a full refund. A blocked queue raises a **notification** for the player (notification system not yet designed).
- Shared with the other factions' capacities (factions/gdo/overview.md > Unit Control, 2026-11-24): several waiting buildings are served in a fixed internal order; a unit with TunnelSpaceCost 0 always passes.
