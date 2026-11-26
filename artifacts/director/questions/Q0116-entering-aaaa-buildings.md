# Q0116 — Entering a building whose sides are all alike (Garrison, Temple, Shrine)

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — proceeding on assumption: (a), the side nearest the unit.
- **Docs**: `systems/controls/commands.md > Enter`; `factions/cults/structures/garrison.md`, `temple.md`, `shrine.md`; `core/entities.md > Side lettering convention`

## Question
The Garrison, Temple and Shrine are AAAA, and none names an entrance side. Enter says "The unit walks to the structure's entrance side and goes inside". Which side do units go in by?

## Context
- The Armory, Altar, Cathedral, Chop Shop and Airstrip name side B.
- The Garrison's units "leave through any side". For a defensive building, a long walk round to one fixed side under fire matters.
- The Armory's slice, now being built, takes the top side for an AAAA building, for now.

## Options
- **(a) Any side:** the side nearest the unit when it is ordered in.
- **(b) One fixed side:** the top in the default orientation, turning with the building.
- **(c) Per building:** (a) for the Garrison, and (b) for the Temple and the Shrine.

Proposed: (a).
