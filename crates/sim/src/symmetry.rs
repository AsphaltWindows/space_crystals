//! Map symmetry (`core/map_editor.md > Symmetry`): the six symmetry modes, and pure transforms that map tiles,
//! corners, points and building placements onto their images.
//!
//! A symmetric map consists of 1, 2 or 4 images of one part of it. Each image is an [`Orientation`] of the
//! square (a rotation by a multiple of 90°, possibly with a mirror) applied about the map's center: a
//! [`MapTransform`]. Coordinates are mapped exactly, with integer arithmetic on doubled coordinates (the map's
//! center may lie on a tile edge or in a tile's middle), in 64 bits so that no input can overflow.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::object::{Placement, Rotation};
use crate::units::{CornerPos, SU_PER_GU, TilePos};

/// `core/map_editor.md > Symmetry`: how the parts of a map mirror each other.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Symmetry {
    /// No symmetry (Custom maps only).
    None,
    /// Rotational, 180° (two-fold).
    Rotate180,
    /// Rotational, 90° (four-fold). Only possible on a square map.
    Rotate90,
    /// Mirror on the horizontal axis: the top and bottom halves mirror each other.
    MirrorHorizontalAxis,
    /// Mirror on the vertical axis: the left and right halves mirror each other.
    MirrorVerticalAxis,
    /// Mirror on both axes (four-fold).
    MirrorBothAxes,
}

impl Symmetry {
    pub const ALL: [Symmetry; 6] = [
        Symmetry::None,
        Symmetry::Rotate180,
        Symmetry::Rotate90,
        Symmetry::MirrorHorizontalAxis,
        Symmetry::MirrorVerticalAxis,
        Symmetry::MirrorBothAxes,
    ];

    /// Whether a `width × height` map can have this symmetry: 90° rotation needs a square map.
    pub fn fits(self, width: u32, height: u32) -> bool {
        self != Symmetry::Rotate90 || width == height
    }

    /// The orientation of each image, the identity (the part itself) first: 1, 2 or 4 of them. For the four-fold
    /// symmetries, the images go round the map: clockwise for 90° rotation; left-right, top-bottom, then both for
    /// the mirrors.
    pub fn images(self) -> &'static [Orientation] {
        use Orientation as O;
        match self {
            Symmetry::None => &[O::IDENTITY],
            Symmetry::Rotate180 => &[O::IDENTITY, O::ROTATE_180],
            Symmetry::Rotate90 => &[O::IDENTITY, O::ROTATE_90, O::ROTATE_180, O::ROTATE_270],
            Symmetry::MirrorHorizontalAxis => &[O::IDENTITY, O::MIRROR_Y],
            Symmetry::MirrorVerticalAxis => &[O::IDENTITY, O::MIRROR_X],
            Symmetry::MirrorBothAxes => &[O::IDENTITY, O::MIRROR_X, O::MIRROR_Y, O::ROTATE_180],
        }
    }

    /// The transform of each image on a `width × height` map, the identity first, or `None` if the symmetry
    /// doesn't fit the map ([`Symmetry::fits`]).
    pub fn transforms(self, width: u32, height: u32) -> Option<Vec<MapTransform>> {
        self.fits(width, height).then(|| {
            self.images()
                .iter()
                .map(|&orientation| MapTransform::new(orientation, width, height))
                .collect()
        })
    }
}

/// One of the eight orientations of a square: a rotation by a multiple of 90°, with or without a mirror. It is a
/// linear map of offsets in the sim's axes (+x east, +y south): `(x, y) → (xx·x + xy·y, yx·x + yy·y)`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Orientation {
    xx: i8,
    xy: i8,
    yx: i8,
    yy: i8,
}

impl Orientation {
    pub const IDENTITY: Self = Self::new(1, 0, 0, 1);
    /// 90° clockwise as seen on screen: east turns south, south turns west.
    pub const ROTATE_90: Self = Self::new(0, -1, 1, 0);
    pub const ROTATE_180: Self = Self::new(-1, 0, 0, -1);
    /// 90° counterclockwise as seen on screen.
    pub const ROTATE_270: Self = Self::new(0, 1, -1, 0);
    /// Mirrors left and right (across a vertical axis): east turns west.
    pub const MIRROR_X: Self = Self::new(-1, 0, 0, 1);
    /// Mirrors top and bottom (across a horizontal axis): north turns south.
    pub const MIRROR_Y: Self = Self::new(1, 0, 0, -1);
    /// All eight: the four rotations, then the four rotations after [`Orientation::MIRROR_X`].
    pub const ALL: [Self; 8] = [
        Self::IDENTITY,
        Self::ROTATE_90,
        Self::ROTATE_180,
        Self::ROTATE_270,
        Self::MIRROR_X,
        Self::new(0, 1, 1, 0),
        Self::MIRROR_Y,
        Self::new(0, -1, -1, 0),
    ];
    const ROTATIONS: [(Rotation, Self); 4] = [
        (Rotation::R0, Self::IDENTITY),
        (Rotation::R90, Self::ROTATE_90),
        (Rotation::R180, Self::ROTATE_180),
        (Rotation::R270, Self::ROTATE_270),
    ];

    const fn new(xx: i8, xy: i8, yx: i8, yy: i8) -> Self {
        Self { xx, xy, yx, yy }
    }

    /// `self` applied after `first`.
    pub fn after(self, first: Orientation) -> Orientation {
        let (a, b) = (self, first);
        Self::new(
            a.xx * b.xx + a.xy * b.yx,
            a.xx * b.xy + a.xy * b.yy,
            a.yx * b.xx + a.yy * b.yx,
            a.yx * b.xy + a.yy * b.yy,
        )
    }

    /// The offset `(x, y)` turned by this orientation.
    pub fn apply(self, (x, y): (i64, i64)) -> (i64, i64) {
        let m = |a: i8, b: i8| i64::from(a) * x + i64::from(b) * y;
        (m(self.xx, self.xy), m(self.yx, self.yy))
    }

    /// The offset `v` turned by this orientation.
    pub fn apply_vec(self, v: Vec2) -> Vec2 {
        let m = |a: i8, b: i8| f32::from(a) * v.x + f32::from(b) * v.y;
        Vec2::new(m(self.xx, self.xy), m(self.yx, self.yy))
    }

    /// Whether it turns by 90° or 270°, so that a `w × h` footprint becomes `h × w`.
    pub fn swaps_axes(self) -> bool {
        self.xx == 0
    }

    /// Whether it mirrors (turns the map over), rather than only rotating.
    pub fn mirrors(self) -> bool {
        self.xx * self.yy - self.xy * self.yx < 0
    }

    /// The orientation of a building placed with `rotation` and flips (`core/entities.md > Structure Instance`):
    /// the building is flipped first, in its own frame, then rotated clockwise. `flip_h` mirrors it left and
    /// right, `flip_v` top and bottom.
    pub fn of_building(rotation: Rotation, flip_h: bool, flip_v: bool) -> Orientation {
        let turn = Self::ROTATIONS
            .iter()
            .find(|(r, _)| *r == rotation)
            .map_or(Self::IDENTITY, |(_, o)| *o);
        let mut flips = Self::IDENTITY;
        if flip_h {
            flips = Self::MIRROR_X.after(flips);
        }
        if flip_v {
            flips = Self::MIRROR_Y.after(flips);
        }
        turn.after(flips)
    }

    /// A building rotation and flips with this orientation ([`Orientation::of_building`]): without a vertical
    /// flip, which equals a horizontal flip and a half turn.
    pub fn building(self) -> (Rotation, bool, bool) {
        let flip_h = self.mirrors();
        // A mirroring orientation is a rotation after MIRROR_X, which undoes itself.
        let turn = if flip_h { self.after(Self::MIRROR_X) } else { self };
        let rotation = Self::ROTATIONS
            .iter()
            .find(|(_, o)| *o == turn)
            .map_or(Rotation::R0, |(r, _)| *r);
        (rotation, flip_h, false)
    }
}

/// One image of a symmetric `width × height` map: an [`Orientation`] applied about the map's center. Every
/// mapping is exact for the transforms of [`Symmetry::transforms`]; a 90° turn of a map that isn't square doesn't
/// map it onto itself, and its results round down.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MapTransform {
    pub orientation: Orientation,
    pub width: u32,
    pub height: u32,
}

impl MapTransform {
    pub fn new(orientation: Orientation, width: u32, height: u32) -> Self {
        Self {
            orientation,
            width,
            height,
        }
    }

    /// Maps a point given in doubled tile units (so that the map's center, `(width, height)` doubled, and the
    /// centers of tiles are whole numbers).
    fn doubled(self, x2: i64, y2: i64) -> (i64, i64) {
        let (cx, cy) = (i64::from(self.width), i64::from(self.height));
        let (x, y) = self.orientation.apply((x2 - cx, y2 - cy));
        (x + cx, y + cy)
    }

    /// The image of tile `t`. Tiles off the map map off the map.
    pub fn tile(self, t: TilePos) -> TilePos {
        let (x, y) = self.doubled(2 * i64::from(t.x) + 1, 2 * i64::from(t.y) + 1);
        TilePos::new(narrow((x - 1).div_euclid(2)), narrow((y - 1).div_euclid(2)))
    }

    /// The image of corner `c`: the corner grid is one larger than the tile grid, `(width + 1) × (height + 1)`.
    pub fn corner(self, c: CornerPos) -> CornerPos {
        let (x, y) = self.doubled(2 * i64::from(c.x), 2 * i64::from(c.y));
        CornerPos::new(narrow(x.div_euclid(2)), narrow(y.div_euclid(2)))
    }

    /// The image of point `p` (space units).
    pub fn point(self, p: Vec2) -> Vec2 {
        let center = Vec2::new(self.width as f32, self.height as f32) * (SU_PER_GU / 2.0);
        self.orientation.apply_vec(p - center) + center
    }

    /// The image of a `w × h` footprint with its top-left tile at `origin`: the image's top-left tile and size
    /// (`h × w` after a quarter turn).
    pub fn footprint(self, origin: TilePos, w: u32, h: u32) -> (TilePos, u32, u32) {
        let (w2, h2) = if self.orientation.swaps_axes() { (h, w) } else { (w, h) };
        let (x, y) = self.doubled(
            2 * i64::from(origin.x) + i64::from(w),
            2 * i64::from(origin.y) + i64::from(h),
        );
        let corner = |center: i64, size: u32| narrow((center - i64::from(size)).div_euclid(2));
        (TilePos::new(corner(x, w2), corner(y, h2)), w2, h2)
    }

    /// The image of a building's placement (`core/map_editor.md > Symmetry > Editing a symmetric map`: building
    /// orientations are rotated or flipped to match). A rotation keeps the building's flips and adds to its
    /// rotation; a mirror gives the equivalent rotation and horizontal flip.
    pub fn placement(self, p: &Placement) -> Placement {
        let (origin, w, h) = self.footprint(p.origin, p.w, p.h);
        let (rotation, flip_h, flip_v) = if self.orientation.mirrors() {
            self.orientation
                .after(Orientation::of_building(p.rotation, p.flip_h, p.flip_v))
                .building()
        } else {
            let (rotation, _, _) = self
                .orientation
                .after(Orientation::of_building(p.rotation, false, false))
                .building();
            (rotation, p.flip_h, p.flip_v)
        };
        Placement {
            origin,
            w,
            h,
            rotation,
            flip_h,
            flip_v,
        }
    }
}

/// `v` as an `i32`, saturated: coordinates far off the map stay far off the map.
fn narrow(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn t(x: i32, y: i32) -> TilePos {
        TilePos::new(x, y)
    }

    fn c(x: i32, y: i32) -> CornerPos {
        CornerPos::new(x, y)
    }

    #[test]
    fn the_orientations_form_the_eight_symmetries_of_the_square() {
        let all: BTreeSet<_> = Orientation::ALL.iter().map(|o| (o.xx, o.xy, o.yx, o.yy)).collect();
        assert_eq!(all.len(), 8);
        for a in Orientation::ALL {
            // Closed under composition, every one has an inverse, and composing is applying one after the other.
            assert!(Orientation::ALL.iter().any(|b| b.after(a) == Orientation::IDENTITY));
            for b in Orientation::ALL {
                let ab = a.after(b);
                assert!(Orientation::ALL.contains(&ab));
                for v in [(3, -7), (0, 1), (-5, 0)] {
                    assert_eq!(ab.apply(v), a.apply(b.apply(v)));
                }
                assert_eq!(ab.mirrors(), a.mirrors() != b.mirrors());
            }
        }
        // Clockwise on screen: east (+x) turns south (+y).
        assert_eq!(Orientation::ROTATE_90.apply((1, 0)), (0, 1));
        assert_eq!(Orientation::ROTATE_90.apply((0, 1)), (-1, 0));
        assert_eq!(
            Orientation::ROTATE_270.after(Orientation::ROTATE_90),
            Orientation::IDENTITY
        );
        assert_eq!(
            Orientation::MIRROR_X.after(Orientation::MIRROR_Y),
            Orientation::ROTATE_180
        );
        assert!(Orientation::ROTATE_90.swaps_axes() && !Orientation::MIRROR_Y.swaps_axes());
    }

    #[test]
    fn building_orientations_round_trip() {
        for o in Orientation::ALL {
            let (rotation, flip_h, flip_v) = o.building();
            assert!(!flip_v);
            assert_eq!(Orientation::of_building(rotation, flip_h, flip_v), o);
        }
        // Flipped first, then rotated.
        let o = Orientation::of_building(Rotation::R90, true, false);
        assert_eq!(o.apply((1, 0)), Orientation::ROTATE_90.apply((-1, 0)));
        // Both flips are a half turn.
        assert_eq!(
            Orientation::of_building(Rotation::R0, true, true),
            Orientation::ROTATE_180
        );
        assert_eq!(
            Orientation::of_building(Rotation::R90, false, true).building(),
            (Rotation::R270, true, false)
        );
    }

    #[test]
    fn the_modes_have_their_images_and_90_degrees_needs_a_square() {
        let counts: Vec<usize> = Symmetry::ALL.iter().map(|s| s.images().len()).collect();
        assert_eq!(counts, [1, 2, 4, 2, 2, 4]);
        for s in Symmetry::ALL {
            assert_eq!(s.images()[0], Orientation::IDENTITY, "{s:?}");
            assert!(s.fits(12, 12));
            assert_eq!(s.fits(12, 8), s != Symmetry::Rotate90);
        }
        assert!(Symmetry::Rotate90.transforms(10, 8).is_none());
        assert_eq!(Symmetry::Rotate90.transforms(8, 8).unwrap().len(), 4);
    }

    #[test]
    fn tiles_corners_and_points_map_across_the_map() {
        let (w, h) = (10, 8);
        let map = |o| MapTransform::new(o, w, h);
        use Orientation as O;
        assert_eq!(map(O::ROTATE_180).tile(t(1, 2)), t(8, 5));
        assert_eq!(map(O::MIRROR_X).tile(t(1, 2)), t(8, 2));
        assert_eq!(map(O::MIRROR_Y).tile(t(1, 2)), t(1, 5));
        // The corner grid is (w + 1) × (h + 1).
        assert_eq!(map(O::ROTATE_180).corner(c(0, 0)), c(10, 8));
        assert_eq!(map(O::MIRROR_X).corner(c(3, 1)), c(7, 1));
        assert_eq!(map(O::MIRROR_Y).corner(c(3, 1)), c(3, 7));
        assert_eq!(map(O::MIRROR_X).point(Vec2::new(10.0, 20.0)), Vec2::new(630.0, 20.0));
        assert_eq!(map(O::ROTATE_180).point(Vec2::new(10.0, 20.0)), Vec2::new(630.0, 492.0));
        // 90°, clockwise on a square map: the top-left tile goes to the top right.
        let square = |o| MapTransform::new(o, 6, 6);
        assert_eq!(square(O::ROTATE_90).tile(t(0, 0)), t(5, 0));
        assert_eq!(square(O::ROTATE_90).tile(t(1, 0)), t(5, 1));
        assert_eq!(square(O::ROTATE_90).corner(c(0, 0)), c(6, 0));
        assert_eq!(square(O::ROTATE_270).tile(t(0, 0)), t(0, 5));
        assert_eq!(square(O::ROTATE_90).point(Vec2::new(64.0, 0.0)), Vec2::new(384.0, 64.0));
    }

    /// Every transform of every mode agrees with itself: a tile's image has the images of its corners as its
    /// corners and the image of its center as its center; repeating a transform gets back to the start.
    #[test]
    fn tile_corner_and_point_images_agree() {
        for (w, h) in [(10, 8), (9, 7), (6, 6), (5, 5)] {
            for s in Symmetry::ALL {
                let Some(transforms) = s.transforms(w, h) else {
                    continue;
                };
                for m in transforms {
                    for y in -1..=h as i32 {
                        for x in -1..=w as i32 {
                            let image = m.tile(t(x, y));
                            let corners: BTreeSet<CornerPos> = t(x, y).corners().map(|c| m.corner(c)).into();
                            let expected: BTreeSet<CornerPos> = image.corners().into();
                            assert_eq!(corners, expected, "{s:?} {m:?} ({x}, {y})");
                            assert_eq!(m.point(t(x, y).center()), image.center());
                            let back = (1..4).fold(image, |p, _| m.tile(p));
                            assert_eq!(back, t(x, y), "4 applications are the identity");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn footprints_cover_the_images_of_their_tiles() {
        let footprint_tiles = |o: TilePos, w: u32, h: u32| -> BTreeSet<TilePos> {
            (0..h as i32)
                .flat_map(|dy| (0..w as i32).map(move |dx| t(o.x + dx, o.y + dy)))
                .collect()
        };
        for s in Symmetry::ALL {
            let Some(transforms) = s.transforms(12, 12) else {
                continue;
            };
            for m in transforms {
                for (o, w, h) in [(t(1, 2), 2, 2), (t(3, 0), 3, 1), (t(0, 5), 1, 4), (t(5, 5), 2, 2)] {
                    let (image, w2, h2) = m.footprint(o, w, h);
                    let mapped: BTreeSet<TilePos> = footprint_tiles(o, w, h).iter().map(|p| m.tile(*p)).collect();
                    assert_eq!(footprint_tiles(image, w2, h2), mapped, "{s:?} {m:?} {o:?} {w}×{h}");
                }
            }
        }
        // A 2×2 station at the center of a square map is its own image under every transform.
        for m in Symmetry::Rotate90.transforms(12, 12).unwrap() {
            assert_eq!(m.footprint(t(5, 5), 2, 2), (t(5, 5), 2, 2));
        }
    }

    #[test]
    fn building_orientations_turn_and_flip_with_the_map() {
        let building = Placement {
            origin: t(1, 2),
            w: 3,
            h: 2,
            rotation: Rotation::R90,
            flip_h: false,
            flip_v: true,
        };
        let before = Orientation::of_building(building.rotation, building.flip_h, building.flip_v);
        for o in Orientation::ALL {
            let m = MapTransform::new(o, 12, 12);
            let image = m.placement(&building);
            let after = Orientation::of_building(image.rotation, image.flip_h, image.flip_v);
            assert_eq!(after, o.after(before), "{o:?}");
            assert_eq!(
                (image.origin, image.w, image.h),
                m.footprint(building.origin, building.w, building.h)
            );
            if !o.mirrors() {
                assert_eq!((image.flip_h, image.flip_v), (false, true), "rotations keep the flips");
            }
        }
        let turned = MapTransform::new(Orientation::ROTATE_90, 12, 12).placement(&building);
        assert_eq!((turned.rotation, turned.w, turned.h), (Rotation::R180, 2, 3));
    }

    #[test]
    fn extreme_coordinates_never_overflow() {
        for o in Orientation::ALL {
            let m = MapTransform::new(o, 256, 256);
            for v in [i32::MIN, i32::MAX, 0, -1] {
                let _ = m.tile(t(v, i32::MIN));
                let _ = m.corner(c(i32::MAX, v));
                let _ = m.footprint(t(v, v), 16, 3);
                let _ = m.point(Vec2::new(f32::MAX, f32::NAN));
            }
        }
    }
}
