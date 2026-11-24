# 0050 — Correction: empty-click rules are NOT decided yet

- **Date**: 2026-11-23T19:57:42-05:00
- **Answers**: —
- **Docs**: `systems/controls/selection_and_hud.md > BoxSelection > Empty clicks and boxes`

## What changed and why
**Corrects 0049.** Only **Shift + box adds** was decided by the user. The empty-ground rules (plain empty click or box clears; Shift/Ctrl-click on empty ground does nothing; Ctrl wins over Shift) were **not** confirmed: I wrongly took silence as agreement. They're now marked TBD, with your current behaviour as the placeholder.

## Implementation impact
Keep the current behaviour. Decisions will follow in separate entries.

## Verification
—
