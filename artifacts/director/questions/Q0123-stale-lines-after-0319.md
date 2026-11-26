# Q0123 — Stale lines after 0319

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — doc cleanup; the build follows 0319.
- **Docs**: `factions/cults/units/recruit.md > Unit Commands`, `> Right-Click Resolution`; `systems/controls/commands.md > Enter`; `factions/cults/structures/chop_shop.md`, `airstrip.md`, `shrine.md`, `garrison.md`, `README.md > Cults Vehicles`

## Question
Can these lines be brought up to date?
1. **Gunworks and Machine Shop.** 0319 says they have "no commands and can't be entered (a right-click on it with Recruits is a Move)". Three places still say otherwise:
   - `recruit.md > Unit Commands`: "**Enter Gunworks** / **Enter Machine Shop** (staff research)";
   - `recruit.md > Right-Click Resolution`: "Own Gunworks / Machine Shop → Enter (staff research)";
   - `commands.md > Enter`: "**Cults — Temple, Gunworks, Machine Shop**: Recruits only".
2. **The Garrison.** `recruit.md > Right-Click Resolution` has no line for it, though `garrison.md` lets Recruits enter it by right-click.
3. **Old TBD lines** that the same files' Placeholder stats now settle:
   - `chop_shop.md > Production`: "Internal capacity, queue size, sides: TBD";
   - `airstrip.md > Production`: "(number per aircraft TBD)" and "Capacity, queue size, sides: TBD";
   - `shrine.md > Abilities`: the TBD cells of the table;
   - `structures/README.md > Cults Vehicles`: "Vehicles (not yet designed)", "Aircraft (not yet designed)".
4. **A typo** in `garrison.md`: "Whether an Disciple".

## Context
Found while planning the rest of the Cults for M5. 0322 settled the Recruit's panel letters, but not these lines.

## Options
- **(a)** Update them as listed: until their research is designed, a right-click on the Gunworks or Machine Shop is a Move, and "Own Garrison → Enter Garrison" is added.
- **(b)** Leave them.

Proposed: (a).
