//! Design change 0159 (`systems/controls/behaviors.md > PathKnowledge`, map-placed objects; answering question Q0033
//! point 3): every neutral object the map maker placed is known to every player from the start, "pathing and
//! memory", like terrain: each player has a record of it from the start, as of the start. The players' starting
//! layouts are never pre-known ("would leak start position and faction"), nor is anything placed after the start.
//! Elsewhere: a Gather on a map patch never seen, and a patch placed later (`gathering_rules/remembered.rs`); the
//! Drones' search (`gathering_rules/knowledge.rs`); a never-seen crystal field routed around
//! (`path_knowledge/plans.rs`); the records themselves (`vision_memory.rs`).
//!
//! Every `TestSim` checks the invariants after each tick, V1 among them (the records are those a fresh build keeps).

use sc_sim::nav::NavClass;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SU_PER_GU, SpawnAt, Target, TargetView,
    TilePos, Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;

/// A point given in tiles.
fn at(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

/// 40 × 20 open tiles: `kind` of `owner` with its footprint's corner on tile (15, 4), and player 1's Peacekeeper 2
/// on (4.5, 6.0), more than 10 tiles west of it, out of sight (5 tiles); no unit fights by itself.
fn scene(kind: ObjectKind, owner: Option<PlayerId>) -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(40); 20].join("\n"))
        .spawn(kind, owner, SpawnAt::Tile(TilePos::new(15, 4))) // 1
        .spawn(PK, Some(P1), SpawnAt::Point(at(4.5, 6.0))) // 2
        .start();
    t.without_auto_targeting();
    t
}

/// Whether player 1 plans around tile `(x, y)`: its known grid blocks it.
fn known_blocked(t: &TestSim, x: i32, y: i32) -> bool {
    let nav = t.sim().known_nav(P1).expect("player 1 has a walking unit");
    nav.grid(NavClass::GroundRugged, 1)
        .unwrap()
        .is_blocked(TilePos::new(x, y))
}

/// The first plan of a Move of the Peacekeeper to `(26.5, 6.0)`, through the object's footprint.
fn first_plan(t: &mut TestSim) -> Vec<Vec2> {
    let goal = at(26.5, 6.0);
    t.order_at(1, P1, &[ObjectId(2)], OrderKind::Move(Target::Point(goal)), false);
    t.step();
    t.mover(2).path.clone()
}

/// 0159's Verification: "A neutral pre-placed obstacle is routed around before ever being seen; an enemy's starting
/// DC isn't." The map's Supply Delivery Station is in both players' records from the start, never seen (its tiles
/// Unexplored, its record as of tick 0), and blocks player 1's plans: the first plan goes around it. Player 2's
/// Deployment Center, of its starting layout, is in no record of player 1's, a target player 1 doesn't know, and
/// open ground for its plans: the first plan goes straight through (collision still keeps the unit out of it,
/// `path_knowledge/collision.rs`).
#[test]
fn a_neutral_map_object_is_routed_around_before_it_is_ever_seen_and_an_enemy_dc_is_not() {
    let mut t = scene(ObjectKind::SupplyDeliveryStation, None);
    assert_eq!(
        t.sim().vision().state(P1, TilePos::new(15, 5)),
        VisibilityState::Unexplored
    );
    for player in [P1, P2] {
        let record = t.sim().vision().player(player).unwrap().record(ObjectId(1)).cloned();
        assert_eq!(
            record.map(|r| (r.kind, r.seen)),
            Some((ObjectKind::SupplyDeliveryStation, 0))
        );
    }
    assert!(matches!(
        t.sim().target_view(P1, ObjectId(1)),
        TargetView::Remembered { .. }
    ));
    assert!(known_blocked(&t, 15, 4) && known_blocked(&t, 16, 5));
    let plan = first_plan(&mut t);
    assert!(plan.len() > 2, "around it: {plan:?}");

    let mut t = scene(ObjectKind::DeploymentCenter, Some(P2));
    assert_eq!(t.sim().vision().player(P1).unwrap().records().count(), 0);
    assert_eq!(t.sim().target_view(P1, ObjectId(1)), TargetView::Gone);
    let attack = PlayerCommand {
        player: P1,
        command: Command::Order {
            units: vec![ObjectId(2)],
            kind: OrderKind::Attack(ObjectId(1)),
            queue: false,
        },
    };
    assert_eq!(t.sim().validate(&attack), Err(Reject::NotVisible(ObjectId(1))));
    assert!(!known_blocked(&t, 15, 6) && !known_blocked(&t, 18, 6));
    let plan = first_plan(&mut t);
    assert_eq!(plan, [at(4.5, 6.0), at(26.5, 6.0)], "straight through");
}
