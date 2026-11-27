# Q0166 — The Subterranean Transport: firing out, passengers in the network, and Tunnels while burrowed

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — ST1 (the Transport above ground: loading, unloading, Tunnels) and ST2 (burrowing) build the placeholders in Options.
- **Docs**: `factions/syndicate/units/subterranean_transport.md`; `factions/syndicate/tunnels.md > Tunnel Network`, `> Tunnel`; `factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff`, `> Placeholder stats`; `factions/gdo/structures/bunker.md > Garrison`

## Question
1. Do passengers fire out of the Transport?
2. While a loaded Transport is in the network, are its passengers outside the network, as the doc assumes, for regeneration and the Null Chamber?
3. Can a burrowed Transport enter a Tunnel from underground?

## Context
- **The Transport:** "AttackAttributes - none (confirmed, Q0071)". It carries "infantry only": "Capacity 4" heavy infantry. "If it is destroyed, all infantry inside die with it".
  - In a Bunker, by contrast, "Units inside fire their own weapons from the Bunker."
- **In the network:** "It can drive into a Tunnel with infantry inside". "Inside the network the passengers are still locked inside the transport".
  - "every unit inside the Tunnel Network slowly regenerates HP: 1% of its MaxHP per second, placeholder. This is the Syndicate's only healing". But "Units carried inside a Subterranean Transport don't count as being in the network themselves (assumed; the transport itself regenerates)."
  - The Null Chamber buffs "every manned Syndicate unit that exits the Tunnel Network through any Tunnel": for 20 s, +2 grid units of sight and 20% less damage taken, after a 2 s turn inside.
- **Entering:** units enter at a Tunnel's "Side A: Unit entrance/exit (units enter and emerge from the Tunnel Network here)". Loading needs the Transport above ground. Burrow and Surface take 48 frames (3 s) each, and "The last order given meanwhile is held and tried first once it's done".

## Options
1. **(a) No** (the placeholder): passengers are carried, not fighting. **(b) Yes, above ground, as from a Bunker:** a loaded Transport fights with up to 4 infantry's weapons.
2. **(a) Outside, as assumed** (the placeholder): passengers don't regenerate, get no Null Chamber turn, and come out unbuffed. **(b) Inside:** they regenerate with the Transport, and are buffed when it leaves through a Tunnel, the 20 s running while they are aboard.
3. **(a) No: it surfaces (3 s), then drives in** (the placeholder). **(b) It enters from below,** staying hidden throughout. **(c) An Enter order surfaces it by itself,** then it drives in.
