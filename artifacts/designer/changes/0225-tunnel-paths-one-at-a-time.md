# 0225 — Tunnel Side A paths take one unit at a time

- **Date**: 2026-11-25T07:10:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/tunnels.md > Tunnel` (Side A)

## What changed and why
User: each of Side A's two paths (enter, exit) takes **one unit at a time** and fixes where units go in and come out. Entering and exiting units don't block each other; armies pass single file.

## Implementation impact
- Tunnel entry and exit are serialised per path: one unit in each path at a time, at the path's fixed position.
- Throughput depends on how long a unit occupies a path (TBD; e.g. until it has fully cleared it).
- Whether the air doors are also one at a time is TBD.

## Verification
- Order 10 Guards into a Tunnel: they enter one after another through the entry path.
- Eject units at the same time: they come out of the exit path, unhindered by those entering.
