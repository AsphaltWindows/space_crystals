# 0124 — Colonists panels: Beacon (two Warp In menus), Portals, Drone, transforming objects

- **Date**: 2026-11-24T15:00:00-05:00
- **Answers**: Q0040 (point 8)
- **Docs**: `factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState` (new), `> Beacon Deployment > While transforming: only Cancel` (hotkey X); `factions/colonists/structures.md > Robo Portal ObjectInterfaceState`, `> Bio Portal ObjectInterfaceState` (new); `factions/colonists/units.md > Drone ObjectInterfaceState`

## What changed and why
The Director's proposed layouts, plus one user change: the Beacon gets **two Warp In menus** (14 buildings don't fit 11 slots), like the DC's two build menus.
- Beacon: Q Warp In (Basic), W Warp In (Advanced).
  - Basic: Q Egress Node, W Robo Portal, E Bio Portal, A Alloy Furnace, S Essence Refinery, D Flux Reactor, Z Back.
  - Advanced: Q Exophysics Engine, W Skylance, E Fusion Core, R Plasma Turret, A Relay Array, S Navigation Spire, D Psionic Amplifier, F Structural Analyzer, Z Back.
  - Split and names provisional (user: easy to reshuffle later).
- Transforming objects: X Cancel (Deployment / Warp In) only.
- Robo Portal: Q Drone, W Scout, X Cancel Production, C Set Rally Point.
- Bio Portal: Q Sabre Legionnaire, W Lancer, E psionic unit (TBD), X, C.
- Drone: Q Move, A Deploy, S Set Drop-off, X Stop. No Hold Position/Patrol/Attack (confirmed).
- Egress Node: F Select Returning Drones (unchanged).

## Implementation impact
Buildings/units whose costs are TBD appear in their slot but unavailable. W/E slots on Portals (Scout, Lancer, psionic unit) follow the Produces order; they're placeholders until those units are designed.

## Verification
Select a Beacon: Q/W open the two menus; Z returns. Select a warping-in building: only X. Portals and Drone panels match the grids.
