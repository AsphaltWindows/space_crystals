# 0291 — ExpandMenu: all nine expansions hold fixed slots; the turret is always 10th

- **Date**: 2026-11-26T09:55:00-05:00
- **Answers**: Q0081
- **Docs**: `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > ExpandMenu`

## What changed and why
User chose **A**: every expansion is listed in table order. Unavailable ones are greyed, including ones not in the build yet. The Laser Turret is always the tenth slot, so hotkeys are stable.

## Implementation impact
As proposed.

## Verification
The ExpandMenu shows nine expansion slots and then the turret.
