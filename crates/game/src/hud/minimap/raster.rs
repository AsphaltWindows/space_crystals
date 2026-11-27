//! The minimap's image, texel by texel (`systems/controls/hud.md > Minimap > What it shows`), as plain data: the
//! terrain, baked once per map and frame ([`Terrain`]); over it the fog of war, tile by tile as the main map has it
//! (`core/vision.md > VisibilityState`, `crate::fog::Sight::tile_states`); and over that what the local player is
//! shown of the objects ([`marks`]). [`paint`] puts them together.
//!
//! - **Terrain** "in simplified colours": each tile flat, in the colour the terrain mesh shades its faces from
//!   (`crate::map_render::tile_base_color`: its look, lightened by its height, rock on a cliff face).
//! - **The fog**, without the main map's soft edges: an Unexplored tile black ("fully black", `core/vision.md >
//!   Unexplored`), an Explored one dimmed as the fog layer dims the terrain ([`KEPT`] of its colour in linear light,
//!   `crate::fog::EXPLORED_ALPHA`), a Visible one as it is.
//! - **The objects** ([`marks`]), by the predicates the main map's views ask ([`Sight`]), never by reading the sim's
//!   objects around them: on Explored tiles the records of the structures and resources the player remembers, in their
//!   last-known state, dimmed as the main map's records are (`Sight::remembered`, `crate::object_render::remembered`);
//!   then the live objects the player sees (`Sight::sees`: their own, and the others on Visible tiles; never an
//!   object hidden from them, which no record is taken of either, `sc_sim::vision::hidden`). A structure or a map
//!   resource is its footprint ("buildings as larger squares (their footprint at minimap scale)"), a unit a dot
//!   ("units as small dots"), at its position on the map (`Object::map_pos`: an air unit past the edge at the edge);
//!   each in its owner's colour, map resources and objects without an owner in the neutral one
//!   (`crate::object_render::player_color`). Records go first, then map resources, structures over them (a plate
//!   over its patch), and units on top.

use bevy::prelude::*;
use sc_sim::{Category, Map, Object, ObjectKind, Placement, PlayerId, SU_PER_GU, Sim, TilePos, VisibilityState};

use super::frame::MinimapFrame;
use crate::fog::{EXPLORED_ALPHA, Sight};
use crate::map_render::tile_base_color;
use crate::object_render::player_color;

/// What an Explored texel keeps of its colour, in linear light: what the fog layer's black at [`EXPLORED_ALPHA`]
/// leaves of the terrain under it, and what the main map's records keep of theirs (`crate::object_render::remembered`).
pub const KEPT: f32 = 1.0 - EXPLORED_ALPHA;
/// An Unexplored texel: black (`core/vision.md > Unexplored`).
pub const UNEXPLORED: [u8; 4] = [0, 0, 0, 255];
/// A unit's dot: this fraction of the image's longer side, at least [`MIN_DOT_TEXELS`] texels square. Placeholder
/// sizes (the design: "units as small dots"): 2 texels at 720p, 3 at 1080p.
const DOT_FRACTION: f32 = 1.0 / 80.0;
const MIN_DOT_TEXELS: f32 = 2.0;

/// The RGBA8 sRGB texel of `color`.
pub fn texel(color: Color) -> [u8; 4] {
    color.to_srgba().to_u8_array()
}

/// `color` as an Explored tile shows it: [`KEPT`] of it, in linear light.
pub fn dimmed(color: Color) -> Color {
    let linear = color.to_linear();
    Color::linear_rgba(linear.red * KEPT, linear.green * KEPT, linear.blue * KEPT, linear.alpha)
}

/// The terrain as the minimap paints it, for one map and one frame: the tile under each texel, and each tile's texel
/// as Visible and as Explored.
pub struct Terrain {
    /// The map's width in tiles.
    width: usize,
    /// The tile column under each column of texels, and the tile row under each row: the tiles at the texels'
    /// centres.
    columns: Vec<usize>,
    rows: Vec<usize>,
    /// Each tile's texel, row by row from the north-west corner: Visible, then Explored.
    colors: Vec<[[u8; 4]; 2]>,
}

impl Terrain {
    /// The terrain of `map` in `frame`.
    pub fn bake(map: &Map, frame: &MinimapFrame) -> Self {
        let (width, height) = (map.width() as usize, map.height() as usize);
        let tile = |su: f32, tiles: usize| ((su / SU_PER_GU).max(0.0) as usize).min(tiles.saturating_sub(1));
        let columns = (0..frame.texels.x)
            .map(|x| tile(frame.texel_center(UVec2::new(x, 0)).x, width))
            .collect();
        let rows = (0..frame.texels.y)
            .map(|y| tile(frame.texel_center(UVec2::new(0, y)).y, height))
            .collect();
        let mut colors = Vec::with_capacity(width * height);
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let color = Color::from(tile_base_color(map, TilePos::new(x, y)));
                colors.push([texel(color), texel(dimmed(color))]);
            }
        }
        Self {
            width,
            columns,
            rows,
            colors,
        }
    }
}

/// How the minimap draws an object.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Shape {
    /// A structure's or a map resource's footprint, a ground rectangle (space units).
    Footprint(Rect),
    /// A unit, at this ground point.
    Dot(Vec2),
}

/// An object as the minimap draws it: its shape, in its texel.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mark {
    pub shape: Shape,
    pub color: [u8; 4],
}

/// The footprint of `placement`, in space units.
fn footprint(placement: &Placement) -> Rect {
    let corner = placement.origin.corner();
    Rect::from_corners(
        corner,
        corner + Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU,
    )
}

/// The colour of an object of `kind` that `owner` owns: its owner's, and the neutral one for a map resource (as the
/// main map draws them, `crate::object_render`) and for an object without an owner.
fn object_color(kind: ObjectKind, owner: Option<PlayerId>) -> Color {
    player_color(owner.filter(|_| kind.def().category != Category::Resource))
}

/// When an object of `kind` is drawn among the live ones: map resources, then structures and (M6-GK) mines, then
/// units.
fn layer(kind: ObjectKind) -> u8 {
    match kind.def().category {
        Category::Resource => 0,
        Category::Structure | Category::Mine => 1,
        Category::Unit => 2,
    }
}

/// The live `object` as the minimap draws it.
fn live_mark(map: &Map, object: &Object) -> Mark {
    let shape = match (object.kind.def().category, object.placement.as_ref()) {
        (Category::Structure | Category::Resource, Some(placement)) => Shape::Footprint(footprint(placement)),
        _ => Shape::Dot(object.map_pos(map)),
    };
    Mark {
        shape,
        color: texel(object_color(object.kind, object.owner)),
    }
}

/// What `sight` shows of the objects of `sim` (module docs), in drawing order: the records it shows on Explored
/// tiles, dimmed (map resources first), then the live objects it sees, map resources, structures and units.
pub fn marks(sim: &Sim, sight: &Sight) -> Vec<Mark> {
    let mut records: Vec<(u8, Mark)> = sight
        .remembered()
        .map(|record| {
            let color = texel(dimmed(object_color(record.kind, record.owner)));
            let shape = Shape::Footprint(footprint(&record.placement));
            (layer(record.kind), Mark { shape, color })
        })
        .collect();
    records.sort_by_key(|(layer, _)| *layer);
    let mut live: Vec<(u8, Mark)> = sim
        .objects()
        .filter(|object| sight.sees(object))
        .map(|object| (layer(object.kind), live_mark(sim.map(), object)))
        .collect();
    live.sort_by_key(|(layer, _)| *layer);
    records.into_iter().chain(live).map(|(_, mark)| mark).collect()
}

/// A unit's dot in `frame`, texels square.
pub fn dot_texels(frame: &MinimapFrame) -> f32 {
    (frame.texels.max_element() as f32 * DOT_FRACTION)
        .round()
        .max(MIN_DOT_TEXELS)
}

/// Paints the minimap's image of `frame` into `data` (RGBA8 sRGB, row by row from the top-left texel): `terrain`
/// under the fog of `states` (each tile's state, row by row: `Sight::tile_states`), then `marks` in their order. A
/// footprint covers the texels whose centres it covers, one at least; a dot is [`dot_texels`] square around the
/// unit's position. Nothing for `data` of another size than the frame's.
pub fn paint(data: &mut [u8], frame: &MinimapFrame, terrain: &Terrain, states: &[VisibilityState], marks: &[Mark]) {
    let width = frame.texels.x as usize;
    if data.len() != width * frame.texels.y as usize * 4 || terrain.columns.len() != width {
        return;
    }
    for (row, texels) in terrain.rows.iter().zip(data.chunks_exact_mut(width * 4)) {
        let first = row * terrain.width;
        for (column, texel) in terrain.columns.iter().zip(texels.as_chunks_mut::<4>().0) {
            let tile = first + column;
            *texel = match states.get(tile) {
                Some(VisibilityState::Visible) => terrain.colors[tile][0],
                Some(VisibilityState::Explored) => terrain.colors[tile][1],
                Some(VisibilityState::Unexplored) | None => UNEXPLORED,
            };
        }
    }
    let dot = dot_texels(frame);
    for mark in marks {
        let (min, max) = match mark.shape {
            Shape::Footprint(rect) => {
                let min = frame.texel_at(rect.min).round();
                (min, frame.texel_at(rect.max).round().max(min + 1.0))
            }
            Shape::Dot(at) => {
                let min = (frame.texel_at(at) - dot / 2.0).round();
                (min, min + dot)
            }
        };
        fill(data, frame.texels, min, max, mark.color);
    }
}

/// Fills the texels from `min` to `max` (texel coordinates, `max` excluded, cut to an image of `size`) with `color`.
fn fill(data: &mut [u8], size: UVec2, min: Vec2, max: Vec2, color: [u8; 4]) {
    let cut = |p: Vec2| p.clamp(Vec2::ZERO, size.as_vec2()).as_uvec2();
    let (lo, hi) = (cut(min), cut(max));
    for y in lo.y..hi.y {
        let start = (y * size.x + lo.x) as usize * 4;
        let end = (y * size.x + hi.x) as usize * 4;
        data[start..end].as_chunks_mut::<4>().0.fill(color);
    }
}
