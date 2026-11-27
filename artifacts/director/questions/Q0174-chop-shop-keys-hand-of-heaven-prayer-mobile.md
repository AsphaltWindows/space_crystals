# Q0174 — The Chop Shop's keys for the Hand of Heaven and the Prayer-mobile

- **Date**: 2026-11-26T23:06:28-05:00
- **Blocking**: no. Slices M5-N10 (the Hand of Heaven) and M5-N11 (the Prayer-mobile) proceed on this assumption:
  - in the kind menu after Q: Build Seeker, **W: Build Hand of Heaven** and **E: Build Prayer-mobile**;
  - E is greyed with "Requires a Temple" until a Temple stands, as for other unit prerequisites.
- **Docs**:
  - `factions/cults/structures/chop_shop.md > Production`, `> Placeholder stats`;
  - `factions/cults/units/hand_of_heaven.md`;
  - `factions/cults/units/prayer_mobile.md`;
  - `factions/cults/structures/armory.md` (kinds, then Q: Train Militant, W: Train Gunner).

## Question
1. Which keys build the Hand of Heaven and the Prayer-mobile in the Chop Shop's kind menu?
2. The Chop Shop's Production list names only the Seeker and the Hand of Heaven. Does it also produce the Prayer-mobile, as `prayer_mobile.md` says ("Produced at - Chop Shop", needing a Temple)?

## Context
- The Chop Shop's panel reads "**Q: Build Seeker**, **A: Release** in the kind menu", and names no key for its other vehicles.
- The Hand of Heaven's doc says it is "built at the Chop Shop" (3 Recruits, 150 SC, 30 s).
- The Prayer-mobile's doc says "Produced at - Chop Shop (consumes **2 Recruits**...)", with a Temple prerequisite (125 SC, 25 s).
- The Armory's kind menu is Q: Train Militant, W: Train Gunner, then A: Release. The assumption follows that pattern, in the order the vehicles were designed.
- **M5-N6**, building the Chop Shop and the Seeker now, builds Q only. N10 and N11 add theirs.

## Options
1. **W: Build Hand of Heaven, E: Build Prayer-mobile** (the assumption).
2. **Other keys**, for example matching the vehicles' order in the Production list, or keys freed for something else.
3. **The Prayer-mobile isn't the Chop Shop's,** and `prayer_mobile.md` is corrected: say where it is built.
