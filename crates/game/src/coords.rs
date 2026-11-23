//! Sim ↔ Bevy coordinates, the terrain's height on screen, picking, and draw order (`ARCHITECTURE.md > Client
//! architecture > Visuals`, `> Height and picking`).
//!
//! 1 Bevy world unit = 1 space unit. Three spaces:
//! - **ground**: the sim's positions (space units, +y south);
//! - **view**: where a ground point is drawn. [`drawn_position`] shifts it up the screen (north) by its terrain
//!   height, [`HEIGHT_OFFSET_PER_LEVEL`] per level (`core/art_direction.md > Elevation Display`); +y still points
//!   south. On flat ground at height 0, view and ground coincide;
//! - **Bevy world**: view space with +y up, `bevy_y = -view_y` ([`view_to_world`]).
//!
//! Every screen → ground conversion goes through [`ground_under`], the inverse of [`drawn_position`]. Convert only
//! through these helpers, never ad hoc.

use bevy::math::{Quat, Rect, Vec2, Vec3};
use sc_sim::{CornerPos, Map, SU_PER_GU, TilePos};

/// `core/art_direction.md > Elevation Display`: how far up the screen terrain one level higher is drawn, in grid
/// units per level. Placeholder until art tuning (the design's "TBD"): a quarter tile, so a rise of 4 levels lifts
/// the ground by a whole tile, and the 3-level plateau of the test map stands out clearly without hiding much
/// behind it.
pub const HEIGHT_OFFSET_PER_LEVEL: f32 = 0.25;

/// How far up the screen ground at `height` (levels, possibly fractional) is drawn, in space units.
pub fn lift(height: f32) -> f32 {
    height * HEIGHT_OFFSET_PER_LEVEL * SU_PER_GU
}

/// Where the ground point `ground` is drawn (view space): lifted by the terrain height there ([`Map::height_at`]).
pub fn drawn_position(map: &Map, ground: Vec2) -> Vec2 {
    drawn_at_height(ground, map.height_at(ground))
}

/// Where something at `ground` is drawn when it stands at `height` levels.
pub fn drawn_at_height(ground: Vec2, height: f32) -> Vec2 {
    Vec2::new(ground.x, ground.y - lift(height))
}

/// How far t may stray outside a tile row before a solution counts as outside it (float rounding at row edges).
const ROW_TOLERANCE: f32 = 1e-4;

/// Picking: the ground point drawn at the view point `view`, the inverse of [`drawn_position`], so that
/// `drawn_position(map, ground_under(map, v)) == v` (up to float rounding).
///
/// Where terrain overlaps itself on screen (a cliff or a hill drawn over the ground behind it), the frontmost
/// surface wins: the southernmost of the ground points drawn there, because southern terrain draws over northern
/// terrain. Beyond the map's north and south edges the ground continues flat at the edge's height, and beyond its
/// east and west edges at the height of the nearest point on the map (as [`Map::height_at`] does), so every view
/// point has a ground point. A non-finite point comes back unchanged.
///
/// It solves column by column: at a fixed x the bilinear height is linear across a tile row, so the drawn y is
/// linear in the ground y there, and the rows are searched from south to north.
pub fn ground_under(map: &Map, view: Vec2) -> Vec2 {
    if !view.is_finite() {
        return view;
    }
    let (width, height) = (map.width() as i32, map.height() as i32);
    let k = lift(1.0);
    let x = view.x;
    // The column of tiles at x and the position across it, clamped onto the map like `Map::height_at`.
    let gx = (x / SU_PER_GU).clamp(0.0, width as f32);
    let column = (gx.floor() as i32).min(width - 1);
    let u = gx - column as f32;

    for row in (0..height).rev() {
        let [nw, ne, sw, se] = map
            .tile_corner_heights(TilePos::new(column, row))
            .unwrap_or([0; 4])
            .map(f32::from);
        // Height across the row at this x: a + b·t, t from 0 (north edge) to 1 (south edge).
        let a = nw + (ne - nw) * u;
        let b = (sw - nw) + (nw - ne - sw + se) * u;
        // Drawn y: SU_PER_GU·(row + t) − k·(a + b·t) = start + rate·t.
        let start = row as f32 * SU_PER_GU - k * a;
        let rate = SU_PER_GU - k * b;
        let t = if rate.abs() > f32::EPSILON {
            (view.y - start) / rate
        } else if (view.y - start).abs() <= ROW_TOLERANCE * SU_PER_GU {
            // The row is seen edge-on: its whole depth is drawn on one line. Its southern edge is in front.
            1.0
        } else {
            continue;
        };
        // Slightly outside the row is still on the same continuous surface.
        if (-ROW_TOLERANCE..=1.0 + ROW_TOLERANCE).contains(&t) {
            return Vec2::new(x, (row as f32 + t) * SU_PER_GU);
        }
    }
    // Below or above everything drawn in this column: the flat continuation south or north of the map.
    let south_edge = height as f32 * SU_PER_GU;
    let beyond_south = view.y + k * map.height_at(Vec2::new(x, south_edge));
    if beyond_south >= south_edge {
        return Vec2::new(x, beyond_south);
    }
    Vec2::new(x, view.y + k * map.height_at(Vec2::new(x, 0.0)))
}

/// The area the map is drawn on, in view space: its full width, from its highest-drawn corner to its lowest-drawn
/// one (the drawn y along a tile edge is linear between its corners, so the corners are the extremes). The camera
/// keeps its view inside it, so raised ground at the north edge stays reachable.
pub fn drawn_bounds(map: &Map) -> Rect {
    let (width, height) = (map.width() as i32, map.height() as i32);
    let (mut top, mut bottom) = (f32::MAX, f32::MIN);
    for cy in 0..=height {
        for cx in 0..=width {
            let c = CornerPos::new(cx, cy);
            let y = drawn_at_height(c.pos(), f32::from(map.corner(c).unwrap_or(0))).y;
            top = top.min(y);
            bottom = bottom.max(y);
        }
    }
    Rect::new(0.0, top, width as f32 * SU_PER_GU, bottom)
}

/// The height a tile-based object stands at: the mean height of its footprint's corners on the map, which is the
/// footprint's level when it is flat (`core/map.md > Elevation > Map resources only on flat terrain`). A footprint
/// entirely off the map stands at the height of the nearest ground.
pub fn footprint_height(map: &Map, origin: TilePos, w: u32, h: u32) -> f32 {
    let (mut sum, mut count) = (0.0, 0u32);
    for dy in 0..=i64::from(h) {
        for dx in 0..=i64::from(w) {
            let corner = (i64::from(origin.x) + dx, i64::from(origin.y) + dy);
            let (Ok(cx), Ok(cy)) = (i32::try_from(corner.0), i32::try_from(corner.1)) else {
                continue;
            };
            if let Some(height) = map.corner(CornerPos::new(cx, cy)) {
                sum += f32::from(height);
                count += 1;
            }
        }
    }
    if count == 0 {
        let center = origin.corner() + Vec2::new(w as f32, h as f32) * SU_PER_GU / 2.0;
        return map.height_at(center);
    }
    sum / count as f32
}

/// View position (+y south) → Bevy world position (+y up).
pub fn view_to_world(p: Vec2) -> Vec2 {
    Vec2::new(p.x, -p.y)
}

/// Bevy world position (+y up) → view position (+y south).
pub fn world_to_view(p: Vec2) -> Vec2 {
    Vec2::new(p.x, -p.y)
}

/// A sim facing θ (0 = east, positive = clockwise on screen) is a Bevy rotation of −θ about z.
pub fn facing_to_rotation(facing: f32) -> Quat {
    Quat::from_rotation_z(-facing)
}

/// Interpolates a facing along the shortest arc.
pub fn lerp_facing(from: f32, to: f32, alpha: f32) -> f32 {
    from + sc_sim::math::wrap_angle(to - from) * alpha
}

/// Draw layers in StarCraft 1 order, back to front. Each has its own z band; within a band, z grows with the ground
/// y, so that southern objects draw in front, and southern terrain over the northern terrain it overlaps on screen.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrawLayer {
    Terrain,
    GroundDecals,
    GroundObjects,
    Air,
    Overlays,
}

/// Distance between the bases of adjacent z bands.
const Z_BAND: f32 = 100.0;
/// z added per space unit of ground y inside a band. A 1024-tile-tall map spans 65.5 z, which stays inside a band.
const Z_PER_SU: f32 = 0.001;

/// The z offset inside a band of something whose ground y is `ground_y`.
pub fn depth_offset(ground_y: f32) -> f32 {
    ground_y * Z_PER_SU
}

impl DrawLayer {
    /// The z of this layer's northern edge (ground y = 0). The 2D camera sees z in (-1000, 1000).
    pub fn base_z(self) -> f32 {
        let index = match self {
            DrawLayer::Terrain => 0.0,
            DrawLayer::GroundDecals => 1.0,
            DrawLayer::GroundObjects => 2.0,
            DrawLayer::Air => 3.0,
            DrawLayer::Overlays => 4.0,
        };
        -500.0 + index * Z_BAND
    }

    /// The y-sorted z of something anchored at ground y `ground_y`.
    pub fn z_at(self, ground_y: f32) -> f32 {
        self.base_z() + depth_offset(ground_y)
    }
}

/// Translation of a visual in `layer` anchored at `ground` and standing at `height` levels: drawn lifted by its
/// height, y-sorted by its ground position.
pub fn ground_translation(layer: DrawLayer, ground: Vec2, height: f32) -> Vec3 {
    view_to_world(drawn_at_height(ground, height)).extend(layer.z_at(ground.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 0.01
    }

    /// 4 × 4 tiles: low ground in row 0, a cliff row (row 1) rising 0 → 8 towards the south, which folds on
    /// screen, and a plateau at level 8 in rows 2 and 3.
    fn cliff_map() -> Map {
        Map::from_ascii(
            "
            ....
            cccc
            ....
            ....

            00000
            00000
            88888
            88888
            88888
            ",
        )
        .unwrap()
    }

    #[test]
    fn y_is_flipped_both_ways() {
        let p = Vec2::new(100.0, 250.0);
        assert_eq!(view_to_world(p), Vec2::new(100.0, -250.0));
        assert_eq!(world_to_view(view_to_world(p)), p);
    }

    #[test]
    fn higher_ground_is_drawn_further_up() {
        let map = Map::from_ascii("..\n..\n\n000\n012\n024").unwrap();
        let k = HEIGHT_OFFSET_PER_LEVEL * SU_PER_GU;
        assert_eq!(drawn_position(&map, Vec2::new(0.0, 20.0)), Vec2::new(0.0, 20.0));
        // Corner (2, 2) is at height 4; the center of tile (1, 1) at (1 + 2 + 2 + 4) / 4.
        assert_eq!(
            drawn_position(&map, Vec2::new(128.0, 128.0)),
            Vec2::new(128.0, 128.0 - 4.0 * k)
        );
        let center = TilePos::new(1, 1).center();
        assert_eq!(drawn_position(&map, center), center - Vec2::new(0.0, 2.25 * k));
        assert_eq!(lift(4.0), 4.0 * k);
    }

    #[test]
    fn picking_inverts_the_drawing_everywhere() {
        let maps = [
            Map::new(5, 4),
            Map::from_ascii("...\n...\n\n0123\n1234\n2345").unwrap(),
            cliff_map(),
            sc_sim::scenarios::dev_skirmish().map.terrain,
        ];
        for map in &maps {
            let bounds = drawn_bounds(map);
            for iy in -10..=60 {
                for ix in -3..=40 {
                    let view = Vec2::new(ix as f32 * 37.3, bounds.min.y - 100.0 + iy as f32 * 61.7);
                    let ground = ground_under(map, view);
                    let back = drawn_position(map, ground);
                    assert!(close(back, view), "{view} → {ground} → {back}");
                }
            }
        }
        assert!(ground_under(&Map::new(2, 2), Vec2::NAN).is_nan());
    }

    #[test]
    fn flat_ground_is_picked_where_it_lies() {
        let map = Map::new(4, 3);
        assert!(close(ground_under(&map, Vec2::new(70.0, 90.0)), Vec2::new(70.0, 90.0)));
        // Off the map too.
        assert!(close(
            ground_under(&map, Vec2::new(-5.0, -30.0)),
            Vec2::new(-5.0, -30.0)
        ));
        assert!(close(
            ground_under(&map, Vec2::new(50.0, 500.0)),
            Vec2::new(50.0, 500.0)
        ));
        // On raised flat ground, the ground drawn at a view point lies its lift further south.
        let raised = Map::from_ascii("..\n\n444\n444").unwrap();
        let k = HEIGHT_OFFSET_PER_LEVEL * SU_PER_GU;
        assert!(close(
            ground_under(&raised, Vec2::new(70.0, 0.0)),
            Vec2::new(70.0, 4.0 * k)
        ));
    }

    /// Where cliffs overlap on screen, the frontmost surface is picked: the plateau in front of the cliff that
    /// rises towards it and of the low ground behind both.
    #[test]
    fn picking_takes_the_frontmost_surface_where_cliffs_overlap() {
        let map = cliff_map();
        let k = HEIGHT_OFFSET_PER_LEVEL * SU_PER_GU; // 16: the plateau's 8 levels lift it 128 su
        assert_eq!(k * 8.0, 128.0);
        // View y 40 at x 32 shows three surfaces: the low row 0 (ground y 40), the folded cliff row 1, and the
        // plateau's row 2 (ground y 168). The plateau is in front.
        let picked = ground_under(&map, Vec2::new(32.0, 40.0));
        assert!(close(picked, Vec2::new(32.0, 168.0)), "{picked}");
        assert_eq!(TilePos::containing(picked), TilePos::new(0, 2));
        // Above the plateau's drawn edge only the low ground shows.
        let picked = ground_under(&map, Vec2::new(32.0, -20.0));
        assert!(close(picked, Vec2::new(32.0, -20.0)), "{picked}");
        // A cliff that falls towards the viewer is seen face-on: a point on its face picks the cliff tile.
        let falling = Map::from_ascii("..\n..\n..\n\n888\n000\n000\n000").unwrap();
        let picked = ground_under(&falling, Vec2::new(10.0, 0.0));
        assert_eq!(TilePos::containing(picked), TilePos::new(0, 0));
        assert!(close(drawn_position(&falling, picked), Vec2::new(10.0, 0.0)));
    }

    #[test]
    fn the_drawn_bounds_reach_from_the_highest_to_the_lowest_corner() {
        assert_eq!(drawn_bounds(&Map::new(4, 3)), Rect::new(0.0, 0.0, 256.0, 192.0));
        let map = cliff_map();
        // The north edge is at height 0, the south edge (ground y 256) at 8: drawn at 256 − 128.
        assert_eq!(drawn_bounds(&map), Rect::new(0.0, 0.0, 256.0, 128.0));
        let raised_north = Map::from_ascii("..\n..\n\n444\n000\n000").unwrap();
        assert_eq!(drawn_bounds(&raised_north), Rect::new(0.0, -64.0, 128.0, 128.0));
    }

    #[test]
    fn footprints_stand_at_their_corners_mean_height() {
        let map = Map::from_ascii("...\n...\n\n2220\n2221\n2222").unwrap();
        assert_eq!(footprint_height(&map, TilePos::new(0, 0), 2, 2), 2.0);
        assert_eq!(footprint_height(&map, TilePos::new(2, 0), 1, 1), 1.25);
        // Off the map: the nearest ground's height; never a panic.
        assert_eq!(footprint_height(&map, TilePos::new(i32::MAX, i32::MAX), 16, 16), 2.0);
        assert_eq!(footprint_height(&map, TilePos::new(i32::MIN, 0), 2, 2), 2.0);
    }

    #[test]
    fn southern_things_draw_in_front_and_layers_stack() {
        let north = DrawLayer::GroundObjects.z_at(64.0);
        let south = DrawLayer::GroundObjects.z_at(640.0);
        assert!(south > north);
        // The deepest point of a huge map stays below the next layer.
        assert!(DrawLayer::GroundObjects.z_at(1024.0 * 64.0) < DrawLayer::Air.base_z());
        assert!(DrawLayer::Terrain.base_z() > -1000.0 && DrawLayer::Overlays.z_at(65_536.0) < 1000.0);
        // Lifted by its height, sorted by its ground.
        let t = ground_translation(DrawLayer::GroundObjects, Vec2::new(10.0, 100.0), 2.0);
        assert_eq!(
            t,
            Vec3::new(10.0, -(100.0 - lift(2.0)), DrawLayer::GroundObjects.z_at(100.0))
        );
    }

    #[test]
    fn facing_turns_clockwise_on_screen() {
        // Sim facing +90° points south (+y sim) = down on screen = -y in Bevy.
        let v = facing_to_rotation(std::f32::consts::FRAC_PI_2) * Vec3::X;
        assert!((v - Vec3::NEG_Y).length() < 1e-6);
    }

    #[test]
    fn facing_interpolates_along_the_shortest_arc() {
        let pi = std::f32::consts::PI;
        let mid = lerp_facing(pi - 0.1, -pi + 0.1, 0.5);
        assert!((sc_sim::math::wrap_angle(mid) - pi).abs() < 1e-5);
    }
}
