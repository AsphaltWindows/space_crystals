# Client architecture — Panel visuals, placement, Bevy code and client tests

Read this when you touch the panel's visuals, placement, Bevy code conventions, the client's tests or the generated docs.

- **Visuals**: the panel's buttons (`hud/command_panel.rs`) are rebuilt when the panel, the bindings or the surface
  changes; each shows its command's label (`PanelSlot::label`) and its hotkey from the bindings, and an EjectMenu tile
  its count in its bottom-right corner. **Every label keeps
  the full label size at 720p: six characters at most** (slice M3-K2a; a unit test measures every label of every
  panel on surfaces down to 800 × 600): what a button makes by its type's tag ("PK" trains a Peacekeeper, "GN" builds
  a Generator, "RP" warps in a Robo Portal, as `short_label` shows them on the map), "Place" for a ready building,
  "Cancel" for every Cancel, else the command's name, its first word or a shortening ("Raise", "Drop", "Rev.",
  "AtkGnd", "Adv."). **Never add a label longer than that**; its full name goes in `PanelCommand::name`. The
  InfoPanel names every button under the pointer in full, with its hotkey (`hud/info_panel/hint.rs`, `Panel::name`:
  "R: Raise Shield", "Q: Train Peacekeeper"; the client has no tooltips; slices M3-S and M3-K2a), after the
  ActiveGroup's type where the design names a command so (`PanelCommand::name_on`: a Beacon's "Warp In (Basic)",
  "Cancel Deployment" and "Cancel Warp In"; a Convert names its target form, "Convert to Rifle Legionnaire"), and with
  a Build's price, the sim's (`PanelSlot::price`: "W: Build Tunnel (1 Supply)", slice M4-J1). A label never wraps
  (`TextLayout::no_wrap`): one too long for its button at `LABEL_VH` (on a surface too small for it) is drawn smaller
  to fit on one line inside a lit button's thicker border (`label_px`, from the default font's fixed advance, 0.6 em,
  which a unit test checks against the font), so a label keeps its size when its button lights up. The indicators
  (`command_ui/indicators.rs`, the mesh in `indicators/draw.rs`) are
  one mesh in the overlays layer, rebuilt every frame while anything shows: markers at the targets of the selected
  own units' current orders (and queued ones while Shift is held), a flash where an order was just given
  (`feedback.flash_ticks`), and while Shift is held each unit's route through its waypoints as dashed lines; colours
  follow `systems/controls/hud.md > Color language` (an Attack Ground's marker, slice M4-O, is a Location in
  red, `systems/controls/hud.md > Indicator assignments`, and its route leg ends at its point; the D
  slot's AwaitingTarget gives the order on the ground only, `command_ui/resolution.rs`). **A marker or a route leg that names an object goes by what
  the local player knows of it, never by the live object** (`indicators/targets.rs`, `Sight::target_view`: the sim's
  `TargetView` through the client's `Sight`; design change 0139, slice M3-K2a): where the object is drawn while the
  player sees it, around its record (last-known position and footprint) while the player remembers it, wherever the
  object really is (a Beacon whose deployment was cancelled walks away as a Drone) and whether it still exists, and
  nothing while the player knows nothing of it (a unit out of sight). `marker_ring` is where a marker is drawn; the
  rally lines end at it. The selected own producers' rally points (`indicators/rally.rs`, design
  changes 0087, 0088) show a marker, green on the ground and around an own or neutral object, red around an enemy
  (the sim turns an enemy rally point it loses sight of into a ground one, so the marker turns green there), and a
  solid line from the footprint's edge to it; nothing while the producer isn't selected or has none; a portal rallied
  on a patch rings the patch in green (slice M3-J2). The InfoPanel's rally line names the target as the player knows
  it too (`activity::rally_line`). The gathering markers (`indicators/gathering.rs`, design change 0201 point 4,
  slice M3-J3; placeholder looks, each a shape and a colour of its own, `Indicator::Gathering` with a
  `GatheringMarker`) take the Move marker's place: the Gather marker, cyan corner brackets round a Gather's source;
  the Return Resources marker, a gold ring with four arrowheads pointing in, round a Drop Off's drop-off; and the Set
  Drop-off marker, a wider dashed violet ring round each selected worker's AssignedDropOff (`drop_off_indicators`), as
  a rally point shows while its producer is selected (a reading: Set Drop-off applies at once and gives no flash; the
  design gives the marker, not when it shows). The pointer
  (`command_ui/cursor.rs`) shows the `PointerLook`: images drawn in code and cached per look, set as the window's
  `CursorIcon`, and drawn as a UI node at the injected cursor in QA runs, which render no system cursor; Mixed is
  striped in the Attack red and the Move green (`MIXED_COLORS`, `MIXED_STRIPE_PX`), GatherResources is the crystals'
  cyan (`GATHER_COLOR`) and ReturnResources gold (`RETURN_COLOR`), the Gather and Return Resources markers' colours. The invalid-target feedback
  (`command_ui/feedback.rs`) is the message `INVALID_TARGET_TEXT` above the HUD Bar for `feedback.feedback_ticks` sim
  ticks after the latest `InvalidTarget` (`TargetFeedback`), never in the pointer's way (`Pickable::IGNORE`). The same
  band gives the insufficient-resources feedback (0083): `NO_CRYSTALS_TEXT` after the sim's `ProductionRefused` for
  a local player's producer that a Train didn't reach for want of crystals (a full queue gives none); the
  placeholder of design change 0143's notification, the army cap's name (`army_cap_text`: `UNIT_CONTROL_TEXT`,
  `BEACON_CAPACITY_TEXT`, `TUNNEL_SPACE_TEXT`), after the sim's `ProductionWaiting` for a local player's producer
  whose unit begins to wait for room (its InfoPanel shows "Waiting for Unit Control", "Waiting for Beacon Capacity"
  or "Waiting for Tunnel Space" while it does, `hud/info_panel/activity.rs`); a placement's (`placement_text`, the
  check's first problem, the Tunnel Area's three rules included): an
  `InvalidPlacement` or the sim's `PlacementRefused`; and `TARGET_TOO_CLOSE_TEXT` after the sim's `TargetTooClose`,
  or `LocationTooClose` (slice M5-O3), for a local player's unit (design change 0182).
- **Placement visuals** (`placement.rs`, `placement/`): while the local player places a building (`placing_now`: a
  game mode, AwaitingPlacement, a commandable selection). `PlacementGhost` holds the `Ghost` under the cursor (the
  footprint as turned, the `PlacementCheck` there now, the side letters of `sc_sim::sides`), worked out every frame;
  its one mesh is rebuilt in place, under one handle, only when that changed: each tile on the terrain's surface,
  green or red by its own problems, red throughout when the footprint breaks a whole rule (0093), an outline
  green or red for the verdict, and an arrow at each doorway (0390, `door_arrows.rs`: `door_arrows` from the type's
  defs and its sides as placed, in at `HoldsDef::entrance`, out at each exit side of `exit_rule`, two-way where they
  are one door, out of each open side of a portal, in and out at a Tunnel's two paths, `network::Paths`, in at an
  Airstrip's B and out at its C along the runway, a placeholder reading, slice M5-N7; the same arrows show on the
  selected buildings, `selection/doors.rs`, 0394); five `Text2d` labels carry the side letters
  (0079) and the building's tag, never
  hidden and never culled, only emptied (Bevy lays a `Text2d` out only while a camera saw a sprite or a text the frame
  before, and drops a change made in a frame without one: a text that changes must keep one in view). It lies
  over the map's other overlays, under the indicators: it is the cursor, so no pointer shows over the map
  (`PointerLook::placing`). `BuildAreaOverlay` shows `Sim::build_area`, the area only, as one mesh in the terrain's
  z band just in front of the fog layer (`overlay::AREA_Z`), but on the tiles the local player hasn't explored, which
  stay black (slice M3-J3, `BuildAreaOverlay::unexplored`), rebuilt only when the area differs from the one it last
  drew (`BuildArea: PartialEq`), its unexplored tiles changed or the map meshes were rebuilt (`MapView::builds`). The ghost's check is its
  builder's (`ghost::check_now`: `Sim::placement_check`, `warp_in_check`, `deploy_check` for the Drone a click
  would go to, or `build_check` for an Agent's Build; the last two by the player's knowledge, Reserved over a
  lingering ghost included); the overlay shows for a constructor's placement only. A Tunnel ghost (slice M4-J1) has
  its Tier 1 Tunnel Area drawn around it (`GhostArea`, `placement/ghost_area.rs`: `ShownArea::of`), green or red by
  the check's Tunnel Area problems alone (`area_is_clear`: the `WholeProblem::TunnelArea*`), on its explored tiles,
  just in front of the area overlays and behind the lingering ghosts (`GHOST_AREA_Z`). `LingeringGhosts`
  (`placement/lingering.rs`, `core/entities.md > Placement Validation > Lingering build ghosts`) draws the local
  player's pending worker builds (`Sim::pending_builds`, never another player's: Deploys, and an Agent's Tunnels with
  their Tier 1 areas, `lingering_sites`) in the game modes, whatever the interface: each area tinted on its explored
  tiles with its outline, then each site's footprint tinted on the terrain with an outline, one mesh rebuilt only when
  the sites, the areas' unexplored tiles or the map change, just in front of the build area overlay (`LINGERING_Z`).
  A ghost goes with its order, when
  the deployment or the site starts or the order is cancelled, replaced or fails. **The Tunnel Area display**
  (`TunnelAreaOverlay`, `placement/tunnel_areas.rs`; slices M4-J2 and M4-J3, `factions/syndicate/tunnels.md > Tunnel
  Area display`, design changes 0299 and 0300) shows, in the game modes, two things and never an enemy's area
  (neither seen nor remembered; the ghost still judges by the known ones, red with its reason), in:
  - **the selected areas** (`selected_areas`, a reading): the area of each own Tunnel in the selection, sites
    included, at its tier now (`tunnel::area::of`: an upgrading Tunnel's current area, not the one it reserves), in
    violet; it hides as the selection changes. An expansion's placement is its Tunnel's, selected all along (M4-F);
  - **a Tunnel ghost's rings** (`ghost_rings`, `tunnel_areas/rings.rs`): for each tier of the kind's table above the
    base, the cells its area adds to the tier below's (`Ring`: Tier 2's 36, Tier 3's 44), each ring in its tint.
    Red (`Ring::blocked`) are the cells that would block the upgrade (0242): off the map, not Drillable as far as the
    player knows (the sim's vision: an unexplored tile passes, as on the ghost, `Mode::Known`), or in an own area as
    the ghost's check counts them (`rings::own_areas`, a reading: `tunnel::area::reserved` of every own Tunnel and
    site, the upgrade's target tier's, and the Tier 1 areas of the own pending Tunnel ghosts but the placing Agent's
    without Shift, as `Sim::build_check` leaves them). Off-map cells can't be seen (the camera keeps to the map), so a
    red line along the map's edge marks where a ring runs off it (`rings::edge_marks`, a presentational reading).
    The Tier 1 area is the ghost's own (`GhostArea`).
  - **one mesh** under one handle, with the build area overlay's look (`overlay::tint_area`) and z, but on unexplored
    tiles and off the map, rebuilt only when the areas, the rings (as the ghost moves to another tile: it runs after
    `ghost::update_ghost`), their unexplored tiles or the map changed (`TunnelAreaOverlay::builds`); no view-mode
    toggles.

  **What the area displays share** (slice M5-J2, `placement/area_overlay.rs`), written once: the entity of one
  blended mesh under a reserved handle in the terrain's z band (`spawn_area_mesh`), its state (`AreaMesh`: rebuilt
  only when `is_stale`, what it shows or the map changed; `drawn` for its visibility, `show`), the unexplored tiles left
  out (`unexplored`), and the selection's own objects (`selected_own`). The tiles are drawn by `overlay::tint_tiles`
  (a tint per tile) and `overlay::outline` (an edge where the area ends), which `overlay::tint_area` combines. A new
  area display uses them rather than a copy. **The Recruitment Area display** (`RecruitmentAreaOverlay`,
  `placement/recruitment_areas.rs`; `factions/cults/structures/recruitment_hall.md > Recruitment Area display`,
  design changes 0350, 0351 and 0352, answering Q0107) shows, in the game modes, never an enemy's area:
  - **the preview of a Hall placed** (`ghost_preview`, the sim's `Preview::of`, after `ghost::update_ghost`): its 10 ×
    10 area tinted tile by tile by what the preview finds (`fill_of`) within its outline, on the map's tiles only (a
    Hall may stand right at the edge, design change 0387: its tiles off the map are not Recruitable, counted among the
    100, neither tinted nor outlined), and its Effectiveness as a `Text2d` north of the ghost (`EffectivenessLabel`,
    always visible and emptied rather than hidden, as the ghost's letters; "Effectiveness: 60%"), or south of it where
    north would be off the map and out of the view (`label_ground`, slice M5-S4, a placeholder). The claimants (`known_claimants`) are what the local player knows, never the hidden
    truth: their own completed Halls; the other players' completed Halls they see (`Sight::sees`) or remember
    (`Sight::remembered`, as last seen: a Hall destroyed unseen counts until its place is seen, one never seen doesn't,
    a site claims nothing), ranked as completed at tick 0 (Placeholder (not in the design: what the player knows holds
    no completion tick)); their own sites by id, then their pending Hall Builds by `PendingBuild::group` (Placeholder
    (not in the design: the sim keeps no placement time for a site, so the sites rank first)). Without Shift, the
    ghosts the placing Recruits alone hold (`command_ui::interface::recipients`) don't count, as in
    `Sim::group_build_check`;
  - **a selected own Hall's claims** (`selected_claims`): the tiles it claims in the game (`Sim::claimed_tiles`, what
    its InfoPanel's Effectiveness counts), in a selected Tunnel's violet; a site claims none. Its **true** claims, even
    where an unseen enemy Hall has taken tiles (design change 0373, answering Q0133: "Unlike the placement preview, it
    may reveal where an unseen enemy Hall took tiles");
  - its colours and z are its own constants in one place (`RECRUITMENT_AREA_Z`, just in front of the Tunnel Area
    overlay), its look the Tunnel display's.
- The design leaves some things open (labels, colours, the flash, the cursor's shapes and the Mixed look, the
  invalid-target and insufficient-resources feedbacks' form and duration, the targeting click allowance's value, when
  Shift counts for a targeting click, Shift with a rally point; the ghost's and the overlay's looks, the placement
  messages, the orientation a placement starts with, the hidden slots of buildings not yet in the game; the Warp In
  menus' labels, the lingering ghosts' look, a greyed-out button's messages); each placeholder is marked where it
  is made.

**Bevy code**
- Write one plugin per feature module (camera, map rendering, HUD, input, …). Keep systems small. Bevy caps a system
  at 16 parameters, and the prototype's UI systems hit that cap.
- Keep any state a test must set or inspect in a `Resource`, not a `Local`.
- Send one-shot happenings as sim events or Bevy messages, never as marker components that are added and removed.

**Client tests** advance an exact number of ticks deterministically, never by wall-clock time. Insert
`TimeUpdateStrategy::FixedTimesteps(1)`, so that each `App::update()` runs FixedUpdate exactly once and real time
advances by one tick. The very first update only starts the clock, so assert on `Sim::tick()`, not on the number of
updates. To draw between ticks, advance by part of a tick with `TimeUpdateStrategy::ManualDuration`. A client run's
`LoadedScenario` (whose setup holds the game options) and `RecordedInputs` are a `Replay`: playing it back in a bare
`Sim` gives the same `state_hash` at every tick. `tests/replay.rs` records games through the client and plays them
back (`AppConfig::record`, `AppConfig::playback`).

**Generated docs** (slice M5-Ma, `tests/playtest_docs/`): two pages of the playtest kit are generated from the code
between marker lines, `<!-- generated: <name>, begin; ... -->` and `<!-- generated: <name>, end -->`, the text
around them written by hand. `docs/playtest/controls.md` is every key of the bindings table and every faction's
panels, as `command_ui::interface::panel` builds them from the templates; its test fails when the page differs,
naming the first line that does and the command that regenerates it, so a change to a template, a command's name or
hint, a prerequisite or a binding regenerates it and commits it with the change. `docs/playtest/placeholders.md` lists
every `Placeholder (` mark in the comments of `crates/` (*Tunables*), by faction and system; nothing fails when it is
stale, and the milestone's QA regenerates it. The tests read `UPDATE_DOCS`, as the goldens read `SC_BLESS`:

    UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::

rewrites both (`playtest_docs::controls` or `playtest_docs::placeholders` one).
