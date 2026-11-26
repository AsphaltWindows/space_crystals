//! The Recruitment Area display (slice M5-J2; `factions/cults/structures/recruitment_hall.md > Recruitment Area
//! display`, design changes 0350, 0351 and 0352, answering Q0107): "as the Tunnel Area display"
//! (`super::tunnel_areas`), whose mechanics it shares (`super::area_overlay`).
//!
//! - **The preview of a Hall being placed** ([`ghost_preview`]): "**Placing a Hall** shows its 10x10 area: the tiles it
//!   would claim, the tiles already claimed by your own Halls, the non-Recruitable tiles, and its Effectiveness in %",
//!   by the sim's rule of claiming (`sc_sim::recruitment::preview`), among the Halls the local player knows of
//!   ([`known_claimants`]), the Effectiveness as a label by the ghost:
//!   - their own completed Halls, as they claim now;
//!   - the other players' completed Halls they see, as they are, or remember, as last seen (design change 0352: "The
//!     preview reads the player's memory of enemy Halls, not the true state"): a remembered Hall counts until they see
//!     its place without it, though it may be gone, and a Hall they never saw doesn't (`core/vision.md > Explored`;
//!     "no information leak"). Nothing of what they don't see reaches the preview: it reads the records of their
//!     sight (`crate::fog::Sight::remembered`) and the objects it shows them (`Sight::sees`);
//!   - their own Halls not completed yet, "under construction and placed ghosts", in their order of placement
//!     (design change 0351): the sites in the order they started, before the ghosts in the order they were placed
//!     (`sc_sim::PendingBuild::group`). **Placeholder (not in the design: the sim keeps no time of placement for a
//!     site, so the sites, already started, rank before every ghost still pending).** Without Shift, the click
//!     replaces the orders of the Recruits placing, whose ghosts then don't count, as in their placement check
//!     (`sc_sim::Sim::group_build_check`).
//! - **The claims of a selected own Hall** ([`selected_claims`]), design change 0373 (answering Q0133): "**A selected
//!   own Hall** shows its claimed tiles: its **true** claims, matching its InfoPanel's Effectiveness and what it
//!   produces", the ones it claims in the game (`Sim::claimed_tiles`), which its InfoPanel's Effectiveness counts
//!   (`crate::hud::info_panel`); "Unlike the placement preview, it may reveal where an unseen enemy Hall took tiles".
//!   A site claims none (design change 0341: "A Hall under construction claims nothing").
//! - **Never an enemy's area**: "Enemy Halls' areas are never drawn as areas of their own"; their claims show only as
//!   the tiles taken from the Hall placed.
//!
//! It shows in the game modes only. **The look**, **Placeholder (Q0001)** until the art, its colours and z in one place
//! (below), consistent with the Tunnel Area display's: the tiles tinted on the terrain's surface, by what the preview
//! finds on them, within the area's outline; a selected Hall's claimed tiles in a selected Tunnel's violet, outlined;
//! not on the tiles the player hasn't explored, which stay black (`core/vision.md > Unexplored`), nor off the map. One
//! mesh, under one handle, rebuilt in place only when what it shows, the unexplored tiles among it or the map changed.

use std::collections::HashSet;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use sc_sim::orders::build::footprint;
use sc_sim::recruitment::def_of;
use sc_sim::recruitment::preview::{Claimant, Claimer, Preview, PreviewTile, Rank};
use sc_sim::{Map, ObjectId, Placement, PlayerId, SU_PER_GU, Sim, TilePos, VisibilityState};

use super::area_overlay::{AreaMesh, selected_own, show, spawn_area_mesh, unexplored};
use super::ghost::{Ghost, PlacementGhost};
use super::overlay::{AREA_Z, outline, tint_area, tint_tiles};
use super::placing_now;
use super::tunnel_areas::rings::BLOCKED_FILL;
use super::tunnel_areas::{OWN_EDGE, OWN_FILL};
use crate::ClientMode;
use crate::command_ui::Builder;
use crate::command_ui::interface::recipients;
use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::fog::{LocalSight, Sight};
use crate::input::{ActionState, InputAction};
use crate::map_render::MapView;
use crate::selection::{ControlStates, Selection};
use crate::shapes::ColorMesh;

/// The tints of the previewed area's tiles (**Placeholder (Q0001)**): those the Hall would claim green, as a valid
/// ghost area; those an own completed Hall has in a selected Tunnel's violet ([`OWN_FILL`]), brighter; those an own
/// Hall not completed yet has in the lingering ghosts' blue (`super::lingering`); those another player's Hall has red,
/// as a Tunnel ring's blocked cells ([`BLOCKED_FILL`]: "shown as unclaimable"); the non-Recruitable ones dark.
pub const CLAIMABLE_FILL: Color = Color::srgba(0.25, 0.95, 0.35, 0.26);
pub const OWN_TAKEN_FILL: Color = Color::srgba(0.70, 0.45, 1.0, 0.34);
pub const PENDING_TAKEN_FILL: Color = Color::srgba(0.55, 0.85, 1.0, 0.30);
pub const OTHER_TAKEN_FILL: Color = BLOCKED_FILL;
pub const NOT_RECRUITABLE_FILL: Color = Color::srgba(0.05, 0.05, 0.08, 0.50);
/// The previewed area's outline.
pub const PREVIEW_EDGE: Color = Color::srgba(0.90, 0.95, 1.0, 0.85);
/// A selected own Hall's claimed tiles: a selected Tunnel's area's look.
pub const SELECTED_CLAIMS: (Color, Color) = (OWN_FILL, OWN_EDGE);
/// The overlay's z above the terrain entity's: in front of the Tunnel Area overlay's ([`AREA_Z`]), under the Tunnel
/// ghost's area and the lingering ghosts' (`super::ghost_area`, `super::lingering`).
pub const RECRUITMENT_AREA_Z: f32 = AREA_Z + 0.002;
/// The Effectiveness label (**Placeholder (not in the design: where and how the Effectiveness shows)**): its z over the
/// overlays' band base (the ghost's letters', `super::ghost`), its text height in space units, its colour, and how far
/// north of the footprint its centre stands, in tiles.
const LABEL_Z: f32 = -1.0;
const LABEL_SU: f32 = 20.0;
const LABEL_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.95);
const LABEL_NORTH_GU: f32 = 1.5;

/// The fill of a tile the preview finds `found`.
pub fn fill_of(found: PreviewTile) -> Color {
    match found {
        PreviewTile::Claimable => CLAIMABLE_FILL,
        PreviewTile::Taken { claimer, .. } => match claimer {
            Claimer::Own => OWN_TAKEN_FILL,
            Claimer::OwnPending => PENDING_TAKEN_FILL,
            Claimer::Other => OTHER_TAKEN_FILL,
        },
        PreviewTile::NotRecruitable => NOT_RECRUITABLE_FILL,
    }
}

/// The preview of the Hall the local player places, as the display shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShownPreview {
    /// The ghost's footprint.
    pub placement: Placement,
    pub preview: Preview,
}

impl ShownPreview {
    /// The Effectiveness label's text. **Placeholder (not in the design: it asks for "its Effectiveness in %", not the
    /// wording; the InfoPanel's, `activity::recruitment`, without the tiles).**
    pub fn label(&self) -> String {
        format!("Effectiveness: {}%", self.preview.effectiveness_percent())
    }
}

/// The tiles a selected own Hall claims ([`selected_claims`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedClaims {
    pub hall: ObjectId,
    /// In tile order (`Sim::claimed_tiles`).
    pub tiles: Vec<TilePos>,
}

/// The claimed tiles of each of `player`'s own completed Halls in `selection`, in id order (module docs); none of
/// another player's Hall, nor of a site.
pub fn selected_claims(sim: &Sim, player: PlayerId, selection: &Selection) -> Vec<SelectedClaims> {
    let halls = selected_own(sim, player, selection).filter(|o| def_of(o.kind).is_some());
    halls
        .map(|o| SelectedClaims {
            hall: o.id,
            tiles: sim.claimed_tiles(o.id),
        })
        .filter(|claims| !claims.tiles.is_empty())
        .collect()
}

/// The Halls the player of `sight` knows of, as the preview of their Hall placed counts them, in the order they claim
/// (module docs): their own completed ones; the other players' completed ones they see or remember; then their own
/// sites, and their pending Builds of a Hall but those the Recruits `replaced` alone hold
/// (`Sim::pending_without`). **Placeholder (not in the design: what the player knows doesn't say when another
/// player's Hall was completed):** another player's Hall ranks as completed at the start, the tie rule then splitting
/// its tiles with an own Hall's by distance (as two starting Halls really do); which of the two has them only shows
/// whose they are, not how many the Hall placed would get.
pub fn known_claimants(sim: &Sim, sight: &Sight, replaced: &[ObjectId]) -> Vec<Claimant> {
    let player = sight.player();
    let halls = sim.objects().filter(|o| def_of(o.kind).is_some());
    let mut claimants = Vec::new();
    let mut sites = Vec::new();
    for o in halls {
        let Some(placement) = o.placement else {
            continue;
        };
        let own = o.owner == Some(player);
        if own && o.construction.is_some() {
            sites.push((o.id, o.kind, placement));
            continue;
        }
        let rank = match (own, &o.production) {
            (true, Some(production)) if o.is_operational() => Rank::Completed(production.completed_at),
            (false, _) if sight.sees(o) && o.is_operational() => Rank::Completed(0),
            _ => continue,
        };
        let claimer = if own { Claimer::Own } else { Claimer::Other };
        claimants.push(Claimant {
            id: Some(o.id),
            kind: o.kind,
            placement,
            claimer,
            rank,
        });
    }
    // The other players' Halls remembered, as last seen: a site claims nothing.
    let remembered = sight
        .remembered()
        .filter(|r| def_of(r.kind).is_some() && r.owner != Some(player));
    for record in remembered.filter(|r| r.site.is_none()) {
        claimants.push(Claimant {
            id: Some(record.id),
            kind: record.kind,
            placement: record.placement,
            claimer: Claimer::Other,
            rank: Rank::Completed(0),
        });
    }
    let mut ghosts: Vec<_> = (sim.pending_without(player, replaced).into_iter())
        .filter(|pending| def_of(pending.kind).is_some())
        .collect();
    ghosts.sort_by_key(|pending| pending.group);
    let pending = (sites.into_iter().map(|(id, kind, site)| (Some(id), kind, site)))
        .chain(ghosts.into_iter().map(|pending| (None, pending.kind, pending.site)));
    for (n, (id, kind, placement)) in (0u32..).zip(pending) {
        claimants.push(Claimant {
            id,
            kind,
            placement,
            claimer: Claimer::OwnPending,
            rank: Rank::Pending(n),
        });
    }
    claimants
}

/// The preview of `ghost`, a Hall's ghost of the player of `sight`'s (module docs), among the Halls they know of
/// ([`known_claimants`]), the Recruits `replaced` dropping their orders; `None` for another kind's ghost. The ground is
/// known where they explored.
pub fn ghost_preview(sim: &Sim, sight: &Sight, ghost: &Ghost, replaced: &[ObjectId]) -> Option<ShownPreview> {
    def_of(ghost.kind)?;
    let placement = footprint(ghost.kind, ghost.origin, ghost.orientation)?;
    let claimants = known_claimants(sim, sight, replaced);
    let known = |t: TilePos| sight.tile(t) != VisibilityState::Unexplored;
    let preview = Preview::of(ghost.kind, placement, &claimants, sim.map(), known)?;
    Some(ShownPreview { placement, preview })
}

/// The tiles of what the display shows: the selected Halls' claimed tiles, then the previewed area's.
fn cells<'a>(selected: &'a [SelectedClaims], preview: Option<&'a ShownPreview>) -> impl Iterator<Item = TilePos> + 'a {
    let claimed = selected.iter().flat_map(|claims| claims.tiles.iter().copied());
    claimed.chain(
        preview
            .into_iter()
            .flat_map(|shown| shown.preview.tiles.iter().map(|&(t, _)| t)),
    )
}

/// The overlay's mesh: each of the `selected` Halls' claimed tiles ([`SELECTED_CLAIMS`], [`tint_area`]), then the
/// previewed area, each tile tinted by what the preview finds on it ([`fill_of`], [`tint_tiles`]) within the area's
/// [`outline`], on their tiles on `map` that are `drawn`, with no edge where the area goes on past the map's edge or
/// into a tile not drawn.
pub fn overlay_mesh(
    map: &Map,
    selected: &[SelectedClaims],
    preview: Option<&ShownPreview>,
    drawn: impl Fn(TilePos) -> bool,
) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    let on_map = |t: TilePos| map.in_bounds(t) && drawn(t);
    for claims in selected {
        tint_area(&mut mesh, map, claims.tiles.iter().copied(), on_map, SELECTED_CLAIMS);
    }
    if let Some(shown) = preview {
        for &(t, found) in &shown.preview.tiles {
            tint_tiles(&mut mesh, map, [t], on_map, fill_of(found));
        }
        let tiles = shown.preview.tiles.iter().map(|&(t, _)| t);
        outline(&mut mesh, map, tiles, on_map, PREVIEW_EDGE);
    }
    mesh
}

/// What the Recruitment Area display shows.
#[derive(Resource, Debug, Default)]
pub struct RecruitmentAreaOverlay {
    /// The selected own Halls' claimed tiles its mesh was last built from.
    pub selected: Vec<SelectedClaims>,
    /// The preview of the Hall placed it was built from.
    pub preview: Option<ShownPreview>,
    /// The tiles of those on the map it left out, in their order: those the local player hadn't explored.
    pub unexplored: Vec<TilePos>,
    /// Its mesh's state.
    mesh: AreaMesh,
    /// Whether it shows now: something to show, in part on explored tiles at least.
    pub visible: bool,
    /// How many times its mesh was built.
    pub builds: u32,
    /// The Effectiveness label's text by the ghost: empty while no Hall is placed.
    pub label: String,
}

/// The overlay's mesh.
#[derive(Component)]
pub struct RecruitmentAreaMesh;

/// The Effectiveness label by the ghost: always visible, its text empty while there is none (a `Text2d` hidden and
/// shown again may not draw: it is never hidden).
#[derive(Component)]
pub struct EffectivenessLabel;

#[derive(Resource)]
pub(super) struct RecruitmentAreaAssets {
    mesh: Handle<Mesh>,
}

pub(super) fn spawn_overlay(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let assets = (meshes.as_ref(), materials.as_mut());
    let marker = RecruitmentAreaMesh;
    let mesh = spawn_area_mesh(
        &mut commands,
        assets,
        "Recruitment Area overlay",
        marker,
        RECRUITMENT_AREA_Z,
    );
    commands.insert_resource(RecruitmentAreaAssets { mesh });
    commands.spawn((
        Name::new("Effectiveness label"),
        EffectivenessLabel,
        Text2d::default(),
        TextFont::from_font_size(FontSize::Px(LABEL_SU)),
        TextColor(LABEL_COLOR),
        Transform::default(),
        Visibility::Inherited,
        NoFrustumCulling,
    ));
}

/// Shows the local player's selected own Halls' claims and the preview of the Hall they place in the game modes
/// (module docs), but on the tiles they haven't explored, redrawn when what it shows or those tiles changed, and the
/// preview's Effectiveness by the ghost. It runs after the ghost's update (`super::ghost::update_ghost`), with Shift as
/// the ghost's check has it.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn draw_overlay(
    mode: Res<ClientMode>,
    local: LocalSight,
    states: Res<ControlStates>,
    ghost: Res<PlacementGhost>,
    actions: Res<ActionState>,
    view: Res<MapView>,
    assets: Res<RecruitmentAreaAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut overlay: ResMut<RecruitmentAreaOverlay>,
    mut shown: Query<&mut Visibility, With<RecruitmentAreaMesh>>,
    mut labels: Query<(&mut Text2d, &mut Transform), With<EffectivenessLabel>>,
) {
    let (sim, sight) = (local.sim(), local.get());
    let player = sight.player();
    let (selected, preview) = if mode.is_game() {
        let selection = &states.of(player).selection;
        let placing = placing_now(*mode, sim, player, &states);
        let replaced = match placing {
            Some(placing) if matches!(placing.builder, Builder::Recruits { .. }) => {
                let queue = actions.held(InputAction::Queue);
                if queue {
                    Vec::new()
                } else {
                    recipients(selection, placing.command())
                }
            }
            _ => Vec::new(),
        };
        let preview = (ghost.shown.as_ref()).and_then(|g| ghost_preview(sim, &sight, g, &replaced));
        (selected_claims(sim, player, selection), preview)
    } else {
        (Vec::new(), None)
    };
    let unexplored = unexplored(sim.map(), &sight, cells(&selected, preview.as_ref()));
    let changed = overlay.selected != selected || overlay.preview != preview || overlay.unexplored != unexplored;
    let build = || {
        let hidden: HashSet<TilePos> = unexplored.iter().copied().collect();
        let any = !selected.is_empty() || preview.is_some();
        any.then(|| overlay_mesh(sim.map(), &selected, preview.as_ref(), |t| !hidden.contains(&t)))
    };
    if overlay.mesh.is_stale(changed, &view) {
        let target = (meshes.as_mut(), &assets.mesh);
        if overlay.mesh.redraw(&view, target, "Recruitment Area overlay", build) {
            overlay.builds += 1;
        }
        let label = preview.as_ref().map(ShownPreview::label).unwrap_or_default();
        if overlay.label != label {
            overlay.label = label;
        }
        overlay.selected = selected;
        overlay.preview = preview;
        overlay.unexplored = unexplored;
    }
    let visible = overlay.mesh.drawn();
    if overlay.visible != visible {
        overlay.visible = visible;
    }
    show(&mut shown, visible);
    let at = overlay
        .preview
        .as_ref()
        .map(|shown| label_at(sim.map(), &shown.placement));
    for (mut text, mut transform) in &mut labels {
        if text.0 != overlay.label {
            text.0.clone_from(&overlay.label);
        }
        if let Some(at) = at {
            transform.set_if_neq(Transform::from_translation(at));
        }
    }
}

/// Where the Effectiveness label of a Hall's ghost on `footprint` stands (Bevy world coordinates): north of the
/// footprint, between it and the area's edge, over every map object.
fn label_at(map: &Map, footprint: &Placement) -> Vec3 {
    let ground = Vec2::new(
        footprint.center().x,
        (footprint.origin.y as f32 - LABEL_NORTH_GU) * SU_PER_GU,
    );
    let at = view_to_world(drawn_at_height(ground, map.height_at(ground)));
    at.extend(DrawLayer::Overlays.base_z() + LABEL_Z)
}

#[cfg(test)]
mod tests;
