//! (M5-N5) The buff invariant F1 (`Object::buffs`, `crate::buffs`): no object holds two buffs of the same ability
//! ("effects of the same ability **don't stack**", `factions/cults/structures/shrine.md > Placeholder stats`), every
//! buff ends within its ability's duration from now and works one tick more at least (a buff is dropped in the tick of
//! its last effect), and only a kind the ability affects holds its buff, a unit held off the map as one on it.

use super::Violation;
use crate::object::Object;
use crate::sim::Sim;
use crate::units::Tick;

/// F1: every object's buffs, and every held unit's (module docs).
pub(super) fn buffs(sim: &Sim) -> Result<(), Violation> {
    let now = sim.tick();
    (sim.objects().chain(sim.held().map(|held| &held.object))).try_for_each(|o| its_buffs(o, now))
}

/// F1 for one object, at the end of tick `now`.
fn its_buffs(o: &Object, now: Tick) -> Result<(), Violation> {
    let problem = |problem: String| Violation {
        invariant: "F1",
        message: format!("#{} ({:?}) {problem}", o.id.0, o.kind),
        at: o.pos,
    };
    for (i, buff) in o.buffs.iter().enumerate() {
        let ability = buff.ability;
        if o.buffs[..i].iter().any(|before| before.ability == ability) {
            return Err(problem(format!("holds {ability:?} twice")));
        }
        if !ability.affects(o.kind) {
            return Err(problem(format!("holds {ability:?}, which doesn't affect its kind")));
        }
        let end = now.saturating_add(ability.duration());
        if buff.until <= now.saturating_add(1) || buff.until > end {
            let until = buff.until;
            return Err(problem(format!(
                "holds {ability:?} until tick {until}, not after tick {} and by tick {end}",
                now + 1
            )));
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
    use crate::defs::{AreaAbility, ObjectKind};
    use crate::ids::PlayerId;
    use crate::scenario::SpawnAt;
    use crate::testing::ScenarioBuilder;
    use crate::testing::invariants::Invariants;

    const P1: PlayerId = PlayerId(1);

    /// A Disciple (1) and a Recruit (2) of player 1 on a flat map, as saved before its first tick (tick 0).
    fn saved() -> serde_json::Value {
        let builder = ScenarioBuilder::new(&"..........\n".repeat(8))
            .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(Vec2::new(100.0, 100.0)))
            .spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(Vec2::new(300.0, 100.0)));
        let sim = Sim::new(&builder.build());
        assert_eq!(sim.tick(), 0);
        serde_json::to_value(&sim).expect("a state serializes")
    }

    /// The checker's verdict on `json` loaded, with object `n`'s buffs set to `buffs`.
    fn check_with(mut json: serde_json::Value, n: u32, buffs: serde_json::Value) -> Result<(), Violation> {
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        let object = objects.iter_mut().find(|o| o["id"] == n).expect("in the state");
        object["buffs"] = buffs;
        let sim: Sim = serde_json::from_value(json).expect("the edited state loads");
        Invariants::new(&sim).check(&sim, &TickInputs::default())
    }

    fn buff(ability: AreaAbility, until: Tick) -> serde_json::Value {
        json!({ "ability": ability, "until": until })
    }

    /// The negative test: a state edited so that a unit holds an ability's buff twice, a buff that outlasts its
    /// ability's duration or whose time is up, or a buff of an ability that doesn't affect its kind, breaks F1, which
    /// the buffs a cast gives keep.
    #[test]
    fn two_buffs_of_an_ability_too_long_a_buff_or_one_of_another_kind_breaks_f1() {
        let (health, energy) = (AreaAbility::HealthRegeneration, AreaAbility::EnergyRegeneration);
        let both = json!([buff(health, 240), buff(energy, 320)]);
        assert_eq!(check_with(saved(), 1, both), Ok(()), "a cast's, in tick 0");
        assert_eq!(check_with(saved(), 2, json!([buff(health, 2)])), Ok(()), "a tick more");
        let twice = check_with(saved(), 1, json!([buff(health, 100), buff(health, 200)])).unwrap_err();
        assert_eq!(twice.invariant, "F1", "{twice:?}");
        assert!(twice.message.contains("twice"), "{twice:?}");
        let long = check_with(saved(), 2, json!([buff(health, 241)])).unwrap_err();
        assert_eq!(long.invariant, "F1", "{long:?}");
        assert!(long.message.contains("by tick 240"), "{long:?}");
        let over = check_with(saved(), 2, json!([buff(health, 1)])).unwrap_err();
        assert_eq!(over.invariant, "F1", "{over:?}");
        let recruit = check_with(saved(), 2, json!([buff(energy, 100)])).unwrap_err();
        assert_eq!(recruit.invariant, "F1", "{recruit:?}");
        assert!(recruit.message.contains("doesn't affect its kind"), "{recruit:?}");
    }
}
