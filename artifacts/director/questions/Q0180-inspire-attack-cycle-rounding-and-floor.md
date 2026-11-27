# Q0180 — Inspire's faster attack: whole frames, which phase, and the 2-frame floor

- **Date**: 2026-11-27T04:13:34-05:00
- **Blocking**: no. M5-N11 built the placeholders below. They stay so until the answer, and the playtest notes say so.
- **Docs**:
  - `factions/cults/units/prayer_mobile.md > Placeholder stats` (Inspire: "Recruits also attack **25% faster** (their attack cycle × 0.75)"; "Stacks with augmentation");
  - `factions/cults/augmentation.md` (the Metabolism increment: "−1 frame of FiringDuration per increment ... FiringDuration never below 2 frames"; "**No floors** on the results");
  - design change 0365.

## Question
An inspired Recruit's attack cycle is × 0.75. The game counts frames in whole numbers, and an attack is phases: Aim, Firing, Cooldown and Reload.
1. **Rounding:** how does × 0.75 round to whole frames?
2. **Which phases** get shorter?
3. **The floor:** does the Augmentations' "never below 2 frames" for FiringDuration apply when Inspire shortens it?

## Context
M5-N11 built Inspire. The cases:
- **A plain Recruit:** cycle 12 → 9 (exactly 25%), its Firing 8 → 5.
- **An Improved Metabolism Recruit:** cycle 10 → 7.5, which rounds to 8 (20% faster), its Firing 6 → 4.
- **An Improved Metabolism Level 2 Recruit:** cycle 6 → 4.5, which rounds to 5, its Firing 2 → 1. That is below the Augmentations' 2-frame floor, which Inspire doesn't respect as built.

**As built (placeholders):**
- the cycle × 0.75 rounds to the nearest frame, halves up;
- the frames saved come off FiringDuration only, as the Metabolism increment shortens Firing;
- no floor: Firing can go to 1.

**For your information, on energy** (no change asked): the Prayer-mobile starts with 50 energy and regains 0.5 a second.
- Its first Inspire (75) comes at about 50 s, and its first Suppress (100) at about 100 s.
- Inspire then covers 10 s of every 150 s, and recasting before the 10 s end needs 150 energy banked.

## Options
**1. Rounding:**
- **(a) Nearest, halves up** (as built): sometimes less than 25% (10 → 8).
- **(b) Round down:** always at least 25% (10 → 7).
- **(c) Exact on average:** a fractional cycle carried from one attack to the next, so 10 → 7.5 alternates 7 and 8. This is more code, and the attack timeline is no longer the same every cycle.

**2. Which phases:**
- **(a) Firing only** (as built), like the Metabolism increment.
- **(b) Every phase × 0.75,** each rounded.
- **(c) Cooldown or Reload only.**

**3. The floor:**
- **(a) No floor** (as built): Firing can reach 1 frame.
- **(b) Firing never below 2 frames,** as for the Augmentations: a Level 2 Recruit then gains nothing from Inspire's faster attack.
