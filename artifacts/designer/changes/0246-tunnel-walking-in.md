# 0246 — Units walk into a Tunnel single file at normal speed; produced units exit without delay

- **Date**: 2026-11-25T10:55:00-05:00
- **Answers**: Q0063 point 7 (and part of point 2)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Network` (Transit delay, Walking in), `> Tunnel > Side A`

## What changed and why
User:
- **Produced units** have no transit delay; they can exit as soon as they are produced (confirms 0245's reading).
- **Entering**: units walk into the entry path at their normal speed, one behind another with normal spacing, like any one-unit-wide passage. **There is no fixed hold per unit**, so there are no large traffic jams. This replaces the Director's 8-frame hold proposal (pt 7) and clarifies 0225's "one unit at a time" as "one unit wide".

## Implementation impact
- The entry path is a single-file passage; units are removed from the map once they have walked into the Tunnel.
- Throughput follows unit speed and spacing.
- Exits are covered with Q0064.

## Verification
20 Guards ordered to Enter file in continuously, without pausing between units, and the whole line is in within a few seconds.
