# 0264 — Building armor rule: PointArmor n, FullArmor n × size in grid squares

- **Date**: 2026-11-25T14:35:00-05:00
- **Answers**: —
- **Docs**: `core/entities.md > Structure Type > Building armor rule` (new); `factions/syndicate/structures.md > Tunnel Expansions` (High armor underground), `> Laser Turret` (placeholders); `systems/combat.md > AoEDamage` (note)

## What changed and why
User: building armor should generally be **PointArmor n, FullArmor n × size in grid squares** (exceptions possible). Existing buildings already fit (Hideout 1/4, Tunnel 1/16).

Placeholders corrected to the rule:
- underground expansions: n = 6, so 6/24 for 2x2 (replaces 0262's 6/200);
- Laser Turret (1x1): raised 1/1, underground 6/6 (replaces 0260's 1/6 and 6/30).

## Implementation impact
- Update the placeholder armor values above.
- Note for tuning: under this rule a blast covering part of a big building meets about n per covered square. A Grenadier blast (100) over a hidden 2x2 expansion with n = 6 deals about 93.

## Verification
—
