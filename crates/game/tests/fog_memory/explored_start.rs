//! (Slice M5-S7) The map explored from the start (design change 0416, `core/vision.md > Starting exploration`), as the
//! client shows it at 0:00: the fog dims every tile out of the local player's sight instead of hiding it in black, and
//! there the neutral objects show as remembered, at their starting state, while no other player's object shows,
//! neither live nor remembered (bases must still be scouted); the live objects show only where Visible. Without it,
//! the map starts unexplored as before: black out of sight, and nothing remembered.

use sc_sim::VisibilityState;
use sc_sim::testing::ScenarioBuilder;
use space_crystals::fog::{EXPLORED_ALPHA, FogAssets, UNEXPLORED_ALPHA};

use super::*;

/// The objects of [`explored_world`], by id.
const OWN_GENERATOR: u32 = 1;
const ENEMY_GENERATOR: u32 = 2;
const ENEMY_PK: u32 = 3;
const NEAR_PATCH: u32 = 4;
const FAR_PATCH: u32 = 5;
const FAR_STATION: u32 = 6;

/// A flat 48 × 12 map: player 1's Generator at (1, 1), with a patch beside it at (4, 3); far east, player 2's Generator
/// at (44, 1) and Peacekeeper; in between, out of both players' sight, a patch at (24, 2) and a station at (24, 8).
/// The map starts explored, or not.
fn explored_world(explored: bool) -> App {
    let map = vec![".".repeat(48); 12].join("\n");
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let scenario = ScenarioBuilder::new(&map)
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Generator, Some(P2), tile(44, 1))
        .spawn(ObjectKind::Peacekeeper, Some(P2), SpawnAt::Point(centre(42, 6)))
        .spawn(ObjectKind::CrystalPatch, None, tile(4, 3))
        .spawn(ObjectKind::CrystalPatch, None, tile(24, 2))
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(24, 8))
        .build();
    let mut config = AppConfig::new(scenario);
    config.scenario.setup.config.start_explored = explored;
    headless(config)
}

/// The fog texture's alpha over tile (`x`, `y`).
fn fog_alpha(app: &App, x: u32, y: u32) -> u8 {
    let world = app.world();
    let image = world
        .resource::<Assets<Image>>()
        .get(&world.resource::<FogAssets>().texture)
        .unwrap();
    image.data.as_ref().unwrap()[((y * image.width() + x) * 4 + 3) as usize]
}

fn byte(alpha: f32) -> u8 {
    (alpha * 255.0).round() as u8
}

/// Design change 0416's Verification, the client's half: "Start a game: the map shows dimmed terrain and crystal
/// patches; the enemy base isn't shown."
#[test]
fn at_0_00_the_map_shows_explored_with_its_neutral_objects_and_no_enemy_base() {
    let mut app = explored_world(true);
    assert_eq!(tick(&app), 0);
    app.update();
    let sim = app.world().resource::<SimRes>();
    let sight = Sight::new(sim, P1, true);
    let (w, h) = (sim.map().width() as i32, sim.map().height() as i32);
    let tiles = (0..h).flat_map(|y| (0..w).map(move |x| TilePos::new(x, y)));
    assert!(
        tiles.clone().all(|t| sight.tile(t) != VisibilityState::Unexplored),
        "every tile explored"
    );
    assert_eq!(sight.tile(TilePos::new(24, 2)), VisibilityState::Explored);
    assert_eq!(fog_alpha(&app, 24, 2), byte(EXPLORED_ALPHA), "dimmed, not black");
    assert_eq!(fog_alpha(&app, 44, 1), byte(EXPLORED_ALPHA), "the enemy's tiles too");
    assert_eq!(
        remembered(&app),
        [FAR_PATCH, FAR_STATION],
        "the neutral objects out of sight, no enemy's"
    );
    let sim = app.world().resource::<SimRes>();
    let patch = sim
        .vision()
        .player(P1)
        .unwrap()
        .record(ObjectId(FAR_PATCH))
        .expect("remembered");
    assert_eq!(patch.seen, 0, "as at the start");
    assert_eq!(
        patch.resource,
        sim.object(ObjectId(FAR_PATCH)).unwrap().resource,
        "its starting amount"
    );
    for (id, shown) in [
        (OWN_GENERATOR, true),
        (NEAR_PATCH, true),
        (ENEMY_GENERATOR, false),
        (ENEMY_PK, false),
        (FAR_PATCH, false),
        (FAR_STATION, false),
    ] {
        assert_eq!(drawn(&app, id), shown, "object {id}: live only where Visible");
    }
}

/// With the option off the map starts unexplored, as before: black out of sight, nothing remembered.
#[test]
fn without_it_the_map_starts_unexplored() {
    let mut app = explored_world(false);
    app.update();
    assert_eq!(fog_alpha(&app, 24, 2), byte(UNEXPLORED_ALPHA), "black");
    assert_eq!(remembered(&app), Vec::<u32>::new());
    assert!(!drawn(&app, FAR_PATCH) && !drawn(&app, ENEMY_GENERATOR));
}
