# 0159 — All neutral map-placed objects are known from the start

- **Date**: 2026-11-24T20:00:00-05:00
- **Answers**: Q0033 (point 3; Q0033 done)
- **Docs**: `systems/controls/behaviors.md > PathKnowledge` (Map-placed objects)

## What changed and why
Option B. Every neutral object placed by the map maker is known from the start (pathing and memory), like terrain. Players' starting layouts are never pre-known (would leak start position and faction).

## Implementation impact
Extend "known from start" from resources to all neutral map-placed objects. Starting-layout objects stay unknown until seen.

## Verification
A neutral pre-placed obstacle is routed around before ever being seen; an enemy's starting DC isn't.
