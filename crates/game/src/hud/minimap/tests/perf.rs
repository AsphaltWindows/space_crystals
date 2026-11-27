//! Timings of the minimap's drawing (slice M5-MM; the Director's target: "under 0.5 ms average per frame on
//! `maps/playtest.json` with about 300 objects, measured in a release build"). Ignored by default:
//!
//! ```sh
//! cargo test -p space_crystals --release --lib minimap_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `maps/playtest.json` with both players' GDO bases, its patches and its station, and 140 Peacekeepers of each
//! player's on a grid over the open ground of their half of the map; each side is sent to the other's base (one group
//! order each), so that the sights move and the fog changes every tick. Auto-targeting is off. With the fog on, as
//! player 1 sees it, for each of 480 ticks (30 s) the sim steps outside the timing, then a redraw is timed: the tile
//! states, the marks and the painting over the baked terrain, what `draw_minimap` does when a redraw is due; and so is
//! the work of every frame, the frame, the layers' key and the camera's outline. On three surfaces: the default
//! window, 1280 × 720; 1920 × 1080; and 3840 × 2160 physical pixels at scale 2.
//!
//! It prints the mean, the 99th percentile and the worst redraw, and the average a frame at 60 frames a second with
//! at most [`REDRAWS_PER_SECOND`] redraws, one frame in four: the work of every frame and a quarter of the mean
//! redraw. Not timed: the image's upload to the GPU (4 bytes a texel, at most 15 times a second), the renderer's.
//! Timings are printed, never asserted.

use std::hint::black_box;
use std::time::{Duration, Instant};

use sc_sim::testing::TestSim;
use sc_sim::{GameSetup, MapDef, ObjectKind, OrderKind, Scenario, Target, TilePos, TilePresetId};

use super::super::raster::{Terrain, marks, paint};
use super::*;
use crate::camera::viewport_rect;
use crate::coords::ground_under;
use crate::fog::Sight;
use crate::hud::HUD_INSETS;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const UNITS_EACH: usize = 140;
const TICKS: u32 = 480;

/// How long `work` takes, by the wall clock: the only clock this test reads.
fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let out = work();
    (out, start.elapsed())
}

/// The game: `maps/playtest.json` as `--map` loads it, both players' bases, then [`UNITS_EACH`] Peacekeepers each
/// spawned at tick 1 on open ground (Plane tiles, two tiles apart, none within three tiles of a structure), player 1's
/// in the map's west half and player 2's in its east half; at tick 3 each side is sent to the other's start.
fn game() -> TestSim {
    let map: MapDef = serde_json::from_str(include_str!("../../../../../../maps/playtest.json")).unwrap();
    let starts = [map.slots[0].start, map.slots[1].start];
    let setup = GameSetup::default_for(&map);
    let mut game = TestSim::new(Scenario { map, setup });
    game.without_auto_targeting();
    let sim = game.sim();
    let map = sim.map();
    let structures: Vec<Rect> = sim
        .objects()
        .filter_map(|o| o.placement)
        .map(|p| {
            Rect::from_center_size(p.center(), UVec2::new(p.w, p.h).as_vec2() * SU_PER_GU).inflate(3.0 * SU_PER_GU)
        })
        .collect();
    let open = |t: TilePos| {
        map.tile(t).is_some_and(|tile| tile.preset == TilePresetId::PLANE)
            && !structures.iter().any(|r| r.contains(t.center()))
    };
    let half = map.width() as i32 / 2;
    let sites = |west: bool| {
        let columns = (1..map.width() as i32 - 1)
            .step_by(2)
            .filter(move |&x| (x < half) == west);
        let tiles = columns.flat_map(|x| (1..map.height() as i32 - 1).step_by(2).map(move |y| TilePos::new(x, y)));
        tiles.filter(|&t| open(t)).take(UNITS_EACH).collect::<Vec<_>>()
    };
    let sites = [(P1, sites(true)), (P2, sites(false))];
    for (player, tiles) in &sites {
        assert_eq!(tiles.len(), UNITS_EACH, "room for {player:?}'s units");
        for &t in tiles {
            game.spawn_unit_at(1, ObjectKind::Peacekeeper, *player, t.center());
        }
    }
    game.run_until(2);
    for (player, goal) in [(P1, starts[1]), (P2, starts[0])] {
        let units: Vec<_> = game
            .sim()
            .objects()
            .filter(|o| o.owner == Some(player) && o.kind == ObjectKind::Peacekeeper)
            .map(|o| o.id)
            .collect();
        assert_eq!(units.len(), UNITS_EACH, "{player:?}'s units spawned");
        game.order_at(3, player, &units, OrderKind::Move(Target::Point(goal.center())), false);
    }
    game
}

/// The mean, the 99th percentile and the worst of `times`, in milliseconds.
fn stats(mut times: Vec<Duration>) -> (f64, f64, f64) {
    times.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let mean = ms(times.iter().sum::<Duration>()) / times.len() as f64;
    let p99 = ms(times[(times.len() * 99).div_ceil(100) - 1]);
    (mean, p99, ms(*times.last().unwrap()))
}

#[test]
#[ignore = "perf: run in release, with --nocapture"]
fn minimap_perf() {
    let surfaces = [
        (UVec2::new(1280, 720), 1.0),
        (UVec2::new(1920, 1080), 1.0),
        (UVec2::new(3840, 2160), 2.0),
    ];
    for (physical, scale_factor) in surfaces {
        let size = SurfaceSize { physical, scale_factor };
        let mut game = game();
        let extent = crate::camera::map_extent_su(game.sim().map());
        let frame = MinimapFrame::on(size, extent);
        let terrain = Terrain::bake(game.sim().map(), &frame);
        let mut data = vec![0; (frame.texels.x * frame.texels.y * 4) as usize];
        let (mut redraws, mut frames) = (Vec::new(), Vec::new());
        let mut shown = 0;
        for _ in 0..TICKS {
            game.step();
            let sim = game.sim();
            let sight = Sight::new(sim, P1, true);
            let (_, every_frame) = timed(|| {
                let frame = MinimapFrame::on(size, crate::camera::map_extent_su(sim.map()));
                let key = LayersKey {
                    generation: 0,
                    tick: sim.tick(),
                    player: sight.player(),
                    revision: sight.revision(),
                    records: sight.records_revision(),
                    map_build: 0,
                    texels: frame.texels,
                };
                let due = redraw_due(Some(&(key, 0.0)), black_box(&key), 1.0, false);
                let (_, viewport) = viewport_rect(size, HUD_INSETS);
                let visible = crate::camera::visible_extent_su(viewport, size.physical.x, 1.0);
                let outline = frame.camera_rect(ground_under(sim.map(), extent / 2.0), visible);
                black_box((due, outline));
            });
            let (count, redraw) = timed(|| {
                let states = sight.tile_states(sim.map().width(), sim.map().height());
                let marks = marks(sim, &sight);
                paint(&mut data, &frame, &terrain, &states, &marks);
                marks.len()
            });
            black_box(&data);
            shown = count;
            frames.push(every_frame);
            redraws.push(redraw);
        }
        let objects = game.sim().objects().count();
        let (mean, p99, worst) = stats(redraws);
        let (frame_mean, ..) = stats(frames);
        let per_frame = frame_mean + mean * REDRAWS_PER_SECOND / 60.0;
        println!(
            "minimap_perf {}×{} at scale {scale_factor} ({}×{} texels), {objects} objects ({shown} marks at the end): \
             redraw mean {mean:.4} ms, p99 {p99:.4} ms, worst {worst:.4} ms; every frame {frame_mean:.5} ms; a frame \
             at 60 fps {per_frame:.4} ms on average",
            physical.x, physical.y, frame.texels.x, frame.texels.y,
        );
    }
}
