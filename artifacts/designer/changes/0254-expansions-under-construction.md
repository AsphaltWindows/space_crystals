# 0254 — Expansions under construction

- **Date**: 2026-11-25T12:20:00-05:00
- **Answers**: Q0065
- **Docs**: `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > Expansions under construction` (new)

## What changed and why
User:
- **Pt 1:** paid when the ghost is placed.
- **Pt 4:** Tunnel Space is used once construction starts. Today ghost placement starts construction at once, so both happen at the click.
  - I read the user's "7" as point 4, about Tunnel Space; point 7 stays as proposed.
  - If expansion queuing is ever added: pay at the ghost, Tunnel Space at the start.
- **Pt 3:** cancelling refunds **75%** (rounded down), not the full cost. The Tunnel's X cancels its current operation.
- **Pts 2, 5, 6, 7, 8** as proposed:
  - ConstructionHP applies to all expansions, with damage staying;
  - no sight;
  - Groupable;
  - destroyed with no refund if the Tunnel falls mid-build;
  - Eject still works meanwhile.

## Implementation impact
As proposed, except for the 75% refund on cancel.

## Verification
Place a Hideout (200 SC), then cancel: 150 SC come back.
