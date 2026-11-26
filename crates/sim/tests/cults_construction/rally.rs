//! (M5-S6) A rally point on a building its Recruits are still building (design change 0396;
//! `systems/controls/selection.md > SelectionGroup`: "a building that will produce or release units can have its
//! **RallyPoint set while still under construction or transformation** [...]. The rally point is kept when it
//! completes"): an Armory's site takes one and keeps it, and its Militants go there; a building that never produces nor
//! releases units takes none.

use sc_sim::{ProductionCommand, Rally, RecruitKind, Target};

use super::*;

/// A ground point south-west of [`SITE`], beyond the Armory's exit side, C, which faces west unturned (x = 1280).
const RALLY: Vec2 = Vec2::new(900.0, 1400.0);

fn rally(objects: &[ObjectId], target: Target) -> Command {
    Command::Production {
        objects: objects.to_vec(),
        command: ProductionCommand::SetRally(target),
    }
}

/// Player 1's `command` in the next tick, which runs ([`step`]).
fn next(t: &mut TestSim, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command);
    step(t);
}

/// What `id`'s production holds: its rally point and how many entries it has queued; `None` without one.
fn held_rally(t: &TestSim, id: ObjectId) -> Option<(Option<Rally>, usize)> {
    (object(t, id).production.as_ref()).map(|production| (production.rally, production.queue.len()))
}

/// Design change 0396's Verification: "Start an Armory, select it while it's being built, and right-click ground: a
/// rally marker appears. After completion, trained units go there." The sim's half (the client's is
/// `crates/game/tests/commands/recruit_ui/unfinished_rally.rs`): player 1's four Recruits build an Armory at [`SITE`],
/// whose site takes a rally point on the ground, as on the finished building. Its production holds that alone, and
/// nothing else of a producer's works yet: a Train goes to no producer ([`Reject::NoProducers`]). A rally point on the
/// site itself clears it (design change 0086), and the site has no production again. Complete, the Armory keeps the
/// point, and the Militant it trains of a Recruit that went in comes out of side C and walks there.
#[test]
fn an_armory_under_construction_takes_a_rally_point_and_its_militants_go_there() {
    let mut t = world(4);
    t.at(2, P1, build(&recruits(4), ARMORY, SITE, false));
    run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    assert_eq!(held_rally(&t, site), None, "its production held back");
    let ground = rally(&[site], Target::Point(RALLY));
    assert_eq!(gate(&t, P1, ground.clone()), Ok(()));
    next(&mut t, ground.clone());
    assert_eq!(held_rally(&t, site), Some((Some(Rally::Point(RALLY)), 0)));
    let train = Command::Production {
        objects: vec![site],
        command: ProductionCommand::TrainFrom {
            kind: ObjectKind::Militant,
            from: RecruitKind::Plain,
        },
    };
    assert_eq!(gate(&t, P1, train.clone()), Err(Reject::NoProducers), "not at work yet");
    next(&mut t, rally(&[site], Target::Object(site)));
    assert_eq!(held_rally(&t, site), None, "cleared");
    next(&mut t, ground);
    run_to(&mut t, 1000, "the Armory is built", |t| !built(t).is_empty());
    assert!(!object(&t, site).is_site());
    assert_eq!(held_rally(&t, site), Some((Some(Rally::Point(RALLY)), 0)), "kept");
    next(&mut t, order(&[2], OrderKind::Enter(site), false));
    run_to(&mut t, WALK, "the Recruit goes in", |t| {
        !t.sim().held_in(site).is_empty()
    });
    assert_eq!(gate(&t, P1, train.clone()), Ok(()));
    next(&mut t, train);
    let militant_at_rally =
        |t: &TestSim| (t.sim().objects()).any(|o| o.kind == ObjectKind::Militant && o.pos.distance(RALLY) < 40.0);
    run_to(&mut t, 600, "the Militant walks to the rally point", militant_at_rally);
}

/// A building that never produces nor releases units takes no rally point unfinished, as it takes none finished
/// ([`Reject::NoProducers`]): a Storage's site, and a Garrison's, which will let its units out but "has no rally point"
/// (`factions/cults/structures/garrison.md > Placeholder stats > Panel`). The Garrison stands behind an Armory, its
/// TechPrerequisite, which player 1 has from the start (2); their Recruits are 3 to 6.
#[test]
fn a_building_that_produces_nothing_takes_no_rally_point_unfinished() {
    let mut scenario = cults().spawn(ARMORY, Some(P1), tile(2, 6));
    for i in 0..4 {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(start_of(i, 4)));
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    t.at(2, P1, build(&[3, 4], STORAGE, SITE, false));
    t.at(2, P1, build(&[5, 6], ObjectKind::Garrison, SITE_2, false));
    run_to(&mut t, WALK, "both sites start", |t| sites(t).len() == 2);
    let point = Target::Point(RALLY);
    for site in sites(&t) {
        let kind = object(&t, site).kind;
        assert_eq!(
            gate(&t, P1, rally(&[site], point)),
            Err(Reject::NoProducers),
            "{kind:?}"
        );
    }
    assert_eq!(gate(&t, P1, rally(&sites(&t), point)), Err(Reject::NoProducers));
    assert_eq!(gate(&t, P1, rally(&[id(2)], point)), Ok(()), "the Armory takes one");
}
