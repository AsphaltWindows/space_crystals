# Q0062 — Tunnel Areas and the two layers

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slices M4-E and M4-F build.
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Area`, `> Tunnel Area Non-Overlap Rule`, `> Tunnel Space`; `factions/syndicate/structures.md > Tunnel Expansions`; `core/entities.md > Direct Placement`, `> All placement methods: no buildings on resources`; `core/map.md > DefaultTilePresets`.

## Question
1. **Non-overlap between players.** "Tunnel Areas cannot overlap with each other." Does this include other players' Tunnels? **Proposal:** the player's own Tunnels only. Enemy areas may overlap yours, and each player's expansions occupy their own layer.
2. **Area cells off the map.** A Tunnel near the map's edge has part of its area outside the map. **Proposal:** those cells aren't tunnel-region: Tunnel Space counts only cells on the map.
3. **What an expansion's placement checks.** Direct Placement lists surface checks: Buildable, no units, no structure overlap. For expansions it says only "validated against the relevant underground spatial rules (e.g., Tunnel Area bounds, no overlap)". **Proposal:**
   - (a) every tile is **Drillable**: Plane, Rugged and Cliff yes; Mountain and Water no;
   - (b) the owner's and neutral underground structures block it;
   - (c) surface structures, units and lingering ghosts don't;
   - (d) resources do ("whatever the placement method");
   - (e) the Tunnel's own footprint does, since it isn't tunnel-region.
4. **Surface placement over expansions.** **Proposal:**
   - surface structures ignore underground ones;
   - a Tunnel is the exception: its footprint counts in both layers, so it can't stand on an expansion its owner can see, whether its own or neutral;
   - an enemy's hidden expansion never blocks anything, so no placement can reveal it.
5. **Tunnel-region over terrain no expansion can use** (Water, Mountain). Does it still count for Tunnel Space? **Proposal:** yes. Only the map's bounds matter (point 2).

## Context
The underground and the surface share the grid. Every point in which an enemy's hidden expansion could refuse a placement, or change a ghost's colour, would leak it. The proposals keep each player's underground private.

## Options
- **Point 1:**
  - A, own Tunnels only, as proposed;
  - B, every Tunnel. The ghost then judges the enemy Tunnels the player knows of, at their seen tier (see Q0069 point 6), and the check on arrival uses the truth.
- **Point 3a:**
  - A, Drillable, as proposed;
  - B, Buildable, as for surface buildings;
  - C, no tile rule underground.
- **Points 2 and 5:** A, as proposed. B, the whole square counts. C, only cells an expansion could use count.
