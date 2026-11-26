# Q0114 — Augmentation: HP when MaxHP changes, the reserved unit, and the weakest Recruit

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — proceeding on assumption: the proposal below.
- **Docs**: `factions/cults/augmentation.md > Increments`, `> Augmentation Effects`, `> Augmentation Orders`; `systems/combat.md > MinimumDamage`

## Question
1. **HP when MaxHP changes.** An augmentation changes MaxHP by 10 per Constitution increment, up or down. What happens to a damaged unit's HP? Can a negative increment kill a badly damaged unit?
2. **Which unit is reserved.** An order "reserves one unit of the selected type and AugmentationState". If several match, which one?
3. **The weakest Recruit.** Improved Metabolism at Level 2 (+6 Metabolism / −3 Constitution) gives a Recruit:
   - MaxHP 10 (40 − 30);
   - melee Damage 0 (3 − 3), so each hit does the 0.5 minimum;
   - FiringDuration 2 and speed 7.

   Is that intended?

## Context
- Elsewhere a change of maximum keeps the damage: a Warp In's HP follows its maximum (0110), and the planned live tuning works the same way.
- The Armory reserves "the Recruit inside the longest" (Q0105's placeholder).

## Options
1. **(a)** the damage is kept, but HP never drops below 1, so an augmentation never kills; **(b)** HP is kept, capped at the new maximum; **(c)** HP scales with the maximum.
2. **(a)** the matching unit inside the longest; **(b)** the most damaged; **(c)** the least damaged.
3. **(a)** as written, for the numbers pass; **(b)** a floor (e.g. MaxHP at least half the base, and Damage at least 1).

Proposed: 1a, 2a, 3a.
