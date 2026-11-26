//! (M5-H1) The Glider invariant G1 (`movement::glider`, plan decision T81): a Glider in flight never stops and keeps
//! within its speeds. Every Glider on the map that was in the game in the tick before (one that came in this tick
//! hasn't flown yet) flies (Locomotion Moving), at a speed (its momentum's length, `Mover::momentum`) between its
//! IdleSpeed and its MaxSpeed (`systems/units.md > GliderMovement`: "Unit must always maintain movement to stay
//! airborne"), its own move in the tick being that flight (`Mover::velocity`: the soft separation of air units
//! comes on top, `movement::air`), and where it counts as being (`Object::map_pos`) is on the map, wherever it flies
//! (`systems/units.md > Air Collision`: "stays visible and attackable as if it were at the nearest edge tile").
//!
//! It also holds the exemptions the Gliders and the soft separation need from I3, I5 and I7 (`super::check_units`):
//! a Glider moves, and has momentum, without a goal, and an air unit is pushed apart from another by at most
//! [`SEPARATION_PUSH_SU`] a tick, beyond its own move.

use std::collections::BTreeMap;

use super::{Seen, Violation};
use crate::defs::{Domain, MovementAttrs, UnitDef};
use crate::ids::ObjectId;
use crate::mover::Locomotion;
use crate::object::Object;
use crate::sim::Sim;
use crate::tuning::air::SEPARATION_PUSH_SU;

/// The rounding a speed may carry: a flight is its facing's unit vector times its speed (`movement::glider`).
const SPEED_TOLERANCE: f32 = 1e-4;

/// G1 over the units on the map (module docs); `seen` are those of the tick before.
pub(super) fn gliders(sim: &Sim, seen: &BTreeMap<ObjectId, Seen>) -> Result<(), Violation> {
    let research = sim.research();
    for o in sim.objects().filter(|o| seen.contains_key(&o.id)) {
        let (
            Some(mover),
            Some(MovementAttrs::Glider {
                idle_speed, max_speed, ..
            }),
        ) = (&o.mover, o.movement(research))
        else {
            continue;
        };
        let fail = |message: String| {
            Err(Violation {
                invariant: "G1",
                message: format!("#{} ({:?}) {message}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        let speed = mover.momentum.length();
        if mover.locomotion != Locomotion::Moving {
            return fail(format!("is a Glider, and isn't flying: {mover:?}"));
        }
        let slowest = idle_speed * (1.0 - SPEED_TOLERANCE) - SPEED_TOLERANCE;
        if speed.is_nan() || speed < slowest {
            return fail(format!(
                "flies at {speed} su a tick, slower than its IdleSpeed of {idle_speed}"
            ));
        }
        if speed > max_speed * (1.0 + SPEED_TOLERANCE) + SPEED_TOLERANCE {
            return fail(format!(
                "flies at {speed} su a tick, faster than its MaxSpeed of {max_speed}"
            ));
        }
        if mover.velocity != mover.momentum {
            return fail(format!(
                "moved by {} in its own flight, which was {}",
                mover.velocity, mover.momentum
            ));
        }
        let at = o.map_pos(sim.map());
        if !sim.map().contains_point(at) {
            return fail(format!("at {} counts as being at {at}, off the map", o.pos));
        }
    }
    Ok(())
}

/// Whether `o` moves by GliderMovement (`UnitDef::glides`).
pub(super) fn glides(o: &Object) -> bool {
    (o.def().unit.as_ref()).is_some_and(UnitDef::glides)
}

/// The SeparationRadius of `o` if it is an air unit that keeps apart from others (`movement::air`).
fn separation_radius(o: &Object) -> Option<f32> {
    let unit = o.def().unit.as_ref()?;
    (unit.base_def().domain == Domain::Air && o.mover.is_some()).then_some(unit.separation_radius?)
}

/// How far the soft separation may push `o` in a tick beyond its own move, space units: [`SEPARATION_PUSH_SU`] for an
/// air unit with a SeparationRadius, 0 for any other unit (I3).
pub(super) fn push_allowance(o: &Object) -> f32 {
    separation_radius(o).map_or(0.0, |_| SEPARATION_PUSH_SU)
}

/// Whether `o`, which moved without a goal of its own, may have been pushed by the soft separation (I5, I7): it is an
/// air unit with a SeparationRadius, it moved by at most [`SEPARATION_PUSH_SU`], and another such unit is near enough
/// to have pushed it: within the larger of their two radii of where `o` was before the push (its last position),
/// give or take the other's own push.
pub(super) fn pushed(sim: &Sim, o: &Object) -> bool {
    let Some(radius) = separation_radius(o) else {
        return false;
    };
    let moved = o.pos.distance(o.prev_pos);
    moved <= SEPARATION_PUSH_SU * (1.0 + SPEED_TOLERANCE) + SPEED_TOLERANCE
        && sim.objects().any(|other| {
            other.id != o.id
                && separation_radius(other)
                    .is_some_and(|r| other.pos.distance(o.prev_pos) < radius.max(r) + SEPARATION_PUSH_SU)
        })
}

#[cfg(test)]
mod tests {
    use glam::Vec2;
    use serde_json::{Value, json};

    use crate::command::TickInputs;
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::scenario::SpawnAt;
    use crate::sim::Sim;
    use crate::testing::invariants::{Invariants, Violation};
    use crate::testing::{ScenarioBuilder, TestSim};

    /// A Sparrow (1) of player 1, circling over a flat map for 20 ticks (the checks of every tick pass), as saved.
    fn saved() -> Value {
        let scenario = ScenarioBuilder::new(&"..........\n".repeat(8))
            .spawn(
                ObjectKind::Sparrow,
                Some(PlayerId(1)),
                SpawnAt::Point(Vec2::new(320.0, 256.0)),
            )
            .build();
        let mut t = TestSim::new(scenario);
        t.run(20);
        serde_json::to_value(t.sim()).expect("a state serializes")
    }

    /// The rules' verdict (`Invariants::check_rules`) on `json` loaded, with the Sparrow's mover's `field` set to
    /// `value`.
    fn check_with(mut json: Value, field: &str, value: Value) -> Result<(), Violation> {
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        let sparrow = objects.iter_mut().find(|o| o["id"] == 1).expect("in the state");
        sparrow["mover"][field] = value;
        let sim: Sim = serde_json::from_value(json).expect("the edited state loads");
        Invariants::new(&sim).check_rules(&sim, &TickInputs::default())
    }

    /// The negative test: a Sparrow edited to fly faster than its MaxSpeed of 10 su a tick, slower than its IdleSpeed
    /// of 4, not at all, or by another move than its flight, breaks G1, which the state as saved keeps.
    #[test]
    fn a_glider_out_of_its_speeds_or_not_flying_breaks_g1() {
        let json = saved();
        let sim: Sim = serde_json::from_value(json.clone()).expect("the state loads");
        assert_eq!(Invariants::new(&sim).check_rules(&sim, &TickInputs::default()), Ok(()));
        let momentum = json["objects"][0]["mover"]["momentum"].clone();
        for (field, value, problem) in [
            ("momentum", json!([0.0, 12.0]), "faster than its MaxSpeed"),
            ("momentum", json!([0.0, 3.0]), "slower than its IdleSpeed"),
            ("locomotion", json!("Stationary"), "isn't flying"),
            ("velocity", json!([0.0, 0.0]), "in its own flight"),
        ] {
            let broken = check_with(json.clone(), field, value).unwrap_err();
            assert_eq!(broken.invariant, "G1", "{broken:?}");
            assert!(broken.message.contains(problem), "{field}: {broken:?}");
        }
        assert!(momentum.is_array(), "the Sparrow flies: {momentum}");
    }
}
