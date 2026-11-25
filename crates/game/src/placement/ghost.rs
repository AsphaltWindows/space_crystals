//! The placement ghost (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter] >
//! AwaitingPlacement`): "A ghost preview of the building follows the cursor, snapped to the grid. The ghost is tinted
//! green when the placement is valid and red when invalid. [...] whether a given tile is valid right now (flat,
//! Visible, unoccupied, not a resource) is shown per tile on the ghost" (design change 0093). "Side labels (A/B/C/D
//! per SymmetryType) are displayed on the ghost and update with rotation/flipping" (design change 0079).
//!
//! [`PlacementGhost`] holds what it shows ([`Ghost`]): the footprint where its builder puts it (`Placing::origin`:
//! under the cursor, or on the Beacon of a Warp In), turned as the player turned it, and the check its click would
//! face now ([`check_now`], the rules the command gate applies), worked out every frame while a building is placed
//! and, but for a Warp In, the cursor is over the map. Its one mesh is rebuilt in place, under one handle, only when
//! that changes; its letters are five text entities spawned once, emptied rather than hidden when they show nothing
//! ([`spawn_ghost`] says why).
//!
//! The Colonists' ghosts (slice M3-J1): a Drone's Beacon follows the cursor, green as far as the player knows (design
//! change 0120: `Sim::deploy_check`, which also makes it red over a lingering ghost, a Reserved tile); a Warp In's
//! building stands on its Beacon, centred, an odd size's extras to the bottom-right, and turns about the Beacon's
//! centre (design change 0121: "moving the mouse never changes the ghost"), red where blocked or not Visible (design
//! change 0119: `Sim::warp_in_check`).
//!
//! The look, placeholders until the art (Q0001):
//! - each footprint tile lies on the terrain as the terrain mesh draws it, green if it breaks no rule of its own, red
//!   if it breaks one; every tile is red when the footprint breaks a rule as a whole (no tile in the build area; a
//!   plate not on a patch), as the whole ghost is then invalid ([`Ghost::tile_is_green`]);
//! - an outline around the footprint, green when the placement is valid and red when it isn't: the verdict on the
//!   whole ghost;
//! - the letter of each side at its edge, inside the footprint (`sc_sim::sides`), except on a single tile, whose
//!   sides are all alike (the Extraction Plate's AAAA); and the building's tag in the middle, as its placeholder on
//!   the map shows it (`crate::kinds::short_label`), a Warp In's a little below it, clear of the tag
//!   of the Beacon under the ghost ([`TAG_BELOW_SU`]);
//! - over everything on the map, under the command indicators: it is the cursor.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{
    BuildingOrientation, CornerPos, Map, ObjectKind, PlacementCheck, PlayerCommand, PlayerId, SU_PER_GU, SideLetter,
    Sim, TilePos, sides,
};

use super::placing_now;
use crate::ClientMode;
use crate::command_ui::interface::{deploy_recipient, deploying};
use crate::command_ui::{Builder, Placing};
use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::input::{ActionState, InputAction};
use crate::kinds::short_label;
use crate::map_render::tile_surface;
use crate::selection::{ControlStates, Selection};
use crate::shapes::ColorMesh;
use crate::sim_driver::{LocalPlayer, PendingCommands, SimRes};

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
/// How far south of the middle a Warp In ghost's tag stands, space units: between the tag of the Beacon under it, at
/// the Beacon's centre, and the south side's letter, on the Egress Node's 2 × 2 too.
const TAG_BELOW_SU: f32 = 26.0;
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
    /// Everything that keeps the placement from being valid now ([`check_now`]).
    pub check: PlacementCheck,
    /// The letters of its sides facing north, east, south and west, as turned (`sc_sim::sides`).
    pub sides: Option<[SideLetter; 4]>,
    /// A Warp In's ghost, which stands on its Beacon (design change 0121).
    pub on_beacon: bool,
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
            on_beacon: matches!(placing.builder, Builder::Beacon(_)),
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
    /// south and west, inside their edges (none on a single tile), then the tag in the middle (a Warp In's
    /// [`TAG_BELOW_SU`] south of it).
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
        let tag_below = if self.on_beacon { TAG_BELOW_SU } else { 0.0 };
        let side = |i: usize| {
            let letters = self.sides.filter(|_| lettered)?;
            Some((format!("{:?}", letters[i]), mids[i]))
        };
        [
            side(0),
            side(1),
            side(2),
            side(3),
            Some((short_label(self.kind).to_string(), self.center() + Vec2::Y * tag_below)),
        ]
    }
}

/// What the placement ghost shows now ([`Ghost`]), `None` while it shows nothing: nothing is being placed, or the
/// cursor isn't over the map (but for a Warp In).
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

/// Works out the ghost while the local player places a building ([`placing_now`]): its footprint where its builder
/// puts it (`Placing::origin`: under the cursor while it is over the map; a Warp In's on its Beacon, wherever the
/// cursor is), as turned, and the check its click would face there now ([`check_now`], with Shift as it is held).
pub(super) fn update_ghost(
    mode: Res<ClientMode>,
    actions: Res<ActionState>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    states: Res<ControlStates>,
    pending: Res<PendingCommands>,
    mut ghost: ResMut<PlacementGhost>,
) {
    let pointer = actions.pointer();
    let over_map = !pointer.over_ui && pointer.view.is_some();
    let ground = pointer.ground.filter(|_| over_map);
    let queue = actions.held(InputAction::Queue);
    let selection = &states.of(player.0).selection;
    let shown = placing_now(*mode, &sim, player.0, &states).and_then(|placing| {
        let origin = placing.origin(&sim, ground)?;
        let check = check_now(&sim, player.0, selection, placing, origin, queue, &pending.0)?;
        Ghost::of(placing, origin, check)
    });
    if ghost.shown != shown {
        ghost.shown = shown;
    }
}

/// The check that a click on `player`'s ghost of `placing` at `origin` would face now, by its builder:
/// - a constructor's building: `Sim::placement_check` (the tiles in sight, the build area);
/// - a Warp In: `Sim::warp_in_check` (the tiles in sight, design change 0119: a tile out of sight is red; the
///   Beacon's own footprint consumed);
/// - the Drones' Beacon: `Sim::deploy_check`, as far as the player knows (design change 0120), for the Drone the
///   click would go to (`deploy_recipient`, Shift as `queue` says; the first one when none of them could take a
///   Shift-click's), whose sites a plain click's order replaces; the lingering ghosts' sites (the player's pending
///   Deploys, those sent for the next tick among `unsent` included) are reserved (design change 0125).
///
/// `None` when the builder is gone, or the ActiveGroup no longer has a Drone.
pub fn check_now(
    sim: &Sim,
    player: PlayerId,
    selection: &Selection,
    placing: Placing,
    origin: TilePos,
    queue: bool,
    unsent: &[PlayerCommand],
) -> Option<PlacementCheck> {
    match placing.builder {
        Builder::Constructor(_) => Some(sim.placement_check(player, placing.kind, origin, placing.orientation)),
        Builder::Beacon(beacon) => sim.warp_in_check(beacon, placing.kind, placing.orientation),
        Builder::Drones { .. } => {
            let deploying = deploying(sim, player, unsent);
            let drone = deploy_recipient(selection, queue, &deploying)
                .or_else(|| deploy_recipient(selection, false, &deploying))?;
            sim.deploy_check(drone, origin, queue)
        }
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
mod tests;
