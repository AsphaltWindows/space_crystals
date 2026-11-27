# 0431 — The Phantom descends and rises at the air doors over 16 frames, open to anti-air

- **Date**: 2026-11-26T19:25:00-05:00
- **Answers**: Q0165
- **Docs**: `factions/syndicate/units/phantom.md > Placeholder stats`

## What changed and why
Answered from the docs ("It **rises vertically** out of a Tier 3 Tunnel's air door") with a designer placeholder timing (standing permission): option (b). Descending into the entry door and rising from the exit door each take **16 frames (1 s)**, like a Hornet at the Airfield; it is an **air target** meanwhile, as the Specter (24 frames) and every aircraft landing or taking off (0428).

## Implementation impact
Replaces the placeholder (a) "at once". The door rate (one per 8 frames) is unchanged, so two Phantoms can be rising at the same time.

## Verification
Eject a Phantom: it rises from the exit door over 1 s and an anti-air weapon can hit it while it rises. Send one home: it descends into the entry door over 1 s, hittable until it's in.
