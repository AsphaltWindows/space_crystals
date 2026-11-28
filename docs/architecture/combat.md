# Combat

Read this when you touch weapons, damage, projectiles, shields, energy, armor or anything that kills.

## Combat

Slice W (`combat.rs`, `combat/`, `elevation.rs`; `systems/combat.md`). A unit that can attack has an `AttackState`:
the target its behaviour engages (`target`, set by its Attack order in 5a), the phase of the running sequence and the
ticks spent in it. Hashed and serialized like all state.

- **Phases** (`combat/phases.rs`, its tests in `combat/phases/tests.rs`): Aiming → Firing → Cooldown → Reloading, each
  lasting exactly its duration in ticks (zero included). A sequence starts in 5e when none runs and the engaged target
  is valid and in range; the next one at the earliest in the tick after Reloading ends, so a cycle takes Aim + Firing +
  Cooldown + Reload ticks (Peacekeeper 17, Guard 24: its laser rifle, design change 0186). Ordered in tick `t`, the
  first hit lands at the end of tick `t + Aim + Firing − 1`.
  Aiming turns the base towards the target at TurnRate (the attack writes `facing`) and lasts until it faces it
  within `Combat::facing_tolerance`. Aiming and Reloading are interruptible: another order, or a target that is gone or out
  of range, cancels an aim; a reload keeps counting. Firing and Cooldown run on whatever happens.
- **Queries**: `combat::may_move(&Object)` and `may_turn(&Object)`: whether the attack leaves the base free to move
  or turn this tick (`systems/combat.md > UnitBaseSource`: not while Aiming, Firing or in Cooldown; turret units
  always), and never while the unit converts its weapon (see Weapon Conversion). Phase 5d
  holds a unit that may not: it stands, keeping its goal, path and stuck clock (`movement::wait`). The behaviours
  start a chase only when `may_move` holds.
- **One turn budget per tick**: a unit turns at most its TurnRate per tick in all. Phase 5d turns it first and notes
  the angle (`movement::Turns`, in the movement scratch); the aim in 5e turns it only by what is left
  (`Shooter::turn_left`). Both turn through `math::turn_toward`. It matters in the tick a unit that moved comes into
  range and begins to aim. The record is transient: 5d writes it afresh and 5e of the same tick reads it, so it is
  neither hashed nor saved; it follows from the state the tick began with and the tick's inputs.
- **Range** (`shape.rs`, `combat/targeting.rs`; `systems/combat.md > Range Measurement`, design change 0031): **edge
  to edge**, the gap between the attacker's shape and the target's. `Shape::of(object)` is a unit's Silhouette or a
  tile-based object's footprint; `Shape::gap` measures between two, `Shape::distance_to` to a point (ground-location
  targets). A Silhouette that isn't square turns with its unit's facing (slice M5-V1, design change 0033:
  `Shape::silhouette` takes the facing; `ObjectDef::silhouette_turns`), and the gap is then the least distance from
  a corner of one shape to the other; square ones stay axis-aligned. Range and proximity changed in `shape.rs` alone.
  `combat::reaches(attrs, attacker shape, elevation, target shape, elevation)` is the one range check: within Range
  and beyond MinRange, both inclusive, with the ElevationModifier. `can_attack_now(map, vision, attacker, target)`
  adds the rest for two objects: destructible, seen, an allowed domain.
- **Vision** (`systems/combat.md > ValidTarget` 2): a sequence starts, and an aim goes on, only while the attacker's
  owner sees the target (`PlayerVision::sees_object_box`: the fog delay and a reveal count), by the vision 5f left
  at the end of the tick before. What a unit does about a target out of range or out of sight is its behaviour's; a
  neutral attacker sees nothing. Once the tick's hits are applied, each one reveals its attacker to the owner of the
  object hit if that player doesn't see it (`Vision::reveal`, see Vision).
- **ElevationModifier** (`elevation.rs`, shared with vision): `Elevation::of` an object is its elevation level (the
  height rounded down), or `Exempt` for air units; `modifier(source, target)` is +1/−1/0 grid unit, binary.
  `attack_range` and `sight_range` apply it; melee attacks don't take it.
- **Melee** (`systems/combat.md > FullyConnected > Melee`: "Short reach: 8 space units"; the Sabre Legionnaire):
  every melee attack has the Range `defs::MELEE_CONTACT_GU`, a gap of 8 su, the one that counts as Proximity
  (`tuning::orders::Orders::proximity_su`), and no MinRange (`defs_are_consistent` checks both). It is a placeholder (design
  change 0149): exactly 0 would be fragile, since a unit closing in may stop a little short of touching. `max_range`
  gives melee no ElevationModifier.
- **Hits** (`combat/damage.rs`): every attacker acts against one snapshot of the targets, and an attack's effect
  lands at the end of Firing as a `Hit` on the tick's list. The list is applied once all have acted, so every hit
  counts, including those on a unit another hit has killed and those of a unit that dies in the same tick.
  `hit_damage` follows `systems/combat.md > SingleTargetDamage` and `systems/combat.md > DamageModifierOrder` (pre-armor
  multipliers, PointArmor, directional multiplier, MinimumDamage); HP is fractional, nothing is rounded. PointArmor is
  the target's `ObjectDef::armor`, a structure's too: a Peacekeeper's 10 damage takes 9 HP off a Generator and 8 off a
  plate. An object at 0 HP or less dies; a structure that dies frees its tiles (the navigation grids are rebuilt).
- **Shields** (`shields.rs`, `combat/damage.rs`; `systems/combat.md > Shields`, slice M3-B, plan decision T18): an
  object whose kind has MaxShields (the Colonists') has `Object::shields`, full when it is inserted (and when a save
  from before Shields loads). A `Hit` carries the raw damage and the target's modifiers as the shot saw them
  (pre-armor multiplier, PointArmor, directional multiplier), and `damage::resolve` settles it when the list is
  applied, hit by hit, so the second hit of a tick meets the Shields the first left. The Shields take the damage
  first, at full value; an overflow goes through `hit_damage` with the MinimumDamage capped at it (`systems/combat.md >
  MinimumDamage > Shield overflow`: 10.25 on 10 Shields and PointArmor 1 takes 0.25 HP); a hit the Shields take whole
  costs no HP, and no armour is consulted. No modifier touches the Shields: the pre-armor multipliers and the
  DirectionalArmor touch the overflow only (design change 0147). `SimEvent::Hit` says what each hit took (`damage` of
  the HP, `shield_damage`). The Shields regenerate 0.25 a tick (`Shields::shield_regen_per_tick`; `systems/combat.md >
  ShieldRegenRate`: "continuously (not gated by a delay after taking damage)") up to the maximum now, a neutral object's
  too (design change 0146), in their own step (see *Tick pipeline*). A transforming object's follow its interpolated
  maximum as its HP do and regenerate up to it (design changes 0110 and 0146, `transform::follow_max`), and a cancel
  keeps their fraction, with no minimum (0111, `transform::keep_fraction`). The vision's records keep no Shields.
- **Energy** (`energy.rs`; slice M5-N4a, plan decision T75; `factions/cults/units/disciple.md > Energy`): an object
  whose kind has an energy pool (`ObjectDef::energy`, an `EnergyDef` in points as the design gives them: its maximum,
  its start and its regeneration, `EnergyRegen::PerSecond` or, for the Shrine (slice M5-N5), `PerUnitInside`: so
  many a second for each unit inside, everyone held counted, those on their way out too, a Placeholder) has
  `Object::energy`, a whole number of `energy::ONE`s (`RATIO_ONE`) a point, so that the design's rates add up exactly
  (0.5 a second is 2048 a tick, `energy::per_tick`: 50 to 200 in 4800 ticks, 300 s to the tick) and a spell's
  threshold compares exactly. It has its type's start as it comes into the game (`energy::start`, `Sim::new_object`)
  or gains a pool (`energy::after_change`), and regenerates in `Phase::Energy` (`energy::step`) up to its maximum,
  whoever owns it, a unit held inside a building too (Placeholder: "It regenerates naturally over time" names no
  exception). Hashed (tag 7) and saved only while set, so that a save from before loads with none and a game without
  a pool hashes as it did. Spending is the spells' (Shroud's drain, slice M5-N4b; the Shrine's casts, M5-N5;
  Mirage's cost, M6-CM). The client reads it in points (`energy::of`): the energy bar and the InfoPanel's line, through
  `Sim::energy_shown`, which for an illusion, which has no pool, is its parent's, live (design change 0452). E1
  (*Movement and collision > Invariants*) holds it within the pool.
- **Buffs** (`buffs.rs`, `defs/cults/abilities.rs`, `sim/casts.rs`; slice M5-N5, plan decisions T75 and T76;
  `factions/cults/structures/shrine.md > Abilities`): an area ability (`AreaAbility`, the Shrine's two, Q Health
  regeneration and W Energy regeneration; its numbers an `AbilityDef`: cost, radius, duration, the kinds it affects,
  its `BuffEffect`) is cast by a structure whose type lists it (`StructureDef::abilities`), at a point
  (`Command::Cast`, *Commands, events and replays*), and every unit of the caster's owner on the map whose centre is
  in its circle as the last tick left it (`buffs::in_circle`, the edge included: a Placeholder), of a kind it affects,
  gets its `Buff` until the cast's tick plus its duration (`Object::buffs`: one of an ability at most, a recast
  restarting it, so that effects never stack). It works every tick from its cast's on (*Tick pipeline*: Health
  regeneration's 0.125 HP a tick in the Buffs step, up to `Object::max_hp_with`; Energy regeneration's 4096 in the
  energy step, up to the MaxEnergy), wherever the unit goes, into a building too (a Placeholder), and the unit keeps it
  through a change of kind its ability still affects. Units inside a building or a Tunnel Network aren't in the area,
  and the buff stays when its caster is gone (a Placeholder). Hashed (tag 9) and saved only while a unit has one.
  (Slice M6-SN) `Buff::ability` is a `BuffKind`: an area ability's (`BuffKind::Area`), or a buff no cast gives, the
  Null Chamber exit buff's (`BuffKind::NullChamber`, *The Null Chamber*; `BuffKind::duration` and `affects` in place
  of an `AbilityDef`'s, `effect()` `None`); an area ability's hashes and saves as the ability did, so no state moved.
  (Slice M6-SB) So is the sludge's slow (`BuffKind::Sludge`, *The Vault and the Sludge*): held while a unit's centre
  is in a sludge patch and for 2 s after, its top speeds × 0.6 (`patches::speed_factor`).
  F1 (*Movement and collision > Invariants*) checks them. The client marks each buff a seen unit has beside its HP
  bar's place (`combat_effects/buffs.rs`, *Client architecture*).
- **Inspire and Suppress** (`buffs.rs`, `suppress.rs`, `defs/cults/units/prayer_mobile.rs`; slice M5-N11,
  `factions/cults/units/prayer_mobile.md`, design changes 0364 and 0365): the Prayer-mobile's two casts are area
  abilities too, which a unit casts at once from where it stands (`UnitDef::spells`, `SpellDef::Cast`: R and F,
  "instant, no target"; `buffs::casts`), its area taken from the caster as the last tick left it
  (`AbilityDef::area`: the Shrine's `AtPoint`, Inspire's `AroundCaster`, a circle of 4 grid units round it, and
  Suppress's `Ahead { arc }`, 90° ahead of its facing and 5 grid units long, `suppress::in_arc`, its bounds and edge
  inside: a Placeholder). The caster isn't in its own area. `Affects::Own(kinds)` takes in its owner's units of those
  kinds, Inspire's the four infantry kinds; `Affects::EveryUnit`, Suppress's, every unit of any owner, air units too,
  but no structure. **Inspire** (`BuffEffect::Inspire`) adds 1.5 su a tick to the unit's MaxSpeed on top of its
  Augmentations' (`buffs::max_speed_bonus`, in `Object::movement`), and gives a Recruit's own weapon its attack cycle
  × 0.75, to the nearest tick, the ticks saved taken off its FiringDuration (`buffs::inspired`, after
  `augmentation::augmented`: 12 → 9, a Firing of 8 → 5; an Improved Metabolism Recruit's 10 → 8): **Placeholder (not
  in the design: which phases)**, Firing being its longest phase, the one Improved Metabolism shortens too.
  **Suppress** (`BuffEffect::HoldFire`) is the status "can't attack" for 4 s: a unit that holds fire
  (`suppress::holds_fire`) starts no attack sequence in phase 5e, on the map or from inside a building, but moves,
  takes its orders, channels Shroud and builds as ever; a Rocketman's Target Lock goes on counting and stays locked,
  and its first missile's sequence starts after the status. **Placeholder (not in the design)**, an attack under way as
  it lands: its aim is cancelled, and a Firing, a Cooldown or a reload under way runs on, the Firing's hit landing (a
  Recruit's punch up to 7 ticks after the cast). A recast restarts either (one buff of an ability at most). Both are
  `Buff`s, hashed and saved only while set; their ends are the Buffs step's. Tests: `crates/sim/tests/prayer_mobile/`
  (`energy.rs`, `inspire.rs`, `suppress.rs`, `production.rs`, `saves.rs`), `suppress.rs`'s and `buffs/tests.rs`'s unit
  tests, and the golden `cults_prayer_mobile` (*Replays and golden hashes*).
- **Scan and cooldowns** (`cooldown.rs`, `sim/casts.rs`, `defs/gdo/structures/radar_station.rs`; slice M6-GR, plan
  decision T92; `factions/gdo/structures/radar_station.md > Placeholder stats`): the Radar Station's Scan is an area
  ability (`AreaAbility::Scan`; its numbers the GDO's rules', `GdoRules::scan`: radius 6, 8 seconds, a 60-second
  cooldown, free), cast at any point on the map (`AbilityArea::AnyPoint`: no sight needed), whose effect is sight
  (`BuffEffect::Sight`: no buff, its owner's scan, *Vision > Scans*). An ability with a cooldown
  (`AbilityDef::cooldown`, seconds) starts its caster's `Object::cooldown` as it is cast, a `Progress` of its frames,
  which the economy phase advances by the owner's Power ratio, the cast's tick the first, and drops once done
  (`cooldown::step`): a station that scans in tick `t` at full Power is refused through `t + 959`
  (`Reject::CoolingDown`) and scans again in `t + 960`; at 30/51 its cooldown takes 1632 ticks, 102 s (design change
  0423: "Everything a consumer does is slowed"). It replaces the plan's `cooldown_until`, a tick that the ratio
  couldn't slow. Hashed (tag 10) and saved only while set. Each station has its own: Ungroupable (design change 0422),
  each press goes to one. Tests: `crates/sim/tests/radar/` (`building.rs`, `scans.rs`, `cooldown.rs`, `power.rs`,
  `saves.rs`), `vision/scans.rs`'s unit test and the golden `gdo_defense`.
- **Powered weapons** (`combat/power.rs`; slice M6-GR, design change 0423): a structure that consumes Power and
  attacks, the Missile Turret, runs its whole attack cycle (Aim, Firing, Cooldown, Reload) at its owner's Power ratio.
  Each tick it banks the ratio (`AttackState::power_carry`, in `RATIO_ONE`s), and its phases take a tick only once a
  whole one is banked (`phases::Shooter::speed`, `power::spend`): at 30/51 the turret's 24-tick cycle takes 41 and its
  launch comes in tick 10 of a sequence, not 5. Each sequence starts with nothing banked, and a cancelled attack
  drops what was (`AttackState::cancel`); at full Power nothing is ever banked, so that every other weapon is as
  before. A consumer whose owner generates nothing never fires (ratio 0). `combat::step` reads the summaries only while
  such a weapon exists (`power::summaries`). Hashed (the attack state's tag 4) and saved only while not 0. **The
  Missile Turret** (`defs/gdo/structures/missile_turret.rs`) fires a homing missile, slice M5-K's HeadDisjointed
  (*HeadDisjointed and Target Lock* below; Universal, 18 damage, range 6, 16 su a frame; Aim 4, Fire 2, Cooldown 4,
  Reload 14), picking its targets as the Laser Turret does:
  `crates/sim/tests/defense_system/turret.rs` has it hit a circling Sparrow and a running Peacekeeper, never missing,
  from its first tick.
- **Weapon Conversion** (`conversion.rs`, `sim/modes.rs`; `factions/colonists/units/weapon_conversion.md`, plan
  decision T26, design change 0198): `Ability::Convert` flips the target form of each Legionnaire it goes to, a
  Sabre's to the Rifle Legionnaire and a Rifle's to the Sabre (`conversion::converts_to`): its type switches at the
  press, and the unit converts to it (*Object model > Type and form*). The gate drops the units without another form
  (`Reject::LacksAbility` when none has one) and takes every other, whatever it is doing: a press is never refused.
  `Object::conversion` holds the form the unit has (`Conversion::form`, whose weapon it fights with,
  `combat::weapon`) and the frames left of the conversion under way. **Pressed mid-attack** (design change 0153:
  "deferred, not rejected"), a unit Firing or in Cooldown fights on with its form's weapon
  (`conversion::waits_to_convert`, *Changes that wait for an attack* below), and its conversion begins in the first
  Modes step after its Cooldown's last tick; so one R converts a whole group in a fight, the mid-swing units 1 to 4
  frames later (a Sabre's Firing and Cooldown). **Pressed again** (`factions/colonists/units/weapon_conversion.md > A
  toggle sets a target form`): a unit that waits takes the new target, and doesn't convert at all if that is its form
  (`conversion::retarget`); one converting finishes, then converts back (8 frames for the round trip); one converted
  converts back. A conversion cancels an Aim or a Reload, keeping the target (design change 0152: the new weapon starts
  ready), and lasts `CONVERT_TICKS`, 4 frames: the unit holds still and doesn't turn (`may_move`, `may_turn`; its
  behaviours see it can't move; `Mover::speed_factor` 0, so it yields to nobody), its attack idles, and it can be
  attacked throughout (`factions/colonists/units/weapon_conversion.md > Vulnerability`). Pressed in tick `t`, it is of
  its new type from `t` on, converts through tick `t + 3`, and the Modes step of `t + 4` gives it its other form, HP and
  Shields unchanged (their maxima are equal), so it fights as that from `t + 4`. Nothing cancels a conversion under way
  (`factions/colonists/units/weapon_conversion.md > Completion`), a Stop changes nothing. The orders stay, and it
  carries on with the other weapon (`factions/colonists/units/weapon_conversion.md > Orders are kept`, design change
  0150). Convert isn't queued (design change 0151: "holding Shift changes nothing").
- **Changes that wait for an attack** (`sim/modes.rs`, design changes 0153 and 0172): a conversion or a raise of the
  Riot Shield whose unit is Firing or in Cooldown (`combat::mid_attack`: the phases that can't be interrupted) when it
  would begin waits, the unit's type switched already: the first Modes step after the Cooldown's last tick begins
  it, before the Reloading that followed runs a tick, so the unit acts as if pressed in that tick and the Reloading
  never runs (a raised shield's Reloading goes on behind it: the weapon is kept). No ability is stored: the type and
  the form's state say it (`conversion::waits_to_convert`, `ShieldMode::Deferred`). A lowering never waits (a shield
  off the ground has no shot to wait for), and one pressed while a raise waits drops it: the shield never starts
  rising (design change 0198).
- **Removal path**: `Sim::remove_objects` (`sim/removal.rs`) is the one way objects leave the game, for deaths and
  debug removals alike. It emits `Removed` and clears every reference to the removed: a current order naming one
  completes, queued ones are dropped, a move towards one ends `TargetLost`, an attack on one stops (an aim is
  cancelled, a shot being fired hits nothing). An order outlives its target by one rule (`Order::outlives_target`):
  an Attack, a Move to an object or a Gather on an object its holder's owner still remembers goes on to the
  last-known position (design change 0139: "nothing about the real state leaks"), and a Gather whose patch leaves
  heads for where the patch was (`Gather::spot`, see *Behaviours > Gathering*); the next tick's snapshot sees such a
  target Remembered until its place is seen, and Gone then (`sim/removal.rs::forget_seen_gone` ends what names it).
  `testing::dangling_references` checks that nothing else names a removed object. A Tunnel taken, whatever took it,
  leaves its expansions neutral, destroys its unfinished ones and strands the network units no remaining Tunnel lets
  out, once everything of the call has gone (`Sim::lost_tunnels` before, `Sim::lose_tunnels` after; slice M4-H,
  *Object model > Losing Tunnels*). The units inside a building that holds units come out alive inside its former
  footprint in the same way (`Sim::evacuations` first, before the refunds, `Sim::evacuate` once the building and
  every reference to it are gone; slices M5-F1 and M5-F3, *Object model > Buildings that hold units*).
- **Riot Shield** (`riot_shield.rs`; `factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`, slice
  M3-S): shield down and shield up are two unit types, `Peacekeeper` and `PeacekeeperShielded` (design change 0173),
  which carry the same `RiotShieldDef`, naming both; the Barracks trains the first, and the Unit Control, a sum over
  the objects, is the same. `Object::riot_shield` (`RiotShield { up }`) is how many ticks of the toggle time the
  shield is up (`RiotShieldDef::toggle_ticks`, 6, design change 0174); the type says which way it goes (*Object model
  > Type and form*), and `riot_shield::mode` tells it as the design does (`ShieldMode`: Lowered, Deferred, Raising,
  Raised, Lowering). `Command::Ability` switches the type at the press, a raise that waits for a shot included
  (design change 0198; `Sim::change_kind` from `sim/modes.rs`: the orders, the move, the HP and the whole attack
  kept), and the Modes step takes the shield a tick on (`riot_shield::step`), so a toggle in tick `t` is done at the
  end of tick `t + toggle_ticks − 1`; a reversal turns back from where the shield is (0171). A raise pressed during
  Firing or Cooldown waits for the shot, the shield down meanwhile (0172, `ShieldMode::Deferred`, *Changes that wait
  for an attack*), and a lowering pressed then drops it. From the start of a raise to the end of a lowering the unit
  moves at half speed (`Mover::speed_factor`, from how far up the shield is: `riot_shield::speed_factor`) and can't
  attack (`riot_shield::allows_attack(object)`, the one "may it attack now" check: the behaviours, 5e and
  auto-targeting ask it; the client draws the shield from `riot_shield::mode`). Which orders it takes follows its
  type (design change 0202, `riot_shield::takes_attack_orders(kind)`, which `combat::takes_attack_orders(kind)` joins
  to the weapon: the gate, the default command and the client's panel and right-click ask the latter): a Peacekeeper
  (Shielded) takes no Attack (the gate drops it from new Attack orders, `Reject::CannotAttack` when none is left;
  design change 0075; the client's Attack gives it a Move on its target instead, design change 0397), and a
  right-click on an enemy makes it follow (0074); so (slice M5-N4a) does a type without a
  weapon (`UnitDef::attack` none, the Disciple: "AttackAttributes - None"), which has no `attack` capability and so
  never auto-targets;
  a Peacekeeper whose shield is on its way down takes one, "accepted and held", and attacks once the shield is down.
  Meanwhile its behaviours see no attack (`Unit::attrs`: no auto-targeting, Hold disengages), and 5e cancels an
  aim (`can_attack_now` is false); a shot already being fired lands. **Orders are kept** (design change 0150): an
  Attack it had goes on hold, following its target as a Move to it would (`AttackOrder::on_hold`), and attacks again
  once the shield is down; an AttackMove or a Patrol only moves meanwhile (0076); the toggle never touches the queue,
  and Shift means nothing for it (0151). Only once the shield is all the way up is the damage it takes halved, a
  pre-armor multiplier (`riot_shield::damage_factor` in `damage::pre_armor_multiplier`, with the Rugged bonus; 0171:
  "a half-raised shield gives none"). A mode is not a task: the unit keeps its orders and yields like any other
  (design change 0023). The placeholders are listed in the module docs. The toggle doesn't use the conversion's
  machinery (`conversion.rs`), whose change runs to its end once begun and holds the unit still; they share the
  type-and-form model, `Sim::change_kind` at the press, the wait for a shot and the Modes step.
- **Projectiles** (`combat/projectiles.rs`, slice M4-O, plan decisions T45 and T48; `systems/combat.md >
  DoublyDisjointed`): a DoublyDisjointed attack (`Shooter::locks`) locks its target's position at the end of Aiming
  (`AttackPhase::FiringAt { at, target }`) and at the end of Firing launches a `Projectile` into `Sim::projectiles`, in
  launch order: its shooter and the shooter's owner, the object it was locked on (cleared when that object leaves:
  `projectiles::forget`; (M5-K) a homing one's flight, *HeadDisjointed and Target Lock* below), its start, the locked
  point, its arrival tick (`flight_ticks`: the distance over the attack's ProjectileSpeed, rounded up: 32 ticks for 4
  GU at 8 su per tick), and the attack's Damage, AoE radius and TargetDomain, copied at launch (slice M5-S8: the
  TargetDomain its blast hits, `area::blast_domain`, the Rocketman's the domain of what it was fired at). The unit goes
  on into Cooldown and Reloading, its base free from the launch (`base_is_free`). In 5e, after every attacker has acted,
  the due projectiles land in launch order (`projectiles::land`), whether their shooter lives or not: each a
  `SimEvent::Blast` and its area's hits, put on the tick's list before the single-target hits. The store is serialized
  and hashed only while it holds a projectile, and `AttackState::location` only while set, so a game without them
  hashes as before (the goldens unchanged). Its golden, a fight of test throwers (`combat::testing`), is a unit test
  (`combat/projectiles/golden.rs`, see *Replays and golden hashes*), and so is the ignored `projectiles_perf` (200 in
  flight). The game's throwers are the Grenadier (slice M4-U, `defs/syndicate/units/grenadier.rs`), played with its
  own numbers in `tests/grenadier.rs`, and (slice M5-S8) the Rocketman, whose missiles burst
  (`tests/rocketman/bursts.rs`); the mechanics' tests keep their test throwers, whose numbers stay put, among them a
  single-target dart (`combat::testing::DART`, the Rocketman's missile before design change 0445), since no type of
  the game fires a single-target projectile now.
- **TailDisjointed attacks** (`combat/phases.rs`, slice M5-O3; `systems/combat.md > TailDisjointed`, the Cults'
  Gunner's, design change 0304): locked as a DoublyDisjointed attack is (`Shooter::locks`, `AttackPhase::FiringAt`),
  at an object or a location, with no projectile. At the end of Firing it strikes the locked spot (`phases::strike`):
  a single-target attack hits what stands there then (`phases::at_the_spot`: a destructible object whose shape, a
  Silhouette or a footprint, covers the point, in a domain the attack can target, whoever owns it, seen or not), with
  its single-target hit and a `SimEvent::Attacked` drawn to the spot; with nothing there it misses
  (`SimEvent::Missed`), so a target that has left the spot dodges. Placeholder (not in the design): of two objects at
  the spot, the one aimed at while it covers it, else the first in id order. An area one (the Artillery Rig's, not
  built) blasts the spot as a landing projectile does. The base stays held through Firing and Cooldown, as for a
  FullyConnected attack. No unit walks off the spot in the Gunner's two frames of Firing: each one's Silhouette
  reaches further from its centre than it moves in two frames (the nearest, the Drone's: 18 su against 16). So its
  tests move the target with a debug teleport (`tests/gunner.rs`), and the dodge on foot is tested with a longer
  Firing (`combat/phases/tests/tail_disjointed.rs`).
- **Area damage** (`combat/area.rs`, plan decision T46; `systems/combat.md > AoEDamage`): the one path of every area
  attack, a blast's hits. A blast takes every object whose domain its TargetDomain hits (`hits_domain`: Ground hits
  ground and underground objects, Air the air, Universal all; design change 0262: underground and hidden ones too),
  that TargetDomain fixed as the attack fires by its def (slice M5-S8, `TargetType::AoE`'s `BlastDomain`,
  `area::blast_domain`): the attack's own for every area attack but the Rocketman's, whose missiles keep the domain
  of what they were fired at (`factions/gdo/units/rocketman.md > AttackAttributes (normal attack)`, design change
  0445: Air at an aircraft, Ground at anything else or the ground; Placeholder: Ground for a missile whose target
  left the game during its Firing, launched at the spot alone),
  whoever owns it, its attacker and the attacker's own objects included (design change 0293, answering Q0083 point
  1: friendly fire), by the exact overlap of its circle with the object's shape (`area::overlap`, a closed form
  through `math::asin` and `math::sqrt`, whose bits a test pins): a share of the Damage of overlap / circle, the
  FullArmor times overlap / shape, the directional zone from the blast's centre, a minimum of 0.5 × the overlap
  fraction (`Hit::minimum`), the Shields first at full value (`damage::resolve`). Nothing in it asks `hidden_from`.
  The reveal after the hits (see Vision) shows the attacker, if it still exists, to each damaged object's owner, when
  the damage lands: a dodged grenade reveals nothing (design change 0293, point 2).
- **Attack Ground** (`orders/attack_ground.rs`, plan decision T47; `systems/controls/command_behaviors.md >
  AttackingLocation`): `OrderKind::AttackGround(point)` sets `AttackState::location`; 5e aims at a point in reach
  (`reaches_point`, from the unit's shape) and fires. The behaviour walks the unit into range, and backs it off a point
  inside its MinRange by an Attack's own rule and code (`engage::too_close`, slice M5-O3; *Behaviours*), letting go of
  the location while it does; a unit that can't back off gives the order up with `SimEvent::LocationTooClose` ("Target
  too close"). A unit that can't turn in place stops only where the location will be in its arc at rest, and one in
  range that can't aim from where it stands repositions as for an Attack (slice M6-S6a, design change 0460:
  `aim::Mark::point`, `AttackGround::repositioning`; *Behaviours*).
  **It keeps firing** (design change 0293, answering Q0083 point 3): the location stays engaged after each shot (a
  launch, or a TailDisjointed strike: the Gunner's, slice M5-O3), so
  the attack cycles at it until another order or a Stop, and the orders queued behind it never start on their own;
  only the Failure rule, or a weapon that can no longer attack the location, ends it. The gate takes it only for units
  whose weapon can target the ground (`combat::can_target_ground`, `Reject::CannotTargetGround`).
- **Structures that attack** (`combat/structures.rs`, plan decision T49; slice M4-S1, the Laser Turret,
  `factions/syndicate/structures/laser_turret.md`): **`StructureDef::attack`** is the type's AttackAttributes
  (`combat::weapon_of` reads a unit's or a structure's), and **`phases::Shooter::new(map, object, attrs, sight,
  turned)`** takes the attack and the object, a unit's silhouette or a structure's footprint (`Shape::of`,
  `Elevation::of_object`), so a structure runs the units' phases and deals their damage, with no turn rate (it fires
  in any direction: `turn_left` infinite, it faces its target as a turret does) and no orders. In 5e, before its
  sequence, a complete structure (`Object::is_operational`: a site holds its fire, and is no threat) **picks its own
  target** (`structures::pick`; design change 0298 point 8, `factions/syndicate/structures/laser_turret.md >
  Targeting`: "targets like a unit holding position: BaseAutoTargeting within Range ..., then the closest ... It
  **keeps its target** while the target stays valid and in range"): an enemy's ValidTarget within Range, threatening
  targets first (`TargetView::threat`, `combat::threat`: an attack that can target the ground; a manned Garrison's
  occupants', M5-S2b), then the smallest gap, then the lowest id; none while auto-targeting is off. Its `facing` snaps
  to what it fires at
  (`structures::face`), for the client's barrel only. Its `AttackState` comes with its kind (`sim/kinds.rs`,
  `Capabilities`), so hashed and saved as a unit's; the goldens, which have no such structure, are as they were.
  **A pop-up fires only while raised** (slice M4-S2, `popup::may_fire`; "It can't fire while rising or retracting"):
  lowered, rising and retracting it picks and faces its target as ever and starts no sequence (an aim is cancelled, a
  reload runs on); **and only while active**, inside its Tunnel's area (design change 0360): an inactive one, a
  neutral one among them, stays down and never fires. Its armour is `Object::armor()`, the underground one while
  fully lowered (*Object model > The pop-up form*).
- **Units' turrets** (`combat/turret.rs`, `orders/turret.rs`; slice M5-U1, plan decision T80; `systems/combat.md >
  TurretSource`, `systems/controls/commands.md > TurretCommandState`): a unit whose type has TurretAttributes
  (`UnitDef::turret`) attacks from its turret (`turret::Source`). **`AttackState::turret: Option<TurretState
  { facing, locked }>`**: its facing relative to the base's, within ±TurnAngle/2, and its LockedTarget; none until it
  first locks or turns (a default one is a turret centred with nothing locked), so it is saved and hashed (a tag) only
  while set, and every state without one, the goldens', is as it was. `AttackState::target` stays what the base's
  behaviour engages; the turret's sequence runs at `locked` (`Shooter::aimed`). **In 5e**, before the phases
  (`turret::step`): a lock its attack can't target at all is let go, then the turret turns toward an Attack Ground's
  location or its LockedTarget, the latter only while its owner sees it (design change 0376, answering Q0137: "A
  turret turns only toward a lock its owner sees"), by its TurnRate, a full circle (`turret::full_turn`, within `Combat::facing_tolerance`
  of 360°) the shorter way, a limited arc only within it, to its edge toward a point outside; not when the tick
  starts in Firing or Cooldown (their TurretSourceActions are None). A sequence starts at a seen lock in range and
  in the arc (`turret::in_arc`, bounds included), Aiming lasts until the turret faces it too and is cancelled when
  it leaves the arc, and an Attack Ground's location takes the same arc. The base turns only in 5d: `base_is_free`
  holds throughout. **(M5-N10) A hull that aims too** (`turret::aims_by_hull`: a turret on a base that turns in place,
  the Hand of Heaven's TrackedVehicle, whose launcher turns 20° in all; `factions/cults/units/hand_of_heaven.md`:
  "stop and turn its hull"): its behaviour stops it in range wherever its target is (`Unit::takes_aim` in `close_in`
  and the Attack Ground) rather than approach, which heads where its path does and at a building's corner kept the
  target's centre out of the arc all the way in; while the target is outside the arc, it asks 5d to turn its hull
  toward it (`Unit::aim_hull`: `Mover::turn_to`, taken in the same tick, so neither saved nor hashed). 5d turns a
  unit at rest without a goal so (`movement::turn_in_place`, `Tracked::spin`): at its SpinRate, as far as its
  Silhouette stays free (`World::turn`); a spin cut short backs off, or creeps one tick's Acceleration toward the
  target, out of the contact (I5 and I7 allow it that), and an own idle unit that stopped it is asked to make way;
  the spin crushes as any spin in place does (M5-V4). (M6-S6a) One whose turn the tiles cut where it stands and
  wherever that creep would take it can't aim from where it stands, and repositions (design change 0454; *Behaviours*,
  `orders/aim.rs`). Only what its base engages turns its hull: a lock its turret
  keeps on its own (on Hold, on a Move) turns none. **(M6-SV) A base that can't turn in place with a narrow turret**
  (`engage::brakes_into_arc`: FixedTurnRadiusMovement and an arc short of a full turn, the Assault Bike's 40°, the
  Raptor's 60°) stops for a target in range only where the target will still be in its arc at rest
  (`engage::rests_in_arc`: the point the coast's braking along its facing brings it to, its facing as it is, since
  "Stopping + Turning: invalid"); until then `close_in` keeps driving it toward the target, turning. Stopping as soon
  as the target came into its arc, it coasted on straight, the target slipped out again, and it ended beside the
  target and gave the Attack up. A target inside a circle it turns on, or less than a grid unit beyond its rim, it
  reached only in contact, where it gave up or fired from within its own round's blast (question Q0184's readings):
  since slice M6-S6a it can't aim from where it stands, and repositions first, straight on (a vehicle backs up only on
  the Reverse order) until the target lies beyond by the margin and its weapon's MinRange, from where it comes round
  to rest beyond that (design change 0454; *Behaviours*). An Attack Ground's location takes the same rules (design
  change 0460, answering Q0184: under the literal rule the Bike circled a location in range far out of its arc for
  good, halting once a lap), and the Bike's mortar has a MinRange of a grid unit (0460), so that it never fires from
  within its own round's blast.
  **In 5a**, right after the base's behaviour, the turret's own (`orders/turret.rs::run`): it keeps
  a lock its base engages; any other it keeps only while it could fire at it from where it stands (`fireable`: seen,
  a valid domain, Range and MinRange, the arc; `systems/controls/commands.md > TurretCommandState > When a lock
  ends`, design change 0376), else lets it go; with nothing locked it scans (`World::turret_pick`: `World::pick` with `Scan::Weapon` and the arc as
  `admit`, a Move's tick included) and locks the best. The rotation a scan measures, the base's
  (`BaseAutoTargeting`) and the turret's alike, is from the turret's facing (`Unit::aim_facing`, design change
  0179). **Who sets the lock**: `close_in` once the target is in range and in the arc (out of the arc the approach
  goes on, the move turning the base; a hull that aims too, in range wherever the target is), the idle and the AttackMove acquisitions; a Hold's base engages nothing and
  leaves it to the turret's scan; a Stop clears it (`orders::run::start`; 0376: "**Stop clears the lock**"), and
  the turret scans in the same tick; every other order leaves it ("When the base behavior does not specify a target,
  the turret retains its previous TurretCommandState"; 0376: a Move keeps a lock from an Attack, a friendly one too); the removal path
  clears it (`AttackState::forget`); a kind change keeps it only for a type with a turret (`sim/kinds.rs`). **Not
  shared with the Laser Turret** (*Structures that attack*), which has no arc and no turn rate, snaps its facing and
  picks by the gap in 5e: its code and its golden (`cults_vs_syndicate`) are as they were. Tests on test turret units
  (`combat/turret/testing.rs`: the Guard on a Mech's base, a 90°, 45° or 360° turret, a fixed gun), in
  `combat/turret/tests.rs` and `orders/turret/tests.rs`; the ignored `turrets_perf` (`combat/turret/perf.rs`, 96
  turret units passing each other) runs as `projectiles_perf` does.
- **Units that fire from inside a building** (`combat/occupants.rs`, plan decision T72; slice M5-N2, the Garrison,
  `factions/cults/structures/garrison.md > Garrisoning`): **`StructureDef::occupants_fire: Option<OccupantFire {
  stand_in, range_bonus_gu, bonus_for }>`**, the Garrison's: the Recruit fires the Militant's AttackAttributes
  ((M6-CM) design change 0450: "**Disciples and illusions don't fire** from inside", so the Disciple, which has no
  weapon, has no stand-in either, and a Garrison holding only Disciples and illusions fires nothing and is no threat),
  the Militant and the Gunner their own (`combat::weapon`) with **+1 grid unit of Range**
  (`OccupantFire::weapon_kind`, `range_bonus`; `occupants::weapon` returns the attack as fired, owned, so the
  stand-in's is the Militant's def as it is: "Augmentations don't change the borrowed attack"). In 5e, after every
  object's own attack (`occupants::firing`, then `occupants::step`, a Garrison in operation from before the tick, its
  units by `(building, since, id)`), each unit inside runs its own `AttackState` (it keeps the one it had; one whose
  type has none would have one made for it, none since 0450) with a **`Shooter::occupant`**: the Garrison's
  footprint, elevation, owner and vision, firing any way as a structure does, its id the unit's (its hits and events
  name the unit). Its
  target is **picked as a structure's** (`structures::pick`, T49's rule reused: kept while valid and in range,
  threatening first, then the smallest gap, then the id; none while auto-targeting is off), so each picks its own.
  Range and MinRange are measured from the footprint, as a structure's are (`targeting::reaches` on its shape), so a
  Gunner's MinRange of 2 counts from the edge. **Its shots leave from the footprint's edge nearest the target**
  (`Shooter::muzzle`: the target's point clamped into the footprint; a unit's or a structure's own is its centre):
  `SimEvent::Attacked::from`, `Missed::from`, a projectile's start and the side directional armour is hit from.
  A hit's reveal (`vision.reveal`) names the Garrison (`occupants::building_of`), which the victim then sees. **No
  spells from inside** (Q0115): only the attack runs. **It is a threat while manned** (slice M5-S2b; design change
  0377, answering Q0139: "enemies' target scans treat the Garrison as a threatening target while any occupant could
  attack the scanner's domain", `systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection
  Priority` 1, the Militant stand-in included; "An empty Garrison, or one with no weapon that can reach the scanner's
  domain, is not a threat"): **`combat::Threats::of(objects, held)`**, one pass over the held units as each snapshot is
  taken, gives each building in operation whose units inside have an attack there the TargetDomain all of theirs
  cover (`occupants::domain_inside`: `occupants::weapon`'s, as they fire; Ground with Air makes Universal, which no
  building's units can mix today), and both snapshots add it to the building's own threat (`World::with_occupants` in
  5a, `Targets::with_occupants` in 5e), so a scan still reads one `threat` a target (tests:
  `combat/occupants/tests/threat.rs`, 0377's Verification among them, and `combat/structures/tests.rs`). Leaving clears
  the unit's attack (`Sim::put_on_map`, a made one gone). Tests: `combat/occupants/tests.rs` (the weapons, the muzzle)
  and `tests/garrison/fire.rs`, `tests/garrison/protection.rs` (0319's "A Garrison with 4 Militants fires at range 5":
  4.9 grid units off the footprint hit, 5.1 never; the stand-in at 4 with an augmented Recruit's blow unchanged;
  the Gunner's band from the edge; no fire while empty; (M6-CM) 0450's Verification, Disciples inside firing nothing
  and a Recruit with them the Militant's attack, illusions nothing, `tests/mirage/garrison.rs`; each its own target;
  no damage inside, a Grenadier's blasts on the footprint included).
  - **Measured** (`tests/garrisons_perf.rs`, ignored, release): ten full Garrisons, forty inside, under ten waves of
    30 Peacekeepers walking down the lane between their rows (all 300 killed, 5430 shots from inside): a mean of
    0.064 ms and a p99 of 0.12 ms a whole tick, the combat phase's 0.018 ms (0.007 ms and 0.003 ms with nothing in
    range) (load average 1.7). `structures_perf` and `holding_perf` (no Garrison in them) ran A/B against master
    (slice M5-N8's build), 12 interleaved rounds: the same games, the same timings within noise (medians:
    `structures_perf` 0.371 and 0.374 ms a whole tick, its combat phase 0.026 ms on either side; `holding_perf`
    cycling 0.036 and 0.037 ms, the evacuation's one tick 0.134 and 0.140 ms) (load average 1.5 to 2.6).
- **Shroud** (`shroud.rs`, `orders/shroud.rs`, `combat/damage.rs`; slice M5-N4b, plan decisions T76 and T77 as design
  change 0334 amends them; `factions/cults/units/disciple.md > Spells > Shroud`): the Disciple's channel is its order,
  **`OrderKind::Shroud`**, which the units whose type casts the spell take (`UnitDef::spells`, `SpellDef::Shroud`, its
  `ShroudDef`: the radius, the drain, the 10 energy it needs to start, which the gate checks, and the start of a
  queued one again: below it the order ends at once). The unit stands; each tick it pays the drain from its pool, its
  regeneration going on, and the channel ends when the pool can't pay, at a Stop or any other order, or with the unit
  (`SimEvent::ShroudStarted`, `ShroudEnded`). **`Sim::clouds`** (`shroud::Clouds`) keeps the clouds: each a connected
  component of the channelling units' circles (touching or overlapping), with its buffer and its timer, updated at the
  end of 5a and again at the end of the tick (*Tick pipeline*). (Slice M6-CM) **A channel's circle lingers**: a channel
  that ends, for any reason, its unit's death included, leaves its circle where it was for `Spells::linger` ticks, 24
  (design change 0448: "the cloud **stays for 1.5 s (24 frames, placeholder)** and keeps absorbing as usual"; a
  Placeholder), a member of its cloud as any other (`shroud::Member::until`, the tick it vanishes in, hashed under a
  sub-tag of its own only while set), which joins, splits, absorbs and releases as any; a channel whose circle moves
  under it (a debug teleport, new rules) goes on, its old circle replaced, not lingering. A cloud whose members change
  (a circle's vanishing, a split, a join, a new channel joining it) releases its buffer over its own circles as they
  were, and the clouds that follow start empty. In 5e, while there is a cloud, **`damage::absorb`** takes off the
  tick's list each hit on an object whose centre is under a cloud, by an anti-ground attack (`hits_domain(Ground)`:
  ground and underground
  targets; an air unit's hits stay, slice M5-H1's to test), whose attack doesn't leave from under the same cloud
  (`damage::Origins`, where each attack left, kept only while there is a cloud; Placeholder: a projectile's launch
  point, judged as it lands), into the cloud's buffer at its raw damage; the first starts the 8-tick timer
  (`tuning::spells::Spells::release_delay`). An absorbed hit still reveals its attacker to the owner of the object it was
  aimed at, as if it had landed (design change 0386, answering Q0143: "whether a direct shot, a projectile or an area
  hit"). Then **`damage::release`** turns each buffer due into one area hit: `SimEvent::ShroudReleased`, which every
  player is told of, then a `Hit` for each object under the cloud by its overlap share of the cloud's area (counted on
  a lattice, `shroud::lattice`), under the AoE rules (`area::share`) but without directional armour; the share over
  empty cloud is lost, and a release reveals no one (0386: "The cloud's later release reveals nobody"; its attacker
  is the cloud's first member). Hashed under tag 3 of `Sim::state_hash` and saved only while there is a cloud or a
  buffer waiting, so that a game without Shroud hashes as before; C1 (*Movement and collision > Invariants*) holds the
  clouds to the channels. Without a cloud none of it runs. Tests: `tests/shroud/` (absorption, the clouds, the
  channel, (M6-CM) the linger after a Stop and after a death, 0448's Verification step 8, their state through splits
  and joins, a replay and a save, and what an absorbed hit and a release reveal) and the golden `cults_shroud`
  (*Replays and golden hashes*; slice M6-CM's linger moved it, and only it: with `Spells::linger` at 0 it gives its
  old hashes); a 3x3 building wholly under a circle, which no Disciple can stand in, is a unit test's
  (`combat/damage/tests.rs`). The client draws the clouds (`shroud_view`, *Client architecture > Visuals*), a
  lingering circle as any, (M6-CM) the channelling stance (`shroud_view::stance`), and puts the spell in the
  Disciple's R.
  - **Measured** (`tests/shroud_perf.rs`, ignored, release): eight Disciples, four clouds of two circles, over sixty
    Militants under the fire of forty (711 ticks, 236 releases, 3540 hits, none killed): over six runs, a mean of
    0.032 to 0.047 ms and a p99 of 0.067 to 0.10 ms a whole tick, the combat phase's 0.010 to 0.014 ms (p99 0.04 to
    0.06 ms); the same game without the clouds 0.014 ms (p99 0.045 ms), its combat phase 0.002 ms (load average 0.8 to
    1.0). Without a cloud there is nothing to measure: `structures_perf` and `movement_perf` ran A/B against master
    (slice M5-S2b's build), interleaved, 12 and 8 rounds: medians of 0.358 and 0.360 ms a whole tick for
    `structures_perf` (its combat phase 0.026 ms on either side), 0.475 and 0.472 ms for `movement_perf` (load
    average 0.9 to 1.0).
- **Mirage** (`mirage.rs`, `sim/mirage.rs`, `orders/run.rs`, `orders/shroud.rs`; slice M6-CM, design change 0448 with
  0450's and 0452's; `factions/cults/units/disciple.md > Spells > Mirage`): the Disciple's second spell
  (`SpellDef::Mirage`, its `MirageDef`: the cost, 50; the illusions, 4; their offset, 1 grid unit; the damage they
  take, twice), an **`Ability::Mirage`** the gate takes of a unit of a type that casts it with the energy
  (`Reject::NotEnoughEnergy`) and no illusion (`Reject::IllusionsCantCast`, which refuses an illusion's Shroud too).
  **`Sim::cast_mirage`** casts it at once in the commands phase: it spends the cost, ends a Shroud channel under way
  (its circle lingers), removes the old set and makes the new one north, east, south and west of the Disciple
  (`mirage::compass`), a blocked spot's illusion at the first free point of the rings round it (Placeholder:
  `Spells::mirage_search_step_su`, 8, apart out to `mirage_search_radius_gu`, 3; none made without one), each a new
  object of the Disciple's kind and owner, with its HP, its Augmentations (so its MaxHP and its speed) and its facing,
  idle, with no energy, and **`Object::illusion_of`** the Disciple's id (`SimEvent::Mirage`, told to its owner only,
  after the illusions' `Spawned`). **An illusion is a Disciple**: its kind is its parent's, so every path that shows
  an enemy a Disciple (the map, the InfoPanel, the computer players' `View`, every other event) shows one of an
  illusion with no masking code, and what the link makes of it is read where each rule is: `mirage::damage_taken`
  doubles the HP each hit takes, a release's share too (`combat::damage::apply`); `recruitment::Lineages` gives it no
  Unit Control; it has no `Object::energy`, and `Sim::energy_shown` is its parent's, wherever the parent is (0452);
  the Altar, a building that augments, doesn't take it in (`mirage::admitted`: the gate's `Reject::NotAccepted`, the
  right-click's `Aimed::refuses_illusions`); a Garrison does, and it fires nothing there, as a Disciple doesn't (0450);
  a buff affects it as a Disciple. **Mirroring** (`orders::run::update`): in 5a, its parent having had its turn (an
  illusion's id is above its parent's), an illusion whose parent's channel started in this phase takes a Shroud order
  of its own in place of its orders ("An illusion that is moving or busy when the parent starts channelling stops
  and plays the animation too"): it stands as a channel does, drains nothing and casts no circle (`shroud::member`
  leaves it out), completes once its parent channels no more, and ends at a Stop or an order of its own. Placeholder:
  an illusion made in the tick its parent's channel starts doesn't mirror it. **How illusions end**: by death, as any
  unit; with the parent, whose removal (`Sim::remove_objects`, `Sim::remove_held`, `Sim::illusions_of`) takes them
  along, those held too, each with a `Removed` and no death; and at a recast. M1 (*Movement and collision >
  Invariants*) holds the links. Hashed under tag 15 and saved only while set, so that a game without illusions hashes
  as before. Tests: `tests/mirage/` (0448's Verification but step 8, which is `tests/shroud/linger.rs`'s; 0450's
  step 3; 0452's three steps; the Altar, the Shrine, a save and a replay), `tests/garrison/fire.rs` (0450's first two
  steps), the golden `cults_mirage` (a cast, two illusions dying of double damage, a recast, the parent's death
  taking its new set along), the fuzz's Mirages, the soak's (each player's first Disciple casts one,
  `tests/pairings/soak/fight.rs`) and the structure test (`ruleset/tests.rs`). The client puts the spell in the
  Disciple's F (`PanelCommand::Mirage`); its owner sees an illusion washed out, on the map (`MeshKey::illusion`,
  keyed on the local player) and in the SelectionPanel's portraits, and named "Disciple (illusion)", its R and F
  greyed ("Illusions can't cast"), and every other player a
  Disciple (`crates/game/tests/commands/mirage.rs`; the computer players', `crates/ai/src/view/tests.rs`).
- **HeadDisjointed and Target Lock** (`combat/projectiles/homing.rs`, `combat/target_lock.rs`,
  `orders/target_lock.rs`; slice M5-K, design change 0356; `systems/combat.md > HeadDisjointed`,
  `factions/gdo/units/rocketman.md > Target Lock`): a HeadDisjointed attack launches a projectile of the store with a
  flight of its own (`Projectile::homing`: where it is and was), which in each later phase 5e flies its
  ProjectileSpeed at its target's centre and lands on its target as it reaches it, within a step; it can't miss, and
  vision plays no part. Placeholder: one whose target leaves the game flies on to the target's last spot and hits
  nothing (slice M5-S8: the Rocketman's, area damage since design change 0445, bursts there in its target's domain,
  harmlessly or on what of that domain stands there). The Rocketman's Target Lock is an order,
  **`OrderKind::TargetLock`** (by plan decision T76's Shroud), which begins **`AttackState::target_lock`**: in 5e the
  lock counts LockOnDuration (48 ticks) while the unit stands, never yields and fires at nothing, then fires the unit's
  weapon at its target homing, from anywhere within LockRange (7 GU, edge to edge) and inside its MinRange
  (`target_lock::attrs`). It breaks when its target leaves LockRange or its owner's sight, at the start of an order
  given without Shift, or with its target, and the order completes in the next 5a; a Shift-queued order waits behind the
  lock and runs when it ends (design change 0438, answering Q0171). Placeholders: no walk into range (a target beyond
  LockRange or unseen breaks it at once); a locked missile whose Firing began is launched homing, whatever became of the
  lock. The gate takes it for a unit whose type has one, on an enemy object the player sees (`Reject::NotAnEnemy`
  otherwise). `TargetLock` and `Projectile::homing` are serialized and hashed only while set, so the goldens before them
  are unchanged. Reading: the Rocketman's SightRange of 6 is shorter than its LockRange of 7, so without other eyes its
  lock breaks on sight first; a strafing Sparrow's loop leaves the sight of a lone group of Rocketmen on every pass,
  and none of their locks completes (`tests/rocketman/anti_air.rs`).
