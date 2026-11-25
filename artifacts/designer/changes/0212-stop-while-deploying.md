# 0212 — Stop while deploying winds the deploy back

- **Date**: 2026-11-25T05:05:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Laser Battery > Deploy`; `factions/gdo/units.md > Artillery Rig > Modes`

## What changed and why
User: pressing **Stop** while deploying stops the deploy and **undoes the part already done**. The unit winds back from its current point (halfway → half the deploy time) and ends mobile. It can neither move nor attack until it has finished winding back.

Assumed symmetric (my reading): Stop while undeploying winds back to deployed. Whether other orders without Shift (Move etc.) also interrupt a deploy is being settled with the user.

## Implementation impact
- Deploy progress must be reversible.
- Stop triggers the reversal.
- The unit stays the mobile type throughout (0211).

## Verification
Order Deploy, then press Stop halfway. The unit packs back up over about half the deploy time, stays mobile, and can then move.
