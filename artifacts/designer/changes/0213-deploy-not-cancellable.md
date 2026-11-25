# 0213 — Deploying can't be cancelled (replaces 0212)

- **Date**: 2026-11-25T05:10:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Laser Battery > Deploy`; `factions/gdo/units.md > Artillery Rig > Modes`

## What changed and why
User changed their mind: once the deployment process has started it **can't be cancelled**. It always runs to completion, and Stop doesn't interrupt it. **Replaces 0212** (wind-back on Stop). Assumed symmetric (my reading): undeploying also always completes. What happens to orders given during the process is being settled with the user.

## Implementation impact
No reversible deploy progress is needed.

## Verification
Order Deploy, then press Stop halfway. The unit still finishes deploying.
