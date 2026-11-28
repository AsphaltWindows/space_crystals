# Client architecture — Kinds, sim driver and visuals

Read this when you touch the client's skeleton: kinds, the sim driver, drawing objects, terrain, effects, fog or viewer events.

## Client architecture

M0 builds this skeleton, so follow it. The art direction is 2.5D pixel-art sprites in the style of StarCraft 1 and
Age of Empires 2, with the details still open in Q0001. Until that art exists, everything is a placeholder shape laid
out by the same rules.

**Kinds** (`kinds.rs`, slice M4-R2): what the client knows of each object kind beside its def, one module per kind
(`kinds/<faction>/<object>.rs`, named as the design's file of the object is: `kinds/gdo/peacekeeper.rs` holds both
the Peacekeeper's types, `kinds/map/` the map's objects), behind one registry, `kinds::of`, an exhaustive `match`
with one line per kind. A `ClientKind` holds the kind's label (`kinds::short_label`; a Tunnel's names its tier,
`kinds::object_label`), its look (`object_render::Look`: a unit's body shape and whether it carries its label, a
structure, or a map resource's own shape), its shots' style (`combat_effects::ShotStyle`, for a kind with a weapon)
and its panel template (`command_ui::interface::templates::PanelTemplate`). Its name and numbers are the def's. The
drawing code stays in `object_render/` and `combat_effects/`, and nothing else in the client matches on kinds for
these.

**Sim driver**
- The `Sim` lives in a Bevy `Resource`. That is a client newtype, because `sc_sim` can't derive Bevy traits. The sim
  is stepped only in `FixedUpdate`, whose timestep comes from `sc_sim::TICKS_PER_SECOND` (`Time::<Fixed>::from_hz`).
  One FixedUpdate run is one tick. Nothing else steps the sim, and no game logic reads frame time.
- Player input becomes `PlayerCommand`s in a pending-commands `Resource`. The stepping system drains it into that
  tick's `TickInputs`, which is what a replay records.
- Read `Sim::events()` in the same system, right after `step`, and forward them, for example as Bevy `Message`s. The
  next step clears them, and FixedUpdate can run several times in one frame.
- `SimControl` says whether the step may run: `paused`, `step_once` (one step while paused, then it clears itself)
  and QA's `hold_at`; never once the game is over (`SimControl::may_step`, `Sim::outcome`: "the game stops",
  `core/game_rules.md > End of Game`), so a finished game stands at its last tick. `Pace` says how fast, changeable
  at any time: `Scaled(f)` runs `f` × 16 ticks per second of real time through `Time<Virtual>`'s relative speed,
  `Max` (`--fast`) as many ticks per frame as fit in `FAST_FRAME_BUDGET`, drawn exactly at their ticks. A change
  applies in `PostUpdate` (`apply_pace`), so the next frame's time update already goes at the new pace, and at a
  scaled pace the fixed loop never owes more than a frame's worth (`limit_fixed_loop`: 250 ms of scaled time), so
  leaving `Max` runs no burst of ticks. A replay's playback controls both (see Replays and golden hashes).

**Visuals**
- The sim is the source of truth. Visuals are separate entities, each linked to its object by an
  `ObjectVisual(ObjectId)` component. Their `Transform`s are derived from the sim every frame and are never read back
  for gameplay.
- Create and despawn visuals from sim events, or by diffing ids against `Sim::objects()`. Build the initial set from
  `Sim::objects()`, because scenario spawns emit no events.
- **Interpolation**: draw each object at `prev_pos.lerp(pos, a)`, where `a` is `Time<Fixed>::overstep_fraction()`
  read in `Update`. Interpolate facing along the shortest arc.
- **Coordinates** (`coords.rs`): 1 world unit is 1 su, in three spaces. **Ground** is the sim's (+y south).
  **View** is where a ground point is drawn: `drawn_position(map, ground)` shifts it up the screen (north) by
  `lift(height) = height × HEIGHT_OFFSET_PER_LEVEL` GU (`core/art_direction.md > Elevation Display`; 0.25 GU per
  level, a placeholder until art tuning). **Bevy world** is view space with **`bevy_y = -view_y`** (`view_to_world`),
  because Bevy's +y points up. Convert only through these helpers, never ad hoc. A sim facing θ is a Bevy rotation of
  −θ about z.
- **Picking**: every screen → world conversion ends in `coords::ground_under(map, view)`, the inverse of
  `drawn_position`: the ground point drawn at that view point. Where terrain overlaps itself on screen (a cliff or a
  hill in front of the ground behind it), the frontmost, i.e. southernmost, surface wins. It solves column by column
  (at a fixed x the drawn y is linear across a tile row) from south to north; beyond the map's edges the ground
  continues flat. The input layer's `Pointer::ground` is the cursor's ground point (`camera::cursor_to_ground`):
  hover, clicks, the editor's brush, ghosts, markers and object picking all use it.
- **Draw order** follows StarCraft 1. The layers (`coords::DrawLayer`) are terrain < ground decals < ground objects
  < unit decals < ground units < air < overlays, each with its own z band. Within a layer, z grows with **ground** y,
  so southern objects draw in front. Each sprite is anchored at its ground position, the sim `pos`, and that point
  drives its y-sort; it is drawn at its height (`ground_translation`; a tile-based object at its footprint's mean
  corner height, `footprint_height`). The tile-based objects (structures, map resources, the records of remembered
  ones) are ground objects, sorted among themselves (a plate over its patch: `structures::BASE_Z`); units are drawn
  above them whatever the depths (`object_render::draw_layer`, slice M3-U), so a unit crossing a footprint, as one
  leaving a portal's energy doorway does, is never hidden under it (`tests/selection/draw_order.rs`). Selection rings
  are unit decals, between the two; frames lie with their structures; ghosts, HP bars, path and command indicators are
  overlays, above everything. So are the units' tags, just above the command indicators and still y-sorted by their
  units (`units::tag_z`, slice M3-J3), so that no marker on a unit hides its label; a structure's tag stays with its
  structure, under the units that cross it.
- **Air units fly** (`object_render/flight.rs`; slice M5-H1, plan decision T81: "air units are drawn raised, with a
  ground shadow, at their true position, even past the edge"; placeholders until Q0001): an air unit's body is in the
  air layer, drawn `look.flight_lift_levels` (3, 48 su) above the ground under it, and its `FlightShadow` child, a dark
  ellipse, lies on that ground in the unit decals' band, its transform undoing the body's turn. Both are at the sim's
  `pos`, past the map's edge too, where the sim counts the unit as at the edge (`Object::map_pos`). **Whatever is drawn
  on a unit asks `object_render::body_height`**, the ground's height plus the flight's: its picking box and selection
  ring (`Selectable::of`, so a click picks the raised body, in front of a ground unit there, and not the shadow), its
  HP bar, an order's marker on it (`indicators::targets::object_box`) and its shots' ends (`Tracer::lift`). (M5-N11)
  **On its takeoff run** (`Mover::takeoff`, a Sparrow out of the Airstrip; design change 0428: "it appears on the
  runway's B end and rolls into the air toward C") its lift is `object_lift`'s: none until `look.lift_off_su` (128 su, a
  placeholder) before the run's end, then rising evenly to the whole flight at the end, the shadow following
  (`place_shadows`); its shots' ends and a burst on it stay at the flight's height (`flight_lift`, a placeholder).
  (M6-GH2) **Landing on an Airfield's slot** (`sc_sim::landing::lift`) it sinks evenly by the sim's ticks to the
  ground, its shadow under it; **down**, it is drawn on the ground and its shadow hidden (`flight::shadow_shown`); its
  take-off is a rise, drawn as one out of an air door is (`network::door_lift`).
- **Terrain** (`map_render.rs`): one mesh. A flat tile is a quad, a sloped one four triangles around its center (at
  the corners' mean height), each shaded by Lambert's law towards `LIGHT_TOWARDS` (a placeholder: from the north-west,
  45° up), so flat ground keeps its colour. A non-Traversible tile at least `CLIFF_MIN_SLOPE` levels steep reads as a
  cliff face: rock-coloured, with a stratum line at every level. Every vertex's z grows with its ground y, so with the
  depth test southern terrain overdraws the terrain behind it. The grid (G) follows the surface in the terrain's z
  band, slightly in front, so terrain in front hides it too. Both meshes are rebuilt when the sim's map changes.
- **Units** (`object_render.rs`, `object_render/units.rs`; the systems in `object_render/systems.rs`) are placeholder
  shapes until the art (Q0001): a shape per kind, its look's (*Kinds*: the Peacekeeper round, the Guard and the
  Legionnaires square, the Drone an octagon), of the silhouette's size,
  in the owner's colour (`player_color`, the editor's slot colours), with a dark rim and a light tick along the
  facing. Only a round body turns with the interpolated facing (`body_turns`), and (slice M5-V1) a body whose
  Silhouette turns with the facing as its collision shape does (`ObjectDef::silhouette_turns`: the Cults' vehicles,
  their rectangles); such a unit is picked by the box around it as it lies (`selection::pick`). Any other square or
  octagon stays axis-aligned like its collision box, and only its tick turns (a `FacingTick` child, `has_tick`). Structures and map
  resources never turn, whatever their facing (a structure keeps a unit's facing through `Sim::change_kind`). The
  Colonists' units, the Peacekeeper's two types, the Agent and the Grenadier carry a `UnitTag` child, a `Text2d` of
  the type's short label (DR, SL, RL, PK, PS, AG, GR), because the Legionnaire's two forms share a silhouette, the
  Peacekeeper's two types a body (slice M3-J3), and the Agent and the Grenadier the Guard's square (slices M4-A and
  M4-U). A tag's local z lifts it into the
  overlays band (`units::tag_z`, see Draw order), and it stays
  upright on a body that turns: `units::upright_tags`, chained after `place_object_visuals`, gives it the inverse of
  its visual's rotation. Every type's short label (`kinds::short_label`) is its own, and none is a
  planned unit's or building's button label (`Planned::label`: the Scout's is "ST", the patch's "SC"); a unit test
  checks it. One mesh per `MeshKey` (kind, owner, a structure's size as placed, whether it is transforming, and
  (slice M6-CM) whether it is an illusion of the local player's, which its owner alone sees washed out towards a
  pale lilac, `illusion_color`, a placeholder; any other player's looks a Disciple's) is shared, and one tick mesh
  per kind. A visual whose object is drawn otherwise now
  (its `MeshKey` changed: a kind changed in place, `SimEvent::KindChanged`; a transformation began or ended; a
  defeated player's unit turned neutral) is despawned with its children and made anew (`ObjectVisuals` keeps the
  key each was made from). A transforming object, a building being warped in, is tinted half way to a pale cyan
  (`transforming_color`, a placeholder until the art). The map resources' shapes are in
  `object_render/resources.rs`. A unit is drawn as the form it has (`object_render::drawn_kind`, *Object model >
  Type and form*): a Legionnaire whose conversion is pending keeps its old form's look and tag, its selection group
  its new type's. A unit that carries a Riot Shield has a `RiotShieldVisual` child: a pale arc in
  front of it, hidden while the shield is down (a deferred raise's too), growing to its full size as it goes up. It is `Visibility::Hidden` or
  `Inherited`, never `Visible`, so the fog that hides its unit hides it too. A worker has a `LoadMarker` child per
  resource it gathers (`object_render/load.rs`, `load::carried`; slices M3-J2 and M4-J1, placeholders): a crystal
  shard in the crystals' cyan with a dark rim at the top right of its body, and on an Agent an amber supply crate
  there too, each shown the same way while it carries a load of its resource (`carries_of`), to every player who sees
  it.
- **Structures** (`object_render/structures.rs`) are placeholders too: the footprint as placed, in the owner's colour
  inside a dark rim, drawn `INSET_SU` (8) inside its edge so that a selection frame shows around it, and over a map
  resource's placeholder (a plate covers its patch). A `StructureTag` child, a `Text2d` of the type's short label
  (`kinds::short_label`), names it; the fog hides it with its structure. A remembered structure
  (`remembered.rs`) looks the same, in its owner's colour, dimmed, its tag too. **Underground structures** (slice
  M4-A, the Hideout; `structures::underground`) are translucent outlines, a rim and a faint fill in the alpha-blended
  `underground` material, under the units (`artifacts/director/m4-plan.md` §3.3); the sim hides them from other
  players (*Vision > Hidden objects*), so only their owner and the fog-less views draw them. **The Tunnel's look**
  (`object_render/tunnel.rs`, design changes 0224, 0248 and 0251) follows its tier and its sides as placed
  (`TunnelLook`, part of the `MeshKey`): Side A's two paths, ragged at Tier 1 and paved from Tier 2, a hatch on Sides
  B (crystals' cyan) and D (Supplies' amber), none on C, and the Tier 3 air doors over the paths' halves, where the
  sim has them and its size (`Paths::air_exit`, `Paths::air_entry`, `Tunnel::air_door_radius_su`); its tag
  names its tier (T1 to T3, `kinds::object_label`), to enemies too (design change 0258), a record's the tier last seen.
  **A site** (`Object::is_site`, slice M4-J1; `object_render/site.rs`, a placeholder) is drawn by being one, whatever
  its kind (`MeshKey::site`), on the surface or underground (an expansion's outline, slice M4-F): its footprint tinted
  half way to a pale sand and braced at its corners by a scaffold (clear of its tag) instead of its finished look,
  with a progress bar near its south edge (lower and thinner on a roof one tile high, a Laser Turret's, clear of its
  tag), a dark track in its mesh and a `SiteProgress` child stretched to its share done (`place_site_progress`,
  `Progress::ratio`), to everyone who sees it. A record of one is drawn as a site too (`MeshKey::of_record`, slice
  M4-I2), dimmed, its bar fixed at the progress last seen (`LastKnown::site`, `SiteProgress::remembered`, which
  `place_site_progress` leaves alone): the site may have grown or been finished since. **A runway** (`Look::Runway`,
  `object_render/runway.rs`; the Airstrip, slice M5-N7, a placeholder) is a structure's footprint with a dark strip
  down its longer axis, an end bar at each end and a dashed centre line clear of the tag, drawn alike at both ends
  (which is B and which C, the door arrows show). **A structure that attacks**
  (`Look::Turret`, `object_render/turret.rs`; the Laser Turret, slice M4-S1, a placeholder) is a structure's footprint
  with a round head on its roof and, once finished, a `TurretBarrel` child turned by the object's facing
  (`place_barrels`, interpolated as a unit's facing tick), which the sim snaps to what it fires at; a record of one
  seen raised keeps the head and no barrel. **Always a pop-up** (slices M4-S2 and M5-Y, placeholders; design change
  0360): fully lowered (`MeshKey::lowered`, `popup::is_lowered`) it is an underground structure's outline in the
  blending material, with neither head nor barrel, so its owner sees the ground and the units over it and `Sight`
  shows an owned one to nobody else; from its first rising frame it is the raised body, its barrel facing its target
  and growing to its length with `PopUp::height`, and shrinking as it retracts. A neutral one, down for good, shows to
  everyone as its outline in the neutral colour, and so does its record (`MeshKey::of_record`,
  `popup::remembered_lowered`), blended as an underground structure's record is (`object_render/remembered.rs`).
  **A unit with a turret** (slice M5-U1) carries the same `TurretBarrel` on its body, with no head, turned to its
  turret's facing (`sc_sim::combat::turret_facing`); the sim keeps no turret facing of the tick before, so the
  barrel's `TurretTrack` remembers the last two ticks it saw and interpolates between them, and on a round body,
  which turns with the unit, the barrel turns by the turret's facing less the body's.
- **Path overlay** (`path_overlay.rs`, P or `--show-paths`): each unit's remaining path (`Mover::remaining_path`) as
  a thin line from where the unit is drawn, cut into short pieces lifted by the terrain's height, in the overlays
  layer, and a frame around each unit in the colour of its move's status (`status_color`: green moving, amber held
  up, red failed, blue arrived; placeholder colours). Its one mesh is rebuilt every frame while it is shown.
- **Combat effects** (`combat_effects.rs`, its mesh in `combat_effects/draw.rs`; placeholders until Q0001): a tracer for
  each `SimEvent::Attacked` and each `SimEvent::Missed` (`feedback.tracer_ticks`; a TailDisjointed shot's, the Gunner's,
  to the spot it locked, hit or miss, `Tracer::target` none for a miss: slice M5-O3), drawn only along the parts of its
  path over the local player's Visible tiles (`shot_spans`, through `visible_spans`, which cuts it at every tile edge it
  crosses; design change 0177: a shot from the fog shows from the fog edge, one all in the fog not at all; its attacker
  is revealed by the sim), **with no exception** (design change 0293, answering Q0083 point 4: every shot, tracer or
  projectile, whoever fires it and at whatever, a revealed attacker's at the player's own objects too; it replaced
  design changes 0206 and 0244, Q0056 and Q0075, which drew those whole, from the attacker to the target); a
  tracer's look is its weapon's (`ShotStyle::of` on the form that fired, its kind's `ClientKind::shot`; design change
  0207: the Syndicate's lasers, the Guard's and the Laser Turret's, red), HP bars over damaged and selected objects (a
  unit's clear of its silhouette's corners, a structure's along the top edge of its footprint and as wide as it), with a
  shield bar right above for an object that has Shields (`systems/combat.md > Shields > Display`, design change 0148;
  `SHIELD_FILL`, a placeholder colour), shown too while only its Shields are down, and (slice M5-N4a) an energy bar
  right under it for an object that has an energy pool (`HpBar::energy`, from `sc_sim::energy::of`; `ENERGY_FILL`, a
  placeholder violet), shown with the HP bar alone, energy short of its maximum being no damage (Placeholder),
  (slice M6-GH1) an aircraft's ammo as pips under its bars, one square per round of its type's max, filled amber for
  each it has left (`HpBar::ammo`, from `sc_sim::aircraft`; `AMMO_FILL`, a placeholder look), shown with the HP bar,
  (slice M6-SB) a unit's tanks as thin bars under those, as wide as its HP bar, sludge's murky yellow and smoke's grey,
  filled for what is left, lighter while its toggle is on (`HpBar::tanks`, from `sc_sim::patches`; `TANK_FILLS`, a
  placeholder look), shown with the HP bar, (slice M6-GS1) a Supply Chopper's carried Supplies as teal pips under those,
  one square per Supply, at most `SUPPLY_PIPS` (8; `HpBar::supplies`, from `sc_sim::supply_run::carried`; `SUPPLY_FILL`,
  a placeholder look), shown with the HP bar while it carries any, (slice M6-ST1) a carrier's places as pips over its
  bars, above its shield bar if any, one square per place of its capacity, filled pale for each its passengers take
  (`HpBar::passengers`, from `sc_sim::carry::def_of` and `Sim::room`; `PASSENGER_FILL`, a placeholder look), shown with
  the HP bar, to its owner alone with the fog on (Placeholder),
  (slice M5-N5) a mark of each buff a unit the local player sees has, a cross in a dark rim right of its HP bar's
  place, whether the bar shows or not, green for Health regeneration and violet for Energy regeneration, in
  `Object::buffs`' order, their `BuffKind`s' (`combat_effects/buffs.rs`, `buff_marks`; who sees a buff and its look are
  Placeholders, Q0001; (M5-N11) Inspire's a gold chevron, Suppress's a red X; (M6-SN) the Null Chamber exit buff's a
  pale blue diamond; (M6-SB) the Sludge's slow a murky green chevron, downward), (slice M5-N11) where a cast round its
  caster or ahead of it was seen, for `feedback.cast_ticks` (16), Inspire's gold ring of its radius over a faint disc
  and Suppress's red wedge along `SimEvent::Cast::facing`, fading (`combat_effects/casts.rs`, `CastFlash`; a placeholder
  look; under the blasts, as each shows only while its point's tile is Visible), and a fading marker for each
  `SimEvent::Died` and (slice M5-V4) `SimEvent::Crushed` (`feedback.death_marker_ticks`; a crush has no look of its own:
  Placeholder (not in the design: no look of its own)), never for an object that was hidden from the local player
  (`shows_death`, the `Hiding` its death carries; plan decision T52: "nothing is drawn for a hidden victim", as a
  Hideout under a blast; no hit is drawn for any victim). Each effect keeps its event's `Sighting` (*Viewer events*):
  the drawing code only cuts a path to the Visible tiles. Slice M4-O adds the projectiles in flight
  (`combat_effects/projectiles.rs`, `Flight`, from `SimEvent::ProjectileLaunched`): each along the straight path to
  its locked point at the pace of the ticks, lifted by a cosmetic arc (`ARC`) over its shadow, shown while its point
  on the ground is over a Visible tile (`shows_flight`; `core/vision.md > Attacks from outside vision`), whoever
  threw it (design change 0293), in its thrower's style as a tracer is in its shooter's (`Flight::weapon`,
  `ShotStyle::of`; plan decision T52: the Grenadier's `ShotStyle::Grenade`, a grenade with its fuse lit, slice M4-U;
  slice M5-K: the Rocketman's `ShotStyle::Missile`, a small body in its owner's colour with a flame and a nose, flying
  flat from the muzzle to its target's height, an aircraft's in the air, `Flight::lift`), the homing missiles drawn
  from the sim's store where the sim steers them (`homing_missiles`, while their point on the ground is Visible), (slice
  M5-K) the reticle of each Target Lock of the local player's units round its target while they see it
  (`combat_effects/locks.rs`, `lock_marks`; who sees it and its look are Placeholders, Q0001): an amber arc that fills
  clockwise over LockOnDuration outside the order's marker, four ticks closing in and a faint aiming line, a red ring
  once locked; and the
  blasts (`combat_effects/blasts.rs`, from `SimEvent::Blast`, `feedback.blast_ticks`): a ring the size of the area,
  shown while the tile under its centre is Visible (`shows_blast`), (slice M5-S8) a missile's burst the same at its
  radius of 32 su, its ring in its shooter's colour (`blasts::ring_color`, by `Blast::weapon` and `Blast::owner`, from
  the landed flight or `SimEvent::Blast::attacker`) and in the air over an aircraft it was fired at (`Blast::lift`), and
  (slice M5-K) a single-target one's, of radius 0, a small puff at the height it struck (none of the game's types
  fires one now). One alpha-blended mesh in the overlays layer, rebuilt every
  frame while
  anything shows; effects age by the sim's tick and the render alpha, so screenshots stay deterministic.
- **Shroud's clouds** (`shroud_view.rs`, slice M5-N4b; `factions/cults/units/disciple.md > Spells > Shroud`: "the
  cloud is shown to **every player**, over the tiles that are Visible to them, so its edge can be seen even when the
  Disciple can't"): a pale haze over each cloud's circles, one haze where they overlap, and a release's flash
  (`SimEvent::ShroudReleased`) over the circles it went over for `look.release_ticks`, fading; both only over the local
  player's Visible tiles (`shroud_view::texels`), so an enemy who sees a cloud's edge sees that edge. One mesh over the
  box of tiles the clouds and the flashes cover (`shroud_view::Area`), on the terrain's surface as the fog layer's
  is, under the air units and over the ground units (Shroud protects the ground), and one texture of
  `TEXELS_PER_TILE` texels to a tile's edge, each as opaque as the circles cover it, so the edges are smooth. The
  mesh is rebuilt when the box changes, the texture written in place when what it shows changes, and a new texture
  made only when the box's size changes (*Bevy 0.20: verify, don't recall*); hidden, with nothing computed, while no
  cloud or flash shows. (Slice M6-CM) A lingering circle is drawn as any, and **the channelling stance**
  (`shroud_view/stance.rs`) marks each unit that stands channelling (`sc_sim::shroud::stands_channelling`: a Disciple
  channelling, and its illusions mirroring it, which have no cloud) with a ring of its own at its feet, to every
  player who sees the unit, the two alike, so that only the cloud tells them apart (0448: "so a watching enemy can
  tell which one is real while it channels"; a placeholder for the animation, Q0001). Colours
  and lengths are placeholders (Q0001). Tests: `shroud_view/tests.rs` (the texels, the stance's rings),
  `tests/combat/shroud.rs` (each player's Visible tiles, the release's flash, (M6-CM) the circle lingering where it
  was after a Stop).
- **The sludge trails** (`patches_view.rs`, slice M6-SB; `factions/syndicate/units/shadebringer.md > Sludge
  (toggle)`): each sludge patch a murky disc as wide as it, with a darker rim, on the ground under the units
  (`DrawLayer::GroundDecals`, as its mesh's transform's z: alpha blending orders the blended meshes by theirs, so the
  effects' HP bars and buffs' marks come over it), at its centre's height (`drawn_at_height`), shown while the tile
  of its centre is Visible to the local player (`patches_view::sludge_discs`; with the fog off, every one), shrinking
  to 0.4 of its radius over its last `look.sludge_fade_ticks` (32), by the sim's ticks and the render alpha
  (`patches_view::shown_share`). One mesh, all the rims then all the bodies, rebuilt every frame while a patch shows
  and hidden while none does. Its look is a placeholder (Q0001); slice M6-SC's smoke needs its own. Tests:
  `patches_view/tests.rs`, `tests/commands/sludge.rs` (with the slow's mark on another player's unit).
- **Fog of war** (`fog.rs`, `core/vision.md`): `Sight` (`fog/sight.rs`) says what the local player is shown: its own
  objects, and everything on its Visible tiles (`Sight::sees`); on Explored tiles the terrain and the player's records
  of the structures and resources it saw there, in their last-known state, never the live objects (`Sight::remembered`,
  see Vision); on Unexplored tiles nothing (a map object known from the start, design change 0159, shows once a tile
  under it is Explored). Object visuals it doesn't see get `Visibility::Hidden`; the records get
  visuals of their own (`object_render/remembered.rs`, `RememberedVisuals`: one entity per record shown, dimmed
  like the Explored ground, a placeholder; synced when the sight changes). Tracers, death markers, HP bars,
  projectiles and blasts, the
  path overlay, the selection (`selectable_by`) and with it what a click or the pointer targets, the InfoPanel and
  the command indicators ask it too. The fog layer (`fog/layer.rs`) is one mesh that follows the terrain's surface tile
  by tile, in the terrain's z band `FOG_Z` in front of it (so no cliff stratum shows through), and one texture of
  one black texel per tile (`UNEXPLORED_ALPHA`, `EXPLORED_ALPHA`, clear when Visible), sampled linearly. The texture
  is written only when `Sight::revision`, the local player or the sim changes, the mesh only with the map, and a new
  texture, which the material is pointed at, is made only when the map's size changes (slice M5-Z3; *Bevy 0.20:
  verify, don't recall*): from the 1 × 1 stand-in in the first frame the fog applies after frames it didn't (an editor
  play-test, F3 after `--no-fog`), and between play-tests of maps of different sizes.
  A `Sight` looks through the union of the visions it watches (`Sight::watching`): a tile in the best state any of
  them gives it, an object out of sight of all of them as the latest record any of them keeps. A player watches
  their own vision, and a defeated one their allies' too (`Sight::new`, `Sim::allies`; design change 0102: never a
  full map reveal); `Sight::revision` holds whom it watches. Without teams a defeated player watches their own
  vision alone, which has no Visible tile left once the fog delay of their last tiles is over.
  `FogOfWar` turns it off for QA (`--no-fog`, the debug key F3, which a replay's playback takes without debug
  commands too): display only, the sim computes every player's
  vision anyway; the top bar says so meanwhile, "Fog off (F3)" (slice M5-S6, `TopBarNote::FogOff`, below). The editor
  shows no fog; its play-tests do.
- **Viewer events** (`viewer_events.rs`, slice M4-V): **one filter between the sim's events and every view of them.**
  The sim tells every event to everyone. `filter_events` (in `SyncSim`, set `FilterEvents`) alone reads that stream,
  besides the QA state dump (tooling). It gives each event an `Audience` as it arrives, and fills the frame's
  `ViewerEvents`, which every consumer reads instead. A consumer asks how a viewer perceives each event: `Whole`,
  `Clipped` (a path seen only over the viewer's Visible tiles, which the drawing code cuts), or `Nothing`. The
  audiences:
  - `Owner`, the owner alone: feedback, refusals, notices, refunds, production, construction and upgrades;
  - `Object`, whoever `Sight::sees` the object;
  - `Seen`, by place: every shot and projectile is clipped for every viewer, the shooter's owner too (0293); a blast
    shows where its centre is Visible; a death (a `Died`, and slice M5-V4's `Crushed` alike) shows to its owner and
    where it is Visible, never to a player it was hidden from;
  - `Removal`, whoever was shown the object where it went, never a player who remembers it out of sight;
  - `Everyone`: defeats and the end of the game, and (slice M5-N4b) Shroud's releases, which the drawing keeps to the
    viewer's Visible tiles, as the clouds (`shroud_view`).

  The fog off shows everything but a player's own business. Whom an object is hidden from is `Hiding::of`'s alone
  (the hook, *Vision > Hidden objects*). A new `SimEvent` doesn't compile until `Arrival::audience` has its rule, and
  fails the table test (`viewer_events/tests.rs`) until it has a row (`tests/table.rs`, the buildings that hold
  units' in `tests/table/holding.rs`). Never read
  `SimEventMessage` elsewhere.
- Never make one entity per tile. Draw the terrain as one mesh or texture, or a few chunks; the fog of war is one
  texture, never per-tile materials.
- Create mesh and material handles once and cache them in a `Resource`. Never create assets per frame, per object or
  per indicator, because the old prototype ran out of memory doing that.
