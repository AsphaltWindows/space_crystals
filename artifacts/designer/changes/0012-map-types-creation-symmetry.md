# 0012 — Map creation, map types (Skirmish Default / Custom), generated symmetry, Starting Layout rules

- **Date**: 2026-11-23T13:39:17-05:00
- **Answers**: Q0004 (completes it, together with 0008 and 0009)
- **Docs**: `core/map_editor.md` (rewritten: Map Creation, Map Types, Starting Layouts > Overlay editing / Skirmish Default restriction / Outdated layouts, Symmetry > Editing a symmetric map, Custom Tile Presets), `core/map.md > Tile` and `> TilePreset` (custom textures deferred), `README.md`

## What changed and why
**Skirmish Default is now a map type** chosen at creation. Before (0009), it was a per-layout restriction that could be toggled. The other type is **Custom**, which is deferred: it will later get slot restrictions, triggers and so on.

**Map creation settings:** map type, size (min/max TBD), number of player slots (max TBD), symmetry (required for Skirmish Default, optional for Custom), name and description.

**Skirmish Default maps:**
- **Symmetry generated, not drawn.** Only the necessary portion (a half, or a quarter for 90° and both-axes mirrors) is editable, and the rest is generated automatically. **Everything is mirrored:** tiles, corner heights, patches + amounts, stations + settings, start positions + all faction layouts, and neutral objects. Building orientations are transformed to match.
- **No slot restrictions.** The map can't limit factions or fix teams.
- **Starting Layouts:** only the faction's Starting Conditions objects are allowed, and every layout must be **complete** for the map to be playable. The editor shows what's missing.
- **Pre-placed objects:** neutral allowed; player-owned not allowed outside Starting Layouts.

**All maps:**
- Layouts of different factions may overlap, but each must be valid on its own.
- If Starting Conditions change, affected layouts are **flagged as outdated**, with no automatic fixing.
- Custom tile presets: any game-property combination; existing textures only for now (custom textures later).

## Implementation impact
- **Supersedes** 0009's "the restriction can be turned off per layout". Turning it off now means making a Custom map, and the map type is fixed at creation.
- Map file: type, size, slot count, symmetry mode, metadata. A symmetric map can store only the editable portion, or everything; that's your choice, but the generated part must always equal the transform of the editable part.
- Editor: creation dialog; the non-editable region is locked and regenerated live; layout completeness check and missing-object list; outdated-layout flag (needs a way to compare layouts with current Starting Conditions, e.g. a Starting Conditions version or content check).
- Custom maps: only the absence of the restrictions above is defined. Triggers and slot restrictions are **not designed**, so don't build them.
- TBD: min/max map size, max slots, how slot count relates to symmetry (e.g. 4 slots on a 180° map), and objects on the symmetry axis or center. Pick reasonable editor limits as placeholders and flag them.

## Verification
- Create a Skirmish Default 2-player map with 180° symmetry: half the map is editable. Painting a cliff, raising a hill or placing a patch in it makes the rotated copy appear in the other half immediately. The other half can't be edited directly.
- Place the GDO layout at the editable start position: the opposite start gets the rotated layout, with building orientations rotated.
- Leave the Cults layout incomplete: the editor lists the missing Recruits, and the map can't be started.
- Try to place an owned Barracks outside a layout on a Skirmish Default map: refused. A neutral object: allowed.
- 90° symmetry is only offered for square maps.
