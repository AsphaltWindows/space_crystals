# Q0108 — Stale lines in the Cults docs and the Enter command

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — doc cleanup only; the build follows the readings below.
- **Docs**: `systems/controls/commands.md > Enter > Sets BaseCommandState`; `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`, `> Unit Commands`

## Question
Can these lines be brought up to date?
1. `commands.md > Enter > Sets BaseCommandState` says "TargetObject = Tunnel (ObjectInstance)". Enter now targets every structure that holds units (the Armory, Altar, Garrison, the GDO's Bunker).
2. `recruit.md > Unit Commands` says "Assist Construction — via panel button B", as its panel does. That letter waits for Q0095.

## Context
Found while planning M5. The rest of the Cults docs were consistent with one another.

## Options
- **(a)** "TargetObject = the structure (ObjectInstance)", and the Recruit's letter as answered.
- **(b)** Leave them.

Proposed: (a).
