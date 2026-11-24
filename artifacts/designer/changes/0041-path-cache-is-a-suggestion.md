# 0041 — Clarification: the per-player obstacle cache is an implementation suggestion, not design

- **Date**: 2026-11-23T19:34:11-05:00
- **Answers**: —
- **Docs**: `systems/controls/behaviors.md > PathKnowledge`

## What changed and why
The user clarified that the obstacle cache and "re-path when stuck after a cache change" in 0040 were an **implementation idea, not part of the design**. The doc now marks it as a suggestion. The design requirement is only: paths are planned from the owner's knowledge (terrain and own structures always known; others' structures known once seen; seen-but-fogged structures are walls at their last-known position; never-seen ones are open), and units re-route when they discover an obstacle.

## Implementation impact
How to implement this (cache or otherwise), and when to re-path, is the Director's call. The suggestion is there if useful.

## Verification
Same as 0040.
