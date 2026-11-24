# 0123 — A Gather ends when the Drone runs out of patches; queued orders wait for that

- **Date**: 2026-11-24T14:50:00-05:00
- **Answers**: Q0040 (point 7); Q0037 (point 12; point 5 partly: radius around the Drone, per-unit stat)
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)` (Search radius), `> When a Gather ends` (new)

## What changed and why
User's own rule (neither offered option). A Gather ends on its own only when the Drone runs out of crystals to mine: its patch is gone and no patch is left within its search radius. It delivers any load first, then idles or starts the next queued order. Orders Shift-queued behind a Gather (including Deploy) wait for that: "mine this field out, then …". A Drone waiting at an occupied patch is still gathering.

The redirect search ("nearest unoccupied patch") is limited to a **radius around the Drone itself**, an internal per-unit Drone stat (name TBD), **6 grid units placeholder**.

## Implementation impact
- Replaces the Director's proposed "queued order starts after the next drop-off" (Q0037 pt 12 / Q0040 pt 7).
- The radius is centred on the Drone (not on the arrival patch as proposed in Q0037 pt 5). Which patches qualify within it (known only? not under a structure?) is still open with Q0037 pt 5/11.
- Designer's reading, flag if wrong: "no patch left" = no patch at all within the radius; if only occupied patches remain, the Drone keeps gathering (waits at one).

## Verification
- Shift-queue a Move behind a Gather: the Drone keeps mining until its field is exhausted, delivers its last load, then moves.
- A Drone waiting at an occupied patch with a queued order keeps waiting.
- Deplete a field with a patch 8 GU away: Drones within 6 GU of no patch deliver and go idle.
