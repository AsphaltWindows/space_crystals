# 0009 — Starting Layouts: placed from scratch, Skirmish Default restriction (can be turned off)

- **Date**: 2026-11-23T13:28:34-05:00
- **Answers**: Q0004 (further part; still open: completeness, toggle scope, sync on Starting Conditions changes, slot count/teams, pre-placed objects, metadata)
- **Docs**: `core/map_editor.md > Starting Layouts > Placing from scratch`, `> Skirmish Default restriction`

## What changed and why
- Starting Layouts start **empty**. The map maker places each starting object by hand, and the editor does not pre-fill a default layout.
- Under the **Skirmish Default** restriction, which is on by default, a faction's layout can only contain the objects in that faction's standard Starting Conditions (e.g. GDO: Deployment Center, Extraction Facility, 2 Extraction Plates). The map maker can **turn the restriction off** to place any objects (custom or scenario starts). "Skirmish Default" is the user's name for it.

## Implementation impact
- Extends 0008. The editor's palette for a Starting Layout under Skirmish Default offers only that faction's starting object types, up to their counts. With the restriction off, the full object palette is available.
- Each layout stores whether Skirmish Default is on. The scope of the toggle (per map / start position / faction layout) is TBD, so per faction layout is the most flexible assumption until decided.
- TBD: whether a Skirmish Default layout must be complete for the map to be usable; behavior when Starting Conditions change later.

## Verification
- New start position: all four faction layouts are empty.
- With Skirmish Default on for GDO, the editor only offers a Deployment Center, an Extraction Facility and two Extraction Plates. Trying to place a Barracks or a third plate is refused.
- Turn Skirmish Default off: a Barracks can be placed in the layout and appears at game start for a GDO player in that slot.
