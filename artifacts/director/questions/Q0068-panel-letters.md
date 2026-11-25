# Q0068 — Panel letters outside the 4x3 grid

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slices M4-J1 and M4-J2 build.
- **Docs**: `factions/syndicate/units.md > Agent ObjectInterfaceState`; `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`, `> EjectMenu`, `> ExpandMenu`; `systems/controls/selection_and_hud.md > Grid Layout`, `> Standard Slot Assignments`.

## Question
1. **The Agent** has "A: Build Tunnel", "B: Drop Off Resources" and "Build Laser Turret (hotkey TBD)". Move and Stop have no letter, and B isn't a key in the grid (Q W E R / A S D F / Z X C V). **Proposal:** Q Move, A Build Tunnel, S Drop Off, D Build Laser Turret, X Stop. This mirrors the Drone (Q Move, A Deploy, S Set Drop-off, X Stop). D shows greyed until the Laser Turret exists.
2. **Attack, Hold Position, Patrol and AttackMove for the Agent.** None is listed. **Proposal:** none on its panel; a right-click attacks. Taking BasicCombatUnit's would put A Attack against A Build Tunnel.
3. **The Tunnel** has "A: Upgrade", "B: Expand", "C: Eject", "X: Cancel Upgrade". **Proposal:** Q Upgrade, W Expand, E Eject, X Cancel.
4. **The ExpandMenu and EjectMenu slots.** **Proposal:**
   - entries in the order of the Tech Tree table (Hideout Q, Optics Lab W, Chemical Plant E, Vault R, and so on) and of `units.md` (Agent Q, Guard W, Grenadier E, and so on);
   - Z goes back;
   - unavailable entries are greyed.

## Context
These letters predate the 4x3 grid. The Hideout's letters (Q, W, E, X, C) already fit it.

## Options
- **Point 1:** A, as proposed. B, letters you give.
- **Point 3:**
  - A, Q, W and E, as proposed;
  - B, A Upgrade, S Expand, C Eject. C is free, since the Tunnel produces nothing.
