# 0302 — Cults Soldier renamed Militant

- **Date**: 2026-11-26T12:40:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/soldier.md` is now **`factions/cults/units/militant.md`**;
  - every mention is updated in `factions/cults/structures/{armory,garrison,README}.md`, `factions/cults/augmentation.md`, `factions/cults/units/{README,recruit}.md`, `factions/gdo/structures/bunker.md`, `systems/controls/{commands,selection_and_hud}.md` and `README.md`.

## What changed and why
User renamed the Cults' basic gun infantry from **Soldier** to **Militant**. The unit is otherwise unchanged.

## Implementation impact
Rename the unit type, its hotkey label (Armory Q: Train Militant) and display strings. Update code citations of `units/soldier.md`.

## Verification
The Armory's Q reads "Train Militant".
