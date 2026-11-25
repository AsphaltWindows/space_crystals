# 0245 — Tunnel Network: a small flat delay before an entered unit can exit

- **Date**: 2026-11-25T10:45:00-05:00
- **Answers**: Q0063 point 1 (other points pending)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Network` (Transit delay)

## What changed and why
User: after a unit enters the network, there is a small delay before it can exit. After that it can exit from any Tunnel of a sufficient tier. The delay is flat, not by distance: **2 s (32 frames), placeholder** (my number for "small").

My reading, to be confirmed: units produced by expansions are not "entering", so they have no delay.

## Implementation impact
- Each unit inside stores when it entered.
- Ejecting it (or exiting a transport carrying it, if that applies later) is possible only 32 frames after entry.

## Verification
A Guard enters a Tunnel. For 2 s it can't be ejected anywhere; afterwards it can be ejected from any Tunnel.
