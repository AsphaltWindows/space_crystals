# 0391 — Cults augmentations are much faster: 3 s each, 6 s for Both

- **Date**: 2026-11-27T12:30:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/augmentation.md > Altar > Augmentation Orders`

## What changed and why
User: "make Cults augmentations much faster to complete". The numbers are the designer's placeholders under the standing permission.

| Order | Before | Now |
|---|---|---|
| Enhanced Constitution | 160 frames (10 s) | **48 frames (3 s)** |
| Improved Metabolism | 160 frames (10 s) | **48 frames (3 s)** |
| Both | 320 frames (20 s) | **96 frames (6 s)** |

Costs are unchanged: 25 Space Crystals each, 50 for Both.

## Implementation impact
New order durations; nothing else changes.

## Verification
- Queue Enhanced Constitution on a Recruit in the Altar: it exits augmented about 3 s later.
