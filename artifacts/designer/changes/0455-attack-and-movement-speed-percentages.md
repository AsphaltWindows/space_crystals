# 0455 — Metabolism and Inspire become percentages; one rule for attack-speed frames

- **Date**: 2026-11-27T13:40:00-05:00
- **Answers**: Q0180
- **Docs**:
  - `systems/combat.md > AttackSpeed` (new)
  - `factions/cults/augmentation.md > Increments` (Metabolism increment)
  - `factions/cults/units/prayer_mobile.md > Abilities > Inspire` (Stacks) and `> Placeholder stats` (Inspire)

## What changed and why
The user replaced the flat Metabolism effects with percentages, so that augmentations and Inspire work the same way.
- **Metabolism increment.** Each increment gives **+15% movement speed**. For unarmed units, today only the Recruit, it also gives **+15% attack speed**. Increments add, giving a range of −45% (−3) to +90% (+6).
  - This replaces +0.5 su/frame and −1 frame of FiringDuration per increment.
  - Armed infantry still get only the movement part.
- **Inspire** gives ×1.25 movement speed to every own infantry unit, and ×1.25 attack speed to Recruits. It multiplies after augmentations.
  - This replaces +1.5 su/frame and the cycle × 0.75.
- **Frames** (Q0180). The attack cycle is divided by the factor and rounded once, to the nearest frame, halves up. The phases share the new cycle in proportion: **all phases scale**, not only Firing.
  - **Minimums:** Firing at least 2 frames (user). Aim, Cooldown and Reload at least 1 frame (designer's reading).
  - A phase at its minimum passes the rest of the change to the others.
  - The shortest cycle is 5 frames.

## Implementation impact
- **Replaces M5-N11's Inspire placeholders:** rounding, Firing-only and no floor.
- **Replaces the Metabolism increment's flat values:** the Altar and Augmentations.
- **Factor** = (1 + 0.15 × net Metabolism increments) × (1.25 if inspired). It applies to MaxSpeed with no rounding, and to the attack cycle as above.
- How rounding is shared out among the phases is the Director's call, as long as they sum to the rounded cycle.
- All numbers are placeholders: 15% and ×1.25.

## Verification
Recruit attack cycles. The first number is without Inspire, the one in brackets with it:

| Metabolism increments | Cycle (inspired) |
|---|---|
| −3 | 22 (17) |
| −1 | 14 (11) |
| 0 | 12 (10) |
| +2 | 9 (7) |
| +4 | 8 (6) |
| +6 | 6 (5) |

MaxSpeed:
- A Level 2 Improved Metabolism Recruit moves at 7.6 su/frame, and 9.5 when inspired.
- A plain Militant moves 25% faster when inspired.
