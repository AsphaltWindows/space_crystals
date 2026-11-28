# Q0193 — Define each shared mechanic once, in its own systems doc, and have unit docs compose them?

- **Date**: 2026-11-28T00:40:54-05:00
- **Blocking**: no. This is about how the design is organized, not about what it says. The user asked the Director to raise it with the Designer (2026-11-28) if the design's organization makes implementation harder.
- **Docs**:
  - `systems/` (today: `combat.md`, `units.md`, `controls/`);
  - `factions/gdo/units/artillery_rig.md > Modes` (the vehicle deploy model);
  - `factions/gdo/structures/airfield.md` (landing and rearming);
  - `factions/syndicate/units/subterranean_transport.md`, `factions/gdo/units/supply_chopper.md`, `factions/cults/units/pilgrim.md` (carrying units);
  - `factions/cults/units/disciple.md` (channelling).

## Question
Would you move each mechanic that several units share into a systems doc of its own, where it is defined once? Unit and structure docs would then say which mechanics they use, with their own numbers and exceptions.

## Context
- **The user's direction for implementation** (2026-11-28): "most unit additions should more or less be a compilation of existing mechanics into a new entity. If it requires a new behavior [...] that mechanic should be introduced first as a dependency independently, and then implement the [unit]." The Director now plans that way: a new mechanic is built first as its own module, and the unit is then assembled from mechanics. The user added that adding a mechanic to a large project shouldn't cost much more than to a small one, "as long as the working practices, architecture and documentation are effectively and economically organized".
- **The design is organized one file per object** (0282), and that works well for each unit's own rules and numbers. Shared mechanics, though, are defined inside the first unit or structure that had them, and other docs refer to them there or restate them:
  - **The vehicle deploy model** is defined in `artillery_rig.md > Modes`. The Laser Battery, the Prospector and the Subterranean Transport's Burrow and Surface refer to it there.
  - **Carrying units** is restated in each carrier's doc: the Subterranean Transport, the Supply Chopper and the Pilgrim each give loading, unloading and death with small differences. Q0191 asks three details that apply to all three of them.
  - **Aircraft landing and rearming** are in `airfield.md`, with runway variants in the Airstrip's and the Pilgrim's docs.
  - **Channelling** is defined in `disciple.md`. Heals and auras appear in the First Aid Tent's, the Shrine's and the Disciple's docs.
- **For an implementer,** learning one mechanic then means reading several unit docs and reconciling them. A new unit doc tends to restate a mechanic with small differences, and each difference becomes a question or a special case in the code.
- **This isn't about quality:** the docs are compact (124 files, 605 KB, the largest about 23 KB), and inline history (who decided what, and when) is only about 4% of the text.

## Options
- **(a) Mechanics docs.** A folder such as `systems/mechanics/`, one file per shared mechanic, each defined once: vehicle deploy, carrying units, aircraft landing and rearming, runways, channelling, energy pools, heals and auras, mines and other hidden deployed objects, tanks and toggles, and others as they come. Unit and structure docs then say "uses carrying: capacity 16, infantry only, loads only while landed" and list only their own exceptions. A new mechanic gets its doc before the first unit that uses it. That matches the implementation's mechanic-first order, one doc to one module.
- **(b) Lighter.** Keep the files as they are, but mark each mechanic's defining section as the canonical one, and have other docs refer to it rather than restate it.
- **(c) As is.**

The Director recommends (a). The migration can go one mechanic at a time, starting with those the remaining M6 work builds on: carrying units, aircraft landing (with runways), and vehicle deploy.
