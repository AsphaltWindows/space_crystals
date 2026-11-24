# 0063 — Right-click resolves per unit; Mixed cursor when actions differ

- **Date**: 2026-11-23T20:35:28-05:00
- **Answers**: Q0016 (point 1 follow-up)
- **Docs**: `systems/controls/selection_and_hud.md > RightClickResolution`, `> PointerDisplayType` (new Mixed type, Mixed Right-Click Preview, DefaultState rules)

## What changed and why
**Replaces** "resolution is evaluated against the ActiveGroup's capabilities". **Each selected unit resolves a right-click for itself**, so the outcome doesn't depend on which group is active (e.g. Peacekeepers attack and a Supply Chopper follows, even when the Chopper is the ActiveGroup). The pointer previews the per-unit result: the normal pointer if all units would do the same thing, and a new **Mixed** pointer (generic hybrid; visual TBD) if they'd do different things.

## Implementation impact
- Right-click: per-unit resolution, combined with 0062 (non-capable units move or follow).
- Pointer: compute the set of per-unit actions; one distinct action → that pointer; more than one → Mixed.

## Verification
- Peacekeepers + Supply Chopper, Chopper active (via Tab), right-click an enemy: the Peacekeepers attack, and the Chopper follows. Hovering that enemy shows the Mixed cursor.
- Peacekeepers only, hover an enemy: Attack cursor.
