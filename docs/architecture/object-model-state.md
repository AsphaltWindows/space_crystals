# Object model — Carriers, kind changes, augmented stats and adding a type

Read this when you work on carriers, changing kind, type and form, augmented stats, vehicle deploy, sides, or add an object type or a capability.

- **Changing kind** (`sim/kinds.rs`, plan decision T15): an object that transforms (a Drone deploying into a Beacon,
  a Beacon warping in a building, a Legionnaire converting its weapon, a Peacekeeper raising or lowering its Riot
  Shield) stays the same object, its id included.
  `Sim::change_kind(id, kind, placement)` is the one place a kind changes, and it emits `SimEvent::KindChanged`. It
  keeps the id, owner and facing; the position too, unless the object gets a placement, whose centre it then jumps
  to (`prev_pos` too); the HP, clamped to the new MaxHP (callers interpolate HP themselves); the Shields, clamped
  to the new MaxShields (a kind without any has none, one that gains them starts full); and (slice M5-N4a) the energy,
  kept at most the new maximum (`energy::after_change`: a kind without a pool has none, one that gains a pool starts
  at its type's start, so the Recruit a Cathedral trains comes out a Disciple with 50). It rebuilds the
  capabilities from the new def with `kinds::Capabilities::fresh`, which `Sim::insert_object` uses as well: a
  capability whose def part is unchanged keeps its state, a new one starts fresh, one the new kind lacks goes. A
  unit that stays a unit keeps its orders and its move (design change 0150), replanning if its grid changes; a
  kept attack keeps its target and restarts its sequence (0152), or keeps its whole state when the weapon in hand
  stays the same (`combat::weapon`: the Riot Shield's two types, and a Legionnaire pressed to convert, which keeps its
  form's weapon until its conversion ends; a shot or a reload goes on). Paid state the new kind has no place for (a production
  queue, a construction) is refunded first, as on removal. Call it before 5b (in the commands phase, or a step
  right after 5a), or sync the grids yourself (`paths::sync`). A caller whose new kind isn't functional yet (a
  building warping in) holds its functions back itself. Slice E's Warp In and its cancel are its first callers
  (`sim/transform.rs`), then a toggle's press (`Sim::use_ability` in `sim/modes.rs`: slice B's Weapon Conversion and
  slice M3-S's Riot Shield, see *Type and form*), slice F's deployment (`sim/deploy.rs`, the step right after
  5a) and (M6-GA) a vehicle deploy's completion (`sim/vehicle_deploy.rs`, the step right after the Modes step, see
  *Vehicle deploy*). `sim/kinds/tests.rs` unit-tests it. The client rebuilds the visuals of
  a changed object and keeps it selected (see *Commands, events and replays > Events*).
- **Type and form** (design change 0198, slice M3-T): a toggle, the Legionnaires' Convert or the Peacekeeper's Riot
  Shield, sets the state the unit works toward ("however fast R is pressed, every unit that received the presses
  ends up in the same form"). Its **type**, `Object::kind`, is that target: the press switches it at once
  (`Sim::use_ability`, `Ability::switched_kind`, through `Sim::change_kind`, so with a `SimEvent::KindChanged`),
  whatever the unit is doing, a shot it waits for included. What commands the unit reads the type: the gate, the
  default command (`orders::resolution`), the client's selection groups, panel and right-click, so a second press
  goes to the units the first reached, as they are now. Its **form** is what it has in the world now, which the
  mode's own state holds: a Legionnaire's weapon (`Conversion::form` while a conversion is pending, else its type),
  a Peacekeeper's shield (`RiotShield::up`, how far up it is). What the unit does in the world reads the form, each
  through one function: `conversion::form(o)` (what the client draws, `object_render::drawn_kind`) and
  `combat::weapon(o)` (its AttackAttributes: 5e, the behaviours, the threat it is), and `riot_shield::mode(o)`, with
  `allows_attack(o)`, `damage_factor(o)` and `speed_factor(o)` from it. The Modes step works the form toward the type
  (`conversion::step`, `riot_shield::step`): at once, or after a shot under way (Firing and Cooldown,
  `combat::mid_attack`; *Combat > Changes that wait for an attack*). Pressed again, a unit waiting takes the new
  target and drops a change that is back to its form (`conversion::retarget`; a deferred raise never starts), a
  conversion under way finishes and converts back, and a shield turns back from where it is. Which orders a unit
  takes follows its type from the press, carrying them out waits for its form (design change 0202:
  `riot_shield::takes_attack_orders(kind)` for the first, `allows_attack(o)` for the second). A Legionnaire's two
  forms share everything but the weapon, so its shape, sight and speed are its type's and its form's alike.
- **Augmented stats** (slice M5-N1a, plan decision T73 as design changes 0330, 0331, 0336 and 0338 amend it;
  `augmentation.rs`, `augmentation/stats.rs`, `augmentation/levels.rs`; `factions/cults/augmentation.md > Increments`):
  a unit's MaxHP, top speed and weapon are read through accessors that apply its Augmentations, and only through them:
  `Object::max_hp_with(research)`, `Object::movement(research)` and `Object::max_speed(research)` (before the mode's
  `Mover::speed_factor`), and `combat::weapon(o, research) -> Option<AttackAttrs>`, a copy; by reference,
  `Object::movement_ref` and `combat::weapon_ref` (below). The kind's numbers
  (`ObjectDef::max_hp`, `UnitDef::movement`, `UnitDef::attack`) and `Object::max_hp` (a tier's, a site's, a
  transformation's) are the values before them. The effects table is data (`defs/cults/augmentation.rs`), and which
  kinds take Augmentations, armed or not, is `UnitDef::augmentable` (the four Cults infantry types, the Recruit alone
  unarmed).
  - **The amounts follow the owner's current levels**, read whenever a stat is (design change 0336: research is
    retroactive; 0338: every Cults stat upgrade applies to the units that exist). A unit stores which Augmentations it
    has, never their amounts or a level. `Research<'a>` is a view of the players' research: `Levels::of(player)` is the
    one place a player's levels come from: each line's highest level among the technologies they have completed
    (`Player::techs`, slice M5-N3; "No research" without one), in O(1) and allocating nothing; `Research::at(levels)`
    puts everyone at given levels, for tests. **A defeated player's units keep their research** as they turn neutral,
    frozen at the defeat (design change 0379, answering Q0142; `core/game_rules.md > On Defeat`), so nothing on them
    changes: `economy::defeat` sets `Object::kept_research` to their owner on those with an Augmentation (hashed under
    a tag and saved only while set), and `Research::levels_of(o)`, the accessors' one lookup, reads the levels of
    `o.owner.or(o.kept_research)`. No copy of the levels is needed: a defeated player owns no structure (T8), so
    completes no research again. A1 checks that only a neutral unit keeps research, a defeated player's.
  - **Why this shape.** The systems that hold the objects mutably (phases 5a, 5d and 5e, the network's regeneration)
    take a `Research` beside them, a borrow of the players alone, which a `Sim` method couldn't be; code that has the
    `Sim` asks `Sim::research()`. Stats stored on the object would go stale when a level completes or a def changes
    (T73's rejected alternative). A unit without an Augmentation costs a branch (`Augmentation::is_none`): the loops
    over every unit (phases 5a, 5d and 5e) borrow its kind's numbers (`Object::movement_ref`, `combat::weapon_ref`,
    which make an augmented unit's in a slot of the caller's), through accessors inlined into them, their augmented
    paths out of line (`augmented_increments`, `augmented_movement`); an augmented unit looks its owner up among the
    few players. The behaviours keep a unit's increments (`orders::world::Unit::increments`) and work out its top
    speed only where they need it. Measured (release, 10 interleaved rounds against slice M5-N2's build, load average
    1.5 to 2.2): `movement_perf` 0.481 ms and `recruits_perf` 0.078 ms a whole tick on both sides; a first version
    that copied the numbers out (`Object::movement`, `combat::weapon` in the loops) ran `movement_perf` 2% slower.
  - **HP restored** goes up to `max_hp_with`: the repair's (`repair::damage`, whose MaxHP also sets its rate and its
    price's steps, slice M5-N8) and the regeneration's (`crate::regeneration`). No repairer repairs infantry yet.
  - **HP when MaxHP changes** (design changes 0330 and 0331): `augmentation::follow_max`, applied by
    `augmentation::set` (the Altar's orders, slice M5-N1b, and `DebugCommand::SetAugmentation`), by a completed level
    to every unit of its owner's with the line's Augmentation, on the map or held (`Sim::complete_research`, slice
    M5-N3).
    Up, the HP rises as
    much: `transform::follow_max`'s rule, the damage staying. Down, the HP stays unless it is above the new MaxHP:
    there the two rules part, as a transformation's keeps the damage (design change 0110). No floor; an augmentation
    never kills.
  - **Kind changes** (`Sim::change_kind`): a unit keeps its Augmentations as a kind that can have them, loses them as
    any other, its HP within the new kind's MaxHP with those it keeps; `Sim::train` follows the MaxHP with them before
    and after (a Constitution Recruit's 60 is a Militant's 60).
  - **What they don't change** reads the weapon before them (`combat::weapon_in_hand`): the threat a unit is to the
    scans (`combat::threat`, its TargetDomain) and `can_attack_now` (its ranges); and the TurnRate. A weapon borrowed
    from another kind (a Recruit's or a Disciple's in the Garrison, the Militant's, slice M5-N2) is that kind's, read
    from its def (`combat::occupants::weapon`, `weapon_of`), and the effects apply to a unit's own attack only
    (`augmentation::augmented`; `factions/cults/structures/garrison.md`: "Augmentations don't change the borrowed
    attack"); a Militant or a Gunner inside fires its own through `combat::weapon`.
  - **The guard**: `tests/augmentation/raw_reads.rs` scans the sim's and the client's code, outside the defs, the
    accessors and the tests, for the reads past the accessors (`.max_hp()`, a `.movement` field, `weapon_of(`), and
    fails on any its list doesn't allow with a reason (a structure's MaxHP, a turn rate, a kind's weapon). A new read
    goes through the accessor, or joins the list with its reason. A test rather than unreachable fields: the defs are
    public data that their own tests and the client's panels read, and `Object::max_hp` stays the structures' MaxHP.
- **Vehicle deploy** (plan decisions T44 and T87, slice M6-GA; `vehicle_deploy.rs`, `orders/vehicle_deploy.rs`,
  `sim/vehicle_deploy.rs`): a unit of a pair of types becomes the pair's other type in place, by an order that takes
  its type's frames and can't be cancelled, unlike a toggle ("**the deploying process is what actually changes the
  unit**: it stays the mobile type until the process completes"). **Data**: each type of a pair has a row,
  `UnitDef::vehicle_deploy` (`VehicleDeploy { into, frames }`), so a new pair is a def row on each type (the Artillery
  Rig's two now; slices M6-SL's Laser Battery and M6-ST2's Transport). **The order**, `OrderKind::VehicleDeploy {
  into }` (the panel's R, `PanelCommand::VehicleDeploy`), names the type the unit becomes, so a Deploy pressed twice is
  no Undeploy; it starts in 5a (`Behaviour::VehicleDeploy`, a `Progress`), stops the unit where it is and keeps it
  from attacking (`Unit::may_attack`), and the step after the Modes step counts its frames and completes it (*Tick
  pipeline*). **Held orders**: a deploy under way stays the current order whatever is given (`Orders::give`,
  `Sim::drop_orders`): an order given meanwhile is held behind it, Shift-queued ones after that. **The gate** skips its
  per-unit capability filters for a deploying unit (its target checks still apply) and refuses it only another deploy
  (`Reject::Deploying`); every other unit goes through `vehicle_deploy::can_take`, the one rule the gate and the
  completion share, which adds that a type that can't move (`vehicle_deploy::immobile`: MaxSpeed 0, a deployed Rig)
  takes no order that would move it (`vehicle_deploy::moves`; `Reject::CannotMove`). **The client** judges a deploying
  unit as the type it becomes (`Modes::becomes`: its A and its right-click), and says a dropped order as the gate's
  refusal of it (`feedback::dropped_text`). **A site** (T87, slice M6-GP's Prospector, not built): the order's start,
  where a deploy that needs a site would walk to it first, and `Sim::complete_vehicle_deploy`, where the type it
  becomes would be a structure placed on that site.
- **Mechanics implemented once** (plan decision T54, slice M4-R1): `Progress` (`progress.rs`), ticks done of a total
  in the fixed point of plan decision T6, so that a tick at part speed counts its share of a tick; a transformation
  counts its frames with one (`Transform`). The toggle rule (`toggle.rs`, design change 0198, see *Type and form*):
  `toggle::pressed` switches the type at the press and `toggle::toward` works the form toward it, for the Riot Shield
  and the Weapon Conversion alike, each keeping its own state. The other timers don't use `Progress` yet, because
  their hashed bytes would change: production, construction, the Extraction Facility's cycle, the Weapon Conversion,
  the Riot Shield, a worker's mining and unloading, and the Tunnel Network's held units. Each moves at the next
  re-bless of the goldens that touches it. **New timers use `Progress`.**
- **Sides** (`sides.rs`, design change 0079): `sides(kind, orientation)` gives the letters of a structure's sides
  that face north, east, south and west (`[SideLetter; 4]`, `None` for other objects). Unturned, the letters of its
  `SymmetryType` (AAAA to ABCD) run clockwise from the top side; a flip mirrors them, then the rotation turns them
  with the building. A Barracks (ABAC) has its exit B on the east side unturned, on the south side turned 90°
  clockwise.
- **Footprints don't overlap** (design change 0096: no buildings on resources), with one exception, the pair of an
  Extraction Plate and the uncovered crystal patch it is placed onto (`ObjectKind::may_overlap`;
  `factions/gdo/economy.md > ExtractionPlate`). Scenario validation (`scenario/validate/occupied.rs`, for the map's
  objects and the Starting Layouts alike), debug spawns (`Reject::Overlaps`) and the placement check (see *Placement
  and the build area*) apply it. A plate must stand exactly on a patch (`placement::Onto`), which the placement check
  and map validation ask alike; only a debug spawn puts one on bare ground, where it mines nothing (see *Economy*).

**Adding an object type**
1. Add its def module with its `static ObjectDef`, a citation for every value (*Adding a unit or structure*), and its
   line at the end of the kind registry (`kind_registry!` in `defs.rs`), whose doc comment cites the design section.
   That one line makes the `ObjectKind` variant, its place in `ObjectKind::ALL` and its arm of `ObjectKind::def()`.
   Missing numbers become named consts tagged with the question id.
2. Set up its capability state in `kinds::Capabilities::fresh` (`sim/kinds.rs`; units get idle `orders` and
   `mover`), and give it its client module, with its line in the client's kind registry (*Adding a unit or
   structure*). Its per-object values, if it has any, are `SpawnSettings::defaults`' (the editor's fields follow
   them).
3. Test it. Spawn it in a small inline scenario, then check its initial state and its behaviour.

**Adding a capability**
1. Give it its own module with its state type (deriving `Clone, Debug, PartialEq, Serialize, Deserialize`) and logic.
2. Add `pub <name>: Option<State>` to `Object`, with a doc comment citing the design.
3. Put its per-type parameters on `ObjectDef`, with citations, and initialize the state in
   `kinds::Capabilities::fresh`. Decide in `Sim::change_kind`, whose destructuring of `Object` is exhaustive, what a
   changed object does with it: keep it, start it afresh, or refund it.
4. Insert its update phase into `Sim::step` at a deliberate position (see *Tick pipeline*).
5. Add the `Command`s that drive it, with validation, and the `SimEvent`s the client needs. A unit's mode (a
   toggled ability such as the Riot Shield) is not an order: it goes through `Command::Ability` (an `Ability`
   variant, see *Commands, events and replays*), never into the order queue.
6. Write unit tests in the module and a scenario test in `crates/sim/tests/`.
