# 0211 — Deploy is a queueable order, not a mode toggle; no Undeploy until deployed

- **Date**: 2026-11-25T04:55:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Laser Battery > Deploy`; `factions/gdo/units.md > Artillery Rig > Modes`

## What changed and why
User: deploying is different from mode switches (Convert, Riot Shield).
- **Queueable**: Shift-Deploy runs after the earlier orders (move here, then deploy). Without Shift it replaces the current orders.
- **No mid-way reversal**: Undeploy becomes available only once deployment is complete. Undeploying is assumed symmetric (my reading): Deploy is available again only once undeploying is complete.
- **Type/group switch at completion**, not at the press. The group the units join becomes active when their old group disappears (0199).

**Replaces 0210's "Deploy/Undeploy follows the mode toggle rule"** for both the Laser Battery and the Artillery Rig. Two unit types stay.

## Implementation impact
- Deploy/Undeploy are orders in the command queue.
- There is no reverse during the switch.
- The type changes when the switch finishes.
- Interrupting a deploy in progress with other orders (Stop, Move) is being settled with the user.

## Verification
- Shift-Move then Shift-Deploy: the Artillery Rig drives there, then deploys.
- During deploying the panel shows no Undeploy; when deploying ends, the unit joins the Deployed group.
