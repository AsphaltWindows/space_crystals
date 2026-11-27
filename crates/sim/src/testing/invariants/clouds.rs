//! (M5-N4b) The cloud invariant C1 (`crate::shroud`, `Sim::clouds`): every cloud member that channels is a live unit
//! channelling Shroud, in the channel the member names (its order's start, where it stands, its type's radius:
//! `shroud::member`); (M6-CM) every lingering one vanishes after the end of the tick and within `Spells::linger` ticks
//! of it, at the tick it said the tick before (its time only runs down: [`Lingering`]); no channel is a member twice
//! (a unit's lingering circle and its new channel's being two); every unit that channels is in exactly one cloud; a
//! cloud's members are connected, and two clouds never touch ("a connected cloud is simply one cloud",
//! `factions/cults/units/disciple.md > Spells > Shroud`); and a buffer is never below 0, its timer running while it
//! holds anything and never past due.

use std::collections::BTreeMap;

use glam::Vec2;

use super::Violation;
use crate::ids::ObjectId;
use crate::ruleset::rules;
use crate::shroud::{self, Cloud};
use crate::sim::Sim;
use crate::units::Tick;

/// (M6-CM) What C1 remembers of the tick before: each lingering circle's channel (its unit and its start) and the
/// tick it vanishes.
#[derive(Clone, Debug, Default)]
pub(super) struct Lingering(BTreeMap<(ObjectId, Tick), Tick>);

impl Lingering {
    /// The lingering circles of `sim`'s clouds.
    pub(super) fn of(sim: &Sim) -> Self {
        let members = sim.clouds().list().iter().flat_map(|c| &c.members);
        Self(members.filter_map(|m| Some(((m.id, m.since), m.until?))).collect())
    }
}

/// C1 (module docs), `before` the lingering circles of the tick before.
pub(super) fn clouds(sim: &Sim, before: &Lingering) -> Result<(), Violation> {
    let broken = |message: String, at: Vec2| Violation {
        invariant: "C1",
        message,
        at,
    };
    let (list, tick) = (sim.clouds().list(), sim.tick());
    let last = tick.saturating_add(rules().tuning.spells.linger);
    let mut channels: Vec<(ObjectId, Tick)> = Vec::new();
    let mut members: Vec<ObjectId> = Vec::new();
    for (i, cloud) in list.iter().enumerate() {
        let at = cloud.members.first().map_or(Vec2::ZERO, |m| m.at);
        if let Some(problem) = problem(cloud, tick) {
            return Err(broken(format!("cloud {i} {problem}"), at));
        }
        for m in &cloud.members {
            channels.push((m.id, m.since));
            let Some(until) = m.until else {
                if sim.object(m.id).and_then(shroud::member) != Some(*m) {
                    return Err(broken(
                        format!("cloud {i}'s member #{} doesn't channel as it says", m.id.0),
                        m.at,
                    ));
                }
                members.push(m.id);
                continue;
            };
            if until <= tick || until > last {
                return Err(broken(
                    format!(
                        "cloud {i}'s member #{} lingers until tick {until}, at tick {tick}",
                        m.id.0
                    ),
                    m.at,
                ));
            }
            if let Some(&said) = before.0.get(&(m.id, m.since))
                && said != until
            {
                return Err(broken(
                    format!("cloud {i}'s member #{} lingered until tick {said}, now {until}", m.id.0),
                    m.at,
                ));
            }
        }
        if shroud::components(cloud.members.clone()).len() != 1 {
            return Err(broken(format!("cloud {i}'s circles aren't connected"), at));
        }
        let touching = |other: &Cloud| other.members.iter().any(|o| cloud.members.iter().any(|m| m.touches(o)));
        if let Some(j) = (list.iter().enumerate()).position(|(j, other)| j > i && touching(other)) {
            return Err(broken(format!("clouds {i} and {j} touch, which makes them one"), at));
        }
    }
    channels.sort_unstable();
    if let Some(twice) = channels.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(broken(
            format!("#{}'s channel is in two clouds", twice[0].0.0),
            Vec2::ZERO,
        ));
    }
    members.sort_unstable();
    if let Some(twice) = members.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(broken(format!("#{} channels twice", twice[0].0), Vec2::ZERO));
    }
    if let Some(m) = (sim.objects().filter_map(shroud::member)).find(|m| members.binary_search(&m.id).is_err()) {
        return Err(broken(format!("#{} channels in no cloud", m.id.0), m.at));
    }
    for cloud in sim.clouds().ended() {
        let at = cloud.members.first().map_or(Vec2::ZERO, |m| m.at);
        if cloud.members.is_empty() || !(cloud.buffer.is_finite() && cloud.buffer > 0.0) {
            return Err(broken(
                format!("an ended cloud waits with {} to release", cloud.buffer),
                at,
            ));
        }
    }
    Ok(())
}

/// What is wrong with `cloud`'s own state at the end of tick `tick`, if anything (module docs).
fn problem(cloud: &Cloud, tick: u32) -> Option<String> {
    if cloud.members.is_empty() {
        return Some("has no member".to_string());
    }
    if !(cloud.members.windows(2)).all(|pair| (pair[0].id, pair[0].since) < (pair[1].id, pair[1].since)) {
        return Some("lists its members out of id order".to_string());
    }
    if !(cloud.buffer.is_finite() && cloud.buffer >= 0.0) {
        return Some(format!("holds {} in its buffer", cloud.buffer));
    }
    match cloud.release_at {
        None if cloud.buffer > 0.0 => Some(format!("holds {} with no release due", cloud.buffer)),
        Some(at) if at <= tick => Some(format!("missed its release at tick {at}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::command::{Command, PlayerCommand, TickInputs};
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::orders::OrderKind;
    use crate::scenario::SpawnAt;
    use crate::testing::ScenarioBuilder;
    use crate::testing::invariants::Invariants;

    const P1: PlayerId = PlayerId(1);

    /// Disciples 1 and 2 of player 1, 5 GU apart, channelling one cloud, and Disciple 3 far off, idle: as saved.
    fn saved() -> serde_json::Value {
        let builder = ScenarioBuilder::new(&"..............................\n".repeat(12))
            .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(Vec2::new(300.0, 300.0)))
            .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(Vec2::new(620.0, 300.0)))
            .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(Vec2::new(1600.0, 300.0)));
        let mut sim = Sim::new(&builder.build());
        let shroud = Command::Order {
            units: vec![ObjectId(1), ObjectId(2)],
            kind: OrderKind::Shroud,
            queue: false,
        };
        let inputs = TickInputs {
            commands: vec![PlayerCommand {
                player: P1,
                command: shroud,
            }],
        };
        sim.step(&inputs);
        Invariants::new(&sim).check(&sim, &inputs).expect("as saved, C1 holds");
        assert_eq!(sim.clouds().list().len(), 1, "one cloud of two");
        serde_json::to_value(&sim).expect("a state serializes")
    }

    /// The checker's verdict on `json` loaded, after `edit`.
    fn check_with(mut json: serde_json::Value, edit: impl FnOnce(&mut serde_json::Value)) -> Result<(), Violation> {
        edit(&mut json);
        let sim: Sim = serde_json::from_value(json).expect("the edited state loads");
        Invariants::new(&sim).check(&sim, &TickInputs::default())
    }

    fn broken(result: Result<(), Violation>, says: &str) {
        let violation = result.expect_err(says);
        assert_eq!(violation.invariant, "C1", "{violation:?}");
        assert!(violation.message.contains(says), "{violation:?}");
    }

    /// The negative test: a state edited so that a channelling Disciple is in no cloud, a member doesn't channel as it
    /// says, a cloud splits in two that touch, or a buffer holds damage with no release due, breaks C1, which the state
    /// as saved keeps.
    #[test]
    fn a_cloud_that_isn_t_its_channels_breaks_c1() {
        assert_eq!(check_with(saved(), |_| {}), Ok(()));
        broken(
            check_with(saved(), |json| json["clouds"] = serde_json::json!({})),
            "channels in no cloud",
        );
        broken(
            check_with(saved(), |json| {
                json["clouds"]["clouds"][0]["members"][1]["since"] = 7.into()
            }),
            "doesn't channel as it says",
        );
        broken(
            check_with(saved(), |json| {
                let cloud = json["clouds"]["clouds"][0].clone();
                let (mut one, mut two) = (cloud.clone(), cloud);
                one["members"].as_array_mut().unwrap().truncate(1);
                two["members"].as_array_mut().unwrap().remove(0);
                json["clouds"]["clouds"] = serde_json::json!([one, two]);
            }),
            "touch, which makes them one",
        );
        broken(
            check_with(saved(), |json| json["clouds"]["clouds"][0]["buffer"] = 4.0.into()),
            "with no release due",
        );
    }

    /// (M6-CM) [`saved`] a tick on, Disciple 2 stopped in tick 2: its circle lingers until tick 26.
    fn lingering() -> serde_json::Value {
        let mut sim: Sim = serde_json::from_value(saved()).expect("it loads");
        let stop = Command::Order {
            units: vec![ObjectId(2)],
            kind: OrderKind::Stop,
            queue: false,
        };
        let inputs = TickInputs {
            commands: vec![PlayerCommand {
                player: P1,
                command: stop,
            }],
        };
        sim.step(&inputs);
        Invariants::new(&sim).check(&sim, &inputs).expect("lingering, C1 holds");
        let members = &sim.clouds().list()[0].members;
        assert_eq!(members[1].until, Some(26), "{members:?}");
        serde_json::to_value(&sim).expect("a state serializes")
    }

    /// (M6-CM) The negative test of the linger: a lingering circle edited to vanish past the linger, or in the tick
    /// gone, or to another tick than it said the tick before, breaks C1.
    #[test]
    fn a_linger_past_its_time_or_moved_breaks_c1() {
        assert_eq!(check_with(lingering(), |_| {}), Ok(()));
        let until = |value: u32| {
            move |json: &mut serde_json::Value| json["clouds"]["clouds"][0]["members"][1]["until"] = value.into()
        };
        broken(check_with(lingering(), until(27)), "lingers until tick 27, at tick 2");
        broken(check_with(lingering(), until(2)), "lingers until tick 2, at tick 2");
        let sim: Sim = serde_json::from_value(lingering()).expect("it loads");
        let mut moved = lingering();
        until(25)(&mut moved);
        let moved: Sim = serde_json::from_value(moved).expect("the edited state loads");
        broken(
            Invariants::new(&sim).check_rules(&moved, &TickInputs::default()),
            "lingered until tick 26, now 25",
        );
    }
}
