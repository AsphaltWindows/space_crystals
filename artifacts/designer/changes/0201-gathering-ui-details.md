# 0201 — Gathering UI: Drone right-click on enemies, Set Drop-off pointer, load visibility, gathering markers

- **Date**: 2026-11-25T03:00:00-05:00
- **Answers**: Q0052
- **Docs**: `factions/colonists/units.md > Drone > Right-Click Resolution` (existing), `> Who sees the load` (new); `systems/controls/selection_and_hud.md > PointerDisplayType` (list), `> CommandIndicators > Indicator assignments`, `> Gathering markers`

## What changed and why
1. **A Drone right-clicking an enemy**: the docs already say (Right-Click Resolution) that it **follows an enemy unit** and moves to an enemy structure. This is the current M3-J2 behaviour; no change.
2. **Pointer while targeting Set Drop-off**: **ReturnResources** over a valid drop-off (user; as built).
3. **Load visibility** (user): a visual of what the Drone carries is seen by anyone who sees the Drone. The amount, the drop-off and the source are **owner-only**. This matches the current behaviour, and states the source and destination explicitly.
4. **Markers** (user): Gather (on the source: patch or refinery output), Drop Off (Return Resources) and Set Drop-off each get their **own distinct marker** on the target object, replacing the Move marker. Looks are TBD (an art choice).

## Implementation impact
- Only point 4 changes M3-J2: it needs three new indicator types.
- Points 1–3 confirm what is built.

## Verification
- Gather: a distinct marker appears on the patch, not the green Move marker.
- Drop Off: its own marker appears on the Node.
- Set Drop-off: a third distinct marker appears, and the ReturnResources pointer shows while choosing the drop-off.
- An enemy sees the load on the Drone, but not the amount, the destination or the source.
