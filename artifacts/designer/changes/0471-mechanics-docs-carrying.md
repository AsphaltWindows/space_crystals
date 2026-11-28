# 0471 — Mechanics docs (systems/mechanics/); first one: Carrying Units

- **Date**: 2026-11-28T10:45:00-05:00
- **Answers**: Q0193
- **Docs**:
  - `systems/mechanics/carrying.md` (new)
  - `README.md > systems/mechanics/` (new section)
  - `systems/controls/commands.md > Enter` (carriers line)
  - `factions/syndicate/units/subterranean_transport.md` (Uses line; loading lines; Destroyed)
  - `factions/gdo/units/supply_chopper.md > Unit Transport`
  - `factions/cults/units/pilgrim.md > Carrying` (replaces Loading, Drop and Destroyed)

## What changed and why
The user chose option (a). Each shared mechanic gets **one doc** in `systems/mechanics/`, **defined independently of the units**. The user expects "a few archetypes", each defined on its own. Unit and structure docs say which mechanic and archetype they use, give their parameters, and list only their exceptions. A new mechanic gets its doc before the first unit that uses it.

The first one is **Carrying Units**, with these archetypes:
- **Ground carrier**: loads and unloads where it stands, with an optional loading state.
- **Landing carrier**: lands to load and unload, takes off by itself, uses targeted Unload All, and has an optional Load input.
- **Docked carrier**: loads and unloads only while landed on a given own structure, unloading along a given side.
- **Paradrop**: an addition for aircraft carriers.

The doc also holds the shared rules for loading, unloading (Q0191), passengers and carrier destroyed.

**No rule changed**: this is a move. The three carriers now reference the doc:
- Subterranean Transport: Ground carrier.
- Supply Chopper: Landing carrier.
- Pilgrim: Docked carrier + Paradrop.

## Implementation impact
- One carrying module with archetypes. No behaviour changes.
- Next migrations, one at a time, each with its own entry:
  - aircraft landing and rearming, with runways;
  - vehicle deploy;
  - then garrisoning (needed for the new Cults vehicle), channelling, energy pools, heals and auras, hidden deployed objects, and tanks and toggles.

## Verification
No gameplay change. Each carrier behaves as before; the rules are now read from `systems/mechanics/carrying.md` plus each unit's parameters.
