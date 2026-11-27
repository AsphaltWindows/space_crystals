//! The fog and object layers of the minimap's image, painted from real sims (`systems/controls/hud.md > Minimap >
//! What it shows`): "Fog of war, as on the main map": an Unexplored tile black; an Explored one dimmed, showing only
//! the remembered structures and patches, as last seen; a live enemy unit only on a Visible tile; a hidden object
//! never, but to its owner.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{DebugCommand, Faction, Object, ObjectId, ObjectKind, Sim, SpawnAt, TilePos, VisibilityState};

use super::super::raster::{Terrain, UNEXPLORED, dimmed, marks, paint, texel};
use super::*;
use crate::fog::Sight;
use crate::map_render::tile_base_color;
use crate::object_render::player_color;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// The objects of [`scouted`], by id.
const SCOUT: u32 = 1;
const BARRACKS: u32 = 2;
const GENERATOR: u32 = 3;
const PATCH: u32 = 4;
/// Player 2's Peacekeeper beside the scout, and the one beside the Barracks.
const NEAR_ENEMY: u32 = 5;
const FAR_ENEMY: u32 = 6;

/// The centre of tile (`x`, `y`).
fn center(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// A frame of four texels a tile for `sim`'s map.
fn frame_of(sim: &Sim) -> MinimapFrame {
    let extent = crate::camera::map_extent_su(sim.map());
    MinimapFrame::in_box(Rect::from_corners(Vec2::ZERO, extent / 16.0), 1.0, extent)
}

/// The minimap's image of `sim` as `sight` shows it, in `frame`.
fn draw(sim: &Sim, sight: &Sight, frame: &MinimapFrame) -> Vec<u8> {
    let map = sim.map();
    let terrain = Terrain::bake(map, frame);
    let states = sight.tile_states(map.width(), map.height());
    let mut data = vec![0; (frame.texels.x * frame.texels.y * 4) as usize];
    paint(&mut data, frame, &terrain, &states, &marks(sim, sight));
    data
}

/// The texel of `image` (in `frame`) that shows the ground point `ground`.
fn texel_of(image: &[u8], frame: &MinimapFrame, ground: Vec2) -> [u8; 4] {
    let at = frame.texel_at(ground).floor().as_uvec2();
    let i = ((at.y * frame.texels.x + at.x) * 4) as usize;
    image[i..i + 4].try_into().unwrap()
}

/// Tile (`x`, `y`)'s own colour, as Visible ground shows it.
fn terrain(sim: &Sim, x: i32, y: i32) -> Color {
    Color::from(tile_base_color(sim.map(), TilePos::new(x, y)))
}

fn object(sim: &Sim, id: u32) -> &Object {
    sim.object(ObjectId(id)).expect("in the game")
}

/// A flat 32 × 16 map with a water tile at (1, 10): player 1's Peacekeeper, the scout (1), at the centre of tile
/// (3, 8); player 2's Barracks (2) on (20, 3) to (22, 4) and Generator (3) on (28, 12) to (29, 13), a crystal patch
/// (4) on (16, 4), and player 2's Peacekeepers 5 at (6, 8), in the scout's sight, and 6 at (21, 7), by the Barracks.
/// Auto-targeting is off. The scout looks at the Barracks, the patch and Peacekeeper 6 from (18, 5) in ticks 2 and 3,
/// back home from tick 4, and the sim runs on until the Barracks' tiles are Explored, once the fog delay is over.
fn scouted() -> TestSim {
    use ObjectKind::{Barracks, CrystalPatch, Generator, Peacekeeper};
    let mut rows = vec![".".repeat(32); 16];
    rows[10].replace_range(1..2, "~");
    let point = |x, y| SpawnAt::Point(center(x, y));
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let mut sim = ScenarioBuilder::new(&rows.join("\n"))
        .spawn(Peacekeeper, Some(P1), point(3, 8))
        .spawn(Barracks, Some(P2), tile(20, 3))
        .spawn(Generator, Some(P2), tile(28, 12))
        .spawn(CrystalPatch, None, tile(16, 4))
        .spawn(Peacekeeper, Some(P2), point(6, 8))
        .spawn(Peacekeeper, Some(P2), point(21, 7))
        .start();
    let scout_to = |pos| DebugCommand::SetPosition {
        id: ObjectId(SCOUT),
        pos,
    };
    sim.without_auto_targeting()
        .debug_at(2, scout_to(center(18, 5)))
        .debug_at(4, scout_to(center(3, 8)))
        .run_until(4);
    let barracks = TilePos::new(21, 3);
    while Sight::new(sim.sim(), P1, true).tile(barracks) != VisibilityState::Explored {
        assert!(sim.sim().tick() < 64, "the Barracks' tiles never turned Explored");
        sim.step();
    }
    sim
}

/// Change 0415's Verification: "Enemy units appear only where you currently see; a remembered enemy building stays as
/// last seen in Explored areas; unexplored areas are black". With the fog on, as player 1 sees [`scouted`]: the tiles
/// nobody of theirs saw black, the Generator on them too; their scout in their colour, the ground it sees as it is
/// (the water too) with player 2's Peacekeeper on it in player 2's colour; the ground it saw dimmed, with the Barracks
/// as remembered, dimmed in player 2's colour, and the patch in the neutral one, but not the Peacekeeper that stands
/// there now. Then the Barracks is gone, and still shows as last seen.
#[test]
fn the_fog_shows_on_the_minimap_as_on_the_map() {
    use VisibilityState::{Explored, Unexplored, Visible};
    let mut scouted = scouted();
    let sim = scouted.sim();
    let sight = Sight::new(sim, P1, true);
    let frame = frame_of(sim);
    let image = draw(sim, &sight, &frame);
    let at = |ground| texel_of(&image, &frame, ground);
    let tile = |x, y| sight.tile(TilePos::new(x, y));
    let (p1, p2, neutral) = (player_color(Some(P1)), player_color(Some(P2)), player_color(None));

    assert_eq!(
        (tile(30, 1), at(center(30, 1))),
        (Unexplored, UNEXPLORED),
        "unexplored: black"
    );
    assert_eq!(
        at(object(sim, GENERATOR).placement.unwrap().center()),
        UNEXPLORED,
        "never seen"
    );

    assert_eq!(at(object(sim, SCOUT).pos), texel(p1), "the player's own unit");
    assert_eq!((tile(3, 11), at(center(3, 11))), (Visible, texel(terrain(sim, 3, 11))));
    assert_eq!((tile(1, 10), at(center(1, 10))), (Visible, texel(terrain(sim, 1, 10))));
    assert_ne!(
        texel(terrain(sim, 1, 10)),
        texel(terrain(sim, 3, 11)),
        "water isn't plain ground"
    );
    let near = object(sim, NEAR_ENEMY).pos;
    assert_eq!(
        (sight.at(near), at(near)),
        (Visible, texel(p2)),
        "a live enemy on a Visible tile"
    );

    assert_eq!(
        (tile(19, 7), at(center(19, 7))),
        (Explored, texel(dimmed(terrain(sim, 19, 7))))
    );
    assert_eq!(at(center(21, 3)), texel(dimmed(p2)), "the Barracks as remembered");
    let patch = object(sim, PATCH).placement.unwrap().center();
    assert_eq!(at(patch), texel(dimmed(neutral)), "the patch as remembered");
    let far = object(sim, FAR_ENEMY).pos;
    assert_eq!(sight.at(far), Explored);
    assert_eq!(
        at(far),
        texel(dimmed(terrain(sim, 21, 7))),
        "no live enemy on an Explored tile"
    );

    let next = sim.tick() + 1;
    scouted.debug_at(next, DebugCommand::Remove { id: ObjectId(BARRACKS) });
    scouted.run_until(next + 1);
    let sim = scouted.sim();
    assert!(sim.object(ObjectId(BARRACKS)).is_none());
    let image = draw(sim, &Sight::new(sim, P1, true), &frame);
    assert_eq!(
        texel_of(&image, &frame, center(21, 3)),
        texel(dimmed(p2)),
        "as last seen"
    );
}

/// "Space Crystal patches and other neutral objects in a neutral colour", and with the fog off (F3, `--no-fog`)
/// everything as it is: every tile as Visible ground, every object in its owner's colour, the Generator and the
/// Peacekeeper by the Barracks too, and no record.
#[test]
fn without_the_fog_the_minimap_shows_everything_as_it_is() {
    let scouted = scouted();
    let sim = scouted.sim();
    let sight = Sight::new(sim, P1, false);
    let frame = frame_of(sim);
    let image = draw(sim, &sight, &frame);
    let at = |ground| texel_of(&image, &frame, ground);
    let p2 = texel(player_color(Some(P2)));
    assert_eq!(at(center(30, 1)), texel(terrain(sim, 30, 1)));
    assert_eq!(at(object(sim, GENERATOR).placement.unwrap().center()), p2);
    assert_eq!(at(center(21, 3)), p2, "the Barracks");
    assert_eq!(at(object(sim, FAR_ENEMY).pos), p2);
    let patch = object(sim, PATCH).placement.unwrap().center();
    assert_eq!(at(patch), texel(player_color(None)), "the patch, neutral");
    assert_eq!(sight.remembered().count(), 0);
}

/// A flat 24 × 12 map: player 1's Peacekeeper at the centre of tile (8, 5), and player 2, the Syndicate, with a
/// Hideout on (10, 4) to (11, 5) in its sight (a debug spawn, as `tests/fog_memory/area_attacks.rs` has it).
fn hideout() -> TestSim {
    let map = vec![".".repeat(24); 12].join("\n");
    let mut sim = ScenarioBuilder::new(&map)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(center(8, 5)))
        .start();
    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::Hideout,
        owner: Some(P2),
        at: SpawnAt::Tile(TilePos::new(10, 4)),
        orientation: Default::default(),
    };
    sim.debug_at(1, spawn).run_until(3);
    assert!(sim.sim().objects().any(|o| o.kind == ObjectKind::Hideout));
    sim
}

/// "Hidden objects (e.g. underground, cloaked) don't appear for players who can't see them": player 2's Hideout,
/// hidden from player 1 (plan decision T32), never shows on player 1's minimap, though its tiles are Visible to them,
/// and they keep no record of it; player 2's minimap shows it in their colour.
#[test]
fn a_hidden_object_never_shows() {
    let hideout = hideout();
    let sim = hideout.sim();
    let frame = frame_of(sim);
    let p1 = Sight::new(sim, P1, true);
    assert_eq!(p1.tile(TilePos::new(10, 4)), VisibilityState::Visible);
    let image = draw(sim, &p1, &frame);
    let ground = texel(terrain(sim, 10, 4));
    assert_eq!(texel_of(&image, &frame, center(10, 4)), ground, "hidden from player 1");
    assert_eq!(texel_of(&image, &frame, center(11, 5)), ground);
    assert_eq!(p1.remembered().count(), 0, "no record of it");
    let image = draw(sim, &Sight::new(sim, P2, true), &frame);
    assert_eq!(
        texel_of(&image, &frame, center(10, 4)),
        texel(player_color(Some(P2))),
        "its owner sees it"
    );
}

/// The marks come in their drawing order: the records first, then the live map resources, structures and units, so
/// that a unit shows over a footprint.
#[test]
fn units_are_drawn_over_structures_and_records_under_both() {
    let scouted = scouted();
    let sim = scouted.sim();
    let all = marks(sim, &Sight::new(sim, P1, false));
    let rank = |mark: &raster::Mark| match mark.shape {
        raster::Shape::Footprint(_) if mark.color == texel(player_color(None)) => 0,
        raster::Shape::Footprint(_) => 1,
        raster::Shape::Dot(_) => 2,
    };
    let ranks: Vec<u8> = all.iter().map(rank).collect();
    assert!(ranks.is_sorted(), "{ranks:?}");
    assert_eq!(ranks, [0, 1, 1, 2, 2, 2], "the patch, two structures, three units");
    let (p1, p2, neutral) = (player_color(Some(P1)), player_color(Some(P2)), player_color(None));
    let fogged: Vec<[u8; 4]> = marks(sim, &Sight::new(sim, P1, true)).iter().map(|m| m.color).collect();
    assert_eq!(
        fogged,
        [texel(dimmed(neutral)), texel(dimmed(p2)), texel(p1), texel(p2)],
        "the patch's and the Barracks' records, then the scout and the enemy it sees"
    );
}
