# 0449 — Tracked turning at rest follows the tread law (stale "unconstrained" removed)

- **Date**: 2026-11-27T12:10:00-05:00
- **Answers**: Q0176
- **Docs**: `systems/combat.md > LocomotionOrientationConstraints[SpeedTurnRadiusMovement]` (Stationary and Stopping rows)

## What changed and why
Option 1, as built. The tread law from 0417 (`systems/units.md > SpeedTurnRadiusMovement > The turning law`) applies at every speed. At rest a tracked unit spins at SpinRate. While stopping it turns at the rate for its current speed. The "unconstrained" entries in combat.md's table were left over from before 0417 and have been corrected. The Hand of Heaven's "about 19° a frame when stopped" stands.

## Implementation impact
None: M5-V3 already behaves this way.

## Verification
As in 0417. A stopped Hand of Heaven turns about 19° per frame, never instantly. A braking one turns faster as it slows down.
