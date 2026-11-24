# Map Editor

The map editor creates and edits maps. It supports **every object** in the game (user's request), so maps can contain terrain, elevation, resources and starting bases, and anything the game can place. Maps are saved as files the game loads.

What a map contains is defined in core/map.md (tiles, tile presets, corner heights, resources). This file covers map types, creation settings and editor-specific features. All decisions here: user, 2026-11-23.

## Map Creation
When creating a new map, the map maker chooses:
- **Map type**: Skirmish Default or Custom (see Map Types).
- **Size**: width and height in tiles, within the editor's minimum and maximum (values TBD).
- **Number of player slots** (= start positions). Maximum: TBD.
- **Symmetry** (see Symmetry). **Required** for Skirmish Default maps; optional for Custom maps.
- **Name** and **description** (map metadata).

## Map Types

### Skirmish Default
The standard competitive map type.
- **Symmetric by construction**: a symmetry must be chosen at creation, and only the necessary part of the map is editable; the rest is generated automatically (see Symmetry).
- **No slot restrictions**: the map cannot restrict which factions may be picked in a slot or fix teams. Any faction can play any slot.
- **Starting Layouts are restricted** to each faction's standard Starting Conditions and **must be complete** (see Starting Layouts).
- **Pre-placed objects**: neutral objects (resources, Supply Delivery Stations, other neutral objects) are allowed. **Player-owned objects are not allowed** outside the Starting Layouts.

### Custom
Maps without the Skirmish Default restrictions: any objects in Starting Layouts, player-owned pre-placed objects, symmetry optional. Custom maps will later need slot restrictions (allowed factions, fixed teams), triggers and more. **Not designed yet; deferred** by the user ("let's not worry about custom maps yet").

## Per-object Resource Values
Patch amounts and Supply Delivery Station settings are set per object in the editor (core/map.md > SpaceCrystalsPatch, core/map.md > SupplyDeliveryStation).

## Starting Layouts
Starting bases are **placed by the map maker per faction**. For every player start position the map defines a separate starting layout for **each faction**: where each of that faction's starting buildings and units goes (the starting objects themselves are defined in factions/*/economy.md > Starting Conditions and factions/syndicate/tunnels.md > Starting Conditions). At game start, only the layout of the faction that player picked is used.

### Placing from scratch
Starting Layouts start **empty**: the map maker places every starting object by hand (no pre-filled default layout).

### Overlay editing
The editor can show **all factions' starting layouts at the same start position simultaneously, overlaid on each other**, so the map maker can define and compare the positions of all buildings and units for all factions in one place. Layouts of different factions **may overlap** each other; each faction's layout only needs to be valid on its own (normal placement rules for that faction's objects).

### Layout validation
(user, 2026-11-24) Each faction's layout is checked with that faction's **normal in-game placement rules, minus visibility** (nothing is "visible" in the editor). For GDO:
- tiles Buildable and flat; no overlaps, except Extraction Plates on patches; no buildings on resources (core/entities.md > Placement Validation);
- the Extraction Facility and plates must lie within the build area of the Deployment Center placed **in the same layout**.

Starting **units** can't stand on structures or impassable tiles. There are no further checks or warnings (e.g. none about patches outside the starting DC's sight): map makers can place layouts however they like within these rules.

### Skirmish Default restriction
On a Skirmish Default map, each faction's Starting Layout may contain **only** the objects in that faction's standard Starting Conditions (e.g. for GDO a Deployment Center, an Extraction Facility and two Extraction Plates), and it **must be complete**: every starting object of every faction placed at every start position. The editor shows what is still missing, and the map cannot be played until all layouts are complete.

Custom maps have no such restriction.

### Outdated layouts
If a faction's Starting Conditions change after a map was made (e.g. an extra starting unit), the map's layouts for that faction are **flagged as outdated** until the map maker fixes them. Nothing is fixed automatically.

## Symmetry
The symmetry is chosen when the map is created. Supported symmetries:
- **Rotational, 180°** (two-fold).
- **Rotational, 90°** (four-fold). Only possible on a **square** map.
- **Mirror, horizontal axis.**
- **Mirror, vertical axis.**
- **Mirror, both axes** (user expects this to be rarely useful but wants it available).
- **None**: Custom maps only.

### Editing a symmetric map
Only the **necessary portion** of the map (one half, or one quarter for four-fold symmetries) is editable. The rest of the map is **generated automatically** from it, so the map is always symmetric.

**Everything is mirrored**: tiles, corner heights, crystal patches (with their amounts), Supply Delivery Stations (with their settings), start positions with all their faction Starting Layouts, and neutral pre-placed objects. Building orientations are rotated or flipped to match.

TBD: how the number of player slots relates to the symmetry (e.g. 4 slots on a 180° map = 2 start positions in the editable half); objects that straddle or sit on the symmetry axis or center (e.g. a 2x2 station in the exact center).

## Custom Tile Presets
Maps can define **custom tile presets** with any combination of game properties (core/map.md > TilePreset). For now, custom presets use the existing textures; **custom textures** per map are planned for the future.
