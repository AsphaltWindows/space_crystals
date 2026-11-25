//! What Explored tiles show of the structures and resources the local player remembers (`core/vision.md >
//! Explored`: "shown in their last-known state (may have been destroyed or changed since)"): one visual per record
//! the local player's sight shows (`fog::Sight::remembered`), in its kind's placeholder shape (a structure's in its
//! owner's colour, with its tag), where the record has its footprint, dimmed as the fog dims the terrain. The live
//! objects there stay hidden (`fog`), so the view never shows what the player doesn't see. With the fog off there
//! are none: every object shows as it is.
//!
//! Synced only when what it shows may change: the sim, the local player, the fog or the mode, and within a sim the
//! tiles or records of the visions the local player watches (`Sight::revision`, `Sight::records_revision`).

use std::collections::BTreeMap;

use bevy::prelude::*;
use sc_sim::tunnel::TunnelState;
use sc_sim::vision::LastKnown;
use sc_sim::{ObjectId, ObjectKind, Placement, PlayerId};

use super::{MeshKey, ObjectVisualAssets, object_height, object_transform, structures, underground, visual_mesh};
use crate::fog::{EXPLORED_ALPHA, LocalSight, Revision};
use crate::kinds::object_label;
use crate::sim_driver::SimRes;

/// Links a visual entity to the record it shows: the local player's record of object `0`.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct RememberedVisual(pub ObjectId);

/// What a record's visual shows of it: its kind, owner, footprint and a Tunnel's tier.
type Made = (ObjectKind, Option<PlayerId>, Placement, Option<TunnelState>);

/// The visual entity of each record shown.
#[derive(Resource, Default, Debug)]
pub struct RememberedVisuals {
    pub by_id: BTreeMap<ObjectId, Entity>,
    /// What each visual was made from.
    made: BTreeMap<ObjectId, Made>,
    /// What the visuals show.
    shown: Option<Shown>,
}

/// What the visual of `record` shows of it.
fn made(record: &LastKnown) -> Made {
    (record.kind, record.owner, record.placement, record.tunnel.clone())
}

/// Everything the set of records shown depends on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Shown {
    generation: u32,
    player: PlayerId,
    fogged: bool,
    /// The sight's `Sight::revision`: whom it watches, and their tiles.
    revision: Option<Revision>,
    /// Its `Sight::records_revision`: their records.
    records_revision: Option<Revision>,
}

/// How much of its colour a record's visual keeps: the fog's [`EXPLORED_ALPHA`] of black over it, as over the
/// Explored terrain under it. Placeholder: the design doesn't say how remembered objects look.
const KEPT: f32 = 1.0 - EXPLORED_ALPHA;

/// The material of a record's visual: the vertex colours dimmed by [`KEPT`] (in linear colour).
fn dimmed_material() -> ColorMaterial {
    ColorMaterial::from_color(Color::linear_rgb(KEPT, KEPT, KEPT))
}

/// Spawns a visual for each record the local player's sight shows, and despawns those of records it no longer
/// shows (seen again, gone, or another player's or sim's).
pub(super) fn sync_remembered_visuals(
    mut commands: Commands,
    sim: Res<SimRes>,
    sight: LocalSight,
    mut visuals: ResMut<RememberedVisuals>,
    mut assets: ResMut<ObjectVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let seen = sight.get();
    let shown = Shown {
        generation: sim.generation(),
        player: seen.player(),
        fogged: seen.is_fogged(),
        revision: seen.revision(),
        records_revision: seen.records_revision(),
    };
    if visuals.shown == Some(shown) {
        return;
    }
    let RememberedVisuals { by_id, made, .. } = &mut *visuals;
    let wanted: BTreeMap<ObjectId, &LastKnown> = seen.remembered().map(|r| (r.id, r)).collect();
    by_id.retain(|id, entity| {
        let keep = wanted.get(id).is_some_and(|r| made.get(id) == Some(&self::made(r)));
        if !keep {
            commands.entity(*entity).despawn();
            made.remove(id);
        }
        keep
    });
    let assets = &mut *assets;
    for (&id, record) in &wanted {
        if by_id.contains_key(&id) {
            continue;
        }
        // An underground structure's record (a neutral one's: none is taken of a hidden one) blends, as it does live.
        let material = match underground(record.kind) {
            true => assets
                .underground_dimmed
                .get_or_insert_with(|| materials.add(structures::underground_material(KEPT))),
            false => assets.dimmed.get_or_insert_with(|| materials.add(dimmed_material())),
        }
        .clone();
        // Records are of structures, in their owner's colour, and resources.
        let key = MeshKey::of(
            record.kind,
            record.owner,
            Some(&record.placement),
            record.tunnel.clone(),
        );
        let mesh = assets
            .meshes
            .entry(key)
            .or_insert_with(|| meshes.add(visual_mesh(key)))
            .clone();
        let pos = record.pos();
        let height = object_height(sim.map(), pos, Some(&record.placement));
        let entity = commands
            .spawn((
                Name::new(format!("Remembered {:?} #{}", record.kind, id.0)),
                RememberedVisual(id),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                object_transform(record.kind, pos, height, 0.0),
                Visibility::Inherited,
            ))
            .id();
        if let Some(size) = key.size {
            // The text's colour, in sRGB, darkened about as much as the material darkens the mesh.
            let dim = KEPT.powf(1.0 / 2.2);
            let label = object_label(record.kind, record.tunnel.clone());
            commands.spawn((structures::tag(id, label, size, dim), ChildOf(entity)));
        }
        by_id.insert(id, entity);
        made.insert(id, self::made(record));
    }
    visuals.shown = Some(shown);
}
