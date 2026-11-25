//! Placeholder visuals for sim objects (`ARCHITECTURE.md > Client architecture > Visuals`).
//!
//! The sim is the source of truth. Each visual is a separate entity linked to its object by [`ObjectVisual`]. Visuals
//! are created and despawned by diffing them against `Sim::objects()` (scenario spawns emit no events); one whose
//! object is now drawn otherwise ([`MeshKey`]) is made anew: an object that changed its kind in place
//! (`SimEvent::KindChanged`, plan decision T15) gets its new kind's mesh, size and tag, a transformation's start and
//! end change its tint, and a defeated player's units turn neutral (`core/game_rules.md > On Defeat`). A unit is drawn
//! in the form it is in, not by its type ([`drawn_kind`], design change 0198): a Legionnaire pressed with Convert
//! changes its look when its conversion is over, its weapon with it. Placed from the sim every frame at
//! `prev_pos.lerp(pos, alpha)`, drawn at the terrain height there (a tile-based object at its footprint's height,
//! `core/art_direction.md > Elevation Display`) and y-sorted by its ground position in its layer: the units' above the
//! structures' and map resources', so that a unit crossing a footprint (out of a portal's energy doorway) stays in
//! sight ([`draw_layer`]). Placeholder shapes stand in until the art direction (Q0001) is settled, each kind's look in
//! its module ([`Look`], `crate::kinds`; drawn by [`units`], [`structures`] and [`resources`]): a unit is a shape of
//! its own per kind, its silhouette's size, in its owner's colour, with a tick showing its facing. A square or
//! octagonal body (the Guard's, the Colonists' units') stays axis-aligned, like its collision box, and only its tick
//! turns ([`FacingTick`]); the Colonists' units and the Peacekeepers carry a tag naming their type ([`UnitTag`]),
//! upright, above the command indicators so that no marker hides it (slice M3-J3). A Peacekeeper's raised Riot Shield
//! is a pale arc in front of it ([`RiotShieldVisual`]), growing out of its body as the shield goes up and shrinking
//! back as it comes down: the Peacekeeper (Shielded) is the Peacekeeper's round body with its shield up or on its way
//! up (design change 0173), which becomes the Peacekeeper's again, the shield on its way down, when the shield is
//! lowered. A worker that carries a load shows a crystal shard on its body, an Agent that carries a Supply a supply
//! crate ([`LoadMarker`], slices M3-J2 and M4-J1). A structure is its footprint as placed, in its owner's colour, with
//! a tag naming its type ([`structures`]); a Tunnel's names its tier, and its sides show their functions by tier
//! ([`tunnel`], design change 0224); an underground structure (the Hideout) is a translucent outline under the units
//! that walk over it (plan decision T32), which only its owner sees while it has one; a neutral one, which a lost
//! Tunnel left (`factions/syndicate/tunnels.md > Losing Tunnels`, slice M4-H: "visible to everyone"), shows to whoever
//! sees its tiles, in the neutral colour, with its HP bar falling as it decays (`crate::combat_effects::hp_bars`: it is
//! a damaged object). An object transforming into its kind (`Object::transform`: a building being warped in) is tinted
//! towards a pale cyan until it is complete ([`transforming_color`]); a site under construction (`Object::is_site`) is
//! tinted towards a pale sand, braced by a scaffold, and shows its progress ([`site`], slice M4-J1), and so does its
//! record, at the progress last seen (slice M4-I2). The map resources have shapes of their own ([`resources`]).
//!
//! The fog of war hides the visuals of the objects the local player doesn't see (`crate::fog`); Explored tiles show
//! the records of what it remembers instead, as visuals of their own ([`remembered`]).

mod load;
mod look;
mod remembered;
mod resources;
mod site;
mod structures;
mod systems;
mod tunnel;
mod units;

use std::collections::BTreeMap;

use bevy::prelude::*;
use sc_sim::tunnel::TunnelState;
use sc_sim::vision::LastKnown;
use sc_sim::{Category, Map, Object, ObjectId, ObjectKind, Placement, PlayerId};

use crate::ClientSystems;
use crate::coords::{DrawLayer, facing_to_rotation, footprint_height, ground_translation};
use systems::{place_object_visuals, place_riot_shields, sync_object_visuals};
use tunnel::TunnelLook;

/// Links a visual entity to the sim object it shows.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObjectVisual(pub ObjectId);

/// The facing tick of a visual whose body doesn't turn (`units::body_turns`): a child of the object's visual, turned
/// by the object's facing.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FacingTick(pub ObjectId);

/// The Riot Shield of a unit that carries one (`factions/gdo/units/peacekeeper.md > Riot Shield`): a child of the
/// unit's visual, hidden while the shield is down, else inheriting the visual's visibility, so the fog hides it with
/// its unit (`crate::fog`). Placeholder until the art exists (Q0001).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct RiotShieldVisual(pub ObjectId);

pub use load::{LoadMarker, carries};
pub use look::{Look, UnitShape};
pub use remembered::{RememberedVisual, RememberedVisuals};
pub use site::SiteProgress;
pub use structures::{StructureTag, underground};
pub use units::UnitTag;

/// What the mesh of an object's visual depends on ([`MeshKey::of`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct MeshKey {
    kind: ObjectKind,
    /// The owner whose colour it is drawn in: a unit's or a structure's; none for map resources.
    owner: Option<PlayerId>,
    /// A structure's size as placed (`Placement::w`, `Placement::h`, turned).
    size: Option<(u32, u32)>,
    /// Whether the object is transforming into its kind (`Object::transform`), which tints it.
    transforming: bool,
    /// Whether the object is a site under construction (`Object::is_site`), or was when last seen
    /// (`LastKnown::site`), drawn as one ([`site`]).
    site: bool,
    /// A Tunnel's look: its tier and where its sides face ([`tunnel`]).
    tunnel: Option<TunnelLook>,
}

impl MeshKey {
    /// An object of `kind` of `owner`'s, placed at `placement`, with tier state `tunnel` (a Tunnel's), finished.
    fn of(
        kind: ObjectKind,
        owner: Option<PlayerId>,
        placement: Option<&Placement>,
        tunnel: Option<TunnelState>,
    ) -> Self {
        let def = kind.def();
        Self {
            kind,
            owner: owner.filter(|_| def.category != Category::Resource),
            size: placement.filter(|_| def.structure.is_some()).map(|p| (p.w, p.h)),
            transforming: false,
            site: false,
            tunnel: TunnelLook::of(kind, tunnel, placement),
        }
    }

    /// The live `object`'s, in the form it is in ([`drawn_kind`]).
    fn of_object(object: &Object) -> Self {
        let placement = object.placement.as_ref();
        Self {
            transforming: object.is_transforming(),
            site: object.is_site(),
            ..Self::of(drawn_kind(object), object.owner, placement, object.tunnel.clone())
        }
    }

    /// The object of `record` as last seen (`remembered`): a site if it was one then.
    fn of_record(record: &LastKnown) -> Self {
        Self {
            site: record.site.is_some(),
            ..Self::of(
                record.kind,
                record.owner,
                Some(&record.placement),
                record.tunnel.clone(),
            )
        }
    }
}

/// Mesh and material handles, created once per object kind (and owner, for units and structures, and size as
/// placed, for structures) and shared by all its visuals.
#[derive(Resource, Default)]
pub struct ObjectVisualAssets {
    material: Option<Handle<ColorMaterial>>,
    /// The remembered objects' material ([`remembered`]).
    dimmed: Option<Handle<ColorMaterial>>,
    /// The blending materials of underground structures, live and remembered (`structures::underground_material`).
    underground: Option<Handle<ColorMaterial>>,
    underground_dimmed: Option<Handle<ColorMaterial>>,
    meshes: BTreeMap<MeshKey, Handle<Mesh>>,
    ticks: BTreeMap<ObjectKind, Handle<Mesh>>,
    shields: BTreeMap<ObjectKind, Handle<Mesh>>,
    loads: BTreeMap<(ObjectKind, sc_sim::defs::Resource), Handle<Mesh>>,
    /// The fill of a site's progress bar ([`site`]).
    site_fill: Option<Handle<Mesh>>,
}

/// The visual entity of each object.
#[derive(Resource, Default, Debug)]
pub struct ObjectVisuals {
    pub by_id: BTreeMap<ObjectId, Entity>,
    /// What each visual was made from.
    made: BTreeMap<ObjectId, MeshKey>,
    /// The `SimRes::generation` the visuals belong to.
    generation: Option<u32>,
}

pub struct ObjectRenderPlugin;

impl Plugin for ObjectRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectVisualAssets>()
            .init_resource::<ObjectVisuals>()
            .init_resource::<RememberedVisuals>()
            .add_systems(Update, sync_object_visuals.in_set(ClientSystems::SyncSim))
            .add_systems(
                Update,
                (
                    (place_object_visuals, units::upright_tags).chain(),
                    place_riot_shields,
                    load::place_load_markers,
                    site::place_site_progress,
                    remembered::sync_remembered_visuals,
                )
                    .in_set(ClientSystems::Present),
            );
    }
}

/// The height an object stands at: its footprint's for a tile-based object, else the terrain's at `pos`.
pub fn object_height(map: &Map, pos: Vec2, placement: Option<&Placement>) -> f32 {
    match placement {
        Some(p) => footprint_height(map, p.origin, p.w, p.h),
        None => map.height_at(pos),
    }
}

/// The layer the visuals of objects of `kind` are drawn in: a unit's above every tile-based object's
/// ([`DrawLayer::GroundUnits`]), so that a unit is never hidden under a structure, also while it crosses one's
/// footprint (a unit walking out of a portal's energy doorway, slice M3-D2); structures and map resources y-sorted
/// among themselves ([`DrawLayer::GroundObjects`]), a plate over the patch it covers (`structures`).
pub fn draw_layer(kind: ObjectKind) -> DrawLayer {
    match kind.def().category {
        Category::Unit => DrawLayer::GroundUnits,
        Category::Structure | Category::Resource => DrawLayer::GroundObjects,
    }
}

/// The transform of the visual of an object of `kind` anchored at ground position `pos`, standing at `height` levels,
/// with sim facing `facing`: y-sorted by its ground position in its kind's layer ([`draw_layer`]).
pub fn object_transform(kind: ObjectKind, pos: Vec2, height: f32, facing: f32) -> Transform {
    Transform::from_translation(ground_translation(draw_layer(kind), pos, height))
        .with_rotation(facing_to_rotation(facing))
}

/// The placeholder shape of each object kind, centered on the object's position; units and structures in the
/// neutral colour, structures unturned.
pub fn placeholder_mesh(kind: ObjectKind) -> Mesh {
    object_mesh(kind, None)
}

/// The placeholder shape of an object of `kind` owned by `owner`, by its kind's look ([`Look`]), a structure
/// unturned. Units and structures take their owner's colour.
pub fn object_mesh(kind: ObjectKind, owner: Option<PlayerId>) -> Mesh {
    match Look::of(kind) {
        Look::CrystalPatch => resources::crystal_patch_mesh(),
        Look::SupplyDeliveryStation => resources::supply_station_mesh(),
        Look::Unit { .. } => units::unit_mesh(kind, player_color(owner), true),
        Look::Structure => {
            let (w, h) = kind.def().tile_size().unwrap_or((1, 1));
            structures::structure_mesh(w, h, player_color(owner))
        }
    }
}

/// The kind a live `object` is drawn as: the form it is in (`sc_sim::conversion::form`, `ARCHITECTURE.md > Object
/// model > Type and form`), which is its type but for a Legionnaire whose conversion hasn't ended yet (design change
/// 0198: its type switches at the press, its weapon when the conversion is over), so that a Sabre still finishing its
/// swing in the Rifle group is drawn as the Sabre it still is. The Peacekeeper (Shielded) has the Peacekeeper's body,
/// its shield drawn by its state ([`place_riot_shields`]).
fn drawn_kind(object: &Object) -> ObjectKind {
    sc_sim::conversion::form(object)
}

/// Whether a visual of `kind` turns with its object's facing: only a unit's round body does (`units::body_turns`).
/// Structures and map resources are drawn as placed, whatever their facing (a structure that was a unit keeps the
/// unit's: `Sim::change_kind`).
fn body_turns(kind: ObjectKind) -> bool {
    kind.def().unit.is_some() && units::body_turns(kind)
}

/// Whether a visual of `kind` shows its object's facing by a tick of its own ([`FacingTick`]): a unit whose body
/// doesn't turn.
fn has_tick(kind: ObjectKind) -> bool {
    kind.def().unit.is_some() && !units::body_turns(kind)
}

/// The mesh of an object's visual: its placeholder shape, a unit's without the tick if its body doesn't turn, a
/// structure in its size as placed, a Tunnel with its look by tier, an underground structure as its outline; tinted
/// while it transforms; a site, on the surface or underground (an expansion, slice M4-F), tinted and braced by its
/// scaffold, with its progress bar's track ([`site`]).
fn visual_mesh(key: MeshKey) -> Mesh {
    let MeshKey {
        kind,
        owner,
        size,
        transforming,
        site,
        tunnel,
    } = key;
    let color = match (transforming, site) {
        (true, _) => transforming_color(player_color(owner)),
        (false, true) => site::site_color(player_color(owner)),
        (false, false) => player_color(owner),
    };
    match size {
        Some((w, h)) => {
            let mut mesh = if underground(kind) {
                structures::underground_shape(w, h, color)
            } else {
                structures::structure_shape(w, h, color)
            };
            if site {
                site::draw(&mut mesh, structures::inner_half(w, h));
            } else if let Some(look) = tunnel {
                tunnel::draw(&mut mesh, look, structures::inner_half(w, h));
            }
            mesh.into_mesh()
        }
        None if kind.def().unit.is_some() => units::unit_mesh(kind, color, units::body_turns(kind)),
        None => object_mesh(kind, owner),
    }
}

/// The pale cyan that a transforming object's colour turns towards.
const TRANSFORMING_TINT: Srgba = Srgba::rgb(0.62, 0.96, 1.0);

/// The colour of a visual whose object is transforming into its kind (`Object::transform`: a building being warped
/// in), from its owner's `color`: half way to a pale cyan. Placeholder until the art direction (Q0001) says how a
/// Warp In looks; the design gives none.
pub fn transforming_color(color: Color) -> Color {
    color.to_srgba().mix(&TRANSFORMING_TINT, 0.5).into()
}

/// Player colours, by player number: those of the editor's slots, whose players they are by default.
const PLAYER_COLORS: [Color; 8] = [
    Color::srgb(0.30, 0.55, 1.00),
    Color::srgb(1.00, 0.35, 0.30),
    Color::srgb(0.35, 0.85, 0.40),
    Color::srgb(1.00, 0.85, 0.25),
    Color::srgb(0.75, 0.45, 1.00),
    Color::srgb(0.30, 0.90, 0.90),
    Color::srgb(1.00, 0.60, 0.20),
    Color::srgb(1.00, 0.50, 0.80),
];
const NEUTRAL_COLOR: Color = Color::srgb(0.70, 0.70, 0.66);

/// The colour of `owner`'s units and structures: grey for neutral ones.
pub fn player_color(owner: Option<PlayerId>) -> Color {
    match owner {
        Some(p) => PLAYER_COLORS[usize::from(p.0.max(1) - 1) % PLAYER_COLORS.len()],
        None => NEUTRAL_COLOR,
    }
}
