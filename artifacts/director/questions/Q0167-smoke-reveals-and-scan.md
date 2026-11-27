# Q0167 — Smoke: revealed attackers, and Scan

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — SC, the Shadebringer's Cover (smoke), builds the placeholders in Options.
- **Docs**: `factions/syndicate/units/shadebringer.md > Cover (toggle)`, `> Placeholder stats`; `core/vision.md > Attacks from outside vision`; `factions/gdo/structures/radar_station.md > Placeholder stats`; `factions/syndicate/units/prism.md > Sight`

## Question
1. An attacker inside or behind smoke damages your unit. Is it revealed to you, as an attacker in the fog is?
2. Does smoke block a Radar Station's Scan?

## Context
- **Smoke** "blocks line of sight into it and behind it for every unit except satellites: it's not possible to see into the smoke, or past it, from the other side". "Units inside the smoke can't see anything".
  - "Structures too": "the smoke blocks buildings' sight the same way as units'. Only satellites see through it."
  - The wall "is 64 su (1 grid unit) wide and blocks sight at any height except satellites'". Each piece lasts 12 s.
  - The Prism, the Syndicate's satellite: "It sees through Shadebringer smoke".
- **Revealed attackers:** "a unit or structure that attacks one of your units or structures while outside your vision becomes visible to you for 1.5 seconds (24 frames), placeholder, after each shot". "For projectiles and area damage ... the reveal happens when the damage lands". "While revealed it is a normal visible object for you: it can be seen, selected and targeted".
  - A unit in or behind smoke can still fire at what its owner sees by other means, e.g. a target spotted by a Prism.
- **Scan** "reveals a circle of radius 6 grid units for 8 seconds", anywhere on the map. It isn't a unit's or a structure's sight.

## Options
1. **(a) Yes, revealed as in the fog** (the placeholder): the reveal isn't sight, so smoke doesn't shield a firing unit from return fire. **(b) No:** smoke hides attackers, as "it's not possible to see into the smoke, or past it" reads. They are revealed only once they leave it, or it clears.
2. **(a) Yes, smoke blocks Scan** (the placeholder), as "Only satellites see through it" reads: a Scan shows nothing inside or behind smoke. **(b) No:** a Scan sees from above, as a satellite does.
