# 0378 — Right-click on a full, damaged own building: Repair

- **Date**: 2026-11-27T10:00:00-05:00
- **Answers**: Q0140
- **Docs**: `factions/cults/units/recruit.md > Right-Click Resolution`; `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All > Entering pool buildings`

## What changed and why
User: **(b)**. A Recruit right-click on a damaged own building that the selection could enter, but that is **full at the moment of the click**, resolves to **Repair**, and the pointer shows Repair. If it isn't full, Enter still wins (0325). This applies to every building that holds units. An Enter certain to fail is a wasted click.

## Implementation impact
- Changes M5-N2's type-based resolution: fullness, counting reserved units, is checked at click time.
- If the building fills up later while the Recruits are walking over, the normal Full-building rule applies.

## Verification
- Fill a Garrison (4 inside) and damage it. Right-click it with Recruits: the Repair pointer shows, and they repair it.
- With a free slot, the same right-click enters.
