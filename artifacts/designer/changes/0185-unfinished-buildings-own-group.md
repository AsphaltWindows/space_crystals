# 0185 — Unfinished buildings form their own SelectionGroup

- **Date**: 2026-11-25T00:05:00-05:00
- **Answers**: Q0048 (point 2; Q0048 done)
- **Docs**: `systems/controls/selection_and_hud.md > SelectionGroup` (Unfinished buildings form their own group)

## What changed and why
Option C, generalised: any building under construction or transformation forms a separate SelectionGroup from finished buildings of its type (own panel, e.g. Cancel only); on completion it joins the finished group. Same idea as Peacekeeper / Peacekeeper (Shielded) (0173).

## Implementation impact
Group key = (type, finished?). Applies to Colonists Warp In, Cults construction, and any other unfinished structures.

## Verification
Select 2 finished Bio Portals and 1 warping in: two groups; the finished group trains normally; Tab to the other shows only Cancel.
