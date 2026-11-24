//! Helpers for the doorway tests (`exit`, `stalls`).

use sc_sim::testing::TestSim;
use sc_sim::{DebugCommand, Doorway, ObjectId, ProductionCommand, Sim, SimEvent, Target, Tick, Vec2};

use super::{P1, PK, production};

pub(super) const DOOR: Vec2 = Vec2::new(704.0, 320.0);

/// Where a unit that walked straight out of [`DOOR`] stands: touching side B from outside.
pub(super) const OUTSIDE: Vec2 = Vec2::new(716.0, 320.0);

/// Fills player 1's bank and queues `n` Peacekeepers at `producer` in tick 1.
pub(super) fn queue(t: &mut TestSim, producer: ObjectId, n: usize) {
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 10_000,
            supplies: 0,
        },
    );
    for _ in 0..n {
        t.at(1, P1, production(&[producer], ProductionCommand::Train(PK)));
    }
}

pub(super) fn rally(t: &mut TestSim, tick: Tick, producer: ObjectId, point: Vec2) {
    t.at(
        tick,
        P1,
        production(&[producer], ProductionCommand::SetRally(Target::Point(point))),
    );
}

/// The units produced so far: `(tick, unit)`.
pub(super) fn produced(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::UnitProduced { unit, .. } => Some((*tick, *unit)),
            _ => None,
        })
        .collect()
}

pub(super) fn exit_of(sim: &Sim, unit: ObjectId) -> Option<ObjectId> {
    sim.object(unit).unwrap().mover.as_ref().unwrap().exit
}

pub(super) fn door_of(sim: &Sim, building: ObjectId) -> Doorway {
    Doorway::of(sim.object(building).unwrap()).unwrap()
}

/// Whether the finished unit of `producer` waits inside.
pub(super) fn waiting(sim: &Sim, producer: ObjectId) -> bool {
    sim.production_status(producer).is_some_and(|status| status.waiting)
}

/// Steps once and checks the doorway rule from outside, for every Peacekeeper of player 1: a unit overlaps a
/// Barracks only while it is that Barracks' released unit, and one that is out never overlaps a Barracks again.
pub(super) fn step_checked(t: &mut TestSim, out: &mut Vec<ObjectId>) {
    t.step();
    let sim = t.sim();
    let doors: Vec<(ObjectId, Doorway)> = sim.objects().filter_map(|o| Some((o.id, Doorway::of(o)?))).collect();
    for o in sim.objects().filter(|o| o.mover.is_some()) {
        let exit = o.mover.as_ref().unwrap().exit;
        let half = o.def().half_extents().unwrap();
        for (building, door) in &doors {
            assert_eq!(
                door.holds(o.pos, half),
                exit == Some(*building),
                "tick {}: #{} at {} and the doorway of #{}",
                sim.tick(),
                o.id.0,
                o.pos,
                building.0
            );
        }
        if exit.is_none() && !out.contains(&o.id) {
            out.push(o.id);
        }
        assert!(exit.is_none() || !out.contains(&o.id), "#{} went back in", o.id.0);
    }
}

pub(super) fn run_checked(t: &mut TestSim, until: Tick) {
    let mut out = Vec::new();
    while t.sim().tick() < until {
        step_checked(t, &mut out);
    }
}
