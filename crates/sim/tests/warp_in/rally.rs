//! (M5-S6) A rally point on a building under Warp In (design change 0396: "A building that will produce or release
//! units can have its RallyPoint set while it is still under construction or transformation, for every faction.
//! Examples: [...] Colonists portals warping in"; over 0116's "no rally point"): a portal takes one and keeps it when
//! it completes; a cancelled Warp In takes it away with the portal; an Egress Node, which produces nothing, takes none.

use sc_sim::Rally;

use super::*;

/// The rally point of the tests: the centre of tile (3, 12), south-west of Beacon 1.
fn rally_point() -> Vec2 {
    at(3, 12).center()
}

fn rally(objects: &[u32], target: Target) -> Command {
    Command::Production {
        objects: objects.iter().map(|&n| id(n)).collect(),
        command: ProductionCommand::SetRally(target),
    }
}

fn validate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// Object `n`'s rally point, and whether it has a production at all.
fn rally_of(t: &TestSim, n: u32) -> Option<Option<Rally>> {
    object(t, n).production.as_ref().map(|production| production.rally)
}

/// Design change 0396 through a transformation: a Robo Portal warping in on Beacon 1 takes a rally point on the ground
/// (a Train still goes to no producer: it doesn't work yet, design change 0116), keeps it when its Warp In is complete
/// at the end of tick 640 ("The rally point is kept when the building completes"), and the Drone it then trains walks
/// there, its default command on the ground (design change 0136).
#[test]
fn a_portal_warping_in_takes_a_rally_point_and_keeps_it() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], warp(RoboPortal)));
    t.step();
    assert_eq!(rally_of(&t, 1), None, "its production held back");
    let ground = rally(&[1], Target::Point(rally_point()));
    assert_eq!(validate(&t, ground.clone()), Ok(()));
    next(&mut t, ground);
    assert_eq!(rally_of(&t, 1), Some(Some(Rally::Point(rally_point()))));
    assert_eq!(
        validate(&t, train(&[1], Drone)),
        Err(Reject::NoProducers),
        "not at work yet"
    );
    t.run_until(640);
    assert_eq!(completions(&t), [(640, 1, RoboPortal)]);
    assert_eq!(rally_of(&t, 1), Some(Some(Rally::Point(rally_point()))), "kept");
    next(&mut t, train(&[1], Drone));
    let drone_at_rally =
        |t: &TestSim| (t.sim().objects()).any(|o| o.kind == Drone && o.pos.distance(rally_point()) < 40.0);
    for _ in 0..600 {
        if drone_at_rally(&t) {
            return;
        }
        t.step();
    }
    panic!("no Drone at the rally point by tick {}", t.sim().tick());
}

/// A Warp In cancelled before it completes (`factions/colonists/economy.md > Cancel Warp In`: "the structure reverts
/// to being a normal, fully-functional Beacon") takes the rally point away with the portal: a Beacon has none, and
/// takes none. An Egress Node warping in, a building that produces nothing, takes none either
/// ([`Reject::NoProducers`]), as it takes none complete.
#[test]
fn a_cancelled_warp_in_loses_its_rally_point_and_an_egress_node_takes_none() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], warp(BioPortal)))
        .at(1, P1, construction(&[2], warp(EgressNode)));
    t.step();
    let ground = || rally(&[1], Target::Point(rally_point()));
    next(&mut t, ground());
    assert_eq!(rally_of(&t, 1), Some(Some(Rally::Point(rally_point()))));
    assert_eq!(
        validate(&t, rally(&[2], Target::Point(rally_point()))),
        Err(Reject::NoProducers),
        "an Egress Node"
    );
    next(&mut t, construction(&[1], CancelTransform));
    assert_eq!(object(&t, 1).kind, Beacon);
    assert_eq!(rally_of(&t, 1), None, "a Beacon has no RallyPoint");
    assert_eq!(validate(&t, ground()), Err(Reject::NoProducers));
    t.run_until(330);
    assert_eq!(completions(&t), [(320, 2, EgressNode)]);
    assert_eq!(
        validate(&t, rally(&[2], Target::Point(rally_point()))),
        Err(Reject::NoProducers),
        "complete, it takes none either"
    );
}
