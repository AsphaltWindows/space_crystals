//! The indicators' colours (`systems/controls/hud.md > Color language`; design change 0201's gathering
//! markers).

use super::*;

#[test]
fn indicator_colours_follow_the_colour_language() {
    let p = Vec2::new(10.0, 20.0);
    assert_eq!(order_color(OrderKind::Move(Target::Point(p))), Some(MOVE_COLOR));
    assert_eq!(
        order_color(OrderKind::Move(Target::Object(ObjectId(3)))),
        Some(MOVE_COLOR)
    );
    assert_eq!(order_color(OrderKind::Attack(ObjectId(3))), Some(ATTACK_COLOR));
    assert_eq!(order_color(OrderKind::AttackGround(p)), Some(ATTACK_COLOR));
    assert_eq!(order_color(OrderKind::AttackMove(p)), Some(AGGRESSIVE_COLOR));
    assert_eq!(order_color(OrderKind::Patrol(p)), Some(AGGRESSIVE_COLOR));
    assert_eq!(order_color(OrderKind::Hold), None);
    assert_eq!(order_color(OrderKind::Stop), None);
    assert_eq!(order_color(OrderKind::Gather(ObjectId(3))), Some(GATHER_COLOR));
    assert_eq!(order_color(OrderKind::DropOff(ObjectId(3))), Some(RETURN_COLOR));
}

/// Design change 0201: each gathering marker has a colour none of the others has, the colour language's
/// (`systems/controls/hud.md > Color language`) included; their shapes are the drawing's own
/// (`gathering::draw`).
#[test]
fn the_gathering_markers_have_colours_of_their_own() {
    let mut colors: Vec<[u8; 4]> = GatheringMarker::ALL
        .map(|m| m.color().to_srgba().to_u8_array())
        .to_vec();
    colors.extend([MOVE_COLOR, ATTACK_COLOR, AGGRESSIVE_COLOR].map(|c| c.to_srgba().to_u8_array()));
    let distinct: BTreeSet<[u8; 4]> = colors.iter().copied().collect();
    assert_eq!(distinct.len(), colors.len(), "{colors:?}");
}

/// (M6-GD) The leash ring: a selected drone, or its selected operator, or both, show its leash once, round the operator;
/// a Peacekeeper, or an operator whose drone is in stock, none, and another player's selection none. Every point of the
/// ring is where the drone's Silhouette would be its leash from the operator's, 8 grid units edge to edge
/// (`sc_sim::tether::past_leash`): just inside it the drone flies on, just outside it falls.
#[test]
fn the_leash_ring_runs_where_the_drone_meets_its_leash() {
    use sc_sim::testing::ScenarioBuilder;
    use sc_sim::{Command, Faction, ObjectKind, PlayerId, SpawnAt, tether};

    use crate::selection::Selection;
    let (p1, operator) = (PlayerId(1), ObjectKind::DroneOperator);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(40); 40].join("\n"))
        .faction(p1, Faction::Gdo)
        .spawn(operator, Some(p1), SpawnAt::Point(Vec2::new(1280.0, 1280.0)))
        .spawn(operator, Some(p1), SpawnAt::Point(Vec2::new(300.0, 300.0)))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(p1),
            SpawnAt::Point(Vec2::new(1280.0, 1380.0)),
        )
        .start();
    let launch = Command::Order {
        units: vec![ObjectId(1)],
        kind: OrderKind::Launch,
        queue: false,
    };
    t.at(2, p1, launch);
    t.run_until(2);
    let drone = ObjectId(4);
    let sim = t.sim();
    assert_eq!(sim.object(drone).map(|o| o.kind), Some(ObjectKind::OperatorDrone));
    let held = leash::Leash {
        operator: ObjectId(1),
        drone,
    };
    let of = |ids: &[u32]| {
        let kind = |n: u32| sim.object(ObjectId(n)).expect("in the game").kind;
        let selection = Selection::new(ids.iter().map(|&n| (ObjectId(n), kind(n))));
        leash::leashes(sim, &selection, p1)
    };
    assert_eq!([of(&[4]), of(&[1]), of(&[1, 4])], [vec![held], vec![held], vec![held]]);
    assert_eq!([of(&[3]), of(&[2])], [vec![], vec![]], "no drone out");
    let theirs = Selection::new([(drone, ObjectKind::OperatorDrone)]);
    assert_eq!(leash::leashes(sim, &theirs, PlayerId(2)), []);

    let ring = leash::ring(sim, held, 1.0).expect("a leash");
    assert_eq!(ring.first(), ring.last(), "closed");
    let (centre, half) = (sim.object(ObjectId(1)).expect("the operator").pos, Vec2::splat(24.0));
    let gap = |p: Vec2| ((p - centre).abs() - half).max(Vec2::ZERO).length();
    for &p in &ring {
        assert!((gap(p) - 512.0).abs() < 0.01, "{p}: {}", gap(p));
    }
    let operator = sim.object(ObjectId(1)).expect("the operator");
    let mut flown = sim.object(drone).expect("out").clone();
    let east = ring[0];
    flown.pos = east - Vec2::new(1.0, 0.0);
    assert!(!tether::past_leash(&flown, operator), "inside");
    flown.pos = east + Vec2::new(1.0, 0.0);
    assert!(tether::past_leash(&flown, operator), "outside");
}
