# 0075 — Shielded Peacekeepers are left out of explicit Attack

- **Date**: 2026-11-24T01:10:00-05:00
- **Answers**: Q0021 (point 2; point 3 still open)
- **Docs**:
  - `factions/gdo/units.md > Peacekeeper > Riot Shield` (new "Explicit Attack" bullet)
  - `systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting` (wording fix only)

## What changed and why
The user picked B. For **A + click on an enemy** with a selection that includes Peacekeepers with shields up:
- The shielded Peacekeepers are **left out** of the order and keep their current orders.
- The units that can attack carry it out.
- This applies per unit: in a Peacekeeper group with mixed shield states, only the lowered ones attack.

Reasoning: an explicit panel command only reaches units that can carry it out (`selection_and_hud.md > CommandPanel`). Two alternatives were rejected:
- **Move to the clicked spot**, the current behaviour: it targets a stale location, and Attack would act as a hidden Move.
- **Follow**: it would make shielded Peacekeepers a special case of the panel rule.

This deliberately **differs from right-click** (0074), where shielded Peacekeepers follow.

Wording fix: the No-smart-casting text now says a panel command goes to everything it is issued to, i.e. the whole Selection for a common command or the ActiveGroup for a group command. Before, it said "the ActiveGroup" only. This isn't a rule change.

## Implementation impact
- This replaces the current behaviour, where shielded units move to the clicked spot on A + click.
- If every unit that would receive the order has its shield up, nothing is issued. With only shielded Peacekeepers selected, Attack is unavailable, as the Riot Shield doc already says.
- **TBD:** A + click on **ground** (AttackMove) with shielded Peacekeepers: left out too, or move along? Keep the current behaviour until it's decided.

## Verification
1. Select 4 shielded Peacekeepers and 4 Rocketmen, which are moving somewhere. Press A and click an enemy. The Rocketmen attack it. The Peacekeepers carry on with their previous move and don't change course.
2. Select 4 Peacekeepers, 2 shielded and 2 not. Press A and click an enemy. Only the 2 unshielded ones attack.
3. Select only shielded Peacekeepers. Attack is unavailable.
