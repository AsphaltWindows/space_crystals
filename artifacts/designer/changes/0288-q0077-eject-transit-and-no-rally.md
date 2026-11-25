# 0288 — Eject during the transit delay is refused; ejected units without a rally stop 1 GU out

- **Date**: 2026-11-26T09:30:00-05:00
- **Answers**: Q0077
- **Docs**: `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > EjectMenu`

## What changed and why
User chose **A** for both points, as built:
1. When every unit of a type is still in the 2 s transit delay, its tile is greyed and the click is refused; nothing is queued.
2. An ejected unit with no rally walks 1 GU straight out of the exit path and idles. Rally points are the player's tool for keeping the exit clear.

## Implementation impact
None; as built.

## Verification
- Eject a Guard right after it entered: its tile is grey for 2 s.
- Eject without a rally: the Guard stops 1 GU out.
