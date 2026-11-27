//! (M5-H1) The Glider invariant G1 (`movement::glider`, plan decision T81): a Glider in flight never stops and keeps
//! within its speeds. Every Glider on the map that was in the game in the tick before (one that came in this tick
//! hasn't flown yet) flies (Locomotion Moving), at a speed (its momentum's length, `Mover::momentum`) between its
//! IdleSpeed and its MaxSpeed (`systems/units.md > GliderMovement`: "Unit must always maintain movement to stay
//! airborne"), its own move in the tick being that flight (`Mover::velocity`: the soft separation of air units
//! comes on top, `movement::air`), and where it counts as being (`Object::map_pos`) is on the map, wherever it flies
//! (`systems/units.md > Air Collision`: "stays visible and attackable as if it were at the nearest edge tile"). One
//! stopped after its flight in the tick (`Mover::stop`: an order that ended in phase 5e, its target gone) shows no
//! move, Stationary, until its next flight, from which it circles where it is; it keeps its speed.
//!
//! It also holds the exemptions the Gliders and the soft separation need from I3, I5 and I7 (`super::check_units`):
//! a Glider moves, and has momentum, without a goal, and an air unit is pushed apart from another by at most
//! `Air::separation_push_su` a tick, beyond its own move ((M5-H2) its base's share of it,
//! `UnitBaseDef::separation_strength`: a Glider's is (M5-N7) a third, design change 0412; a quarter from design change
//! 0405).

use std::collections::BTreeMap;

use glam::Vec2;

use super::{Seen, Violation};
use crate::defs::{Domain, MovementAttrs, UnitDef};
use crate::ids::ObjectId;
use crate::mover::{Locomotion, MoveGoal};
use crate::object::Object;
use crate::ruleset::rules;
use crate::sim::Sim;

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
        let stopped = mover.goal == MoveGoal::None
            && mover.orbit.is_none()
            && mover.strafe.is_none()
            && mover.velocity == Vec2::ZERO
            && mover.locomotion == Locomotion::Stationary;
        if mover.locomotion != Locomotion::Moving && !stopped {
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
        if mover.velocity != mover.momentum && !stopped {
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

/// If `o` is an air unit that keeps apart from others (`movement::air`): its SeparationRadius, and the most the soft
/// separation pushes it in a tick, (M5-H2) its base's share of `Air::separation_push_su`
/// (`UnitBaseDef::separation_strength`), space units.
fn separation(o: &Object) -> Option<(f32, f32)> {
    let unit = o.def().unit.as_ref()?;
    let base = unit.base_def();
    if base.domain != Domain::Air || o.mover.is_none() {
        return None;
    }
    Some((
        unit.separation_radius?,
        rules().tuning.air.separation_push_su * base.separation_strength?,
    ))
}

/// How far the soft separation may push `o` in a tick beyond its own move, space units: the most it pushes an air unit
/// with a SeparationRadius ([`separation`]), 0 for any other unit (I3).
pub(super) fn push_allowance(o: &Object) -> f32 {
    separation(o).map_or(0.0, |(_, push)| push)
}

/// Whether `o`, which moved without a goal of its own, may have been pushed by the soft separation (I5, I7): it is an
/// air unit with a SeparationRadius, it moved by at most the most the separation pushes it ([`separation`]), and
/// another such unit is near enough to have pushed it: within the larger of their two radii of where `o` was before
/// the push (its last position), give or take the other's own push.
pub(super) fn pushed(sim: &Sim, o: &Object) -> bool {
    let Some((radius, push)) = separation(o) else {
        return false;
    };
    let moved = o.pos.distance(o.prev_pos);
    moved <= push * (1.0 + SPEED_TOLERANCE) + SPEED_TOLERANCE
        && sim.objects().any(|other| {
            other.id != o.id
                && separation(other).is_some_and(|(r, other_push)| {
                    other.pos.distance(o.prev_pos) < radius.max(r) + other_push * (1.0 + SPEED_TOLERANCE)
                })
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

    /// The rules' verdict (`Invariants::check_rules`) on `json` loaded, with each of the Sparrow's mover's `fields`
    /// set to its value.
    fn check_with(mut json: Value, fields: &[(&str, Value)]) -> Result<(), Violation> {
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        let sparrow = objects.iter_mut().find(|o| o["id"] == 1).expect("in the state");
        for (field, value) in fields {
            sparrow["mover"][*field] = value.clone();
        }
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
            let broken = check_with(json.clone(), &[(field, value)]).unwrap_err();
            assert_eq!(broken.invariant, "G1", "{broken:?}");
            assert!(broken.message.contains(problem), "{field}: {broken:?}");
        }
        assert!(momentum.is_array(), "the Sparrow flies: {momentum}");
    }

    /// A Sparrow stopped after its flight in the tick (`Mover::stop`, as when the target of its Attack dies in phase
    /// 5e) shows no move until its next flight, and passes G1 so: no goal, no circle, no strafing run, no velocity,
    /// Stationary. It still keeps its speed, and with anything left of a goal it breaks G1.
    #[test]
    fn a_glider_stopped_after_its_flight_keeps_g1_while_it_keeps_its_speed() {
        let stopped = [
            ("goal", json!("None")),
            ("orbit", Value::Null),
            ("velocity", json!([0.0, 0.0])),
            ("locomotion", json!("Stationary")),
        ];
        let json = saved();
        assert_eq!(check_with(json.clone(), &stopped), Ok(()));
        let slow = [stopped.as_slice(), &[("momentum", json!([0.0, 3.0]))]].concat();
        let broken = check_with(json.clone(), &slow).unwrap_err();
        assert!(broken.message.contains("slower than its IdleSpeed"), "{broken:?}");
        let circling = [&stopped[..1], &stopped[2..]].concat();
        let broken = check_with(json, &circling).unwrap_err();
        assert!(broken.message.contains("isn't flying"), "{broken:?}");
    }
}
