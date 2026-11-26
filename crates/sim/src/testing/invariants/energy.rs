//! (M5-N4a) The energy invariant E1 (`Object::energy`, `crate::energy`): an object has energy exactly when its type has
//! an energy pool (`ObjectDef::energy`), and from 0 to the pool's maximum (`energy::max`; never below 0, being a count
//! of `energy::ONE`s), a unit held off the map as one on it.

use super::Violation;
use crate::energy;
use crate::object::Object;
use crate::sim::Sim;

/// E1: every object's energy, and every held unit's, is within its type's pool (module docs).
pub(super) fn energy(sim: &Sim) -> Result<(), Violation> {
    let objects = sim.objects().chain(sim.held().map(|held| &held.object));
    objects.map(within_its_pool).collect()
}

/// E1 for one object.
fn within_its_pool(o: &Object) -> Result<(), Violation> {
    let problem = match (o.energy, energy::max(o.kind)) {
        (None, None) => return Ok(()),
        (Some(energy), Some(max)) if energy <= max => return Ok(()),
        (Some(energy), Some(max)) => format!("has {energy} energy, above its maximum {max}"),
        (Some(energy), None) => format!("has {energy} energy, its type no energy pool"),
        (None, Some(_)) => "has no energy, its type an energy pool".to_string(),
    };
    Err(Violation {
        invariant: "E1",
        message: format!("#{} ({:?}) {problem}", o.id.0, o.kind),
        at: o.pos,
    })
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::command::TickInputs;
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::scenario::SpawnAt;
    use crate::testing::ScenarioBuilder;
    use crate::testing::invariants::Invariants;

    const P1: PlayerId = PlayerId(1);

    /// A Disciple (1) and a Recruit (2) of player 1 on a flat map, the Disciple at its start, as saved.
    fn saved() -> serde_json::Value {
        let builder = ScenarioBuilder::new(&"..........\n".repeat(8))
            .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(Vec2::new(100.0, 100.0)))
            .spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(Vec2::new(300.0, 100.0)));
        let sim = Sim::new(&builder.build());
        assert_eq!(sim.object(crate::ids::ObjectId(1)).unwrap().energy, energy::start(ObjectKind::Disciple));
        serde_json::to_value(&sim).expect("a state serializes")
    }

    /// The checker's verdict on `json` loaded, with object `n`'s energy set to `value`.
    fn check_with(mut json: serde_json::Value, n: u32, value: serde_json::Value) -> Result<(), Violation> {
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        let object = objects.iter_mut().find(|o| o["id"] == n).expect("in the state");
        object["energy"] = value;
        let sim: Sim = serde_json::from_value(json).expect("the edited state loads");
        Invariants::new(&sim).check(&sim, &TickInputs::default())
    }

    /// The negative test: a state edited to have more energy than the pool holds, or energy on a type without a pool,
    /// breaks E1, which a loaded state as saved keeps.
    #[test]
    fn energy_above_the_maximum_or_without_a_pool_breaks_e1() {
        let max = energy::max(ObjectKind::Disciple).unwrap();
        assert_eq!(check_with(saved(), 1, max.into()), Ok(()), "at the maximum");
        let above = check_with(saved(), 1, (max + 1).into()).unwrap_err();
        assert_eq!(above.invariant, "E1", "{above:?}");
        assert!(above.message.contains("above its maximum"), "{above:?}");
        let recruit = check_with(saved(), 2, 5.into()).unwrap_err();
        assert_eq!(recruit.invariant, "E1", "{recruit:?}");
        assert!(recruit.message.contains("no energy pool"), "{recruit:?}");
    }
}
