# 0327 — Recruit build menus: A Basic, W Advanced, fixed slots by branch

- **Date**: 2026-11-26T18:05:00-05:00
- **Answers**: Q0096
- **Docs**: `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`

## What changed and why
User chose **Basic and Advanced**, Q0096's option (a), as on the GDO's Deployment Center.

Recruit panel: Q Move, **A** Basic Construct, **W** Advanced Construct, S Assist, D Repair, X Stop.

The split and slots are designer placeholders: every building has a fixed key and is greyed until available. Rows follow the branches (the A row is Tech/Weaponry, the X row is Mystical); Z goes back.

| Menu | Top row | A row (Tech/Weaponry) | X row (Mystical) |
|---|---|---|---|
| Basic | Q Recruitment Hall, W Storage | A Armory, S Gunworks, D Garrison | X Altar |
| Advanced | — | A Chop Shop, S Machine Shop, D Airstrip, F Scrap Hangar | X Temple, C Cathedral, V Shrine |

## Implementation impact
This replaces the single "A: Construct" menu, and the interim (b) layout if it was built.

## Verification
- Recruits: A then E places an Armory.
- W then A places a Chop Shop (greyed without an Armory).
