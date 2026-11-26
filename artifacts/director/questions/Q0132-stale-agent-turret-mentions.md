# Q0132 — Two doc lines still give the Syndicate Agent-built turrets

- **Date**: 2026-11-26T01:35:00-05:00
- **Blocking**: no — M5-Y built 0359 and 0360 as their entries say; these lines read as leftovers.
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Network > Losing Tunnels`; `factions/syndicate/units/agent.md > Building`

## Question
After 0359 (Agents build only Tunnels) and 0360 (turrets need a parent Tunnel, and don't keep a player alive), should these two lines be updated?

1. `factions/syndicate/tunnels.md > Losing Tunnels`, "Stranded units": "If the last Tunnel of all is destroyed and the player is not defeated (**they still own Laser Turrets**, or a custom scenario), every unit inside the network is destroyed."
   - Since 0360, turrets no longer keep a Syndicate player alive, so "they still own Laser Turrets" can't happen.
   - As built, only a custom scenario leaves a player undefeated with no Tunnel.
2. `factions/syndicate/units/agent.md > Building`, its intro: "Agents construct Tunnels **and surface defensive structures** (see Surface Defensive Structures) on the surface. Defensive structures use the same construction flow as Tunnels".
   - Since 0359, an Agent builds only Tunnels, and a turret is ordered from a Tunnel's ExpandMenu.

## Context
M5-Y (merged `c65cc1b0`) built 0358–0360:
- an Agent's build list is the Tunnel alone, and its panel is Q Move, A Build Tunnel, S Drop Off, D Repair (planned), X Stop;
- turrets start lowered, and are active only while they have a parent Tunnel;
- a turret goes neutral and decays with its Tunnel;
- defeat ignores turrets.

## Options
- **(a) Leftovers:** remove the turret clause from "Stranded units", and say in the Agent's intro that it builds only Tunnels (as built).
- **(b) Either line is meant:** say what it should do instead.
