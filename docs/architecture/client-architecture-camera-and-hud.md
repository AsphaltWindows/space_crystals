# Client architecture — Camera, window, HUD and menus

Read this when you touch the camera, the window, the HUD, the minimap, its alerts, the playtest menu or Restart.

**Camera**: the camera is orthographic and shows exactly 28 GU horizontally (`core/camera.md > Zoom`; see
`ScalingMode::FixedHorizontal`). Its viewport is the area between the UI bars (`ViewportInsets`: the top and bottom
HUD bars in the game, the editor's panels in the editor), so the visible height follows from the aspect ratio.
`CameraView::center` is a **view** point; to look at a ground point, center on its `drawn_position`. It pans with the
arrow keys and at the screen edge, with bindings and speed pending in Q0002, and it stays clamped to the map's drawn
area (`coords::drawn_bounds`, from its highest-drawn to its lowest-drawn corner, so raised ground at the north edge
stays reachable). The middle-mouse grab (`core/camera.md > Panning > Middle-mouse grab`, `camera::GrabPan`) keeps the
view point grabbed on the press exactly under the cursor while the button is held, and stops on the release; the
game and the editor share it. Centering on something snaps instantly (`core/camera.md > Panning`). The camera starts
at the local player's slot's camera start (`Slot::camera_start`), or else on the primary starting structure
(`core/camera.md > Starting Position`): the GDO's Deployment Center, the Syndicate's Tunnel, the Cults' Recruitment
Hall or the Colonists' Robo Portal, where the player's Starting Layout places it (`Scenario::primary_structure`).
Without one (a Custom map's layout may lack it) the slot's
start position stands in, and the map centre for a player without a slot (`camera::start_center`, shown through
`start_view`). Only the editor zooms (`CameraZoom`). Under the debug player switch (F2) each player has a camera of
their own (`camera::PlayerCameras`, `camera/players.rs`, slice M3-J3): when the local player changes within a sim,
the camera keeps the view of the one before and goes to the new one's, at their `start_view` the first time; a new
sim (`SimRes::generation`: a play-test, a loaded scenario) forgets them all and leaves the camera where its switch
put it.

**Window** (`platform/window.rs`; slice M5-S7, design change 0413, `core/camera.md > Window Mode` and
`core/camera.md > Panning > Arrow keys and screen edge`): a game, the editor and a playback open in borderless full
screen on the monitor the window opens on (`window::FULL_SCREEN`, `MonitorSelection::Current`; at creation Bevy logs
that it can't select the current monitor yet and leaves the choice to winit, which takes the one the window opens on),
`--windowed` in a window of `--window-size`, and a QA run always in a window of its fixed size
(`PlatformPlugin::primary_window`: never full screen when `reproducible`), so screenshots keep their size.
`ToggleFullscreen` (F11 or Alt + Enter, both contexts; placeholder keys) switches between the two; back in a window, it
takes the size it had before (`WindowedSize`) and is centred. While the window has the focus, the cursor is confined to
it (`CursorOptions::grab_mode`, `CursorGrabMode::Confined`), in a window and in full screen alike: `confine_cursor`
follows the window's `WindowFocused` messages, frees it on a focus loss and confines it again on the focus's return; a
grab the platform refuses (bevy_winit puts the old mode back) is tried again for `CONFINE_TRIES` frames, and macOS,
which has no confinement, leaves it free. A QA run (`RawInput` off) never confines it. The screen edge pans within
`camera.edge_pan_margin_px` (8, the design's placeholder) of the window's edge. Tests: `tests/client/window.rs`.

**HUD**: built with bevy_ui (`core/camera.md > HUD Layout`). It has a top resource bar and, at the bottom, the HUD
Bar with five fixed sections. From left to right they are Minimap, Portrait, InfoPanel, SelectionPanel and
CommandPanel. All five have the same height and keep fixed horizontal slots, and a hidden panel's slot stays reserved
(`systems/controls/hud.md > HUD Bar`). bevy_ui lays out each root node inside its camera's viewport,
so the HUD needs its own full-window UI camera (`IsDefaultUiCamera` / `UiTargetCamera`). The sections sit at the
rectangles `hud::layout` computes from the surface size alone (`section_rect`, `PortraitGrid`); the same geometry
finds the portrait under a click, because headless tests have no UI picking (and `CommandGrid` the command button).
The InfoPanel (`hud/info_panel.rs`) and the SelectionPanel (`hud/selection_panel.rs`) show the selection (see
Selection), the CommandPanel (`hud/command_panel.rs`) its commands (see Commands UI); a section with nothing to show
shows its name (`SectionLabel`). A selected object the player doesn't see shows its record's last-known state and a
"Last seen … ago" line (a placeholder wording), except that a GDO player's remembered Supply Delivery Station shows
the live station's NextDeliveryTime (`core/map.md > SupplyDeliveryStation > InfoPanel`, design change 0057), "No
delivery scheduled" while it isn't empty, and never CurrentSupplies. An object with Shields shows "Shields a / b" after its HP, and (slice M5-N4a)
one with an energy pool "Energy a / b" after those, rounded down (to every player who sees it, placeholders). A
structure's panel follows design change 0105:
the local player's own shows its HP and, in the activity area under it (`InfoText::Activity`, hidden while empty),
what it is doing; another player's (or a neutral one) its owner and HP only. The activity's lines come from
`hud/info_panel/activity.rs::lines`, the one extension point for them: each structure capability adds its lines there
(construction with slice M2-H2, production with M2-H1), and nothing else of the panel changes. A producer's
(`activity::production`) are the unit in production and its progress from `Sim::production_status` (100% while it
waits at the doorway, 0085), the queue against `Production::queue_limit`, and the rally point (placeholder wordings). A
constructor's (`activity::construction`) are the building under construction and its progress (frames done of
its recipe's, whole percent rounded down), or the building ready to place. A Recruitment Hall's
(`activity::recruitment`, slice M5-C, `Sim::hall_state`) are its Effectiveness with its claimed tiles, "Effectiveness:
49% (49/100 tiles)", and its "Unit Control: usage / capacity" rounded down, before its production's "Recruiting Recruit:
50%" and rally point, with no queue line. A building that holds units (`activity::holding`, slice M5-F2) shows "Inside:
3 / 10", everyone inside of its capacity, and a pool building its free units by kind, "Free: Plain 2, Constitution 1,
Metabolism 0, Both 0", one that takes in several types, the Altar, by type, "Free: Recruit 0, Militant 2, Gunner 0,
Disciple 1" (slice M5-N1b, a placeholder; another, the Garrison, slice M5-N2, the kinds inside with their counts, in
its `accepts` order,
"Occupants: Militant 2, Gunner 1": a placeholder, the design has no GarrisonInstanceState; a Temple, slice M5-N3, its
staff, "Staff: 3 / 6", those not on their way out, `Sim::staff`, and their kinds as the pool's, a placeholder too; a
Shrine, slice M5-N5, its participants alike, "Participants: 8 / 8", a placeholder, beside its energy, "Energy 40 / 200",
the body's line of any pool); its
queue line names the entries and the units they reserve of its capacity, "Queue: Militant, Release (2 / 10)" (slice
M5-N6: a Seeker's entry reserves two, so the Chop Shop's two are "Queue: Seeker, Seeker (4 / 10)"), and a
Release under way is "Releasing Recruit: 50%", an Altar's order "Augmenting Militant (Enhanced Constitution): 45%",
its queue line naming its orders, a Temple's research "Researching Enhanced Constitution Level 1: 45%", its queue
line naming the technologies, of its research queue, "Queue: Enhanced Constitution Level 1 (1 / 5)", and no rally
point. A transforming structure's
(`activity::transformation`, first; slice M3-E) is the building it becomes and its progress, "Warping in: Robo Portal
45%" ("Deploying" from a unit, slice M3-F), and its HP is out of its interpolated MaxHP (`Object::max_hp`), as its HP
bar is. A plate
adds the remaining crystals of the patch under it for its owner only (0099 as design change 0155 limits it, slice
M3-J2): another player's plate is another player's structure, its owner and HP; an uncovered patch shows its count to
everyone. A worker of the local player's adds what it carries, "Carrying 8 Space Crystals", and its AssignedDropOff,
"Drop-off: Egress Node" (`worker_lines`, placeholder lines its owner alone sees). The group of a type's unfinished
buildings (0185) is "(unfinished)" in the InfoPanel's title, and its portraits are dimmed as the map dims a transforming
object (`PortraitCell::Object::unfinished`, `transforming_color`). The top bar shows the player on the left,
the tick on the right (in a replay's playback also the replay's last tick and a note on the playback, `ReplayText`:
speed, paused or end, and a desync in red), and between them, centred, the local player's resources as their faction's
DisplayHud lists them (`hud/resources.rs`, one `ResourceFigure` text each, `resource_bar` for their readings): for
the GDO Space Crystals and Supplies (the banks), Power as consumed / generated (`Sim::power`), red while consumption
exceeds generation (0090), and Unit Control as used / `GdoRules::unit_control_cap`, read through `Sim::unit_control` alone, so
that the units in production (design change 0143) count as the sim counts them; for the Colonists (slice M3-C,
`factions/colonists/overview.md > DisplayHud`) Space Crystals, Alloys, Essence and Flux, and Beacon Capacity as used
/ available (`Sim::army_cap`; red over capacity, 0112), with no Power; for the Syndicate (slice M4-B,
`factions/syndicate/overview.md > DisplayHud`) Space Crystals, Supplies and Tunnel Space as used / available
(`Sim::army_cap`; red over capacity, as `factions/syndicate/tunnel_area_and_space.md > Tunnel Space` has it, "12/10");
for the Cults (slice M5-C, `factions/cults/overview.md > DisplayHud`) Space Crystals and Unit Control as used /
available (`Sim::army_cap`: their Halls' usage and orphaned points over their Halls' capacities rounded down; over
capacity after a Hall is lost, not in red: the design gives the figure no warning). The name and the tick take equal
shares of the width the resources leave, which keeps them centred; what doesn't fit in a share is clipped there, never
drawn over the resources. (M5-S6) **Notes** (`TopBarNote`, placeholders) say how this client shows the game while it
applies, in amber, in the bar's row: "Fog off (F3)" after the name while `FogOfWar` is off, in games and replays alike
(the user took the map revealed by F3 for a bug), and "Play-test (F5)" before the tick in the map editor's play-test,
where a replay's note would be (a play-test is never a replay), in place of the banner the editor laid over the bar's
figures (found by slice M5-Z3). A note takes room of its own, so it never covers a figure, and gives
it up first: short of room, it shrinks, clipped at its end, while the name, the figures and the tick keep their size
(`spawn_note`). The bar is laid out in `vh`, so what fits depends on the window's shape, not its size (the client sets
no smallest size): at 16:9 and 16:10 every faction's bar fits its notes; at 4:3 a long bar (the Colonists', the GDO's
later in a game) clips their ends. Headless tests have no UI layout to measure, so QA screenshots check it. Tests:
`tests/hud.rs` (F3 and `--no-fog`), `tests/replay.rs` (a playback), `tests/editor/session.rs` (a play-test's note,
in the bar's row). Under the top bar, a placeholder
notice (`hud/notices.rs`, `STATION_EMPTY_TEXT` for `feedback.notice_ticks` sim ticks) tells the local player that a
Supply Delivery Station where their Agents gather is empty (`SimEvent::SupplyStationEmpty`, design change 0256 point 3:
"the owner gets a notification", notifications not designed yet). A second, under it, tells every player who hasn't left
the game (below) of each other player defeated while it goes on, a line each (`DefeatNotice`, `defeat_text`, for
`feedback.notice_ticks` from the defeat; `core/game_rules.md > On Defeat`: "**all players** are told", design change
0357); the defeat that ends the game the banner tells. At the end of the game a banner across the world view
(`hud/outcome.rs`, placeholder look and wording) shows the local player's `Banner`: VICTORY for the winners, DEFEAT for
everyone else, DRAW for everyone when nobody won (design change 0163), and DEFEATED with a spectating line while a
defeated player watches the game go on beside an ally (0102; never yet: the sim has no teams). A player defeated while
the game goes on with no ally left standing has **left for the results screen** (design change 0164, `hud/results.rs`;
`results::has_left`, the rule on its figures `results::left_the_game`): a placeholder, DEFEATED, their name and
faction, the game time and tick of their defeat, and whether the game goes on, on an opaque node over everything
below the top bar (`RESULTS_Z`), which takes the pointer, so nothing of the world shows or can be clicked. They stay
there after the end, and the end's banner shows over the screen (`BANNER_Z`). A side that falls in the game's last
tick never left. The screen is derived each frame from the sim and the local player, so the debug player switch and
a replay's playback show each player's view, and the sim isn't touched. A defeated player, and everyone once the
game is over, gives no commands: the command UI's input set doesn't run (`outcome::may_command`), and nothing is
`commandable` after the end.

**Minimap** (`hud/minimap.rs`, `hud/minimap/`; slice M5-MM, `systems/controls/hud.md > Minimap`, design change
0415): a CPU-side image in the Minimap section, in place of its name, one texel a physical pixel, sampled nearest.
`MinimapFrame` letterboxes the map in the section's content box, its aspect kept, on whole pixels, and holds the one
mapping each way: `ground_at` (a panel point to the ground, clamped onto the map, so a drag goes on beyond the panel)
and `screen_at`. The terrain is baked once per map and frame size (`raster::Terrain`: each tile flat in
`map_render::tile_base_color`, the terrain mesh's colour before its faces are shaded, and that dimmed); the fog and
the objects are painted over it (`raster::paint`) only when what they show changed (`LayersKey`: the sim, its tick,
the local player, `Sight::revision` and `records_revision`, the map), at most `REDRAWS_PER_SECOND` (15) times a
second of real time, and at once while the sim stands still (`SimControl::may_step`), so screenshots show their tick.
The layers read the sim's objects only through `Sight`: the tile states (`Sight::tile_states`, the fog layer's too),
then the records on Explored tiles, dimmed by `raster::KEPT` (what the fog's `EXPLORED_ALPHA` leaves, in linear
light), then the live objects `Sight::sees`, map resources under structures under units: footprints for structures
and patches, `raster::dot_texels` squares for units at `Object::map_pos`, in `object_render::player_color` (map
resources neutral). The camera's outline is a bordered node placed every frame after `apply_camera`
(`MinimapFrame::camera_rect` of `viewed_ground`: the world view's extent around the ground under its centre, cut to
the map). Its input
(`hud/minimap/input.rs`, `MinimapPointer`) is read in `Actions` before the command UI's: the panel is over the UI, so
no `MapPress`, `MapClick` or `SelectionBox` starts there, and nothing reaches the selection. A left press on the map
area (`MinimapPress`) centres the camera on the ground under the cursor each time that changes
(`follow_minimap_press`), unless the command flow claims it: `click_targets` takes it while a target is awaited, with
the map's click allowance, and resolves it on `MapTarget::on_ground` (ground only, the designer's placeholder). A right
press (`MinimapClick`) is the map's ground right-click there: `right_click_orders` gives both through one
`right_click`. Placement reads only `MapClick`s, so in AwaitingPlacement a minimap click only moves the camera (the
Director's reading). Measured (`hud/minimap/tests/perf.rs`, ignored, release; `maps/playtest.json`, 305 objects, the
fog on; load average 7.6): a redraw 0.029 ms at 1280 × 720, 0.039 ms at 1920 × 1080 (worst 0.049), 0.092 ms at
3840 × 2160 (worst 0.33), so 0.008 to 0.023 ms a frame on average at 60 frames a second. Tests: `hud/minimap/tests.rs`
and `tests/layers.rs` (the frame and the mappings, the outline at two zooms, the redraws, the fog layers),
`tests/commands/minimap.rs` (the clicks through the virtual mouse, and the panel).

**Minimap alerts** (`hud/minimap/alerts.rs`; slice M5-MM2, `systems/controls/hud.md > Minimap > Alerts`, design
change 0443): `take_alerts` runs in `Present` after `apply_camera`, as the outline does, over the frame's
`ViewerEvents` as the local player perceives them: a `SimEvent::Hit` with damage or shield damage on an object of
theirs (wherever it came from: placeholders) whose position (the object's now, or its `Died`'s in the same frame)
lies outside `viewed_ground`, the outline's rectangle, is an alert there. `MinimapAlerts` keeps the pings of the last
`feedback.ping_throttle_ticks` (160) and each player's latest, by the sim's generation; `MinimapAlerts::alert` throttles
one within `feedback.ping_area_gu` (8) of a ping of the player's still in its 10 s, faded or not (`tuning/feedback.rs`).
Each ping is the local player's of the moment and shows to them alone, so after F2, in a game or a playback, the other
player's hits ping, and F2 back shows the first player's again.
`place_pings` shows each ping of the local player as a round bordered node (`PingRing`, a pool kept hidden when
unused) in `PingLayer`, a child of the image node clipped to it, sized and faded by `ring_at` of its age as drawn
(`combat_effects::age`, so a held tick shows it exactly) until `feedback.ping_ticks` (32). `InputAction::JumpToAlert`
(Space, the design's key; the Game context) centres the camera on the latest ping's `drawn_position` in `Actions`
(`jump_to_alert`, after the frame's pans), nothing before the first or once the player has left the game. Measured
(`hud/minimap/alerts/tests.rs`, `alerts_perf`, ignored, release; load average 16): 200 hits on the player's objects
every tick, all over a 128 × 128 map, through the throttle and the rings, 0.017 ms a frame (worst 0.022), with up to
187 pings kept; a game has a few hits a tick and a few rings. Tests: `hud/minimap/alerts/tests.rs`
(the throttle, the latest, the ring, a new sim), `tests/minimap_alerts.rs` (off and on screen, a long fight and two
far apart, the ring's 2 s, Space, a restart).

**Playtest menu** (`playtest_menu.rs`, `playtest_menu/`; slice M5-S7, design change 0414, `core/game_rules.md >
Playtest menu (temporary)`): games only, not the editor, its play-tests or a playback (`build_app`). `PlaytestMenu` is
`Closed`, `Paused` (F10, `InputAction::PlaytestMenu`: Resume, Restart and Quit, under "PAUSED") or `GameEnd` (opened
when `Sim::outcome` is set: Restart and Quit, under the outcome's banner; F10 leaves it open). `use_menu` runs in
`PreUpdate` after `ReadActions`, so the pause (`SimControl::paused`, the designer's placeholder) holds from the frame
F10 is read, before that frame's `FixedUpdate`: nothing steps, the computer players don't think, and the command UI's
input set doesn't run (`menu_closed`). While it is open the menu holds the input (`input::Modal`, *Input*). Its
buttons are bevy_ui nodes in the HUD's colours (`playtest_menu/panel.rs`, a placeholder look); `MenuLayout` computes
their rectangles from the surface size alone, and a click is the `Select` press under the cursor, as the HUD's
headless-testable sections are. Quit writes `AppExit::Success`; there is no confirmation (the designer's placeholder).
**Restart** (`playtest_menu/restart.rs`) starts the game over in-process, as a fresh launch of it would (`Launch`: the
scenario with its setup and game options, the local player, the computer players and the recording as launched): a
new `Sim` of the scenario, the local player, the pending and recorded inputs, the pause, the camera's start view, zoom
and grab, the minimap's press, new computer players (`AiPlayers::start`) and a new recording (the old one written
complete, `Recorder::close`); the rest of a game's client state starts over by the new sim's generation (selections
and control groups, the players' cameras, the visuals and records, viewer events, combat effects, the Shroud's clouds,
indicators, notices, the fog's and the minimap's layers, the minimap's alerts), and the outcome screens follow the
new sim's `outcome`.
The client's own settings stay: F3, P, G, the pace, the window, the bindings. Tests: `tests/playtest_menu/` (the
pause, Quit, the restart against a fresh launch, AI against AI, and the client state), `tests/game_end/menu.rs`,
`tests/selection/menu.rs` (the input held).
