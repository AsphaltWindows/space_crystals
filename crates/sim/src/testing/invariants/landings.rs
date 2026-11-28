//! (M6-GH2) The landing invariant L1 (`crate::landing`, `AircraftState::landing`): an aircraft on a slot, landing or
//! down, is on an Airfield in the game, within `Air::landing_reach_su` of one of its slots (it lands only that near
//! one, and holds still there, out of the air units' separation: `movement::air`); no Airfield has more aircraft on it
//! than the slots of the placeholder layout (`landing::LANDING_SLOTS`: each lands only on a slot it takes,
//! `landing::claims`); one on a slot rises no more
//! (`Mover::rise`: its take-off ends its landing); it rearms a round exactly while it is down and short of ammo (the
//! economy phase starts its first in the tick it comes down, and ends its last as it is full: `landing::rearm`); and a
//! unit held off the map is on no slot. Its base may be another Airfield: a Rebase queued behind its flight home
//! reassigns it as it is given (`Sim::rebase`), and it takes off once that flight is over.
//!
//! A Supply Chopper's landing (`crate::supply_run`, `ChopperState::landing`) is on a Supply Tower's or a
//! Supply Delivery Station's spot in the game, within the same reach of it, one chopper on a spot at a time (the spot's
//! holder alone lands: `Sim::step_supply_runs`); it rises no more, as an aircraft's, and rearms nothing.

use std::collections::BTreeMap;

use super::Violation;
use crate::aircraft;
use crate::ids::ObjectId;
use crate::landing::{self, LANDING_SLOTS, Spot};
use crate::ruleset::rules;
use crate::sim::Sim;
use crate::supply_run;

/// L1 (module docs).
pub(super) fn landings(sim: &Sim) -> Result<(), Violation> {
    let mut on: BTreeMap<ObjectId, usize> = BTreeMap::new();
    let reach = rules().tuning.air.landing_reach_su;
    for o in sim.objects() {
        let Some(landing) = landing::of(o) else {
            continue;
        };
        let fail = |problem: String| {
            Err(Violation {
                invariant: "L1",
                message: format!("#{} ({:?}) {problem}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        let structure = match landing.at {
            Spot::Structure(structure) => structure,
            // A Field landing: within reach of its point (out of the separation, nothing pushes it off), rising no
            // more and rearming nothing.
            Spot::Field(point) => {
                if point.distance(o.pos) > reach {
                    return fail(format!("is down on open ground off its point {point}"));
                }
                if o.mover.as_ref().is_some_and(|mover| mover.rise.is_some()) || landing.rearm.is_some() {
                    return fail(format!("rises or rearms, down on open ground at {point}"));
                }
                continue;
            }
        };
        let at = structure.0;
        if o.chopper.is_some() {
            let Some(spot) = sim.object(structure).and_then(supply_run::spot) else {
                return fail(format!("is on #{at}, no Supply Tower or station in the game"));
            };
            if spot.distance(o.pos) > reach {
                return fail(format!("is on #{at}, off its spot"));
            }
            let count = on.entry(structure).or_default();
            *count += 1;
            if *count > 1 {
                return fail(format!("is one of more than one on #{at}'s spot"));
            }
            if o.mover.as_ref().is_some_and(|mover| mover.rise.is_some()) || landing.rearm.is_some() {
                return fail(format!("rises or rearms, on the spot of #{at}"));
            }
            continue;
        }
        let Some(airfield) = sim
            .object(structure)
            .filter(|airfield| aircraft::airfield_def(airfield).is_some())
        else {
            return fail(format!("is on a slot of #{at}, no Airfield in the game"));
        };
        if !landing::slots(airfield).any(|slot| slot.distance(o.pos) <= reach) {
            return fail(format!("is on #{at}, off its slots"));
        }
        let count = on.entry(structure).or_default();
        *count += 1;
        if *count > LANDING_SLOTS {
            return fail(format!("is one of more than {LANDING_SLOTS} on #{at}"));
        }
        if o.mover.as_ref().is_some_and(|mover| mover.rise.is_some()) {
            return fail(format!("rises, on a slot of #{at}"));
        }
        let ammo = o.aircraft.map_or(0, |state| state.ammo);
        let short = aircraft::def_of(o.kind).is_some_and(|def| ammo < def.max);
        let rearms = landing.descent.done() && short;
        if landing.rearm.is_some() != rearms {
            let state = if landing.descent.done() { "down" } else { "landing" };
            return fail(format!("{state} with {ammo} rounds, rearms {:?}", landing.rearm));
        }
    }
    for held in sim.held() {
        if landing::of(&held.object).is_some() {
            return Err(Violation {
                invariant: "L1",
                message: format!("#{} ({:?}) is on a slot, held", held.object.id.0, held.object.kind),
                at: held.object.pos,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use glam::Vec2;
    use serde_json::json;

    use super::*;
    use crate::command::TickInputs;
    use crate::defs::ObjectKind;
    use crate::economy::RATIO_ONE;
    use crate::ids::PlayerId;
    use crate::scenario::SpawnAt;
    use crate::testing::ScenarioBuilder;
    use crate::testing::invariants::Invariants;
    use crate::units::TilePos;

    const P1: PlayerId = PlayerId(1);

    /// Player 1's Airfield (1) on tiles (1, 1) to (3, 3), its Generator (2), and its Hornet (3) down on the Airfield's
    /// first slot, full, as saved.
    fn saved() -> serde_json::Value {
        let builder = ScenarioBuilder::new(&"..........\n".repeat(8))
            .spawn(ObjectKind::Airfield, Some(P1), SpawnAt::Tile(TilePos::new(1, 1)))
            .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(6, 1)))
            .spawn(ObjectKind::Hornet, Some(P1), SpawnAt::Point(Vec2::new(112.0, 112.0)));
        let sim = Sim::new(&builder.build());
        let mut json = serde_json::to_value(&sim).expect("a state serializes");
        let hornet = object(&mut json, 3);
        hornet["aircraft"]["base"] = 1.into();
        hornet["landing"] = json!({ "at": { "Structure": 1 }, "descent": { "progress": 16 * RATIO_ONE, "total": 16 } });
        json
    }

    fn object(json: &mut serde_json::Value, n: u32) -> &mut serde_json::Value {
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        objects.iter_mut().find(|o| o["id"] == n).expect("in the state")
    }

    /// The checker's verdict on `json` loaded.
    fn check(json: serde_json::Value) -> Result<(), Violation> {
        let sim: Sim = serde_json::from_value(json).expect("the edited state loads");
        Invariants::new(&sim).check(&sim, &TickInputs::default())
    }

    /// The negative test: a state edited to have a full aircraft rearm, one short of ammo not rearm, one on a slot of
    /// what is no Airfield, one rising on a slot, or one on the Airfield off its slots, breaks L1, which the state as
    /// saved, a full Hornet down on its slot, keeps, and so do one short of ammo that rearms and one nudged a little.
    #[test]
    fn a_rearm_out_of_turn_a_slot_off_an_airfield_or_a_rise_on_a_slot_breaks_l1() {
        assert_eq!(check(saved()), Ok(()), "down, full");
        let broken = |edit: &dyn Fn(&mut serde_json::Value), problem: &str| {
            let mut json = saved();
            edit(&mut json);
            let violation = check(json).unwrap_err();
            assert_eq!(violation.invariant, "L1", "{violation:?}");
            assert!(violation.message.contains(problem), "{problem}: {violation:?}");
        };
        let round = json!({ "progress": 0, "total": 32 });
        broken(
            &|json| object(json, 3)["landing"]["rearm"] = round.clone(),
            "rearms Some",
        );
        broken(&|json| object(json, 3)["aircraft"]["ammo"] = 2.into(), "rearms None");
        broken(
            &|json| object(json, 3)["landing"]["at"] = json!({ "Structure": 2 }),
            "no Airfield",
        );
        broken(&|json| object(json, 3)["mover"]["rise"] = round.clone(), "rises");
        // Moved there as saved, the tick before too (so that it hasn't moved in a tick: invariant I3).
        let placed = |json: &mut serde_json::Value, x: f32| {
            let hornet = object(json, 3);
            (hornet["pos"], hornet["prev_pos"]) = (json!([x, 112.0]), json!([x, 112.0]));
        };
        broken(&|json| placed(json, 130.0), "off its slots");
        let mut nudged = saved();
        placed(&mut nudged, 118.0);
        assert_eq!(check(nudged), Ok(()), "within its slot's reach");
        let mut json = saved();
        object(&mut json, 3)["aircraft"]["ammo"] = 2.into();
        object(&mut json, 3)["landing"]["rearm"] = round.clone();
        assert_eq!(check(json), Ok(()), "down, short and rearming");
    }

    /// The chopper's arm: player 1's Supply Tower (1) on tiles (1, 1) to (3, 3), and its Supply Chopper (2)
    /// attached to it and down on its spot, at its centre, keep L1; a state edited to have the chopper on what is no
    /// tower or station, rising on the spot, or off it breaks it.
    #[test]
    fn a_chopper_off_its_spot_or_rising_on_it_breaks_l1() {
        let builder = ScenarioBuilder::new(&"..........\n".repeat(8))
            .spawn(ObjectKind::SupplyTower, Some(P1), SpawnAt::Tile(TilePos::new(1, 1)))
            .spawn(
                ObjectKind::SupplyChopper,
                Some(P1),
                SpawnAt::Point(Vec2::new(160.0, 160.0)),
            );
        let sim = Sim::new(&builder.build());
        let mut saved = serde_json::to_value(&sim).expect("a state serializes");
        let chopper = object(&mut saved, 2);
        chopper["chopper"]["attached"] = 1.into();
        chopper["landing"] =
            json!({ "at": { "Structure": 1 }, "descent": { "progress": 16 * RATIO_ONE, "total": 16 } });
        assert_eq!(check(saved.clone()), Ok(()), "down on its spot");
        let broken = |edit: &dyn Fn(&mut serde_json::Value), problem: &str| {
            let mut json = saved.clone();
            edit(&mut json);
            let violation = check(json).unwrap_err();
            assert_eq!(violation.invariant, "L1", "{violation:?}");
            assert!(violation.message.contains(problem), "{problem}: {violation:?}");
        };
        broken(
            &|json| object(json, 2)["landing"]["at"] = json!({ "Structure": 2 }),
            "no Supply Tower",
        );
        broken(
            &|json| object(json, 2)["mover"]["rise"] = json!({ "progress": 0, "total": 16 }),
            "rises",
        );
        broken(
            &|json| {
                let chopper = object(json, 2);
                (chopper["pos"], chopper["prev_pos"]) = (json!([200.0, 160.0]), json!([200.0, 160.0]));
            },
            "off its spot",
        );
    }
}
