# 0095 — Extraction Plates need visible tiles

- **Date**: 2026-11-24T04:15:00-05:00
- **Answers**: Q0027 (point 2; points 3–4 still open)
- **Docs**:
  - `core/entities.md > Placement Validation > Direct Placement` (plates aren't exempt)
  - `factions/gdo/economy.md > Opening Economy Baseline > Reference starting base` (map-maker guideline)

## What changed and why
The user chose A. Extraction Plates follow the Direct Placement visibility rule: they can only go on patches the player currently sees.

The user's view: the DC's sight of 6 is enough to see a long row of patches, as long as the starting DC is placed favourably. So the reference starting base now also assumes its patches are **within the starting DC's sight**. Map makers place GDO Starting Layouts with that in mind.

## Implementation impact
- Matches the Director's proposal. No exemption for plates.
- Test map: place each GDO Starting Layout's DC so its sight (6 grid units, plus the elevation modifier if any) covers the main patches.

## Verification
1. On the test map, at frame 0, every main-base patch is Visible, and the EF can plate each one without scouting.
2. A patch inside the build area but out of sight shows a red ghost.
