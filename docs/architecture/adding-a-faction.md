# Adding a faction

Read this when you add a faction.

## Adding a faction

What a faction needs before players can start with it, in the order M3 (the Colonists) built it. Started by slice
M3-C and finished by M3-K2 from what the M3 slices needed (plan `artifacts/director/m3-plan.md`, the Built entries of
`artifacts/director/plan.md`); reviewed against M4 (the Syndicate) by slice M4-M, which added items 15 to 18 and M4's
examples, and against M5 (the Cults) by slice M5-Mb: M5's slices had added the Cults' cases as they went (items 1, 4,
5, 7, 8, 11 and 19), and M5-Mb added item 16's.

1. **Defs** (slice M3-A, laid out by M4-R1): the faction's root module, `defs/<faction>.rs`, with its own rules (its
   caps and divisors), and a def module per kind under `defs/<faction>/`, each with its line at the end of the kind
   registry (the state hash numbers kinds in that order), its `static ObjectDef` with `faction: Some(..)` and, for a
   unit, its army-cap cost in its `UnitDef`; every exhaustive match over `ObjectKind` (sim and client) decided. Each
   kind has its client module (*Adding a unit or structure*), and the editor's unit palette lists the faction's units
   by itself. A cost may share another faction's field when it means the same: the Cults' Unit Control cost, the
   Recruits a unit takes, is `unit_control_cost` (slice M5-A, plan decision T55), whose `defs_are_consistent` check
   names the factions that have one.
2. **Banks**: a resource the faction stores is a `Player` field (`u32`, hashed in `hash.rs`, `#[serde(default)]`),
   in `Banks` and `DebugCommand::SetResources` (`serde(default)` there too, so older command files load). Re-bless
   the goldens (see *Replays and golden hashes*). The Syndicate's Supplies (slice M4-I) took no new field: the bank
   was there, and what was new was gathering it (item 12's workers, from a neutral station).
3. **Starting Conditions** (`scenario/starting.rs`): a `static StartingConditions` table and its arm in
   `StartingConditions::of`. A game then starts the player's banks from it (`Player::starting`), and the camera on
   its primary structure (`Scenario::primary_structure`).
4. **Layouts**: from then on every Skirmish Default map needs the faction's layout at every start (plan decisions
   T11, T24, T63): `scenarios::dev_skirmish_map` (slot 1's; `MapDef::regenerate` makes the images), every Skirmish
   Default file in `maps/` (place them with the editor's Layout tool and save;
   `the_maps_in_the_repository_are_playable` says which), and the tests that build a Skirmish Default map and expect
   it playable (the editor's use `tests/editor/colonists.rs::with_colonists`,
   `tests/editor/syndicate.rs::with_syndicate` and `tests/editor/cults.rs::with_cults`). Check each layout with
   `MapDef::validate` (its structures by `placement::check_layout`, its units by the unit-start rule), and keep the
   starting units clear of the structures' sides and exits. The goldens on `dev_skirmish` embed its map, layouts and
   all, so they are re-blessed for the new layouts, every checkpoint the same (as slices M3-C and M4-B did; *Replays
   and golden hashes*). Every faction has Starting Conditions since slice M5-B (plan decision T63): a test that needs
   a faction without something uses `test_defs`. A faction's own layout rule goes into `check_layout` (the
   Syndicate's expansions inside their Tunnel's area), and its text into the editor's `rule_text` and the client's
   `placement_text`.
5. **Army cap** (`economy/army.rs`): the faction's `ArmyResource` (`ArmyResource::of`, `name`, `cost_of`, and what
   is available in `cap`), or none. A cap may be derived rather than built (the Syndicate's Tunnel Space, plan
   decision T33: its Tunnels' areas less their expansions' cells), and then counts the units off the map too
   (item 15). The start check at production start is the one for every faction (design change
   0143, `sim/producers/step.rs`), and so is its `SimEvent::ProductionWaiting`; the client's `army_cap_text` and the
   InfoPanel's waiting line name the faction's resource. A resource that only shows (the Cults' Unit Control, slice
   M5-C) says so in `ArmyResource::reserves`, and nothing waits for it. A unit costs only in its own faction's
   resource (`ArmyResource::cost_of`).
6. **HUD figures** (`hud/resources.rs`): the faction's `ResourceFigure`s in its DisplayHud order (`ResourceFigure::of`)
   and each new figure's reading in `resource_bar`, in ASCII.
7. **Templates** (`command_ui/interface/templates.rs`, an object's own panel in `templates/<faction>/<object>.rs`): each
   kind's panel template, in its client module (*Adding a unit or structure*), `PanelTemplate::NONE` until its slice
   builds one. A producer takes `InterfaceTemplate::Production`, whose grid comes from its `ProductionDef` (a new unit
   to train shows its label; a producer that recruits by itself, the Recruitment Hall, shows no Train and no X, only
   C), and a pool building `InterfaceTemplate::PoolProducer`, whose KindMenus come from it (slice M5-F2).
   A builder needs its `Builder` (`command_ui/interface/placing.rs`: where the ghost stands,
   which check colours it, what a click sends; slice M3-J1 has a constructor's, the Drones' and the Beacon's), and a
   building not in the game yet holds its menu slot as a `PanelCommand::Planned`. A type that transforms shows only its
   Cancel while it does (`TRANSFORMING_GRID`; `colonists::becomes` lists the types that may).
8. **Producers' exits** (`ProductionDef::exits`, slices M3-D and M3-D2): `ExitRule::Side` for a side doorway
   (the unit appears half inside that side), `ExitRule::Sides` for a door on every side of that letter, the first
   free one in `Doorway::order` taking the unit (slice M5-C), `ExitRule::Centre` for an energy doorway open through the
   sides with the letters given (the unit appears at the centre and walks out through them; its arc comes from the
   sides, so a new portal needs no code; see *Production*). `defs_are_consistent` checks that the letters name the
   type's sides.
   A doorway with a rally point gives the new unit its own right-click order at release (`orders::resolution`).
   `ExitRule::Network` is a producer with no doorway (the Hideout, slice M4-C; the Vehicle Assembly, M6-SV; the
   Aerospace Assembly, M6-SD): its
   units go into the held store
   (item 15), and a rally point brings them out of a Tunnel. `ExitRule::Runway` is a holder's that trains aircraft
   (the Airstrip, slice M5-N7): they take off along the runway between its two sides, its ground units leaving by
   the holder's exit; the client draws it as `Look::Runway`. `ExitRule::Pad` is an Airfield's (slice M6-GH1): its
   aircraft appear (slice M6-GH2) landed on a free slot, assigned to it, and need no doorway
   (`Sim::release_onto_slot`); the client draws its slots as `Look::Pads`.

The faction's own mechanics come next, each only if it has them (M3's and M4's in brackets):

9. **Kinds that change** (slices M3-A, M3-E, M3-F; plan decision T15): every change of kind goes through
   `Sim::change_kind` (see *Object model > Changing kind*: same id and owner, capabilities rebuilt, orders, move and
   target kept, a `SimEvent::KindChanged`). A building that comes out of another object is an `Object::transform`
   (`transform.rs`, `Sim::step_transforms`: a Warp In, a Drone deploying as a Beacon), placed by its own `Method`
   in the one placement check, and the client rebuilds the visual on a `MeshKey` change and moves the object into
   its new group (`ControlState::set_type`; an unfinished building's group is its own, `GroupKey`).
10. **Abilities and modes** (M3-B Weapon Conversion, M3-S the Riot Shield, M3-T the target state): an `Ability`
    (`command.rs`), its rule in the gate (`sim/gate/orders.rs::check_ability`), its use in `sim/modes.rs` (a
    toggle switches the type at the press, `Ability::switched_kind`, and its mode's state holds the form, which the
    Modes step works toward the type, waiting through an attack phase that can't be interrupted,
    `combat::mid_attack`; *Object model > Type and form*); the client's `PanelCommand`, its template slot and a
    label of six characters at most with its full name in the hover hint, and the press shown at once
    (`ControlState::toggle`, `PendingCommands::kind_of`).
11. **Shields** (M3-B): `max_shields` in the def; `Object::shields` absorbs in `combat/damage.rs::resolve` and
    regenerates in `Phase::Shields`; the client draws the bar and the InfoPanel line. **Energy** (M5-N4a) likewise:
    `energy` in the def (an `EnergyDef`); `Object::energy` regenerates in `Phase::Energy`, and E1 bounds it; the
    client draws its bar and line from `energy::of`. **Area abilities** (M5-N5): an `AreaAbility` with its
    `AbilityDef` (`defs/cults/abilities.rs`), listed in its caster's `StructureDef::abilities`; `Command::Cast` and
    its gate (`sim/casts.rs`), `Object::buffs` in `Phase::Buffs` (`buffs.rs`), and F1 bounds them; the client's
    `PanelCommand::Cast` in the caster's template, the cast's circle (`indicators/cast.rs`) and the buffs' marks
    (`combat_effects/buffs.rs`). (M5-N11) A unit casts one as a spell, `SpellDef::Cast` in its `UnitDef::spells`,
    with the area its `AbilityDef::area` says (round it or ahead of it); its effect, a `BuffEffect`, is read where it
    counts (`buffs::max_speed_bonus`, `buffs::inspired`, `suppress::holds_fire`); the client's `templates::cast_slot`
    puts its key, and `combat_effects/casts.rs` draws the cast. (M6-SN) A buff no cast gives is a `BuffKind` variant
    of its own, with its duration and the kinds it affects (`BuffKind::duration`, `affects`), given by
    `buffs::hold`, hashed as a number no ability's takes and saved by a name of its own (`buffs::SavedKind`), and a
    mark in `combat_effects/buffs.rs`.
12. **Workers** (M3-H, M3-H2, M3-H3, M3-K2a): a `WorkerDef` (its carry capacity, mining and drop-off times and
    reaches, search radius and `pass_through`), the Gather and Drop Off orders (`orders/gather.rs`, the gate in
    `sim/workers.rs`), the timed work and the bank credit (`economy/gathering.rs`, `SimEvent::ResourcesDelivered`),
    `StructureDef::drop_off` for the buildings that take loads, and a new drop-off for the workers of one that goes
    (`Sim::reassign_drop_offs`, on the removal path). Units that may overlap go in
    `invariants::overlaps::may_overlap`. In the client: the pointers, the load marker, the InfoPanel lines and a
    select-the-workers command.
13. **A movement model** (M3-G, plan decision T25): a `MovementAttrs` variant with its mover in `movement/`
    (`drag.rs` beside `follow.rs`), the movement invariants amended in writing, and `movement_perf` run A/B.
14. **Orders of its own** (M3-F Deploy): an `OrderKind` with its behaviour in `orders/`, its gate check and its
    events (`SimEvent::DeployFailed`), reservations that others' placements respect (`Sim::pending_builds`), and the
    client's ghost (a `Builder` in `command_ui/interface/placing.rs`). **A worker-built structure** (M4-E) needs no
    order of its own: its def's `StructureDef::site` (`SiteDef`: the price, the frames, the share of its MaxHP it
    starts at, where the builder goes) and its builder's `WorkerDef::builds`, and the Build order, the price and the
    site take it (*Economy > Agent-built Tunnels*).
15. **Units off the map** (M4-C, plan decision T30): a unit inside something, the Syndicate's Tunnel Network, is not
    an object but a `Held` in `Sim::held` (*Object model > Units off the map: the held store*). Its ways in and out
    are `Sim::take_off_map` and `Sim::put_on_map` only (the Enter order and the Tunnel's eject queue,
    `sim/network.rs`), and whatever must count it says so: the army cap, the removal path's references
    (`Sim::forget_references`), a defeated player's units, the state hash and saves, `testing::dangling_references`,
    the client's control groups. A new kind of holder (a transport) gives `Place` a variant and matches it
    everywhere `Place` is matched. The client reads the store for its menus (`Sim::network_counts`, `next_out`).
    (M6-SN) **A network service** (the Null Chamber's, (M6-SB) the Vault's): its queue in
    `network::services::step` (design change 0424: services run side by side, each in its own queue), its turn a
    field of its own in `network::Service`, hashed under a tag of its own only while set, unless the unit's own state
    is its turn (the Vault's: the tanks), and what it gives on the way out in `Sim::leave_network`; the EjectMenu's
    order is untouched.
16. **Sites** (M4-E, M4-F, M4-S1, M4-S2, plan decision T31): a structure built where it stands is the object from its
    first frame, with `Object::construction` while it is a site (*Object model > Sites*): a worker's
    (`StructureDef::site`, the builder held at `Place::Site`) or a Tunnel's expansion (`StructureDef::expansion`,
    `Builder::Tunnel`, the Tunnel's one operation, *Expansions and a Tunnel's operation*). Its maximum HP follows the
    ConstructionHP rule, every system asks `Object::is_operational` before treating it as working, and its cancel and
    refunds are the site's (`SimEvent::SiteCancelled`, `BuildRefunded` for an order that never became a site). A
    builder that isn't a worker or a Tunnel adds a `Builder` variant, as the Cults' Recruits, who build together from
    outside, added `Builder::Recruits` (slice M5-E, *Economy > Cults construction*).
17. **The underground layer** (M4-A, M4-F, M4-H, plan decision T32): `StructureDef::underground` puts a structure in
    a layer of its own: hidden from every player but its owner while owned (*Vision > Hidden objects*), walked over
    (`BlockerKind::UndergroundExpansion`; surface and underground footprints never share a tile, `may_overlap`),
    hit by blasts over it (*Combat*), placed by the expansion method inside its Tunnel's area (*Placement and the
    build area*), and, when its Tunnel goes, neutral and decaying (*Object model > Losing Tunnels*). A form between
    the layers is a pop-up (`StructureDef::popup`, *Object model > The pop-up form*): one function decides each
    reader's answer. The client draws what only its owner sees as an outline.
18. **Prerequisites** (M4-K, plan decision T37): `StructureDef::requires` and `Recipe::requires` name the kinds that
    must stand; one rule for every faction counts them (`tech::counts`, *Object model > Tech prerequisites*), and the
    gate checks them only as the work starts (`Reject::Requires`). The client greys the entry with "Requires a/an
    <name>", so a faction's prerequisites need data only.
19. **Tests and QA** (plan decision T27; slices M3-K1 and M3-K2, M4-L and M4-M): `ScenarioBuilder::faction` sets a
    test player's faction, and `cargo run -- --factions <f1,f2>` starts a game with it, which its replay records. Then:
    - a build line through the gate beside `testing::build_order` (`testing::colonists`: Train, Gather, Deploy and
      Warp In; `testing::syndicate`: Gather, Job, Expand, Upgrade and After, with `SupplyBooks` for the Supplies;
      `testing::cults`: seats, group Builds and Train From),
      whose `Books` count what the faction's income adds;
    - goldens of its opening and of a game against another faction, both players scripted (`tests/golden/colonists.rs`,
      `testing::colonists::skirmish`; `tests/golden/syndicate.rs`, `testing::syndicate::skirmish`;
      `tests/golden/cults.rs`, `testing::cults::skirmish`);
    - its economy oracle against the design's targets (`tests/economy_oracle/`, `docs/economy_oracle.md`);
    - a perf test of its crowds (`tests/drones_perf.rs`, `tests/agents_perf.rs`, `tests/recruits_perf.rs`; the
      Cults' whole economy, `tests/cults_perf.rs`, slice M5-Mb) and an ignored 10-minute soak of the
      golden game (`tests/skirmish_soak.rs`, `tests/syndicate_soak.rs`: invariants, books, no stuck order, the hash
      stable over two runs; the Syndicate's adds the network's invariants, `check_network`, and plays on past the
      golden's end with more on both sides);
    - its opening in the playtest pairings (slice M5-R), one line in `OPENINGS` (`tests/pairings/main.rs`, its hook in
      `openings.rs`): every ordered pair of the four factions (16) plays on every Skirmish Default map
      under the invariants, the books and the gate, until player 2's buildings are removed and player 1 wins; the
      client's `tests/pairings.rs` starts each pair from `--factions`. A faction without a line plays idle;
    - its pairings in the ignored `soak_sweep` (`pairings/soak.rs`, slice M5-Mb), with its own invariants checked
      after every tick (*Sim scenarios and tests*);
    - a scripted walkthrough of the first playable (`--factions` with `--input-script`, see *QA mode*; the Cults',
      `qa/walkthrough/m5/run.sh`), and its playtest notes in `docs/playtest/`, listed in `docs/playtest/README.md`.
20. **A computer player** (slices A1, A2): its module in `crates/ai/src/factions/<faction>.rs` and its `HOOK`
    (*Computer players > The crate*). `--ai` then takes its players, and `ai_games` plays it against every faction
    that has one.
21. **What a unit lays on the ground** (M6-SB, plan decision T93): a `PatchKind` and its `TankDef` in the unit's
    `UnitDef::tanks` (its toggle, its key in R or F and its bar follow), what a patch does where it is read (the
    sludge's slow in `patches::slow`, before the move; slice M6-SC's smoke in the vision), and its look in the client
    (`patches_view.rs`). The store, its grid and its hash are `patches.rs`'s, one for every kind.
