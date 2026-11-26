# Q0138 — Altar: two lines still say fully augmented units leave only by Eject All

- **Date**: 2026-11-26T04:40:17-05:00
- **Blocking**: no. Slice N1b will build 0337: Release works in every state menu, the fully augmented one included.
- **Docs**: `factions/cults/augmentation.md > Altar > Entry`, `> Eject All`, `> ObjectInterfaceState[Altar] > AugmentMenu[UnitType]` and `> AugmentMenu[UnitType, State]`

## Question
Can fully augmented units leave the Altar by Release, as 0337 says? If so, the two older lines need updating.

## Context
- **0337** says: "Release is added in every state menu. The 'Fully augmented' tile is now enabled, with Release as its only option."
- **The menus agree with 0337:**
  - `AugmentMenu[UnitType]`: "**R: Fully augmented**, enabled when there are any (only Release is possible)";
  - `AugmentMenu[UnitType, State]`: "In every state: **A: Release**".
- **Two older lines still say the opposite:**
  - `> Entry`: "they can only leave, via Eject All";
  - `> Eject All`: "This is the only way for fully augmented units to leave."

## Options
- **(a) Release works for fully augmented units, as 0337 says.** The two lines get updated. This is what N1b will build.
- **(b) Eject All is the only way out.** The R tile and its Release are removed.
