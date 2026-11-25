# Q0081 — The ExpandMenu's slots: expansions not in the game yet, and the Laser Turret's slot

- **Date**: 2026-11-25T01:47:32-05:00
- **Blocking**: no — proceeding on assumption: every expansion of the Tech Tree table holds its slot, greyed while unavailable (including the ones M4 doesn't build), and the turret takes the slot after them.
- **Docs**: `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > Panel letters`, `> ExpandMenu`; `factions/syndicate/structures/README.md > Tech Tree Overview`; `factions/syndicate/structures/laser_turret.md` (Hotkey)

## Question
In the ExpandMenu, do expansions that M4 doesn't build yet (Vault, Vehicle Assembly, Forge, Null Chamber, Aerospace Assembly, Orbital Command) hold their slots, greyed, so that the Laser Turret's slot never moves?

## Context
- 0257: "entries take the grid slots in order (Q, W, E, R, A, S, …): expansions in the Tech Tree table's order (Hideout, Optics Lab, Chemical Plant, Vault, …) ...; unavailable entries are greyed."
- The turret "takes the next slot after the expansions". With all nine expansions in the menu it is the tenth slot; with only M4's three it is the fourth, and it would move each time an expansion is added to the game.
- The 4x3 grid has 12 slots and Z goes back, so ten entries fit.
- M4-F builds the menu from a data list, so either answer is a data change.

## Options
- **A (proposed).** All nine expansions are listed, in table order; those not in this build are greyed ("not available yet"). The turret is always the tenth slot, so its letter is stable from M4 on.
- **B.** Only expansions that exist in the build are listed. The menu is shorter now, but the turret's letter changes as expansions arrive, and players' habits break.
- **C.** Only expansions available at this Tunnel's tier are listed. "Unavailable entries are greyed" suggests they are shown, so this departs from 0257.
