# 0058 — Allied units can't be multi-selected; GDO station intel confirmed intended

- **Date**: 2026-11-23T20:24:52-05:00
- **Answers**: Q0013 (point 2; Q0013 now fully answered by 0048–0058); confirms the side effect noted in 0057 (Q0019)
- **Docs**: `systems/controls/selection_and_hud.md > BoxSelection` (allied objects), `core/map.md > SupplyDeliveryStation > InfoPanel`

## What changed and why
- **Allies:** allied units can't be multi-selected. They're other players' objects, selected alone like enemies. For box-priority tiers they're treated like enemy objects until teams are designed. This matches the current behaviour.
- **GDO station intel:** the user confirmed it's **intended** that a GDO player sees a fogged station's countdown start when someone empties it.

## Implementation impact
None beyond 0057.

## Verification
- (Once teams exist.) Box own and allied units together: only own units are selected.
