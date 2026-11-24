# 0097 — Starting Layout validation

- **Date**: 2026-11-24T04:30:00-05:00
- **Answers**: Q0027 (point 4; Q0027 is now fully answered)
- **Docs**:
  - `core/map_editor.md > Starting Layouts > Layout validation` (new)
  - `factions/gdo/economy.md > Opening Economy Baseline > Reference starting base` (the guideline is softened)

## What changed and why
Each faction's Starting Layout is checked with that faction's **normal placement rules, minus visibility**. For GDO:
- tiles must be Buildable and flat;
- no overlaps, except plates on patches;
- no buildings on resources (0096);
- the EF and plates must lie within the build area of the DC placed in the same layout.

Starting units can't stand on structures or impassable tiles.

**No other checks or warnings** (user: "map makers can do whatever they like"). In particular, the editor doesn't warn about patches outside the starting DC's sight. 0095's "patches within DC sight" is an assumption of the reference base only, not an editor rule.

## Implementation impact
- This is the Director's proposal plus the rule for starting units. No visibility checks or sight warnings in the editor.

## Verification
1. In the editor, place a GDO EF outside its layout's DC build area. It's refused.
2. Place it inside, and it's accepted.
3. Place a starting unit on a cliff tile. It's refused.
4. A DC placed where it can't see any patches is accepted without warnings.
