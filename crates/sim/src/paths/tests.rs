//! Whose grids a unit plans on, and the hash of the path knowledge.

use glam::Vec2;

use super::knowledge::PathKnowledge;
use crate::defs::ObjectKind;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::nav::{NavClass, NavRules};
use crate::object::Object;
use crate::scenario::SpawnAt;
use crate::testing::ScenarioBuilder;
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const CLASS: NavClass = NavClass::GroundRugged;

fn hash_of(knowledge: &PathKnowledge) -> u64 {
    let mut h = StateHasher::new();
    knowledge.hash(&mut h);
    h.finish()
}

/// A unit plans on its owner's known grids; a unit without an owner (neutral, or a defeated player's: design change
/// 0101) on the true ones, as nobody's knowledge is its own and it sees nothing to learn from; and a unit standing
/// on a tile its owner's knowledge blocks on the true ones too, until it has seen where it stands.
#[test]
fn units_plan_on_their_owners_grids_and_ownerless_ones_on_the_true_grids() {
    let t = ScenarioBuilder::new(&"....................\n".repeat(8))
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(TilePos::new(15, 3))) // 1: unseen by player 1
        .spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(Vec2::new(96.0, 96.0))) // 2
        .spawn(ObjectKind::Peacekeeper, None, SpawnAt::Point(Vec2::new(96.0, 224.0))) // 3: neutral
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(18, 7))) // 4: assumed by both
        .start();
    let state = t.sim().nav_state();
    let generator = TilePos::new(15, 3);
    let pos = Vec2::new(96.0, 96.0);
    let (truth, grid) = state.plan_grid(None, CLASS, 1, pos).unwrap();
    assert_eq!(truth, 0);
    assert!(grid.is_blocked(generator), "the true grid");
    let (own, grid) = state.plan_grid(Some(P1), CLASS, 1, pos).unwrap();
    assert_eq!(own, 1, "player 1's grids come after the true ones");
    assert!(!grid.is_blocked(generator), "player 1 has never seen it");
    assert!(
        grid.is_blocked(TilePos::new(18, 7)),
        "the map's patch is known from the start"
    );
    // Player 2 has no unit that walks: its grids aren't built, and a unit of its would fall back on the true ones.
    assert!(state.known(P2).is_none());
    assert_eq!(state.plan_grid(Some(P2), CLASS, 1, pos).unwrap().0, 0);
    // A player who isn't in the game.
    assert_eq!(state.plan_grid(Some(PlayerId(7)), CLASS, 1, pos).unwrap().0, 0);
    // Standing on a tile the owner's knowledge blocks (the assumed patch): the true grid.
    let on_the_patch = TilePos::new(18, 7).center();
    assert_eq!(state.plan_grid(Some(P1), CLASS, 1, on_the_patch).unwrap().0, 0);
    // No grid of that class and size at all.
    assert!(state.plan_grid(Some(P1), CLASS, 3, pos).is_none());
    assert!(std::ptr::eq(state.planning(None), state.nav()));
    assert!(std::ptr::eq(state.planning(Some(P1)), state.known(P1).unwrap()));
    assert!(std::ptr::eq(state.planning(Some(P2)), state.nav()));
}

/// The knowledge is hashed whole: versions, who assumes what, and where.
#[test]
fn the_path_knowledge_hash_covers_versions_and_assumed_resources() {
    let t = ScenarioBuilder::new(&"....................\n".repeat(8))
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(18, 7))) // 1
        .spawn(
            ObjectKind::SupplyDeliveryStation,
            None,
            SpawnAt::Tile(TilePos::new(10, 2)),
        ) // 2
        .spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(Vec2::new(96.0, 96.0))) // 3
        .start();
    let base = t.sim().path_knowledge().clone();
    let assumed = |player| -> Vec<ObjectId> { base.player(player).unwrap().assumed().iter().map(|a| a.id).collect() };
    assert_eq!(assumed(P1), [ObjectId(1), ObjectId(2)]);
    assert_eq!(assumed(P2), [ObjectId(1), ObjectId(2)]);
    assert_eq!(
        base.player(P1).unwrap().assumed()[1].footprint.w,
        2,
        "the station's footprint"
    );

    let mut bumped = base.clone();
    bumped.bump(P2);
    assert_eq!((bumped.version(P1), bumped.version(P2)), (Some(0), Some(1)));
    assert_ne!(hash_of(&bumped), hash_of(&base));

    // Without the station among the start's objects: one resource less to assume, for both.
    let objects: Vec<&Object> = t.sim().objects().collect();
    let fewer = PathKnowledge::start([P1, P2], &[objects[0], objects[2]], NavRules::default());
    assert_ne!(hash_of(&fewer), hash_of(&base));
    // Other players, the same resources.
    let others = PathKnowledge::start([P1, PlayerId(3)], &objects, NavRules::default());
    assert_ne!(hash_of(&others), hash_of(&base));
    // A resource elsewhere.
    let mut json = serde_json::to_value(&base).unwrap();
    json[0]["assumed"][0]["footprint"]["origin"]["x"] = 17.into();
    let moved: PathKnowledge = serde_json::from_value(json).unwrap();
    assert_ne!(hash_of(&moved), hash_of(&base));
    assert_eq!(hash_of(&base.clone()), hash_of(&base));
}
