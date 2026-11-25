# 0228 — Phantom turret is a 60–90° total cone; stale fog-chase wording fixed

- **Date**: 2026-11-25T07:40:00-05:00
- **Answers**: — (the doc fix was flagged in Q0058's context)
- **Docs**: `factions/syndicate/units.md > Phantom > TurretAttributes`; `systems/controls/selection_and_hud.md > rally point on an enemy object`

## What changed and why
- User confirmed: the Phantom's 60–90° is the **total** cone (30–45° each side of its facing).
- Clarification, no rule change: the rally-point line no longer says it "matches chasing a target into the fog". Since 0200 an attacking unit that loses its target to the fog stops. The rally rule (a unit produced after the enemy was lost moves to the last-seen position) is unchanged.

## Implementation impact
None beyond the wording.

## Verification
—
