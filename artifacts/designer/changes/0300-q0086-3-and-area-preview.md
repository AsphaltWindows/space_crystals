# 0300 — EjectMenu stays open; area preview marks own overlaps; area shown while placing expansions

- **Date**: 2026-11-26T12:10:00-05:00
- **Answers**: Q0086 point 3 (Q0086 complete: 0285, 0299, 0300)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > EjectMenu`, `> Tunnel Area display`

## What changed and why
User:
1. After an eject click the panel **stays in the EjectMenu**: option (a), as built.
2. The Tier 2/3 placement preview marks red:
   - non-Drillable cells;
   - off-map cells (confirmed);
   - cells overlapping the player's **own** other Tunnels' areas.

   Enemy areas stay hidden.
3. The Tunnel's area is shown while placing an expansion, as built.

## Implementation impact
Add own-area overlap to the preview's red cells.

## Verification
Place a Tunnel next to your own: the future T2/T3 rings show red where they would overlap it.
