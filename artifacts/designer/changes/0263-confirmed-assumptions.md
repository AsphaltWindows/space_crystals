# 0263 — Confirmed: undeploy, held orders, Transport base, Phantom attack

- **Date**: 2026-11-25T14:20:00-05:00
- **Answers**: Q0071 (the "to confirm" items in 5, 6 and 11)
- **Docs**: `factions/syndicate/units.md > Laser Battery > Deploy`, `> Subterranean Transport`, `> Phantom`; `factions/gdo/units.md > Artillery Rig > Modes`

## What changed and why
User confirmed my assumptions:
1. Undeploying always completes, and Deploy is available again only after it.
2. Held orders the deployed unit can't carry out are dropped with unavailable-command feedback.
3. The Subterranean Transport is a DrillUnit with no attack.
4. The Phantom is Groupable, with a FullyConnected (can't-miss) SingleTarget laser.

## Implementation impact
None beyond removing "assumed".

## Verification
—
