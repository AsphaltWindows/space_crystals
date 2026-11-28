# Map model

Read this when you touch the map, tiles, presets, corner heights, slopes or symmetry (`crates/sim/src/map.rs`).

## Map model

The design is `core/map.md`, and the code is `crates/sim/src/map.rs` and `map/`.

- **Tiles**: a `W × H` map has one `Tile` per tile, which is just a `TilePresetId` into the map's preset table: the
  five default presets (`core/map.md > DefaultTilePresets`) at fixed indices, then the map's custom presets. A tile has
  no height of its own. Map sides run from 1 to `map::MAX_MAP_SIDE` (256, a placeholder), checked when a file is read,
  before anything is allocated, and by `MapDef::validate`.
- **Presets** (`core/map_editor.md > Custom Tile Presets`): a `TilePreset` has a name, its five properties and a
  `TileLook` (Plane, Rugged, Cliff, Mountain or Water), which is all the renderer draws by. The default presets are
  fixed. `Map::add_preset`, `set_preset` and `remove_preset` edit the custom ones; removing one moves the later ids
  down and renumbers their tiles, and `preset_use_count` says whether any tile still uses it.
- **Corner heights** (`core/map.md > Elevation`): a `(W + 1) × (H + 1)` grid of levels 0–16 (`CornerPos`,
  `Map::corner`, `TilePos::corners()` in NW, NE, SW, SE order), shared by adjacent tiles, so the terrain is
  continuous. `Map::height_at(p)` interpolates bilinearly inside the tile that contains `p` (written from the NW corner,
  so a flat tile gives its level exactly; off the map it takes the nearest point on it). `elevation_level` floors it. A
  tile is **flat** when its four corners are equal (`is_flat`, `tile_level`), else **sloped**; `tile_slope` is its
  highest corner minus its lowest.
- **Maximum slope**: a Traversible tile may slope at most `MAX_TRAVERSIBLE_SLOPE` (1) level; `is_too_steep` and
  `too_steep_tiles` find violations, and `MapDef::validate` lists them (`ScenarioProblem::TooSteep`). Non-Traversible
  tiles (Cliff, Mountain, Water) may be as steep as they like.
- **Map resources and structures need flat ground** (`core/map.md > Elevation > Map resources only on flat terrain`,
  `core/map.md > Structures only on flat terrain`): every footprint tile of a kind with `ObjectDef::needs_flat_ground`
  must be flat. Validation flags it (`SpawnProblem::NotFlat`) and `Command::Debug(Spawn)` refuses it.
- **Height edits**: `Map::set_corner_heights(targets)` pins the target corners and moves every other corner as little
  as needed for the maximum slope (two corners are linked when they share a Traversible tile, so slopes cascade
  outwards, AoE2-style, and stop at non-Traversible tiles). It returns the corners it changed. On a map without
  violations, targets that keep the rule among themselves (all at one level, or each one level from where it was)
  leave none. `Map::resize` keeps the corners that remain, and new ones continue the old edge flat outwards.
- **Symmetry** (`core/map_editor.md > Symmetry`): `sc_sim::Symmetry` (six modes; 90° rotation only on square maps) gives
  its `MapTransform`s, pure functions that map tiles, corners (the corner grid is one larger than the tile grid),
  points, footprints and building placements (rotation included) to their images, and its `EditableRegion`, the part
  a map maker edits. `MapDef::regenerate` (`scenario/generate.rs`) builds the rest of a symmetric map from that part:
  see Sim scenarios and tests.
