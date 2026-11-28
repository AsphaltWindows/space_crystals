# Vision

Read this when you touch sight, the Shroud's visibility rules or what a player can see.

## Vision

Slices V and F (`vision.rs`, `vision/`; `core/vision.md`). Every tile is `VisibilityState::Unexplored`, `Explored`
or `Visible` for each player, and each player remembers the structures and resources it has seen.

- **Sources**: every object with an owner and a SightRange above 0 (units and structures; an Extraction Plate has
  none); vision goes to its owner. Each sees a disc around its centre (a structure's: its footprint's centre): a
  tile is seen when its centre (`TILE_SIGHT_POINT`) lies within the SightRange (its type's, and, slice M6-SN, the Null
  Chamber exit buff's 2 GU while the unit holds it: `vision/sight.rs`'s `range`), boundary included, with the
  ElevationModifier between the object's elevation and the tile's level at its centre (`elevation::sight_range`: +1
  GU towards lower tiles, −1 towards higher, none for air). No line of sight: terrain blocks nothing.
- **Incremental stamps**: each source keeps its `Eye` (owner, centre, SightRange, elevation) and the sorted tiles
  it stamped. In 5f a source whose eye is unchanged costs nothing; one that changed stamps again and only the
  difference changes the per-player counts of sources seeing each tile; a source that is gone takes its tiles away.
  A tile is Visible while its count is above 0 and for the fog delay after, and Explored once it has been Visible.
  `PlayerVision::revision` changes when any of the player's tiles changes state, so views redraw only then.
- **Fog delay** (`vision/linger.rs`, `core/vision.md > Visible > Fog delay`, design change 0178): a tile that no
  source of the player sees any more in 5f of tick `t` lingers, genuinely Visible, at the end of ticks `t` to
  `t + FOG_DELAY_TICKS − 1` (8, a placeholder); 5f of `t + FOG_DELAY_TICKS` turns it Explored unless a source sees
  it again first. It is the one fix for a target blinking out at the edge of sight: Attack and Follow have no grace
  period of their own, and end once the delay is over (0200). Per player, `Linger` keeps a bit and an end tick
  per tile and a FIFO of `(end, tile)` in the order the delays end (a tile back in sight leaves a stale entry that
  the pop skips), so 5f costs the tiles that leave sight or come back, as before, plus the entries due, never a pass
  over the map. A delay that ends changes the revision, and the memory takes it as the tile leaving sight. It
  bridges sight that recedes from a target (a spotter walks off and a chaser's own sight takes over); a target that
  steps onto a tile never seen, ahead of the front edge of sight, is out of sight at once
  (`tests/fog_delay.rs`).
- **Attackers revealed** (`vision/reveal.rs`, `core/vision.md > Attacks from outside vision`, design change 0177):
  when a hit lands in 5e of tick `t` from an attacker that none of the victim's owner's sources sees (by the vision
  of the end of `t − 1`, the attacker's box where it fires from; the fog delay doesn't count: independent timers),
  the attacker alone, not its tiles, is Visible to that player at the end of ticks `t` to `t + REVEAL_TICKS − 1`
  (24, a placeholder); each such shot renews it. `PlayerVision::reveals` maps each attacker revealed to its end,
  and 5f drops those that end. Every attacker, whatever its range; a neutral victim reveals nothing.
- **Scans** (`vision/scans.rs`; slice M6-GR, plan decision T92; `factions/gdo/structures/radar_station.md >
  Placeholder stats`): a Radar Station's Scan (*Combat > Scan and cooldowns*) is a source of sight of its owner's own:
  the tiles whose centres lie within its radius of its point (6 grid units: 113 tiles round a tile's centre), with no
  ElevationModifier (plan D39), stamped in 5f of its cast's tick and withdrawn in 5f of the tick its 128 end, so that
  they are Visible at the end of the cast's tick and of the 127 after it, then linger the fog delay as any sight does.
  It shows hidden objects no more than other sight does (D39: an enemy's underground expansion stays hidden). Smoke
  over a scan is slice M6-SC's (design change 0433): the reveal is stamped in `Vision::step_scans`.
  `PlayerVision::scans` holds those pending and those active, saved (`SavedVision::scans`, checked as a save loads:
  on the map, a radius finite and above 0, ending after the save's tick and in order) and hashed (tag 1) only while
  there are some; `Vision::check` (V1) holds the stamps to them.
- **Memory** (`vision/memory.rs`, `core/vision.md > Explored`): a `LastKnown` record per tile-based object (a
  structure or a resource) the player has seen and doesn't own: kind, owner, footprint, HP, resource state, a Tunnel's
  tier, a site's progress (`LastKnown::site`, slice M4-I2: the `Progress`, not who builds it; hashed only while set)
  and the tick it was last seen. An object is in a player's sight while a tile under its footprint is Visible; its record
  is rewritten at the end of every such tick (`Vision::remember`, the last phase) and stays as it was out of sight.
  A record whose object is gone, or has moved, is dropped once a tile under the record is Visible: an object
  destroyed in the fog is remembered until its place is seen. An object placed on Explored tiles gets no record, so
  it stays unknown until seen. Own objects are never recorded. **The map's neutral objects are known from the
  start** (design change 0159, `core/vision.md > Explored`: "like terrain"): as the game starts (`Sim::settle_start`)
  `Vision::seed` gives every player a record of each object without an owner (the crystal patches, the Supply
  Delivery Stations), `seen` 0, before the first sight; the players' Starting Layouts are never pre-known, and an
  object that appears later is unknown until seen.
- **The map explored from the start** (slice M5-S7, design change 0416, `core/vision.md > Starting exploration`: "every
  tile starts **Explored** for every player instead of Unexplored"): a game option, `SimConfig::start_explored`. With
  it on, `Sim::settle_start` turns every tile Explored for every player (`Vision::explore_all`) before the first
  sight, beside the neutral objects' records above; no player's object is recorded, and from there memory works as
  ever (a patch mined out of a player's sight keeps its starting amount in their record). The rules that read the
  Explored bits judge from 0:00 as they would later in a game: placement's known terrain (`Mode::Known`: a
  never-explored tile passes, so with every tile explored every terrain problem counts from the start), the Tunnel
  Area's and the Recruitment Hall's known ground, the client's fog (dimmed, not black) and its records out of sight,
  and the areas the overlays leave out as unexplored; path knowledge, targeting remembered objects and the computer
  players' `View` read the records and the Visible tiles, so they don't change. Tests: `tests/starting_exploration.rs`.
- **Memory updates follow the vision's changes**, never a scan of the map or of every object for every player:
  5f's id-order walk over the objects also merges the footprints of the tile-based objects (`Footprints`), so only
  a footprint that came or went touches the index from tiles to objects; `see` and `unsee` note the tiles whose
  state flips; after the walk a noted tile under no footprint and no record costs two bit tests, and on the others
  objects enter or leave the sight and the records seen gone are dropped; `remember` reads only the objects in each
  player's sight. Measured (`tests/vision_perf.rs`, ignored, release): 500 moving units and 168 crystal patches on
  256 × 256, `update` and `remember` 0.17 ms per tick on average, 0.2 ms at the 99th percentile; (M6-GR) the same
  with four scans active all along (`perf_vision_500_units_and_four_scans_on_256`) as fast, within noise of it.
- **State**: the Explored bits, the records, the tiles that linger with their ends and the reveals are history, so
  they are hashed and serialized (each player's rows of `#` and `.`, its `last_known` records, and its `lingering`
  and `revealed`, both `#[serde(default)]`). Eyes, stamps, counts, the footprint index, the delay's queue and the
  objects in sight are derived: a loaded save rebuilds them from the objects (`vision::load`, which rejects a record
  twice, of an own object, off the map, or seen after the save's tick, and a delay or a reveal that ends in the past
  or beyond its length), and invariant V1 (`testing::invariants`) rebuilds them from scratch, from the same history,
  and compares.
- **Queries**: `Sim::vision()`, `Sim::visibility(player, tile)` and `Sim::is_visible(player, object)`: an object is
  seen while it is revealed or when any tile it covers is Visible (its silhouette box or footprint,
  `combat::object_box`). `Vision::player(id)` gives a `PlayerVision` with `state`, `states` (row-major, for the
  client's fog texture), `lingers(tile)`, `is_revealed(id)`, `object_state`, `sees`, `sees_box` (tiles only), and
  `object_box_state` and `sees_object_box`, the one object query that combat, `World::sees`, the gate and the
  client's `Sight` all go through; and for the records `records` (by id), `record(id)`, `remembered` and
  `remembered_record(id)` (those not in sight, which Explored tiles show), `record_tiles` and `is_record_tile` (the
  tiles under any record) and `records_revision` (changes when a record comes, goes or moves; not when only its
  state changes).
- **Hidden objects** (`vision/hidden.rs`, plan decision T32, slice M4-A): an owned underground structure (a
  Syndicate expansion, `StructureDef::underground`) is hidden from every player but its owner; a neutral one isn't
  (`factions/syndicate/tunnels.md > Losing Tunnels`). One predicate, `vision::hidden_from(object, player)`
  (`Sim::hidden_from` for the client), through `Hiding::of(object)`, which the snapshots carry (combat's
  `TargetView::hiding`, `World`'s views) and `object_box_state` takes: a hidden object's state is capped at Explored,
  however Visible its tiles are and whatever is revealed, so targeting, `World::sees`, the gate and the client's `Sight`
  never see it. The memory never records it (`remember` drops it, `Vision::seed` skips it). A neutral expansion (its
  Tunnel lost, slice M4-H) is seen and remembered by whoever sees its tiles; reclaimed, it is hidden again: the other
  players' records of it go when they see its tiles, and its new owner's at once (`Vision::forget_own`: a player never
  remembers its own objects, whether or not it sees them). **Damage never asks**
  (`systems/combat.md > AoEDamage`, design change 0262, keeping 0218: underground objects "take their share of any blast
  over them, even unseen"): a hit on the tick's list lands whoever sees its target (`combat/damage.rs`). Hidden units
  come with the Tunnel network (design change 0219). **A pop-up turret is hidden while fully lowered** (slice M4-S2,
  design change 0296 point 1; `popup::is_lowered`), by its state, which its kind doesn't tell: seen and remembered
  like any structure while it rises, stands or retracts, and its record dropped once its tile is seen with it down.
  A neutral one is lowered for good and hidden from nobody (design change 0360: "visible to everyone"), as a neutral
  expansion is. **The hook for a new arm of the rule** is `Hiding::of` in `vision/hidden.rs`, alone, where that arm is:
  vision, targeting, the records and the client's `Sight` ask it of the live objects, a death carries it
  (`SimEvent::Died::hiding`, slice M4-V), and the client's filter of events takes it for a removal (*Client
  architecture > Viewer events*).
- **Paths** are planned from what each player knows (design changes 0040 and 0042; *Navigation > Path knowledge*):
  the records' footprints are the other players' structures and the resources it knows of, the map's from the start
  (0159); its own come from the live objects, as they are never recorded.
