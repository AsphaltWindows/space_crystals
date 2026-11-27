//! (M6-CM) The illusion invariant M1 (`crate::mirage`, `Object::illusion_of`): an illusion's parent is in the game, on
//! the map or held off it, made before it (the orders' phase gives the parent its turn first: `orders::run::update`),
//! no illusion itself, and of the illusion's own type, a type that casts Mirage; a parent has no more illusions than
//! its Mirage makes ("a Disciple never has more than one set", `factions/cults/units/disciple.md > Spells > Mirage`);
//! and an illusion has no lineage ("They use **no Unit Control**"). That it has no energy is E1's (`energy`). Over the
//! held units too: "An illusion inside a Garrison when it ends simply vanishes".

use std::collections::BTreeMap;

use super::Violation;
use crate::ids::ObjectId;
use crate::mirage;
use crate::object::Object;
use crate::sim::Sim;

/// M1 (module docs).
pub(super) fn illusions(sim: &Sim) -> Result<(), Violation> {
    let in_game = |id: ObjectId| sim.object(id).or_else(|| sim.held_unit(id).map(|held| &held.object));
    let mut sets: BTreeMap<ObjectId, u32> = BTreeMap::new();
    for o in sim.objects().chain(sim.held().map(|held| &held.object)) {
        let Some(parent) = o.illusion_of else {
            continue;
        };
        let fail = |problem: String| {
            Err(Violation {
                invariant: "M1",
                message: format!("illusion #{} ({:?}) {problem}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        let Some(def) = mirage::def_of(o.kind) else {
            return fail("is of a type that casts no Mirage".to_string());
        };
        let Some(of) = in_game(parent) else {
            return fail(format!("has its parent #{} gone", parent.0));
        };
        if let Some(problem) = unlike(o, of) {
            return fail(format!("has a parent #{} {problem}", parent.0));
        }
        if !o.lineage.is_empty() {
            return fail(format!("has a lineage, {:?}", o.lineage));
        }
        let count = sets.entry(parent).or_default();
        *count += 1;
        if *count > def.illusions {
            return fail(format!("is one of more than {} of #{}", def.illusions, parent.0));
        }
    }
    Ok(())
}

/// What is wrong with `parent` as illusion `o`'s parent, if anything (module docs).
fn unlike(o: &Object, parent: &Object) -> Option<String> {
    if parent.id >= o.id {
        return Some("made after it".to_string());
    }
    if mirage::is_illusion(parent) {
        return Some("that is an illusion".to_string());
    }
    (parent.kind != o.kind).then(|| format!("of another type, {:?}", parent.kind))
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::command::{Ability, Command, PlayerCommand, TickInputs};
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::scenario::SpawnAt;
    use crate::testing::ScenarioBuilder;
    use crate::testing::invariants::Invariants;

    const P1: PlayerId = PlayerId(1);

    /// Disciple 1 of player 1 at full energy, with its 4 illusions (3 to 6) from a Mirage, and a Recruit (2): as saved.
    fn saved() -> serde_json::Value {
        let builder = ScenarioBuilder::new(&"..............................\n".repeat(12))
            .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(Vec2::new(600.0, 400.0)))
            .spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(Vec2::new(1200.0, 400.0)));
        let mut json = serde_json::to_value(Sim::new(&builder.build())).expect("a state serializes");
        object(&mut json, 1)["energy"] = crate::energy::max(ObjectKind::Disciple).unwrap().into();
        let mut sim: Sim = serde_json::from_value(json).expect("it loads");
        let mirage = Command::Ability {
            units: vec![ObjectId(1)],
            ability: Ability::Mirage,
        };
        let inputs = TickInputs {
            commands: vec![PlayerCommand {
                player: P1,
                command: mirage,
            }],
        };
        sim.step(&inputs);
        Invariants::new(&sim).check(&sim, &inputs).expect("as saved, M1 holds");
        let illusions: Vec<u32> = (sim.objects().filter(|o| o.illusion_of == Some(ObjectId(1))))
            .map(|o| o.id.0)
            .collect();
        assert_eq!(illusions, [3, 4, 5, 6]);
        serde_json::to_value(&sim).expect("a state serializes")
    }

    /// The checker's verdict on `json` loaded, after `edit`.
    fn check_with(mut json: serde_json::Value, edit: impl FnOnce(&mut serde_json::Value)) -> Result<(), Violation> {
        edit(&mut json);
        let sim: Sim = serde_json::from_value(json).expect("the edited state loads");
        Invariants::new(&sim).check_rules(&sim, &TickInputs::default())
    }

    fn broken(result: Result<(), Violation>, says: &str) {
        let violation = result.expect_err(says);
        assert_eq!(violation.invariant, "M1", "{violation:?}");
        assert!(violation.message.contains(says), "{violation:?}");
    }

    /// Object `id` of the state `json`.
    fn object(json: &mut serde_json::Value, id: u32) -> &mut serde_json::Value {
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        objects.iter_mut().find(|o| o["id"] == id).expect("in the state")
    }

    /// The negative test: an illusion whose parent is gone, an illusion, made after it or of another type, an
    /// illusion with a lineage, and a fifth illusion, break M1, which the state as saved keeps.
    #[test]
    fn an_illusion_without_its_parent_or_one_too_many_breaks_m1() {
        assert_eq!(check_with(saved(), |_| {}), Ok(()));
        broken(
            check_with(saved(), |json| {
                let objects = json["objects"].as_array_mut().expect("the objects are a list");
                objects.retain(|o| o["id"] != 1);
            }),
            "has its parent #1 gone",
        );
        broken(
            check_with(saved(), |json| object(json, 4)["illusion_of"] = 3.into()),
            "that is an illusion",
        );
        broken(
            check_with(saved(), |json| object(json, 3)["illusion_of"] = 4.into()),
            "made after it",
        );
        broken(
            check_with(saved(), |json| object(json, 3)["illusion_of"] = 2.into()),
            "of another type, Recruit",
        );
        broken(
            check_with(saved(), |json| object(json, 3)["lineage"] = serde_json::json!([2])),
            "has a lineage",
        );
        broken(
            check_with(saved(), |json| object(json, 2)["illusion_of"] = 1.into()),
            "is of a type that casts no Mirage",
        );
        broken(
            check_with(saved(), |json| {
                let mut fifth = object(json, 6).clone();
                fifth["id"] = 7.into();
                fifth["pos"] = serde_json::json!([600.0, 600.0]);
                fifth["prev_pos"] = fifth["pos"].clone();
                json["objects"]
                    .as_array_mut()
                    .expect("the objects are a list")
                    .push(fifth);
                json["next_id"] = 8.into();
            }),
            "is one of more than 4 of #1",
        );
    }
}
