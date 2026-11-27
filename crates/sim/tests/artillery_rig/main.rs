//! The Artillery Rig (slice M6-GA; `factions/gdo/units/artillery_rig.md`, `factions/gdo/structures/factory.md >
//! Produces`; design changes 0366 and 0435), through the gate. Its deploy is the vehicle deploy model's
//! (`tests/vehicle_deploy/`). One test binary:
//! - `training`: the Factory's R, behind Advanced Mechanics, for 250 Space Crystals and 5 Supplies, both paid as it is
//!   queued and refunded with a cancel (0435's Verification), refused with fewer than 5 Supplies; it comes out of the
//!   Factory 480 frames on;
//! - `shell`: the deployed Rig's TailDisjointed shell, 300 over a circle of 1 grid unit: an enemy inside its MinRange of
//!   3 ("Target too close" for an Attack; auto-targeting never picks one), one at its Range of 12 and one at 11 seen by
//!   a spotter, a target that leaves the locked spot during the 16 frames of Firing missed, own units in the blast hurt,
//!   Attack Ground, and an Attack on an aircraft refused (`Reject::CannotHit`);
//! - `mobile`: the mobile Rig never fires, whatever is near, and takes no Attack.
//!
//! Auto-targeting is off but where a test turns it on, so that only orders fight.

mod mobile;
mod shell;
mod training;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    AttackPhase, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    ProductionCommand, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2, gu,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RIG: ObjectKind = ObjectKind::ArtilleryRig;
const DEPLOYED: ObjectKind = ObjectKind::ArtilleryRigDeployed;
const PK: ObjectKind = ObjectKind::Peacekeeper;
/// [`field`]'s Rig, the first object.
const UNIT: ObjectId = ObjectId(1);
/// Where [`field`]'s Rig stands, facing east: the centre of tile (10, 15).
const HOME: Vec2 = Vec2::new(672.0, 992.0);
/// Half the Rig's 64 su length.
const RIG_HALF: f32 = 32.0;
/// `factions/gdo/units/artillery_rig.md > Placeholder stats`: "AimDuration 16, FiringDuration **16** (the shell's
/// flight: the gap in which targets can leave the locked area)".
const FIRING: Tick = 16;
/// "AoERadius **64 su (1 GU)**".
const RADIUS: f32 = 64.0;

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

/// Where a unit of half-width `half` stands `gap` grid units east of [`HOME`]'s Rig, edge to edge, along its length,
/// and `dy` space units south.
fn east_of_rig(gap: f32, half: f32, dy: f32) -> Vec2 {
    from_home(RIG_HALF + gu(gap) + half, dy)
}

/// The half-width of a `kind` unit's Silhouette.
fn half(kind: ObjectKind) -> f32 {
    kind.def().silhouette().expect("a unit").x / 2.0
}

/// Player 1's `kind` (1) at [`HOME`], facing east, on a flat map of 50 × 30 tiles, and `more` after it; auto-targeting
/// off. Tick 1 has run.
fn field(kind: ObjectKind, more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(50, 30)).spawn(kind, Some(P1), SpawnAt::Point(HOME));
    let mut t = more(builder).start();
    t.without_auto_targeting();
    t.run_until(1);
    t
}

/// Auto-targeting on from the next tick.
fn auto_targeting(t: &mut TestSim) {
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::SetAutoTargeting { on: true });
}

fn order(units: &[ObjectId], kind: OrderKind) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind,
        queue: false,
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// `player`'s `command` in the next tick, which runs, the gate taking it: that tick.
fn give(t: &mut TestSim, player: PlayerId, command: Command) -> Tick {
    assert_eq!(gate(t, player, command.clone()), Ok(()), "{command:?}");
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    t.step();
    tick
}

fn object(t: &TestSim, n: ObjectId) -> &Object {
    t.sim().object(n).unwrap_or_else(|| panic!("#{} is on the map", n.0))
}

fn current(t: &TestSim, n: ObjectId) -> Option<OrderKind> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    orders.current.as_ref().map(|active| active.order.kind)
}

/// `attacker`'s blasts so far: (tick, centre, radius).
fn blasts_by(t: &TestSim, attacker: ObjectId) -> Vec<(Tick, Vec2, f32)> {
    let blast = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Blast {
            at,
            radius,
            attacker: by,
        } if by == Some(attacker) => Some((*tick, at, radius)),
        _ => None,
    };
    t.events().iter().filter_map(blast).collect()
}

/// The HP damage `n` took so far: (tick, attacker, damage).
fn hits_on(t: &TestSim, n: ObjectId) -> Vec<(Tick, ObjectId, f32)> {
    let hit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit {
            attacker,
            target,
            damage,
            ..
        } if target == n => Some((*tick, attacker, damage)),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

/// Whether `attacker` has attacked at all so far: an attack, a miss, a projectile or a blast of its own.
fn ever_attacked(t: &TestSim, attacker: ObjectId) -> bool {
    t.events().iter().any(|(_, e)| match *e {
        SimEvent::Attacked { attacker: by, .. } | SimEvent::Missed { attacker: by, .. } => by == attacker,
        SimEvent::Blast { attacker: by, .. } => by == Some(attacker),
        SimEvent::ProjectileLaunched { shooter, .. } => shooter == attacker,
        _ => false,
    })
}

/// Runs until the deployed Rig's Aiming ends, and returns the tick and the spot it locked (`AttackPhase::FiringAt`).
fn until_locked(t: &mut TestSim) -> (Tick, Vec2) {
    let until = t.sim().tick() + 300;
    loop {
        assert!(t.sim().tick() < until, "it never locked a spot\n{}", t.ascii(None));
        t.step();
        let attack = object(t, UNIT).attack.as_ref().expect("it attacks");
        if let Some(AttackPhase::FiringAt { at, .. }) = attack.phase
            && attack.elapsed == 0
        {
            return (t.sim().tick(), at);
        }
    }
}
