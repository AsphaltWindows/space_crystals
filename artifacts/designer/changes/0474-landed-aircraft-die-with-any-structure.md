# 0474 — Aircraft landed on any structure are destroyed with it (confirmed)

- **Date**: 2026-11-28T11:45:00-05:00
- **Answers**: —
- **Docs**: `systems/mechanics/aircraft_landing.md > Landed aircraft`

## What changed and why
The user confirmed the rule flagged in 0473. Aircraft landed on **any** structure when it is destroyed are destroyed with it: an Airfield, an Airstrip (the Pilgrim, and therefore its passengers), or a Tunnel's air doors. One still landing counts as landed; one already taking off escapes. It is no longer a placeholder.

## Implementation impact
Apply the rule to every structure aircraft land on, not only the Airfield.

## Verification
1. Destroy an Airstrip with a loaded Pilgrim on it. The Pilgrim and all its passengers die.
2. A Pilgrim that has begun its take-off run escapes.
