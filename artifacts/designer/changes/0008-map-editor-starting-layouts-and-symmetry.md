# 0008 — Map editor: per-faction Starting Layouts with overlay editing; symmetry tools

- **Date**: 2026-11-23T13:26:08-05:00
- **Answers**: Q0004 (partly: how starting bases are placed. Still open: default layouts, layout validity/overlap, keeping maps in sync with Starting Conditions, slot count/teams, pre-placed objects, metadata)
- **Docs**: `core/map_editor.md` (new: Per-object Resource Values, Starting Layouts > Overlay editing, Symmetry Tools), `README.md`, Starting Conditions in `factions/gdo/economy.md`, `factions/syndicate/tunnels.md`, `factions/cults/economy.md`, `factions/colonists/economy.md` (pointer to Starting Layouts)

## What changed and why
**Starting bases are placed by the map maker, per faction** (Q0004 option B, not automatic layout rules). Each start position on a map has a separate Starting Layout for each of the four factions, giving the position of every starting building and unit. At game start, only the picked faction's layout is used. The editor can show **all factions' layouts at the same position overlaid**, so the map maker can define and compare them in one place.

**Symmetry tools:** the editor can make maps symmetric automatically, with 180° rotation, 90° rotation (square maps only), a horizontal-axis mirror, a vertical-axis mirror, and both mirrors together.

## Implementation impact
- Replaces the Director's assumption of "start location + automatic starting-base layout". The map format stores, per start position, a per-faction list of placed starting objects (type, position, building orientation). Spawning Starting Conditions reads the picked faction's layout.
- For the test map: start positions in the corners (0007). The Director authors the four faction layouts there.
- Editor: an overlay view of the four layouts at a start position.
- Symmetry tools: 5 modes as above. Still TBD: live mirroring vs one-shot generation, and exactly which data is mirrored (I've asked: tiles, corner heights, resources, start positions + layouts, pre-placed objects with transformed orientation).
- Still open: whether the editor pre-fills a default layout per faction; how layouts are validated; what happens when Starting Conditions change after a map was made. Until decided, each faction's layout only needs to be valid on its own (normal placement rules), and layouts of different factions may overlap.

## Verification
- In the editor, select a start position and show all faction layouts: the Deployment Center, Tunnel, Recruitment Hall and Egress Node/Robo Portal (plus their units/plates) appear overlaid and can each be moved.
- Start a game on that map as each faction: the starting objects appear exactly where that faction's layout put them.
- Apply 90° rotational symmetry to a square map with one start position: four start positions result, each a rotated copy. On a non-square map the 90° option is unavailable.
