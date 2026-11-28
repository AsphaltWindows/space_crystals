# 0488 — The automatic return drops the aircraft's queued orders

- **Date**: 2026-11-28T15:30:00-05:00
- **Answers**: Q0195
- **Docs**: `systems/mechanics/aircraft_landing.md > Rearming archetypes > Assigned base` (Automatic return)

## What changed and why
Option (a), as built (user). When an aircraft runs out of ammo and returns by itself, the automatic return **replaces all its orders**. Orders queued behind the one it ran out in are dropped, as with the Specter's Network base. It rearms fully and stays landed until the player gives new orders.

0466's rule is unchanged: orders the player Shift-queues **behind a Return to Base** still run as soon as the aircraft lands.

## Implementation impact
None; this matches the build.

## Verification
1. Give a Hornet an Attack Move, then Shift+Move to B. When it runs dry it flies home, rearms fully and stays landed.
2. It never flies to B.
