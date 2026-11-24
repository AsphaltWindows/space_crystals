//! What Explored tiles show of the structures and resources the local player remembers (`core/vision.md >
//! Explored`: "shown in their last-known state (may have been destroyed or changed since)"): one visual per record
//! the local player's sight shows (`fog::Sight::remembered`), in its kind's placeholder shape, where the record has
//! its footprint, dimmed as the fog dims the terrain. The live objects there stay hidden (`fog`), so the view never
//! shows what the player doesn't see. With the fog off there are none: every object shows as it is.
//!
//! Synced only when what it shows may change: the sim, the local player, the fog or the mode, and within a sim the
//! player's tiles or records (`PlayerVision::revision`, `PlayerVision::records_revision`).

use std::collections::BTreeMap;

use bevy::prelude::*;
use sc_sim::vision::LastKnown;
use sc_sim::{ObjectId, ObjectKind, Placement, PlayerId};

use super::{ObjectVisualAssets, object_height, object_transform, visual_mesh};
use crate::fog::{EXPLORED_ALPHA, LocalSight};
use crate::sim_driver::SimRes;

/// Links a visual entity to the record it shows: the local player's record of object `0`.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct RememberedVisual(pub ObjectId);

/// The visual entity of each record shown.
#[derive(Resource, Default, Debug)]
pub struct RememberedVisuals {
    pub by_id: BTreeMap<ObjectId, Entity>,
    /// What each visual was made from.
    made: BTreeMap<ObjectId, (ObjectKind, Placement)>,
    /// What the visuals show.
    shown: Option<Shown>,
}

/// Everything the set of records shown depends on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Shown {
    generation: u32,
    player: PlayerId,
    fogged: bool,
    /// The player's `PlayerVision::revision` and `PlayerVision::records_revision`; `None` for a player who isn't in
    /// the game.
    revisions: Option<(u32, u32)>,
}

/// The material of a record's visual: the vertex colours dimmed as the fog dims the Explored terrain under it
/// ([`EXPLORED_ALPHA`] of black over it, in linear colour). Placeholder: the design doesn't say how remembered
/// objects look.
fn dimmed_material() -> ColorMaterial {
    let kept = 1.0 - EXPLORED_ALPHA;
    ColorMaterial::from_color(Color::linear_rgb(kept, kept, kept))
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
    let vision = sim.vision().player(seen.player());
    let shown = Shown {
        generation: sim.generation(),
        player: seen.player(),
        fogged: seen.is_fogged(),
        revisions: vision.map(|v| (v.revision(), v.records_revision())),
    };
    if visuals.shown == Some(shown) {
        return;
    }
    let RememberedVisuals { by_id, made, .. } = &mut *visuals;
    let wanted: BTreeMap<ObjectId, &LastKnown> = seen.remembered().map(|r| (r.id, r)).collect();
    by_id.retain(|id, entity| {
        let keep = wanted
            .get(id)
            .is_some_and(|r| made.get(id) == Some(&(r.kind, r.placement)));
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
        let material = assets
            .dimmed
            .get_or_insert_with(|| materials.add(dimmed_material()))
            .clone();
        // Records are of structures and resources, which are drawn in no owner's colour.
        let mesh = assets
            .meshes
            .entry((record.kind, None))
            .or_insert_with(|| meshes.add(visual_mesh(record.kind, None)))
            .clone();
        let pos = record.pos();
        let height = object_height(sim.map(), pos, Some(&record.placement));
        let entity = commands
            .spawn((
                Name::new(format!("Remembered {:?} #{}", record.kind, id.0)),
                RememberedVisual(id),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                object_transform(pos, height, 0.0),
                Visibility::Inherited,
            ))
            .id();
        by_id.insert(id, entity);
        made.insert(id, (record.kind, record.placement));
    }
    visuals.shown = Some(shown);
}
