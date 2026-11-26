# 0330 — HP rules: augmentation caps HP at the new max; no floors; vehicles and aircraft come out at full HP; expansion regeneration runs in combat

- **Date**: 2026-11-26T19:00:00-05:00
- **Answers**: Q0114 (points 1 and 3; point 2 below), Q0121 (point 2)
- **Docs**:
  - `factions/cults/augmentation.md > Increments` (HP when MaxHP changes; no floors);
  - `factions/cults/structures/chop_shop.md`, `airstrip.md` (Full HP);
  - `factions/syndicate/structures/README.md > Tunnel Expansions`.

## What changed and why
User:
- **Q0114.1:** when MaxHP changes, if HP is above the new MaxHP it becomes the new MaxHP. Otherwise it is unchanged, so an augmentation never kills. This is option (b): HP is kept, capped. On an increase, HP stays where it was.
- **Q0114.3:** no floors. The MaxHP 10, Damage 0 Recruit is fine.
- **Q0121.2:** vehicles and aircraft come out at **full HP**, regardless of the Recruits' damage.
- **Expansion regeneration** (0329) runs **during combat too**.

Still open with the user: Q0114.2 and Q0121.1 (which unit is reserved), Q0121.3 and Q0105 (whether infantry trained from a damaged Recruit keep its damage). Keep the interim reading ("inside the longest"; keeps damage) until they are answered.

## Implementation impact
- HP cap on MaxHP changes.
- Full HP for Chop Shop and Airstrip products.

## Verification
- A Recruit at 15/40 given Improved Metabolism L2 ends at 10/10.
- A Jackal from two hurt Recruits spawns at 120/120.
