# 0104 — Deployment Center: two build menus

- **Date**: 2026-11-24T05:40:00-05:00
- **Answers**: Q0030 (point 2; point 3 still open)
- **Docs**: `factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]` (DefaultState keys; the "Two build menus" layouts; idle menu entries)

## What changed and why
The DC builds 13 structures, but a panel has only 10 free slots. The user decided to split the menu:
- DC DefaultState: **Q: Basic Construction** and **W: Advanced Construction**. The user is also fine with "Basic/Advanced Structures" as names.
- **BuildMenu[Basic]**:
  - top row Q Generator, W Barracks, E Extraction Facility, R Supply Tower;
  - middle row A Radar Station, S Arsenal, D Career Campus, F Defense System;
  - bottom row Z Back, X Cancel.
- **BuildMenu[Advanced]**, the Factory and everything after it:
  - Q Factory, W Advanced Mechanics, E Airfield, R Propulsion Labs, A Industrial Powerplant;
  - Z Back, X Cancel.

  The user moved the Factory here from the Designer's first draft.
- Unmet prerequisites show greyed out in their slot.
- When a building is ready to place, its own slot becomes "Select completed building". This is the Director's proposal.

The layout is "fine for now" (user).

## Implementation impact
- This replaces the single "Q: Build" BuildMenu.
- M2 needs only Q Generator, W Barracks and E Extraction Facility in BuildMenu[Basic]. The other slots can show greyed placeholders, or be hidden until those buildings exist.
- The idle, constructing and ready menu states work the same in both menus.

## Verification
1. Select the DC. Q opens Basic Construction, and Q again starts a Generator.
2. Z returns to the DC panel.
3. W opens Advanced Construction, where the Factory is greyed out without a Barracks.
4. When the Generator is ready, the Q slot of Basic Construction becomes "Select completed building".
