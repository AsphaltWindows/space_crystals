# 0214 — Orders given while a vehicle is deploying are held; ones the deployed unit can't do are dropped

- **Date**: 2026-11-25T05:20:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Laser Battery > Deploy`; `factions/gdo/units.md > Artillery Rig > Modes` (Orders given while deploying; Not the same as the Colonists Drone's Deploy)

## What changed and why
User: while a Laser Battery or Artillery Rig is deploying, which can't be cancelled (0213):
- the **last order given** (without Shift, replacing earlier ones) is held and is the first thing tried once deployment finishes;
- Shift-queued orders line up behind it;
- at completion each order is checked against the deployed unit, and one it can't carry out (e.g. Move) is dropped, then the next is tried.

Feedback on a dropped order is assumed (the usual unavailable-command feedback, 0162).

Also noted in the docs (user): the Colonists Drone's Deploy is a different model. It becomes a Beacon immediately and can be cancelled (unchanged, 0109/0111).

## Implementation impact
Orders arriving during deploying are stored, not executed or rejected. When deploying completes, they are validated against the deployed type.

## Verification
- Deploy an Artillery Rig and, mid-deploy, right-click an enemy in range. Once deployed it attacks that enemy.
- Mid-deploy, order a Move instead. Once deployed the Move is dropped with feedback, and the unit stays deployed.
