# 0265 — Transport passengers always count against Tunnel Space

- **Date**: 2026-11-25T14:45:00-05:00
- **Answers**: Q0071 (gap: Tunnel Space aboard)
- **Docs**: `factions/syndicate/units.md > Subterranean Transport > Entering the Tunnel Network loaded`

## What changed and why
User chose **A**: infantry aboard a Subterranean Transport keep counting against the Tunnel Space cap, aboard or not, in the network or on the surface. Tunnel Space is an army limit. This corrects 0222's "only the transport's own Tunnel space counts", which conflicted with the cap rule. Passengers still can't be ejected separately.

## Implementation impact
No special case: units count from production for life.

## Verification
—
