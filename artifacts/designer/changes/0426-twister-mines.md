# 0426 — Twister mines: deployed units, selectable; hover units pass over; no blast when destroyed; queued buying

- **Date**: 2026-11-26T18:45:00-05:00
- **Answers**: Q0160
- **Docs**: `factions/gdo/units/twister.md > Mines`, `> Placeholder stats`; `factions/syndicate/units/subterranean_transport.md > Underground movement` (wording)

## What changed and why
- **1**: a mine is "closer to a permanently deployed unit". It is **not a structure** (doesn't count for the Defeat Condition) and **can be selected** by its owner (enemies only when they see it; detection TBD). Designer placeholders: Unit Control 0, Groupable true, an empty command panel for now.
- **2 (b)**: **hover units** (HoverVehicle, HoverCraft) **don't trigger** mines, as with sludge. Designer reading: a blast set off by something else still damages a HoverVehicle in its radius (it is a ground unit; only air is exempt).
- **3 (a)**: a mine destroyed by damage is removed **without** exploding.
- **4 (b)**: Buy Mine presses **queue up to the free stock**, each paid at the press, bought one after another (5 s each).

## Implementation impact
- 1 replaces the placeholder (a) "not selectable": mines need a selection shape and an InfoPanel (16x16, HP 40).
- 2 replaces the placeholder (a).
- 4 replaces the placeholder (a) "refused".

## Verification
- Lay a mine, click it: it is selected and shows its HP; a player with only mines left (no buildings) is defeated.
- A Colonists Drone drives over a mine: nothing happens. A Recruit walks in: it explodes after 8 frames, and a Drone in the radius takes damage.
- Kill a mine with area fire: it disappears without a blast.
- Twister with 0 mines: press F three times: 75 SC paid, three mines arrive at 5, 10 and 15 s.
