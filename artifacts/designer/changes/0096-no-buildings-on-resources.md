# 0096 — No buildings on resources

- **Date**: 2026-11-24T04:20:00-05:00
- **Answers**: Q0027 (point 3; point 4 still open)
- **Docs**: `core/entities.md > Placement Validation > All placement methods: no buildings on resources` (new)

## What changed and why
The user chose A. No structure may cover any tile of a Space Crystals Patch or a Supply Delivery Station, whatever the placement method: Direct, Worker-Built or Warp In. The only exception is a GDO Extraction Plate on a patch. This stops a player from making a resource unusable for everyone by building on it.

## Implementation impact
- Matches the Director's proposal. Resource tiles are invalid on every placement ghost, except for plates.

## Verification
1. Try to place a Generator over a patch. Those tiles show red and placement is refused.
2. Try the same over a Supply Delivery Station. It's refused.
3. A plate on a patch is still allowed.
