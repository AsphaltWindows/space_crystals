//! (M6-GH2) Landing and rearming at the Airfield (`sc_sim::landing`, plan decision T88;
//! `factions/gdo/structures/airfield.md > Aircraft Support`, `systems/mechanics/aircraft_landing.md > Assigned base`;
//! design changes 0428 and 0423), and the Hornet's burst (design change 0458), through the gate. One test binary:
//! - `slots`: four Hornets out of ammo land on the Airfield's 4 slots, each in 16 frames, holding still, and rearm
//!   there; F from anywhere, or a right-click on its own Airfield, flies one home to land and rearm; one landed there
//!   already stays; a right-click on another own Airfield reassigns it, and it lands there;
//! - `domains`: a Hornet landing or taking off is an air target, one down on its slot a ground target (design change
//!   0428 point 1), for the weapons, the gate's `Reject::CannotHit` and the blasts;
//! - `rearm`: RearmTimePerAmmo × the missing ammo, landed after; a Move lifts it off in 16 frames; an order during the
//!   rearm takes it off with the ammo rearmed so far (0428 point 4), and so does one Shift-queued behind its F, as it
//!   lands (design change 0466); F alone rearms it fully, idle, landed; low Power slows it (design change 0423);
//! - `destroyed`: the aircraft landed on a destroyed Airfield, down or still landing, die with it; one taking off
//!   escapes, unassigned (0428 point 3);
//! - `burst`: design change 0458's Verification, the Hornet's missile bursting at the spot it flies to.
//!
//! A new aircraft's appearance on a free slot (0428 point 2) is `tests/airfield`'s (`training`). Auto-targeting is off
//! (`TestSim::without_auto_targeting`), so that only orders fight.

mod burst;
mod destroyed;
mod domains;
mod rearm;
mod slots;

use sc_sim::landing::{self, Landing};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, Sim, SimEvent, SpawnAt,
    Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HORNET: ObjectKind = ObjectKind::Hornet;

/// `factions/gdo/units/hornet.md > AttackAttributes`: "Ammo: **5 missiles**"; "**RearmTimePerAmmo 32 frames** (5
/// missiles in 10 s)".
const AMMO: u32 = 5;
const REARM: Tick = 32;
/// `factions/gdo/structures/airfield.md > Aircraft Support`: "Landing and take-off: **16 frames (1 s)** each
/// (placeholder)".
const LANDING: Tick = 16;

// The objects of [`field`], by id: 1 player 1's Deployment Center on tiles (2, 2) to (5, 5) (Power +30), 2 and 3 its
// Generators on tiles (8, 2) and (8, 5) (+20 each), 4 and 5 its Airfields (−30 each), [`A`] and another on tiles
// (12, 12) to (14, 14), 6 player 2's Deployment Center, far off on tiles (54, 34) to (57, 37). Player 1's Power is at
// full speed.
/// Player 1's Airfield on tiles (12, 3) to (14, 5), and its centre.
const A: u32 = 4;
const A_AT: Vec2 = Vec2::new(864.0, 288.0);
/// [`A`]'s landing slots, in the order they are taken: the centres of its footprint's quarters, north-west,
/// north-east, south-west and south-east (**placeholder**: the slots' layout, `sc_sim::landing::slots`).
const A_SLOTS: [Vec2; 4] = [
    Vec2::new(A_AT.x - 48.0, A_AT.y - 48.0),
    Vec2::new(A_AT.x + 48.0, A_AT.y - 48.0),
    Vec2::new(A_AT.x - 48.0, A_AT.y + 48.0),
    Vec2::new(A_AT.x + 48.0, A_AT.y + 48.0),
];
/// How near its slot an aircraft down on it is: within the reach it lands from (`Air::landing_reach_su`), its own
/// arrival exact, but for the air units' separation, which may nudge it as another flies in over it
/// (`sc_sim::landing`'s module docs).
const ON_SLOT: f32 = 8.0;

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

/// Both players play the GDO on a flat map of 60 × 40 tiles, player 1 with two Airfields, [`A`] and another, and the
/// power for both, and `more` after them (from id 7); auto-targeting off; tick 1 has run.
fn field(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(60, 40))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 5))
        .spawn(ObjectKind::Airfield, Some(P1), tile(12, 3))
        .spawn(ObjectKind::Airfield, Some(P1), tile(12, 12))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(54, 34));
    started(more(builder))
}

/// The game of `builder`, auto-targeting off, tick 1 run.
fn started(builder: ScenarioBuilder) -> TestSim {
    let mut t = builder.start();
    t.without_auto_targeting();
    t.run_until(1);
    t
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

/// [`order`], Shift-queued.
fn queued(units: &[u32], kind: OrderKind) -> Command {
    Command::Order {
        units: units.iter().copied().map(id).collect(),
        kind,
        queue: true,
    }
}

/// The right-click of `units` on `target` (`Sim::default_order`, the first unit's, as the client sends it).
fn right_click(sim: &Sim, units: &[u32], target: Target) -> Command {
    let kind = sim.default_order(id(units[0]), target).expect("a default order");
    order(units, kind)
}

fn object(sim: &Sim, n: u32) -> &sc_sim::Object {
    sim.object(id(n)).unwrap_or_else(|| panic!("#{n} is on the map"))
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
    landing::of(object(sim, n))
}

/// Whether aircraft `n` is down on a slot (`sc_sim::landing::down`).
fn down(sim: &Sim, n: u32) -> bool {
    landing::down(object(sim, n))
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

/// The slot of [`A`] that `pos` is on ([`ON_SLOT`]), by number.
fn slot_of(pos: Vec2) -> Option<usize> {
    A_SLOTS.iter().position(|slot| slot.distance(pos) <= ON_SLOT)
}

/// Steps until `done` holds, at most `ticks` ticks.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&Sim) -> bool) {
    let until = t.sim().tick() + ticks;
    while !done(t.sim()) {
        assert!(t.sim().tick() < until, "{what}: not by tick {until}");
        t.step();
    }
}

/// Steps until aircraft `n`, in the air, is down on a slot, at most `ticks` ticks: the tick its landing began and the
/// tick it was down.
fn until_down(t: &mut TestSim, n: u32, ticks: Tick) -> (Tick, Tick) {
    assert_eq!(landing_of(t.sim(), n), None, "#{n} in the air");
    let until = t.sim().tick() + ticks;
    let mut began = None;
    loop {
        if let Some(landing) = landing_of(t.sim(), n) {
            let began = *began.get_or_insert(t.sim().tick());
            if landing.descent.done() {
                return (began, t.sim().tick());
            }
        }
        assert!(t.sim().tick() < until, "#{n} down: not by tick {until}");
        t.step();
    }
}

/// `points.len()` Hornets of player 1 spawned in the next tick at `points`, ids from `first` on, assigned to
/// `airfield` by a right-click on it in the tick after (`OrderKind::Rebase`), which flies them there to land; stepped
/// until all are down, and home.
fn landed_hornets(t: &mut TestSim, first: u32, points: &[Vec2], airfield: u32) -> Vec<u32> {
    let ids: Vec<u32> = (first..first + points.len() as u32).collect();
    let tick = t.sim().tick() + 1;
    for &pos in points {
        t.spawn_unit_at(tick, HORNET, P1, pos);
    }
    t.step();
    let click = right_click(t.sim(), &ids, Target::Object(id(airfield)));
    assert_eq!(click, order(&ids, OrderKind::Rebase(id(airfield))));
    next(t, P1, click);
    step_until(t, 400, "landed", |sim| ids.iter().all(|&n| home(sim, n)));
    for &n in &ids {
        assert_eq!(base(t.sim(), n), Some(airfield));
    }
    ids
}

/// The missiles launched so far by `shooter`: (tick, target, to, lands_at).
fn launches(t: &TestSim, shooter: u32) -> Vec<(Tick, Option<ObjectId>, Vec2, Tick)> {
    let launch = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::ProjectileLaunched {
            shooter: by,
            target,
            to,
            lands_at,
            ..
        } if by == id(shooter) => Some((*tick, target, to, lands_at)),
        _ => None,
    };
    t.events().iter().filter_map(launch).collect()
}

/// Steps until `shooter` has launched `count` missiles, at most `ticks` ticks.
fn until_launched(t: &mut TestSim, shooter: u32, count: usize, ticks: Tick) {
    let until = t.sim().tick() + ticks;
    while launches(t, shooter).len() < count {
        assert!(
            t.sim().tick() < until,
            "#{shooter}'s {count} missiles: not by tick {until}"
        );
        t.step();
    }
}

/// The hits of `attacker` so far: (tick, target, HP damage).
fn hits_by(t: &TestSim, attacker: u32) -> Vec<(Tick, ObjectId, f32)> {
    let hit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit {
            attacker: by,
            target,
            damage,
            ..
        } if by == id(attacker) => Some((*tick, target, damage)),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

/// The ticks of `attacker`'s hits on `target` so far.
fn hit_ticks(t: &TestSim, attacker: u32, target: u32) -> Vec<Tick> {
    (hits_by(t, attacker).into_iter())
        .filter(|hit| hit.1 == id(target))
        .map(|hit| hit.0)
        .collect()
}

/// The blasts of `attacker`'s shots so far: (tick, at, radius).
fn blasts_of(t: &TestSim, attacker: u32) -> Vec<(Tick, Vec2, f32)> {
    let blast = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Blast {
            at,
            radius,
            attacker: Some(by),
        } if by == id(attacker) => Some((*tick, at, radius)),
        _ => None,
    };
    t.events().iter().filter_map(blast).collect()
}

/// The tick object `n` died, if it has (`SimEvent::Died`).
fn died(t: &TestSim, n: u32) -> Option<Tick> {
    (t.events().iter())
        .find_map(|(tick, e)| matches!(e, SimEvent::Died { id: dead, .. } if *dead == id(n)).then_some(*tick))
}
