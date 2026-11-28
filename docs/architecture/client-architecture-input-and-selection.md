# Client architecture — Input and selection

Read this when you touch input contexts, bindings, actions, text fields, or selection (model, picking, rules, systems).

**Input** (`input.rs`, `input/`): one shared layer turns the keyboard, the mouse, the wheel and the screen edge into
the input of the game and of the map editor, in `PreUpdate` (`ReadActions`, after Bevy's input, UI picking and the
focused text field's key handling), and gameplay and editor code read only what it produces: `InputAction`s, which
become UI state changes, `Command`s and `EditorAction`s. Never read raw keys, buttons, the wheel or the cursor
elsewhere.
- **Input contexts**: the mode picks whose bindings apply (`ClientMode::input_context`): the game's in a game and an
  editor play-test, the editor's in the editor. An action works in its own context only (`InputAction::context`), so
  one key can mean different things: V is a command slot in the game and the Select tool in the editor. The camera's
  pans and grab, the grid (G), `Cancel` (Esc), the editor's play-test switch (F5) and the placement keys (below) are
  shared by both. A button
  held across a context change ends its action. The editor's actions are `InputAction::Editor(EditorInput)`.
- **Bindings table** (`input::Bindings`, a `Resource`): the one list of `(InputAction, Binding)` entries for every
  game and editor action. A `Binding` is `Press(Chord)`, a key or mouse button with the modifiers it needs (Q,
  Ctrl + 1, Shift + Tab); `DoubleTap(Chord)`, the same chord pressed twice within `input.double_tap_seconds` with no
  other bound button in between; `Modifiers(..)`, held while those modifiers are, for the modifier actions (`Queue`,
  `ToggleSelection`, `SelectSameType`); or `Wheel(..)`, the mouse wheel while those modifiers are held (the editor's
  zoom; `ActionState::scrolled` says how far it turned, in lines). A press fires the action of its most specific
  binding in the context: of the bindings whose modifiers are all held, the one needing the most modifiers, then a
  double tap over a single press, then the earlier entry. So Shift + 1 adds to group 1 instead of recalling it, while
  Shift + Q presses slot Q with `Queue` held. One exception: in the editor Ctrl + key is a command of its own, so a
  held Ctrl blocks the editor's key bindings that don't need it (Ctrl + B is nothing, not the Paint tool;
  `InputContext::ctrl_blocks`). Another: the placement actions (`InputAction::is_placement`, today the editor's R,
  Shift + R, F and Shift + F that turn the structure being placed) take a press only while `Placing` says a structure
  is being placed, and then win over the press's other bindings, which they override (`Bindings::resolve_placing`):
  F flips then, and chooses the Fill tool otherwise. Whoever places reports `Placing` (the editor's
  `editor::input::report_placing`, in the editor only); a frame's input follows the report of the frame before. The
  game's AwaitingPlacement shares them (slice M2-H2, risk R11): the four belong to no context, so in a game they turn
  the ghost over the command slots R and F (the editor's `Turn`), and the game reports `Placing` from the local
  ControlState (`command_ui` `report_placing`, in the game modes, outside the command input set, so the keys end with
  the placement whatever stops the input: `Validate` ends a placement in the frame its selection stops being
  commandable, before the report, on a defeat as at the end). Shift + R and Shift + F are theirs while placing, not
  slot R or F with `Queue` held. The action stays held until its button goes up. `Bindings::default()` holds the
  design's defaults (camera keys, G for the grid, P for unit paths, Select and ContextCommand on the left and right
  buttons, Esc to cancel, Tab and Shift + Tab, the control group keys, the 4 × 3 command grid QWER/ASDF/ZXCV with
  the standard slots Z, X and C, Shift to queue) and the editor's (its F1 help lists them). A rebinding UI edits it
  with `bind` and `unbind`. It isn't serializable yet: loading it from a file needs Bevy's `serialize` feature for
  `KeyCode` and `MouseButton`. The selection adds `CenterOnObject` (Alt) to the modifier actions, for clicks on
  portraits, and F2 for the debug player switch, the fog of war F3 for its debug toggle (placeholders: the design
  has no debug keys), and the replay playback keys of the game's context: K pauses and resumes, `.` plays one
  tick, `-` and `=` change the speed (`ReplayPause`, `ReplayStep`, `ReplaySlower`, `ReplayFaster`; placeholders).
  Slice M5-S7 adds F11 and Alt + Enter, in both contexts, for `ToggleFullscreen` (*Window*), and F10 in the game's
  for `PlaytestMenu` (design changes 0413 and 0414; placeholder keys). Slice M5-MM2 adds Space in the game's for
  `JumpToAlert` (design change 0443, the design's key), which `ReplayPause` had until then: the pause moved to K.
  Slice M5-X2 adds F8 in the game's for `DebugRestartWithData` (a placeholder, like the other debug keys): with
  `--tuning`, the game restarts with its data directory read again (*Tunables*).
- **Modal** (`input::Modal`, slice M5-S7): while a menu holds the input (the playtest menu), the frame's presses go to
  `ActionState::modal_pressed` for the menu, and the rest of the client sees only the actions that pass a menu
  (`InputAction::passes_modal`: the window's toggle, F10, and the debug player switch and fog toggle, so hot-seat
  players can still look at their own banners, and slice M5-X2's F8, the menu's Restart with the data directory read
  again): no held action (the camera keys, the screen edge), no wheel, the
  pointer over the UI and no map press, so no selection, command or pan starts, and one under way ends without a click
  or a box. What is still held when it lets go comes back held, never pressed, so the click that closes it does
  nothing else.
- **Output**: `ActionState` (a `Resource`) holds this frame's actions (held, just pressed, just released), the wheel's
  turn, the modifiers, the context, the cursor in logical pixels, and the `Pointer`: the view point and the ground
  point (`coords::ground_under`) under the cursor, whether it is over the UI (a UI node by Bevy's picking, or outside
  the world view: the one over-UI test of the game and the editor), the press in progress and the drag box. A press of
  `Select` or the editor's `UseTool` that goes down on the map is followed until its release (`Pointer::press`, a
  `MapPress`: where it went down, the farthest the pointer has been from there (`moved`), whether that is further
  than `input.drag_threshold_px`, and the ground point under the cursor now, which keeps following it over UI nodes on
  the world view, as the editor's drags do). In the frame its button goes up, the press is `Pointer::released`, for a
  rule of its own about clicks and drags (the command flow's targeting click allowance). `MapClick`
  messages report clicks on the map: `Select` when released, unless it became a drag, and `ContextCommand` and the
  editor's `Eyedropper` when pressed, with the modifiers and a double-click flag. A `Select` drag ends as a
  `SelectionBox` message in view space, kept inside the world view, with the modifiers held at its release. Nothing
  starts over the UI. What the modifiers of a click or a box mean comes from the table
  (`Bindings::holds(InputAction::Queue, click.modifiers)`).
- **Text fields**: while a text field has the keyboard (`TextFocus`), and in the frame after it lost it, keys type
  text and fire no binding, from the real keyboard or the injected one. So the Esc that gives a field up doesn't also
  cancel. A field's own keys (Enter commits, Esc reverts) are Bevy's keyboard input, read by the field
  (`editor/ui/fields.rs`), not bindings.
- Double taps and double-clicks are timed on `Time<Real>`. Placeholders until the design gives values, in
  `tuning/input.rs` and `tuning/camera.rs` (*Tunables*): `input.drag_threshold_px`, `input.double_tap_seconds`,
  `input.double_click_seconds`, `input.double_click_distance_px`, `camera.edge_pan_margin_px`, `input.pixels_per_line`
  (wheels that report pixels).
- **Injection**: tests and QA scripts drive the layer through `InjectedInput`, a virtual keyboard and mouse, without
  winit. Press and release its `keys` and `mouse` (`ButtonInput`s) like Bevy's; the layer clears their press marks
  after reading them. `wheel` turns the wheel by that many lines before the next frame, `cursor` stands in for the
  mouse cursor, `over_ui` puts it over a UI node (headless apps have no UI picking), and `held` and `pressed` inject
  actions directly, past the bindings. Never write to Bevy's own `ButtonInput` resources: its input systems clear
  their presses before ours run. `RawInput` turns the real devices off in QA runs.

**Selection** (`selection.rs`, `selection/`; `systems/controls/selection.md`): client UI state, never sim
state. `ControlStates` holds every player's `ControlState`, a `Selection`, 10 control groups and the
ObjectInterfaceState (see Commands UI); the input changes and the HUD shows the local player's (`LocalPlayer`).
- **Model** (`selection/model.rs`): a selection is a set of object ids with the types they group as, a `GroupType`:
  the kind, and whether the object is unfinished, not in operation (`!Object::is_operational`): a site under
  construction (`Object::is_site`, slice M4-J1) or a transformation (`Object::is_transforming`: a Beacon a Drone
  deploys into, a building warped in). Its SelectionGroups are derived:
  one per `GroupType`, each by id, and one per instance of an ungroupable type (the map resources, which are never
  selected with other objects, design change 0048; the Colonists' Egress Node and Beacon, 0122; the Drone is
  Groupable, 0125). So a type's unfinished buildings are a group of their own, "as if it were another type", with
  their own panel (only Cancel), and one that completes joins the finished group (0185: the group key is "(type,
  finished?)"). A new dimension of the key goes into `GroupType`, which the rules, the panel (`offers`, `commands`)
  and the HUD read; a new kind needs nothing (0173's Peacekeeper (Shielded) is one). The groups come in one stable
  order, by the `GroupKey`: `ObjectKind`, the finished before the unfinished, then the instance (`Selection::groups`),
  which is the order of the SelectionPanel's portraits and of Tab and Shift + Tab (the placeholder of 0055). The
  ActiveGroup is one SelectionGroup, named by its `GroupKey` (0048): Tab and Shift + Tab cycle it with wrap-around,
  adding objects keeps it, and when its last member leaves the selection (dead, or no longer selectable) the next
  group in Tab order becomes active, wrapping around (design change 0199, `Selection::fix_active`); no group left,
  the selection is empty. At most `MAX_SELECTED`
  objects (a placeholder: the order limit).
- **Picking** (`selection/pick.rs`): `Selectables` is what the local player may select this frame (`selectable_by`:
  its own objects, others on Visible tiles; on Explored ones only a remembered Supply Delivery Station, at its
  record, and only for a GDO player, `remembered_selectable_by`, `core/vision.md > Explored Selectability`; with
  the fog off, everything), each object's box as it is drawn: interpolated, lifted by the height it stands at, in
  view space. A click picks the frontmost box under it (grown by `input.pick_slop_su`): a unit before a tile-based
  object, which it is drawn above (see Draw order), then the southernmost, and of boxes drawn at one depth anything
  before a map resource, so a click on a covered patch's tile picks its plate (design change 0099); a box
  (the input layer's `SelectionBox`) applies the priority tiers. `Relation::of` says own, enemy or neutral (no teams
  yet, so no allies); allies would count with enemies in the tiers. `Selectables::target_at` picks a target the same
  way among the selectables and the records the player remembers but may not select (design change 0139:
  targetable, not selectable), for a right-click, a targeting click and the pointer; with the fog off there are no
  records to add.
- **Rules** (`selection/rules.rs`, pure and unit tested): map clicks, boxes, portrait clicks, control group keys and
  Tab on a `ControlState`, under the Constraints: another player's object or a resource is only ever selected alone,
  and any click on one selects it alone (design change 0048). On an own object, a unit or (slice M5-N7, design change
  0410) a building, a click selects it, Shift toggles it, Ctrl or a double-click selects its type on screen, and Ctrl
  + Shift adds that type (0052); "its type" is its SelectionGroup's (`Selectables::own_on_screen` by `GroupType`), so
  an unfinished building takes only the unfinished ones of its kind, and units and buildings may mix. A box selects
  what its tiers give, one own building at most, even of a groupable type (design change 0411: several Halls are
  selected with a double-click, a Ctrl- or Shift-click, or Shift + box), and Shift + box adds that (0049). The add inputs all go through `ControlState::add_to_selection`: they
  remove nothing and add nothing the Constraints forbid, so they do nothing while another player's or a neutral
  object is selected (0053). A plain click on empty ground or an empty box clears the selection; a modified click
  there or an empty Shift + box changes nothing (0051). Control groups take only own objects: Assign and Add with
  another player's or a neutral object selected change nothing (0056). What the modifiers of a click or a box mean
  comes from the bindings (`ClickRoles`; Shift is `ToggleSelection`, the add inputs' modifier). Every rule the design
  leaves open is marked as a placeholder where it is made.
- **Systems** (`SelectionSystems`): `Sync` (in `SyncSim`) removes objects that left the sim from every selection and
  group in the frame of their tick (`SimEvent::Removed`, deaths included, and anything else gone; a station the player
  still selects by its record stays), and those their player may no longer select (an enemy that walked into the fog;
  a placeholder, the design is silent). A selected object the player sees whose `GroupType` changed in place, a new
  kind (`SimEvent::KindChanged`) or a building that completes or starts transforming, stays selected as its new type
  (`ControlState::set_type`). The ActiveGroup stays while it exists (`systems/controls/interface.md >
  ObjectInterfaceState`: a change the player didn't make keeps the state unless the ActiveGroup becomes empty; design
  change 0199: changes in a group that isn't active never change it); one whose members changed type gives way to the
  group they joined (0199), and the ObjectInterfaceState resets with the ActiveGroup. A unit's type there is the sim's
  with the toggles already sent for its next tick applied (`PendingCommands::kind_of`), so the group a press switched
  (`ControlState::toggle`, below) stays switched in the frame before the sim's tick.
  `Sync` also starts every state over when the sim is replaced (`SimRes::generation`). `Input` (in `Actions`) applies
  map clicks, boxes, the control group keys (Recall and Center snaps the camera to the drawn centroid), Tab, and the
  debug player switch (F2, only while `SimConfig::debug`: the client acts as the next player, so "own" and the state
  in use change; a replay's playback switches without debug commands too). The HUD's portrait clicks run after it.
  Systems change a state only through `LocalControl::update`, which marks `ControlStates` changed only on a real
  change, and puts the ObjectInterfaceState back to DefaultState when the Selection or its ActiveGroup changes; a
  control group's recall puts it back even when the selection stays the same (`ControlState::recall_group`, design
  change 0078). While a target is awaited, map clicks and boxes select nothing (the drag box isn't drawn): the
  left-click is the target's.
- **Visuals** (`selection/markers.rs`): one marker entity per selected object: a ring around a unit, in the unit-decal
  layer under it (over the structures), or a frame inside a footprint, which the object's visual would hide there, so
  it is drawn just in front of the object (`FRAME_Z`: half a su of ground y, under the objects south of it); coloured
  by relation, the ActiveGroup's solid and the others' dashed; meshes cached per shape, relation and activity, and a
  marker whose object changed its shape (a kind changed in place) takes its new shape's mesh. The drag box is four
  edges of one shared mesh in the overlay layer, shown while `Pointer::drag_box` is set. The selected buildings' door
  arrows (design change 0394, `selection/doors.rs`: `SelectedDoors`, from `door_arrows`, as the placement ghost has
  them) are one mesh in the overlay layer under the ghost, rebuilt only when the buildings shown or the sim change.
