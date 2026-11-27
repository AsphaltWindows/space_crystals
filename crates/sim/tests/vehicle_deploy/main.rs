//! The vehicle deploy model (slice M6-GA; plan decisions T44 and T87, `sc_sim::vehicle_deploy`;
//! `factions/gdo/units/artillery_rig.md > Modes`, design changes 0211, 0213, 0214 and 0263) on the one pair of types it
//! has today, the Artillery Rig's, through the gate. One test binary:
//! - `timing`: a Deploy takes 48 ticks, the Rig standing still, an Artillery Rig until the last tick, then the deployed
//!   type with its `SimEvent::KindChanged`; Stop doesn't interrupt it; Undeploy is the same the other way, and the
//!   deployed Rig fires no new shell while it undeploys;
//! - `held`: the orders given while it deploys: the last one given without Shift is held and Shift-queued ones line up
//!   behind it; as it completes, each is checked against the deployed type: a Move is dropped with a
//!   `SimEvent::OrderDropped`, an Attack on an aircraft too, an Attack on a ground unit runs;
//! - `gate`: a second deploy refused while one is under way (`Reject::Deploying`), a deploy to a type that doesn't
//!   deploy into the type it names, and the orders that would move a unit whose type can't move (`Reject::CannotMove`);
//! - `queue`: a Move, then a Shift-Deploy, deploys where the move ends;
//! - `obstacle`: deploying and deployed, it never yields: a friendly unit goes round it, and nothing pushes it.
//!
//! Auto-targeting is off (`TestSim::without_auto_targeting`), so that only orders fight.

mod gate;
mod held;
mod obstacle;
mod queue;
mod timing;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SimEvent, SpawnAt,
    Target, Tick, Vec2, gu, vehicle_deploy,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RIG: ObjectKind = ObjectKind::ArtilleryRig;
const DEPLOYED: ObjectKind = ObjectKind::ArtilleryRigDeployed;
const PK: ObjectKind = ObjectKind::Peacekeeper;
/// `factions/gdo/units/artillery_rig.md > Placeholder stats`: "**Deploy 48 frames (3 s), Undeploy 48 frames.**"
const FRAMES: Tick = 48;
/// [`field`]'s unit, the first object.
const UNIT: ObjectId = ObjectId(1);
/// Where [`field`]'s unit stands: the centre of tile (10, 15).
const HOME: Vec2 = Vec2::new(672.0, 992.0);
/// Half the Rig's 64 su length, and half a Peacekeeper's 24 su Silhouette.
const RIG_HALF: f32 = 32.0;
const PK_HALF: f32 = 12.0;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

/// `HOME` and `dx`, `dy` space units.
fn from_home(dx: f32, dy: f32) -> Vec2 {
    HOME + Vec2::new(dx, dy)
}

/// Where a Peacekeeper stands `gap` grid units east of [`HOME`]'s Rig, edge to edge, along its length.
fn east_of_rig(gap: f32) -> Vec2 {
    from_home(RIG_HALF + gu(gap) + PK_HALF, 0.0)
}

/// Player 1's `kind` (1) at [`HOME`], facing east, on a flat map of 40 × 30 tiles, and `more` after it; auto-targeting
/// off. Tick 1 has run.
fn field(kind: ObjectKind, more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(40, 30)).spawn(kind, Some(P1), SpawnAt::Point(HOME));
    let mut t = more(builder).start();
    t.without_auto_targeting();
    t.run_until(1);
    t
}

/// [`field`]'s Rig deployed by its order, with `more`: deployed in tick 2 + 48, which has run.
fn deployed(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let mut t = field(RIG, more);
    let given = give(&mut t, &[UNIT], deploy(), false);
    t.run_until(given + FRAMES);
    assert_eq!(kind_of(&t, UNIT), DEPLOYED);
    t
}

/// R on the mobile Rig: Deploy.
fn deploy() -> OrderKind {
    OrderKind::VehicleDeploy { into: DEPLOYED }
}

/// R on the deployed Rig: Undeploy.
fn undeploy() -> OrderKind {
    OrderKind::VehicleDeploy { into: RIG }
}

fn move_to(to: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(to))
}

fn order(units: &[ObjectId], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind,
        queue,
    }
}

/// What the gate says to player 1's order now (`Sim::validate`).
fn gate(t: &TestSim, units: &[ObjectId], kind: OrderKind, queue: bool) -> Result<(), Reject> {
    let command = order(units, kind, queue);
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// Player 1's order `kind` to `units` in the next tick, which runs, the gate taking it: that tick.
fn give(t: &mut TestSim, units: &[ObjectId], kind: OrderKind, queue: bool) -> Tick {
    assert_eq!(gate(t, units, kind, queue), Ok(()), "{kind:?}");
    let tick = t.sim().tick() + 1;
    t.order_at(tick, P1, units, kind, queue);
    t.step();
    tick
}

fn object(t: &TestSim, n: ObjectId) -> &Object {
    t.sim().object(n).unwrap_or_else(|| panic!("#{} is on the map", n.0))
}

fn kind_of(t: &TestSim, n: ObjectId) -> ObjectKind {
    object(t, n).kind
}

/// Where `n` stands and faces.
fn place(t: &TestSim, n: ObjectId) -> (Vec2, f32) {
    let o = object(t, n);
    (o.pos, o.facing)
}

fn current(t: &TestSim, n: ObjectId) -> Option<OrderKind> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    orders.current.as_ref().map(|active| active.order.kind)
}

fn queued(t: &TestSim, n: ObjectId) -> Vec<OrderKind> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    orders.queue.iter().map(|order| order.kind).collect()
}

/// `n`'s deploy under way: the type it becomes and its frames done of its total.
fn progress(t: &TestSim, n: ObjectId) -> Option<(ObjectKind, u32, u32)> {
    vehicle_deploy::under_way(object(t, n)).map(|(into, progress)| (into, progress.ticks_done(), progress.total))
}

/// The changes of type so far: (tick, object, its new type).
fn kind_changes(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectKind)> {
    let change = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::KindChanged { id, to, .. } => Some((*tick, id, to)),
        _ => None,
    };
    t.events().iter().filter_map(change).collect()
}

/// The orders dropped so far as their deploys completed: (tick, unit, order).
fn dropped(t: &TestSim) -> Vec<(Tick, ObjectId, OrderKind)> {
    let drop = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::OrderDropped { unit, order } => Some((*tick, unit, order)),
        _ => None,
    };
    t.events().iter().filter_map(drop).collect()
}

/// The ticks of `attacker`'s blasts so far.
fn blasts_by(t: &TestSim, attacker: ObjectId) -> Vec<Tick> {
    let blast = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Blast { attacker: by, .. } if by == Some(attacker) => Some(*tick),
        _ => None,
    };
    t.events().iter().filter_map(blast).collect()
}

/// The HP damage `n` took so far: (tick, damage).
fn hits_on(t: &TestSim, n: ObjectId) -> Vec<(Tick, f32)> {
    let hit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit { target, damage, .. } if target == n => Some((*tick, damage)),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}
