# 0210 — Laser Battery needs an Optics Lab; Laser Battery and Artillery Rig deploy as two unit types under the toggle rule

- **Date**: 2026-11-25T04:45:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Laser Battery` (TechPrerequisite, Deploy); `factions/syndicate/structures.md > Vehicle Assembly`, `> Optics Lab`; `factions/gdo/units.md > Artillery Rig > Modes`

## What changed and why
User:
- The Laser Battery requires an **Optics Lab**.
- **Switching modes** works like the Artillery Rig: it takes time, with no moving or attacking meanwhile.
- **Both the Laser Battery and the Artillery Rig** have mobile and deployed as **two unit types**, under the mode toggle rule from 0198:
  - the press goes to the ActiveGroup;
  - type and group switch at the press;
  - every pressed unit ends in the same mode.
- The Laser Battery's battery keeps charging in every mode.

## Implementation impact
- The Artillery Rig (when built) gets two types and the 0198 toggle behaviour.
- The Laser Battery is concept only. Deploy times are TBD for both.

## Verification
Select 3 mobile Artillery Rigs and press Deploy. All three show at once as one Artillery Rig (Deployed) group. Press again quickly: all three end up mobile.
