# Map editor

Read this when you touch the map editor.

## Map editor

`cargo run -- --editor [file.json]` opens the map editor (`crates/game/src/editor.rs` and `editor/`). It edits a map
(`MapDef`) and saves it as a map file (see Sim scenarios and tests). F1 in the editor lists the mouse and keys.

- **Starting**: with a file that exists, the editor opens it. With a file that doesn't exist yet, it starts a new map
  that Ctrl+S saves there. Without a file, it starts a new map with the creation dialog's defaults (Skirmish Default,
  64×48, 180° rotation, 2 slots), and `--editor --scenario <name>` opens a built-in scenario's map. A file that
  doesn't parse exits with code 2. A file that parses but fails validation still opens, with its problems listed, so
  that it can be fixed; `--player` is not checked against it (play-tests fall back to the first slot).
- **Files**: maps live in `maps/` at the repository root (`editor::files::default_maps_dir`), and new maps are saved
  as `maps/<name>.json`. Saves are atomic: `files::save` writes a hidden temporary file next to the target (named
  after the process too, so that two processes writing one file never write into each other's), flushes it and
  renames it over the target. Replays are written the same way (`files::write_atomically`). `maps/dev_skirmish.json` is the built-in `dev_skirmish` saved by the editor. `maps/playtest.json`
  (slice M5-P3) is the map to playtest on (`docs/playtest/README.md > Starting a pairing`): `dev_skirmish`'s terrain,
  each base beside the Cults' reference crystal line. A test checks that every map in `maps/` is playable, that the
  editor saves it unchanged, byte for byte, and that `dev_skirmish.json` still matches the built-in.
- **Actions**: every change is an `EditorAction` (`editor/actions.rs`), queued in the `EditorActions` resource and
  applied in `EditorSystems::Apply`. The input layer's actions, the UI's buttons, the text fields and the window's
  close button only queue actions. The edits are plain functions (`editor/terrain.rs`, `editor/objects.rs`,
  `editor/presets.rs`, `editor/document.rs`) and sim-side helpers (`Map::resize`, `Map::connected_region`,
  `MapDef::resize`, `add_slots`, `remove_slots`, `regenerate`).
- **Input** (`editor/input.rs`): the editor reads only the shared input layer (see Client architecture > Input), in
  the editor's input context. Its keys are `InputAction::Editor(EditorInput)` entries of `Bindings`, rebindable like
  the game's, and `editor_keys` turns the pressed ones into `EditorAction`s (`editor_action`). `editor_pointer` turns
  the `Pointer` into the pointer's actions: its ground point into `Hover`; the `UseTool` press (left button,
  `Pointer::press`) into `Press`, a `Drag` every frame to `MapPress::drag_ground` (over UI nodes on the view too, not
  off it) and `Release`; `Eyedropper` clicks (right button, `MapClick`) into `Pick`; and the `Zoom` wheel
  (`ActionState::scrolled`) into `Zoom` at the cursor. Over the UI nothing starts, by the layer's over-UI test (the
  F1 help is a full-window UI node). While a text field has the keyboard no binding fires. The middle-mouse grab,
  the arrow keys, G and Esc are the shared actions.
- **Document**: `EditorDoc` holds the map, its file and the undo history, with one snapshot per completed action (a
  whole stroke, drag or field edit). Its dirty flag is worked out once per change, not per frame. When the document
  changes, the editor replaces the sim with `Sim::preview` of the map with its default setup. That is a sim at tick 0
  that shows every spawn that fits on the map, even when the map is invalid, so the game's map and object rendering
  show the document.
- **New maps** (`editor/creation.rs`): New opens the creation dialog (`NewMapDialog`, a modal panel), which edits a
  `MapSettings`: type, size, slot count, symmetry, name and description, with the placeholder limits shown and
  enforced (see Sim scenarios and tests). It offers only the slot counts and symmetries the other settings allow, and
  Create stays disabled until `MapSettings::check` passes. While it is open, other edits are refused; Cancel or Esc
  closes it. The map type and the symmetry are fixed once the map exists, and shown as such in the properties panel;
  a 90°-rotated map can only be resized to another square.
- **Map types** (`core/map_editor.md > Map Types`): on a Skirmish Default map, objects outside the Starting Layouts
  are neutral, so the editor offers no owner and refuses owned ones (`PlaceProblem::OwnedOnSkirmishMap`). Custom maps
  have no restrictions. A new Skirmish Default map lists its empty GDO, Syndicate and Colonists layouts as problems
  (`LayoutMissing`), so it can't be play-tested until they are placed with the Layout tool (see Starting Layouts
  below).
- **Starting Layouts** (`editor/layouts.rs` and `layouts/`, slice M2-I; `core/map_editor.md > Starting Layouts`,
  design changes 0008, 0009, 0012 and 0097): the Layout tool (T) edits the layout of `EditorTools::faction` at
  `EditorTools::slot`, an editable slot (the generated ones follow it, turned to match). A click places
  `EditorTools::layout_kind` turned as `EditorTools::orientation` says, or grabs an object of the layout to drag it;
  Delete removes the selected one (`Selection::LayoutObject`); each is one undo step. The palette's Starting layouts
  section (`ui/starting.rs`) picks the slot, the faction and the kind, says what the layout lacks or that it is
  complete, and flags outdated layouts (`layouts::outdated_text`); choosing the tool scrolls the palette to it.
  - **What it offers** (`layouts::palette`): on a Skirmish Default map the faction's Starting Conditions with how many
    the layout may still take (a placed kind moves the tool on to the next one missing); on a Custom map every kind a
    player may own. Every faction has Starting Conditions (plan decision T63), so every faction has a palette.
    Structures snap to tiles and units, such as the Colonists' Drones, go at the point clicked (`objects::snap_spawn`;
    `tests/editor/colonists.rs`). The Syndicate's palette is its Tunnel, Hideout and Agent (slice M4-B); a Tunnel
    whose Tier 1 Tunnel Area reaches off the map or onto undrillable ground, or a Hideout outside that area, shows a
    red ghost and is refused with the rule's text (`rule_text`; `tests/editor/syndicate.rs`). The Cults' is their
    Recruitment Hall, Storage and six Recruits (slice M5-B), and an incomplete Cults layout lists the Recruits it
    lacks and keeps the map from play-testing (0012; `tests/editor/cults.rs`).
  - **Checks** (`layouts/check.rs::check_object`): the editor's own rules first (the Starting Conditions' kinds and
    counts on a Skirmish Default map, the editable part, no overlap with the object's own image), then the sim's:
    `MapDef::validate` of the map with the object in place, keeping only that object's problems. The sim is the
    oracle; the editor has no placement rule of its own, so no sight checks or warnings (0097). The ghost
    (`LayoutPreview`, recomputed only when the document, the tools, the cursor or the gesture changed) shows the
    verdict, red on the refused tiles, and the status line says why.
  - **Overlay** (`overlay/layouts.rs`): while the tool is on, the other factions' layouts at the slot are drawn
    dimmed, the edited one outlined in the slot's colour, its Deployment Center's build area outlined, and the ghost
    with its images; the preview's sim plays the edited faction in every slot (`layouts::preview_scenario`), so it
    shows that faction's layouts. Layout objects with a problem are outlined in red with every tool.
  - **Problems** (`layouts/problems.rs::problem_lines`): the Problems list holds everything that keeps a play-test
    from starting (`PlayTestSetup::problems`: `Scenario::validate` of the map with the play-test setup, listed again
    whenever the document or the setup changes, `sync_problems`), so two slots' layouts in each other's way
    (`LayoutOverlap`) show as soon as the map has no problems of its own, not only when F5 is refused (slice M2-J2).
    It gives each layout that lacks objects one line that names them all and the tool ("S1 GDO layout: add Deployment
    Center, Extraction Facility, Extraction Plate x2 (Layout tool, T); S2 follows"), folding the generated slots'
    identical lines into their editable slot's. A click on a line chooses the slot, the faction and the first missing
    kind, or selects the broken object. A refused play-test (F5) shows the first line.
  - **Outdated layouts** are derived (T11): a non-empty layout that isn't its faction's current Starting Conditions
    is flagged, so one still being placed is too ("Outdated or unfinished"); on a Custom map the line only says that
    the layout differs. Nothing is fixed automatically.
- **Symmetry**: the editor edits only the editable part (`MapDef::region`); the locked part is dimmed
  (`overlay/tints.rs`), and the generated slots' markers are drawn faded. Every edit of the locked part is
  refused with a status message that names it: strokes and fills there, placing, moving or selecting what lies there,
  and the generated slots' markers and values. After every action (`Editor::regenerate`), `MapDef::regenerate`
  rebuilds the locked part when the document changed; a selected object keeps its selection through it. Slots are
  added and removed in whole sets. An object that would overlap its own image is refused
  (`PlaceProblem::OverlapsOwnImage`, the placeholder rule). A file whose locked part differs is listed as
  `NotSymmetric` until its first edit regenerates it. The placement previews show a ghost at each image.
- **Custom tile presets** (`editor/presets.rs`, `core/map_editor.md > Custom Tile Presets`): Add custom preset makes
  "Custom n" with Plane's properties and look and chooses it; the chosen preset's name, properties and look are edited
  in the palette, each one undo step. The default presets are fixed (shown in one line), and a preset some tile still
  uses can't be removed (the status line says how many tiles). `MAX_CUSTOM_PRESETS` and `MAX_PRESET_NAME_CHARS` are
  placeholders.
- **Modes**: `ClientMode` is `Game`, `Editor` or `PlayTest`, and only the HUD, the editor and the input layer (which
  takes the game's bindings in a play-test, the editor's in the editor) look at it. F5 starts a play-test of the
  document with the play-test setup (`PlayTestSetup`, the Play-test panel): each slot's faction, which slot you play,
  the seed, and debug commands on. It is not part of the map. `Sim::try_new` must accept the map; the game runs at
  the game's zoom, and its top bar says "Play-test (F5)" (`hud::TopBarNote::PlayTest`, slice M5-S6). Only F5 returns
  (Esc is the game's Cancel), to the editor with the document, its history and the view unchanged. A play-test has no
  playtest menu (F10; *Client architecture > Playtest menu*), and starts with the map explored as a game does, unless
  `--unexplored` (`cli::into_config` sets the scenario's game options, which the play-test setup keeps).
- **Unsaved changes**: New, Open, Quit and the window's close button (`WindowCloseRequested`, through
  `editor/input.rs::close_requests`; `platform::ConfirmClose` keeps the platform from exiting by itself) ask once with
  a status warning and go ahead when asked again, also during a play-test.
- **Positions**: every position in an `EditorAction` is a ground point (su), the one drawn under the cursor
  (`coords::ground_under`). `CenterOn` centers the view on where its point is drawn, and `Zoom { at }` keeps that
  drawn point where it is on screen.
- **Terrain tools** (`editor/terrain.rs`): Paint and Fill set presets and keep the heights; a Traversible preset is
  refused, with a status warning, on tiles more than 1 level steep (`terrain::refuses`). The Height tool works on
  corners: brush N covers N × N tiles and changes their (N + 1)² corners. Set gives them a level; Raise and Lower move
  each one level, once per stroke however often the brush passes. `Map::set_corner_heights` then cascades the slopes,
  so a height edit never leaves a too-steep tile. It may slope the ground under a resource: that is allowed and
  listed as a problem. The overlay previews each dab: the tiles, the corners it sets (white) and the ones the cascade
  moves (amber), from where they are drawn now to where they will be, and in red the tiles a paint would refuse.
- **Objects**: Place refuses spots off the map, overlapping another footprint or its own image, in the locked part,
  or, for resources and structures, on a sloped tile (the ghost turns red). A plate may go onto a patch that no plate
  covers, the one overlap the sim allows (`ObjectKind::may_overlap`). Structures are placed turned as R, Shift+R, F
  and Shift+F (or the Turn buttons) turned them while the Place or the Layout tool holds one (`editor/orientation.rs`:
  a quarter turn either way, a flip left and right or top and bottom, as seen; `EditorTools::orientation`), their
  footprints and ghosts turned with them; the same keys as the game's placement ghost. New objects get
  `SpawnSettings::defaults`; selecting one shows its values as text fields (the interval in seconds), which commit
  `SetSpawnSettings`, one undo step each.
- **View**: G toggles the grid, K the slope tint (flat green, sloped amber, too steep red, in the terrain's z band).
  The middle-mouse grab pans as in the game. The status line shows the tile under the cursor with its preset, the
  interpolated height there, its level, whether the tile is flat, sloped or too steep, and whether it is locked.
- **Selection** is a spawn index, a slot's marker or a layout object (slot, faction, index). An undo, a redo or a
  resize may shift the indexes, so they clear the selection; so does switching documents.
- **UI work follows changes** (review finding): each list (`ui/lists.rs`) is rebuilt only when what it shows
  changed, keyed so that a field being typed into survives marker moves and preset edits; the buttons are restyled
  when the editor's state changed or the pointer moved over them; the texts and fields refresh only when their values
  changed. Text fields commit in `EditorSystems::Input`, before the frame's actions, so a click elsewhere (Save, a
  tool, another field) applies the typed text first; Enter commits, Esc gives up the text.
- **Tests** (`crates/game/tests/editor/`: `session`, `edits`, `map_edits`, `creation`, `symmetry`, `review`, `input`,
  `units`, `layouts`, `colonists`)
  build a headless app with `AppConfig::editor`, push `EditorAction`s into `EditorActions`, call `app.update()`, and
  assert on `EditorDoc`, the sim, the view and the UI state. Positions are ground points, such as
  `TilePos::center()`. Tests of the input itself (`input`: keys, pointer, wheel, over the UI, typing) go through the
  layer's `InjectedInput` (`tap`, the virtual mouse, cursor and wheel). Never inject Bevy's raw input, except a text
  field's own keys (Enter, Esc) and the window's close button (`WindowCloseRequested`).
- **QA**: `--editor-script` applies its actions in the first frame, before any screenshot. The editor's sim stays at
  tick 0, so take screenshots at tick 0 unless the script play-tests (`"PlayTest"`).

```sh
echo '["FitMap", {"SetTool": "Paint"}, {"Press": [100.0, 80.0]}, "Release"]' > /path/to/edits.json
cargo run -- --editor maps/dev_skirmish.json --editor-script /path/to/edits.json --screenshot 0:/path/to/editor.png
```

- **UI**: bevy_ui with Bevy's `bevy_ui_widgets` feature (headless `Button`, `TextInput`, `ScrollArea` and
  `Scrollbar`) and `ui_picking`. Both side panels scroll with the wheel and with a scrollbar along their inner edge
  (`ui/layout.rs::spawn_side_panel`); the right one lists the problems before the play-test's slots, which run long
  on maps with many slots. Its font has ASCII glyphs only, so UI texts stay ASCII ("Rotate 90", not "90°"). The
  player slot colours, the overlay, locked-part, slope tint, dimmed-layout and build-area colours, the scrollbar's,
  UI sizes, `NEW_MAP_SIZE`, `MAX_DESCRIPTION_CHARS` and the sim's creation limits are placeholders.
