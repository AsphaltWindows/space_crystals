//! (M6-GL) Propulsion Labs and the Dragonfly (`factions/gdo/structures/propulsion_labs.md`,
//! `factions/gdo/units/dragonfly.md`; `systems/mechanics/aircraft_landing.md > Assigned base`; design changes
//! 0366, 0428, 0435 and 0457; plan decision T88), through the gate. One test binary:
//! - `training`: the Deployment Center's Propulsion Labs behind an Airfield, and the Airfield's W, a Dragonfly behind
//!   Propulsion Labs for 175 Space Crystals and 4 Supplies in 320 frames; a new one appears landed on a free slot,
//!   doesn't move there, and takes off straight up for its rally point, leaving at IdleSpeed;
//! - `missile`: its one homing missile, fired along its heading, shoots down a Sparrow; out of it, the Dragonfly flies
//!   home by itself, slows to IdleSpeed over its slot, lands in 16 frames, rearms in 64 and lifts off straight up
//!   (design change 0428's Verification); Hornets and Dragonflies fire nothing on a Move and engage on an Attack Move,
//!   and a Dragonfly launches at a circling Sparrow within two passes (design change 0470's Verification);
//! - `approach`: design change 0428 point 5, its vertical landing: it approaches its slot at IdleSpeed and stops over
//!   it while it lands, never stopping anywhere else in the air, from any heading, three at once among them;
//! - `landing`: `tests/landing`'s rules for a Dragonfly: an air target landing and taking off, a ground target down
//!   (design change 0428 point 1); landed or landing, it dies with its Airfield, and taking off it escapes (point 3);
//!   an order during its rearm lifts it off with the rounds rearmed so far (point 4);
//! - `separation`: two Dragonflies end their SeparationRadius of 48 su apart, pushed at a Glider's third of the
//!   HoverCraft strength (design changes 0405 and 0412).
//!
//! Auto-targeting is off (`TestSim::without_auto_targeting`), so that only orders fight.

mod approach;
mod landing;
mod missile;
mod separation;
mod training;

use sc_sim::landing::Landing;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    AttackPhase, Command, DebugCommand, Mover, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, Sim,
    SimEvent, SpawnAt, Target, Tick, TilePos, Vec2, math,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DRAGONFLY: ObjectKind = ObjectKind::Dragonfly;

/// `factions/gdo/units/dragonfly.md > Placeholder stats`: "IdleSpeed 6, MaxSpeed **14 su/frame**, Acceleration 0.6";
/// "Ammo: **1 missile**"; "**RearmTimePerAmmo 64 frames**".
const IDLE: f32 = 6.0;
const MAX: f32 = 14.0;
const ACCELERATION: f32 = 0.6;
const AMMO: u32 = 1;
const REARM: Tick = 64;
/// `systems/mechanics/aircraft_landing.md > Assigned base`: "Landing and take-off: **16 frames (1 s)** each
/// (placeholder)".
const LANDING: Tick = 16;
/// The rounding a speed may carry (a Glider's flight is its facing's unit vector times its speed).
const SPEED_TOLERANCE: f32 = 1e-3;

// The objects of [`field`], by id: 1 player 1's Deployment Center on tiles (2, 2) to (5, 5) (Power +30), 2 and 3 its
// Generators on tiles (8, 2) and (8, 5) (+20 each), 4 and 5 its Airfields [`A`] and [`B`] (−30 each), 6 player 2's
// Deployment Center, far off on tiles (54, 34) to (57, 37). Player 1's Power is at full speed.
/// Player 1's Airfields: A on tiles (12, 3) to (14, 5), B on tiles (12, 12) to (14, 14); and their centres.
const A: u32 = 4;
const B: u32 = 5;
const A_AT: Vec2 = Vec2::new(864.0, 288.0);
const B_AT: Vec2 = Vec2::new(864.0, 864.0);
/// [`A`]'s and [`B`]'s landing slots ([`slots`]).
const A_SLOTS: [Vec2; 4] = slots(A_AT);
const B_SLOTS: [Vec2; 4] = slots(B_AT);

/// The landing slots of an Airfield centred on `centre`, in the order they are taken: the centres of its footprint's
/// quarters, north-west, north-east, south-west and south-east (**placeholder**: the slots' layout,
/// `sc_sim::landing::slots`).
const fn slots(centre: Vec2) -> [Vec2; 4] {
    [
        Vec2::new(centre.x - 48.0, centre.y - 48.0),
        Vec2::new(centre.x + 48.0, centre.y - 48.0),
        Vec2::new(centre.x - 48.0, centre.y + 48.0),
        Vec2::new(centre.x + 48.0, centre.y + 48.0),
    ]
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn at(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

/// Both players play the GDO on a flat map of 60 × 40 tiles, player 1 with two Airfields, [`A`] and [`B`], and the
/// power for both, and `more` after them (from id 7); auto-targeting off; tick 1 has run.
fn field(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(60, 40))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 5))
        .spawn(ObjectKind::Airfield, Some(P1), tile(12, 3))
        .spawn(ObjectKind::Airfield, Some(P1), tile(12, 12))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(54, 34));
    let mut t = more(builder).start();
    t.without_auto_targeting();
    t.run_until(1);
    assert_eq!((object(t.sim(), A).pos, object(t.sim(), B).pos), (A_AT, B_AT));
    t
}

/// Player 1's bank from the next tick on, which runs: `crystals` Space Crystals and `supplies` Supplies.
fn bank(t: &mut TestSim, crystals: u32, supplies: u32) {
    debug(
        t,
        DebugCommand::SetResources {
            player: P1,
            crystals,
            supplies,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(sim: &Sim, player: PlayerId, command: &Command) -> Result<(), Reject> {
    sim.validate(&PlayerCommand {
        player,
        command: command.clone(),
    })
}

/// `player`'s `command` in the next tick, which runs; the gate takes it.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    assert_eq!(gate(t.sim(), player, &command), Ok(()), "{command:?}");
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    t.step();
}

/// The debug `command` in the next tick, which runs.
fn debug(t: &mut TestSim, command: DebugCommand) {
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, command);
    t.step();
}

fn order(units: &[u32], kind: OrderKind) -> Command {
    Command::Order {
        units: units.iter().copied().map(id).collect(),
        kind,
        queue: false,
    }
}

fn move_to(units: &[u32], to: Vec2) -> Command {
    order(units, OrderKind::Move(Target::Point(to)))
}

/// The right-click of `units` on `target` (`Sim::default_order`, the first unit's, as the client sends it).
fn right_click(sim: &Sim, units: &[u32], target: Target) -> Command {
    let kind = sim.default_order(id(units[0]), target).expect("a default order");
    order(units, kind)
}

fn object(sim: &Sim, n: u32) -> &sc_sim::Object {
    sim.object(id(n)).unwrap_or_else(|| panic!("#{n} is on the map"))
}

fn mover(sim: &Sim, n: u32) -> &Mover {
    object(sim, n).mover.as_ref().expect("a unit")
}

/// Unit `n`'s speed: its own flight's (`Mover::momentum`), which a Glider carries from tick to tick.
fn speed(sim: &Sim, n: u32) -> f32 {
    mover(sim, n).momentum.length()
}

/// Aircraft `n`'s ammo.
fn ammo(sim: &Sim, n: u32) -> u32 {
    object(sim, n).aircraft.expect("an aircraft").ammo
}

/// Aircraft `n`'s base, by number.
fn base(sim: &Sim, n: u32) -> Option<u32> {
    object(sim, n).aircraft.expect("an aircraft").base.map(|b| b.0)
}

/// Aircraft `n`'s landing, if it is landing or down on a slot.
fn landing_of(sim: &Sim, n: u32) -> Option<Landing> {
    sc_sim::landing::of(object(sim, n))
}

/// Whether aircraft `n` is down on a slot (`sc_sim::landing::down`).
fn down(sim: &Sim, n: u32) -> bool {
    sc_sim::landing::down(object(sim, n))
}

/// The kind of what unit `n` is doing: its current order's, else its first queued one's.
fn doing(sim: &Sim, n: u32) -> Option<OrderKind> {
    let orders = object(sim, n).orders.as_ref().expect("orders");
    (orders.current.as_ref().map(|active| &active.order))
        .or(orders.queue.front())
        .map(|order| order.kind)
}

/// Whether aircraft `n` is home: down on a slot, rearmed and its flight home over, idle.
fn home(sim: &Sim, n: u32) -> bool {
    landing_of(sim, n).is_some_and(|landing| landing.descent.done() && landing.rearm.is_none())
        && ammo(sim, n) == AMMO
        && doing(sim, n).is_none()
}

/// Steps until `done` holds, at most `ticks` ticks.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&Sim) -> bool) {
    let until = t.sim().tick() + ticks;
    while !done(t.sim()) {
        assert!(t.sim().tick() < until, "{what}: not by tick {until}");
        t.step();
    }
}

/// `points.len()` Dragonflies of player 1 spawned in the next tick at `points`, ids from `first` on, assigned to
/// `airfield` by a right-click on it in the tick after (`OrderKind::Rebase`), which flies them there to land; stepped
/// until all are down, and home, each never stopping in the air on the way ([`fly_down`]'s rule).
fn landed_dragonflies(t: &mut TestSim, first: u32, points: &[Vec2], airfield: u32) -> Vec<u32> {
    let ids: Vec<u32> = (first..first + points.len() as u32).collect();
    let tick = t.sim().tick() + 1;
    for &pos in points {
        t.spawn_unit_at(tick, DRAGONFLY, P1, pos);
    }
    t.step();
    let click = right_click(t.sim(), &ids, Target::Object(id(airfield)));
    assert_eq!(click, order(&ids, OrderKind::Rebase(id(airfield))));
    next(t, P1, click);
    let until = t.sim().tick() + 600;
    while !ids.iter().all(|&n| home(t.sim(), n)) {
        assert!(t.sim().tick() < until, "landed: not by tick {until}");
        t.step();
        for &n in &ids {
            flies_or_is_held(frame(t.sim(), n));
        }
    }
    for &n in &ids {
        assert_eq!(base(t.sim(), n), Some(airfield));
    }
    ids
}

/// Dragonfly `n` of player 1, spawned south of [`A`] and landed on its first slot, home ([`landed_dragonflies`]).
fn one_landed(t: &mut TestSim, n: u32) {
    landed_dragonflies(t, n, &[Vec2::new(864.0, 900.0)], A);
    assert_eq!(t.pos(n), A_SLOTS[0]);
}

/// The missiles launched so far by `shooter`: (tick, target).
fn launches(t: &TestSim, shooter: u32) -> Vec<(Tick, Option<ObjectId>)> {
    let launch = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::ProjectileLaunched {
            shooter: by, target, ..
        } if by == id(shooter) => Some((*tick, target)),
        _ => None,
    };
    t.events().iter().filter_map(launch).collect()
}

/// The ticks of `attacker`'s hits on `target` so far.
fn hit_ticks(t: &TestSim, attacker: u32, target: u32) -> Vec<Tick> {
    let hit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit {
            attacker: by,
            target: on,
            ..
        } if by == id(attacker) && on == id(target) => Some(*tick),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

/// The tick object `n` died, if it has (`SimEvent::Died`).
fn died(t: &TestSim, n: u32) -> Option<Tick> {
    (t.events().iter())
        .find_map(|(tick, e)| matches!(e, SimEvent::Died { id: dead, .. } if *dead == id(n)).then_some(*tick))
}

/// Dragonfly `n`, landed, shoots down air target `target` on an Attack given in the next tick: it lifts off, flies at
/// it and fires its one missile as its heading faces it (`Combat::facing_tolerance`: its attack comes from its base,
/// which only its flight turns), each tick of its Aiming's end checked; the missile hits, and the target dies. Returns
/// the game in the tick of the hit.
fn shoots_down(t: &mut TestSim, n: u32, target: u32) {
    next(t, P1, order(&[n], OrderKind::Attack(id(target))));
    let tolerance = sc_sim::rules().tuning.combat.facing_tolerance;
    let phase = |sim: &Sim| object(sim, n).attack.as_ref().and_then(|attack| attack.phase);
    let mut before = phase(t.sim());
    let until = t.sim().tick() + 400;
    while hit_ticks(t, n, target).is_empty() {
        assert!(t.sim().tick() < until, "#{n}'s hit: not by tick {until}");
        t.step();
        flies_or_is_held(frame(t.sim(), n));
        let now = phase(t.sim());
        if matches!(before, Some(AttackPhase::Aiming { .. })) && matches!(now, Some(AttackPhase::Firing { .. })) {
            let (o, aimed) = (object(t.sim(), n), object(t.sim(), target));
            let off = math::wrap_angle(math::vector_angle(aimed.pos - o.pos) - o.facing).abs();
            assert!(off <= tolerance + 1e-4, "it fires along its heading: {off} rad off");
        }
        before = now;
    }
    assert_eq!(launches(t, n).len(), 1, "one missile");
    assert_eq!(launches(t, n)[0].1, Some(id(target)));
    assert_eq!(died(t, target), Some(t.sim().tick()), "it shot #{target} down");
}

/// Dragonfly `n`'s flight in a tick, as the tick left it: where it was, its facing and its own flight's speed, whether
/// it was on a slot (landing or down) and down, whether it rose, and whether it was over the point of its landing
/// approach (`Mover::is_over`).
#[derive(Copy, Clone, Debug)]
struct Frame {
    tick: Tick,
    pos: Vec2,
    facing: f32,
    speed: f32,
    on_slot: bool,
    down: bool,
    rising: bool,
    over: bool,
}

fn frame(sim: &Sim, n: u32) -> Frame {
    let mover = mover(sim, n);
    Frame {
        tick: sim.tick(),
        pos: object(sim, n).pos,
        facing: object(sim, n).facing,
        speed: mover.momentum.length(),
        on_slot: landing_of(sim, n).is_some(),
        down: down(sim, n),
        rising: mover.rise.is_some(),
        over: mover.is_over(),
    }
}

/// "It never stops in the air" (`m6-plan.md > GL > Acceptance`): a Dragonfly flies at a speed between its IdleSpeed and
/// its MaxSpeed, unless its landing holds it still, with no speed: on a slot, rising off one, or over the point of its
/// approach (`movement::approach`), the tick it stops there aside, which keeps the speed it came in at.
fn flies_or_is_held(f: Frame) {
    let held = f.on_slot || f.rising || f.over;
    let flies = (IDLE - SPEED_TOLERANCE..=MAX + SPEED_TOLERANCE).contains(&f.speed);
    assert!(flies || (held && f.speed == 0.0), "stopped in the air: {f:?}");
}

/// Steps until Dragonfly `n` is down on a slot, at most `ticks` ticks: its frames, each by [`flies_or_is_held`]'s rule.
fn fly_down(t: &mut TestSim, n: u32, ticks: Tick) -> Vec<Frame> {
    let until = t.sim().tick() + ticks;
    let mut frames = Vec::new();
    while !down(t.sim(), n) {
        assert!(t.sim().tick() < until, "#{n} down: not by tick {until}");
        t.step();
        let f = frame(t.sim(), n);
        flies_or_is_held(f);
        frames.push(f);
    }
    frames
}

/// The landing that `frames` end with ([`fly_down`]'s), on `slot` (`factions/gdo/structures/airfield.md > Returning
/// and rearming`: "it approaches its slot at IdleSpeed, stops over it while it lands (16 frames)"): its approach's last
/// `idle` ticks at IdleSpeed, and its arrival right on the slot at IdleSpeed, where it stops in the air; its landing
/// begins in the next tick, and it holds still there, with no speed, until it is down, 16 ticks on. Returns the
/// arrival's frame.
fn lands_on(frames: &[Frame], slot: Vec2, idle: usize) -> Frame {
    let arrival = frames.iter().position(|f| f.over).expect("it stops over its slot");
    let over = frames[arrival];
    assert_eq!(over.pos, slot, "over its slot: {over:?}");
    assert!(
        (over.speed - IDLE).abs() <= SPEED_TOLERANCE,
        "come in at IdleSpeed: {over:?}"
    );
    assert!(arrival >= idle, "{arrival} ticks of flight before it");
    for f in &frames[arrival - idle..arrival] {
        assert!(!f.on_slot && !f.rising && !f.over, "{f:?}");
        assert!(
            (f.speed - IDLE).abs() <= SPEED_TOLERANCE,
            "the last of its approach at IdleSpeed: {f:?}"
        );
    }
    let landing = &frames[arrival + 1..];
    assert_eq!(
        landing.len(),
        LANDING as usize + 1,
        "landing from the tick after: {landing:?}"
    );
    for (i, f) in landing.iter().enumerate() {
        assert_eq!((f.pos, f.speed), (slot, 0.0), "still on its slot: {f:?}");
        assert!(f.on_slot && f.down == (i == LANDING as usize), "{f:?}");
    }
    assert_eq!(
        landing[LANDING as usize].tick,
        over.tick + 1 + LANDING,
        "down 16 ticks on"
    );
    over
}

/// Dragonfly `n`'s take-off off `slot`, which began in this tick: it rises straight up, held still on the slot's spot
/// with no speed for its rise's first 15 ticks, an air target, and flies in the rise's last, from IdleSpeed: "takes off
/// straight up, leaving at IdleSpeed" (`systems/mechanics/aircraft_landing.md > Assigned base`; `Mover::rise`,
/// slice M6-SD's, as the Hornet's). Returns the game in the tick of its first flight.
fn rises_off(t: &mut TestSim, n: u32, slot: Vec2) {
    assert_eq!(landing_of(t.sim(), n), None, "off its slot");
    for rise in 1..LANDING {
        let f = frame(t.sim(), n);
        assert!(f.rising, "{rise}: {f:?}");
        assert_eq!((f.pos, f.speed), (slot, 0.0), "{rise}: rising straight up");
        t.step();
    }
    let f = frame(t.sim(), n);
    assert!(!f.rising && f.pos != slot, "it flies: {f:?}");
    assert!(
        (IDLE..=IDLE + ACCELERATION + SPEED_TOLERANCE).contains(&f.speed),
        "from IdleSpeed: {f:?}"
    );
}

/// Dragonfly `n`'s take-off on `command`, given in the next tick, off the slot it is on ([`rises_off`]).
fn takes_off(t: &mut TestSim, n: u32, command: Command) {
    let slot = t.pos(n);
    next(t, P1, command);
    rises_off(t, n, slot);
}
