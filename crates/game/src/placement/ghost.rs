//! The placement ghost (`factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter] >
//! AwaitingPlacement`): "A ghost preview of the building follows the cursor, snapped to the grid. The ghost is tinted
//! green when the placement is valid and red when invalid. [...] whether a given tile is valid right now (flat,
//! Visible, unoccupied, not a resource) is shown per tile on the ghost" (design change 0093). "Side labels (A/B/C/D
//! per SymmetryType) are displayed on the ghost and update with rotation/flipping" (design change 0079).
//!
//! [`PlacementGhost`] holds what it shows ([`Ghost`]): the footprint under the cursor ([`Placing::origin_under`]),
//! turned as the player turned it, and the placement check now (`Sim::placement_check`, the rules the command gate
//! applies), worked out every frame while a building is placed and the cursor is over the map. Its one mesh is
//! rebuilt in place, under one handle, only when that changes; its letters are five text entities spawned once,
//! emptied rather than hidden when they show nothing ([`spawn_ghost`] says why).
//!
//! The look, placeholders until the art (Q0001):
//! - each footprint tile lies on the terrain as the terrain mesh draws it, green if it breaks no rule of its own, red
//!   if it breaks one; every tile is red when the footprint breaks a rule as a whole (no tile in the build area; a
//!   plate not on a patch), as the whole ghost is then invalid ([`Ghost::tile_is_green`]);
//! - an outline around the footprint, green when the placement is valid and red when it isn't: the verdict on the
//!   whole ghost;
//! - the letter of each side at its edge, inside the footprint (`sc_sim::sides`), except on a single tile, whose
//!   sides are all alike (the Extraction Plate's AAAA); and the building's tag in the middle, as its placeholder on
//!   the map shows it (`crate::hud::selection_panel::short_label`);
//! - over everything on the map, under the command indicators: it is the cursor.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{BuildingOrientation, CornerPos, Map, ObjectKind, PlacementCheck, SU_PER_GU, SideLetter, TilePos, sides};

use super::placing_now;
use crate::ClientMode;
use crate::command_ui::Placing;
use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::hud::selection_panel::short_label;
use crate::input::ActionState;
use crate::map_render::tile_surface;
use crate::selection::ControlStates;
use crate::shapes::ColorMesh;
use crate::sim_driver::{LocalPlayer, SimRes};

/// A footprint tile that breaks no rule, and one that breaks one (placeholder colours).
pub const VALID_FILL: Color = Color::srgba(0.25, 0.95, 0.35, 0.45);
pub const INVALID_FILL: Color = Color::srgba(1.0, 0.22, 0.18, 0.55);
/// The outline of a valid placement, and of an invalid one.
pub const VALID_EDGE: Color = Color::srgb(0.35, 1.0, 0.45);
pub const INVALID_EDGE: Color = Color::srgb(1.0, 0.30, 0.25);
const LABEL_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.95);
/// The outline's width, space units.
const EDGE_SU: f32 = 4.0;
/// The text heights of the side letters and of the tag (on a footprint at least 2 tiles across, and on a narrower
/// one), space units.
const SIDE_SU: f32 = 18.0;
const TAG_SU: f32 = 22.0;
const SMALL_TAG_SU: f32 = 15.0;
/// How far inside its edge a side letter's centre lies, space units.
const SIDE_INSET_SU: f32 = 13.0;
/// The ghost's z over the overlays' band base: above everything on the map, under the command indicators and the
/// drag box (`DrawLayer::Overlays`). The letters lie just above it; the outline over the tiles.
const GHOST_Z: f32 = -2.0;
const LABEL_Z: f32 = -1.0;
const EDGE_Z: f32 = 0.01;

/// The ghost of the building being placed, as it shows now.
#[derive(Clone, Debug, PartialEq)]
pub struct Ghost {
    pub kind: ObjectKind,
    /// The top-left tile of its footprint.
    pub origin: TilePos,
    pub orientation: BuildingOrientation,
    /// The footprint's size as turned, `(w, h)` tiles.
    pub size: (u32, u32),
    /// Everything that keeps the placement from being valid now (`Sim::placement_check`).
    pub check: PlacementCheck,
    /// The letters of its sides facing north, east, south and west, as turned (`sc_sim::sides`).
    pub sides: Option<[SideLetter; 4]>,
}

impl Ghost {
    /// The ghost of `placing` with the top-left tile of its footprint at `origin`, and its placement `check`.
    pub fn of(placing: Placing, origin: TilePos, check: PlacementCheck) -> Option<Self> {
        Some(Self {
            kind: placing.kind,
            origin,
            orientation: placing.orientation,
            size: placing.size()?,
            check,
            sides: sides(placing.kind, placing.orientation),
        })
    }

    /// Whether the building may be placed here now: green.
    pub fn is_valid(&self) -> bool {
        self.check.is_valid()
    }

    /// The footprint's tiles, row by row from the north-west.
    pub fn tiles(&self) -> impl Iterator<Item = TilePos> + '_ {
        let (w, h) = self.size;
        let at = |origin: i32, d: u32| origin.saturating_add(d as i32);
        (0..h).flat_map(move |dy| (0..w).map(move |dx| TilePos::new(at(self.origin.x, dx), at(self.origin.y, dy))))
    }

    /// Whether footprint tile `t` shows green: it breaks no rule of its own, and the footprint breaks none as a
    /// whole (design change 0093: a tile's validity shows on the ghost, tile by tile; a ghost invalid as a whole is
    /// red throughout).
    pub fn tile_is_green(&self, t: TilePos) -> bool {
        self.check.whole.is_empty() && self.check.tile_is_valid(t)
    }

    /// The colour of footprint tile `t`.
    pub fn tile_color(&self, t: TilePos) -> Color {
        if self.tile_is_green(t) {
            VALID_FILL
        } else {
            INVALID_FILL
        }
    }

    /// The outline's colour: the verdict on the whole placement.
    pub fn edge_color(&self) -> Color {
        if self.is_valid() { VALID_EDGE } else { INVALID_EDGE }
    }

    /// The ground point at the middle of its footprint.
    fn center(&self) -> Vec2 {
        self.origin.corner() + Vec2::new(self.size.0 as f32, self.size.1 as f32) * (SU_PER_GU / 2.0)
    }

    /// The text of each of its labels, and the ground point where it stands: the side letters facing north, east,
    /// south and west, inside their edges (none on a single tile), then the tag in the middle.
    pub fn labels(&self) -> [Option<(String, Vec2)>; 5] {
        let (w, h) = (self.size.0 as f32 * SU_PER_GU, self.size.1 as f32 * SU_PER_GU);
        let nw = self.origin.corner();
        let mids = [
            nw + Vec2::new(w / 2.0, SIDE_INSET_SU),
            nw + Vec2::new(w - SIDE_INSET_SU, h / 2.0),
            nw + Vec2::new(w / 2.0, h - SIDE_INSET_SU),
            nw + Vec2::new(SIDE_INSET_SU, h / 2.0),
        ];
        let lettered = self.size.0.max(self.size.1) > 1;
        let side = |i: usize| {
            let letters = self.sides.filter(|_| lettered)?;
            Some((format!("{:?}", letters[i]), mids[i]))
        };
        [
            side(0),
            side(1),
            side(2),
            side(3),
            Some((short_label(self.kind).to_string(), self.center())),
        ]
    }
}

/// What the placement ghost shows now ([`Ghost`]), `None` while it shows nothing: nothing is being placed, or the
/// cursor isn't over the map.
#[derive(Resource, Debug, Default)]
pub struct PlacementGhost {
    pub shown: Option<Ghost>,
    /// How many times its mesh was built: only when what it shows changed.
    pub builds: u32,
}

/// The ghost's mesh.
#[derive(Component)]
pub struct GhostMesh;

/// One of the ghost's labels: a side letter (north, east, south, west: 0 to 3) or the tag (4), in the order of
/// [`Ghost::labels`].
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct GhostLabel(pub usize);

#[derive(Resource)]
pub(super) struct GhostAssets {
    mesh: Handle<Mesh>,
}

pub(super) fn spawn_ghost(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // The mesh asset arrives with the first ghost: the renderer warns about empty meshes.
    let mesh = meshes.reserve_handle();
    let material = ColorMaterial {
        color: Color::WHITE,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    };
    commands.spawn((
        Name::new("Placement ghost"),
        GhostMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(material)),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Overlays.base_z() + GHOST_Z),
        Visibility::Hidden,
    ));
    for label in 0..5 {
        let size = if label < 4 { SIDE_SU } else { TAG_SU };
        // Always visible and never culled: a label shows nothing while its text is empty. Bevy lays a `Text2d` out
        // only while some camera saw a sprite or a text the frame before, and a change made in a frame without one
        // is lost (`bevy_sprite::text2d::update_text2d_layout`): with no structure in view, a label hidden until the
        // ghost shows would never get its letter. These five keep the world camera seeing one.
        commands.spawn((
            Name::new("Placement ghost label"),
            GhostLabel(label),
            Text2d::default(),
            TextFont::from_font_size(FontSize::Px(size)),
            TextColor(LABEL_COLOR),
            Transform::default(),
            Visibility::Inherited,
            NoFrustumCulling,
        ));
    }
    commands.insert_resource(GhostAssets { mesh });
}

/// Works out the ghost under the cursor while the local player places a building ([`placing_now`]) and the cursor
/// is over the map: its footprint, as turned, and the placement check there now.
pub(super) fn update_ghost(
    mode: Res<ClientMode>,
    actions: Res<ActionState>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    states: Res<ControlStates>,
    mut ghost: ResMut<PlacementGhost>,
) {
    let pointer = actions.pointer();
    let over_map = !pointer.over_ui && pointer.view.is_some();
    let shown = placing_now(*mode, &sim, player.0, &states)
        .filter(|_| over_map)
        .and_then(|placing| {
            let origin = placing.origin_under(pointer.ground?)?;
            let check = sim.placement_check(player.0, placing.kind, origin, placing.orientation);
            Ghost::of(placing, origin, check)
        });
    if ghost.shown != shown {
        ghost.shown = shown;
    }
}

/// Draws the ghost when what it shows changed: its mesh rebuilt in place, its labels moved and rewritten (a label
/// that shows nothing is emptied, never hidden: see [`spawn_ghost`]).
pub(super) fn draw_ghost(
    sim: Res<SimRes>,
    assets: Res<GhostAssets>,
    mut ghost: ResMut<PlacementGhost>,
    mut drawn: Local<Option<Ghost>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh: Query<&mut Visibility, With<GhostMesh>>,
    mut labels: Query<(&GhostLabel, &mut Text2d, &mut TextFont, &mut Transform)>,
) {
    if *drawn == ghost.shown {
        return;
    }
    *drawn = ghost.shown.clone();
    let map = sim.map();
    let visibility = match &ghost.shown {
        Some(shown) => {
            if let Err(error) = meshes.insert(&assets.mesh, ghost_mesh(map, shown).into_mesh()) {
                error!("placement ghost mesh: {error}");
            }
            ghost.bypass_change_detection().builds += 1;
            Visibility::Inherited
        }
        None => Visibility::Hidden,
    };
    for mut shown in &mut mesh {
        shown.set_if_neq(visibility);
    }
    let texts = drawn.as_ref().map(Ghost::labels);
    let narrow = drawn.as_ref().is_some_and(|g| g.size.0.min(g.size.1) < 2);
    for (label, mut text, mut font, mut transform) in &mut labels {
        let Some((wanted, ground)) = texts.as_ref().and_then(|texts| texts.get(label.0)?.clone()) else {
            if !text.0.is_empty() {
                text.0.clear();
            }
            continue;
        };
        if text.0 != wanted {
            text.0 = wanted;
        }
        if label.0 == 4 {
            let px = if narrow { SMALL_TAG_SU } else { TAG_SU };
            if !matches!(font.font_size, FontSize::Px(now) if now == px) {
                font.font_size = FontSize::Px(px);
            }
        }
        let at = view_to_world(drawn_at_height(ground, map.height_at(ground)));
        transform.set_if_neq(Transform::from_translation(
            at.extend(DrawLayer::Overlays.base_z() + LABEL_Z),
        ));
    }
}

/// Where corner `c` of the terrain is drawn (Bevy world coordinates): lifted by its height, off the map at the
/// height of the nearest ground (as `tile_surface` has it).
fn drawn_corner(map: &Map, c: CornerPos) -> Vec2 {
    let height = map.corner(c).map_or_else(|| map.height_at(c.pos()), f32::from);
    view_to_world(drawn_at_height(c.pos(), height))
}

/// The ghost's mesh: each footprint tile lying on the terrain in its colour (a quad if it is flat, else four
/// triangles around its centre, as the terrain mesh draws it), and the outline around the footprint. Bevy world
/// coordinates, z relative to the ghost's entity.
pub fn ghost_mesh(map: &Map, ghost: &Ghost) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    for t in ghost.tiles() {
        let surface = tile_surface(map, t);
        let [nw, ne, se, sw, center] = surface.map(|(ground, height)| view_to_world(drawn_at_height(ground, height)));
        let color = ghost.tile_color(t);
        let flat = surface[..4].iter().all(|&(_, height)| height == surface[0].1);
        if flat {
            mesh.polygon(&[nw, ne, se, sw], 0.0, color);
        } else {
            for (a, b) in [(nw, ne), (ne, se), (se, sw), (sw, nw)] {
                mesh.polygon(&[a, b, center], 0.0, color);
            }
        }
    }
    let (w, h) = (ghost.size.0 as i32, ghost.size.1 as i32);
    let (x0, y0) = (ghost.origin.x, ghost.origin.y);
    let corner = |dx: i32, dy: i32| drawn_corner(map, CornerPos::new(x0.saturating_add(dx), y0.saturating_add(dy)));
    let color = ghost.edge_color();
    for dx in 0..w {
        mesh.segment(corner(dx, 0), corner(dx + 1, 0), EDGE_SU, EDGE_Z, color);
        mesh.segment(corner(dx, h), corner(dx + 1, h), EDGE_SU, EDGE_Z, color);
    }
    for dy in 0..h {
        mesh.segment(corner(0, dy), corner(0, dy + 1), EDGE_SU, EDGE_Z, color);
        mesh.segment(corner(w, dy), corner(w, dy + 1), EDGE_SU, EDGE_Z, color);
    }
    mesh
}

#[cfg(test)]
mod tests {
    use sc_sim::{ObjectId, TileProblem, WholeProblem};

    use super::*;
    use crate::editor::orientation::Turn;

    fn ghost(placing: Placing, check: PlacementCheck) -> Ghost {
        Ghost::of(placing, TilePos::new(3, 2), check).unwrap()
    }

    /// Design change 0093: each tile shows its own validity, and a ghost invalid as a whole is red throughout; the
    /// outline gives the verdict.
    #[test]
    fn tiles_are_green_or_red_by_their_rules() {
        let generator = Placing::new(ObjectId(1), ObjectKind::Generator);
        let valid = ghost(generator, PlacementCheck::default());
        assert_eq!(valid.tiles().count(), 4);
        assert!(valid.tiles().all(|t| valid.tile_is_green(t)));
        assert_eq!(valid.edge_color(), VALID_EDGE);
        let blocked = TilePos::new(4, 3);
        let one_tile = ghost(
            generator,
            PlacementCheck {
                tiles: vec![(blocked, TileProblem::Unit)],
                whole: Vec::new(),
            },
        );
        let red: Vec<TilePos> = one_tile.tiles().filter(|&t| !one_tile.tile_is_green(t)).collect();
        assert_eq!(red, [blocked]);
        assert_eq!(one_tile.tile_color(blocked), INVALID_FILL);
        assert_eq!(one_tile.edge_color(), INVALID_EDGE);
        let outside = ghost(
            generator,
            PlacementCheck {
                tiles: Vec::new(),
                whole: vec![WholeProblem::OutsideBuildArea],
            },
        );
        assert!(outside.tiles().all(|t| !outside.tile_is_green(t)));
    }

    /// Design change 0079: the Barracks (3 wide, 2 tall, ABAC) shows A on top, B on the right, A at the bottom and C
    /// on the left; turned clockwise it is 2 wide and 3 tall, with B at the bottom. A plate's single tile has no
    /// letters.
    #[test]
    fn side_letters_follow_the_turns() {
        let barracks = Placing::new(ObjectId(1), ObjectKind::Barracks);
        let letters = |g: &Ghost| g.labels().map(|l| l.map(|(text, _)| text));
        let plain = ghost(barracks, PlacementCheck::default());
        assert_eq!(plain.size, (3, 2));
        let some = |s: &str| Some(s.to_string());
        assert_eq!(
            letters(&plain),
            [some("A"), some("B"), some("A"), some("C"), some("BR")]
        );
        let turned = ghost(barracks.turned(Turn::Clockwise), PlacementCheck::default());
        assert_eq!(turned.size, (2, 3));
        assert_eq!(
            letters(&turned),
            [some("C"), some("A"), some("B"), some("A"), some("BR")]
        );
        let flipped = ghost(barracks.turned(Turn::FlipHorizontal), PlacementCheck::default());
        assert_eq!(
            letters(&flipped),
            [some("A"), some("C"), some("A"), some("B"), some("BR")]
        );
        let plate = ghost(
            Placing::new(ObjectId(1), ObjectKind::ExtractionPlate),
            PlacementCheck::default(),
        );
        assert_eq!(letters(&plate), [None, None, None, None, some("EP")]);
        // The letters stand inside their edges: north on top, east on the right.
        let labels = plain.labels();
        let (north, east) = (labels[0].clone().unwrap().1, labels[1].clone().unwrap().1);
        assert!(north.y < plain.center().y && east.x > plain.center().x);
    }

    /// A flat tile is a quad, a sloped one four triangles; the outline is a quad per tile edge around the footprint.
    #[test]
    fn the_mesh_lies_on_the_terrain() {
        // Only corner (6, 4) is raised: tile (5, 3) slopes.
        let map =
            Map::from_ascii("......\n......\n......\n......\n\n0000000\n0000000\n0000000\n0000000\n0000001").unwrap();
        let generator = Placing::new(ObjectId(1), ObjectKind::Generator);
        let flat = ghost(generator, PlacementCheck::default());
        // 4 quads, then 8 tile edges.
        assert_eq!(ghost_mesh(&map, &flat).vertex_count(), 4 * 4 + 8 * 4);
        // At (4, 2) its south-east tile (5, 3) slopes.
        let sloped = Ghost::of(generator, TilePos::new(4, 2), PlacementCheck::default()).unwrap();
        assert_eq!(ghost_mesh(&map, &sloped).vertex_count(), 3 * 4 + 4 * 3 + 8 * 4);
    }
}
