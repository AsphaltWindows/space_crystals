# Q0165 — The Phantom at a Tier 3 Tunnel's air doors: descending in and rising out

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — SD, the Tier 3 air doors and the Phantom, builds the placeholder in Options.
- **Docs**: `factions/syndicate/units/phantom.md`; `factions/syndicate/tunnels.md > Tunnel` (Look by tier); `factions/syndicate/tunnel_menus.md > EjectMenu`; `factions/syndicate/units/{specter,prism}.md > Placeholder stats`

## Question
Does a Phantom take time to descend into the entry door and to rise out of the exit door, open to fire meanwhile, or does it go in and come out at once?

## Context
- The Phantom: "It rises vertically out of a Tier 3 Tunnel's air door". Its UnitBase is "HoverCraft (enters and exits only through Tier 3 Tunnels' air doors)". No timings are given.
- **The doors:** "Tier 3: adds two horizontal circular sliding doors, one for flying units entering and one for flying units exiting." "The air doors are on the Tunnel's roof: the entry door over the entry path's half of the Tunnel, the exit door over the exit path's half (Q0064)."
  - Their rate: "one aircraft per 8 frames (0.5 s), placeholder (the same rate for the air entry door)".
- **The other aircraft at the doors have timings:**
  - the Specter: "Lift-off and landing: vertical, 24 frames each at the air door (it can be hit during both, as an air unit)";
  - the Prism: "Launch 32 frames (straight up) and transform 32 frames (vulnerable during both)".

## Options
- **(a) At once** (the placeholder): it is taken in as soon as its centre is over the entry door, and appears over the exit door at flying height.
- **(b) Over some frames, open to fire,** as "rises vertically" suggests: e.g. 24 frames each way as the Specter, so at the doors' rate up to three Phantoms are rising at once.
