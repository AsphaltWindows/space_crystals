# Q0082 — Supplies gathering: the reaches, the nearest Tunnel, and when the old load goes

- **Date**: 2026-11-25T01:47:32-05:00
- **Blocking**: no — proceeding on assumption: M4-I (running) builds the proposals below as placeholders.
- **Docs**: `factions/syndicate/units/agent.md > Gathering > Supplies`, `> Gathering rules`; `core/map.md > SupplyDeliveryStation`

## Question
1. How close must an Agent be to **pick up** at a Supply Delivery Station, and to **drop off** at the Supplies hatch (Side D)?
2. For a Supplies run, is "the nearest own operational Tunnel" measured to its **Supplies hatch**?
3. An Agent switching resources loses its old load "when it picks up the Supply". When exactly: when the pick-up **starts**, or when it **ends**? And the assumed reverse case: when mining starts, or when it ends?

## Context
- `> Supplies`: "CarryCapacity: 1 Supply per trip", "PickUpDuration: 48 frames", "DropOffDuration: 48 frames", "Drop-off side: Tunnel Side D". Unlike `> Space Crystals`, it names no reaches.
- `> Gathering rules` (0255): the nearest Tunnel is chosen by "straight line to its crystal hatch". That text predates Supplies at Side D.
- 0256: "its crystals are **lost when it picks up the Supply**. (Assumed the same the other way: a carried Supply is lost when it mines crystals.)" A Supply also leaves the station "when the pick-up ends".
- M4-D measured crystals with reaches of 8 su.

## Options
1. **Reaches:**
   - **A (proposed).** 8 su for both, as for crystals. Station and hatch work like a patch and a hatch.
   - **B.** Contact (0 su). Agents crowd closer, and the line at Side D gets tighter.
2. **Nearest Tunnel for Supplies:**
   - **A (proposed).** Measured to the Supplies hatch (Side D), since that is where the trip ends.
   - **B.** To the crystal hatch, as the text says. On Tunnels facing different ways, an Agent may then pick a Tunnel whose Side D is farther.
3. **When the old load goes:**
   - **A (proposed).** When the new pick-up (or mining) **ends**, with the new load. An Agent interrupted during the pick-up keeps its old load.
   - **B.** When it **starts**. Simpler, but an Agent ordered away mid-pick-up returns empty-handed.
