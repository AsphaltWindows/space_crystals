//! Fog of war (`core/vision.md`): the local player's view of the map through its vision at the end of the latest
//! tick (`Sim::vision`). Display only: the sim computes every player's vision whatever the client shows.
//!
//! - **What shows** ([`Sight`]): with the fog on, the local player sees its own objects and everything on its
//!   Visible tiles in real time (`> Visible`); on Explored tiles the terrain, and the structures and resources it
//!   remembers in their last-known state, never the live objects there (`> Explored`: the sim's records,
//!   `PlayerVision::remembered`); on Unexplored tiles nothing (`> Unexplored`). The object visuals
//!   ([`hide_unseen_objects`], `object_render::remembered`), the combat effects, the path overlay, the command
//!   indicators, the InfoPanel and the selection (`selection::pick`) all ask it.
//! - **Spectating** (`core/game_rules.md > End of Game`, "Defeated before the end", design change 0102): a defeated
//!   player watches with their allies' vision, never a full map reveal. A sight looks through the union of the
//!   visions it watches ([`Sight::watching`]): a tile shows in the best state any of them gives it, and an object
//!   remembered by several shows as last seen. The sim has no teams yet, so a defeated player has no allies and
//!   watches their own vision alone, which has no Visible tile left: they own nothing.
//! - **The fog layer**: one mesh over the whole map and one texture of one texel per tile, never an entity or a
//!   material per tile (`ARCHITECTURE.md > Client architecture > Visuals`). The mesh follows the terrain's surface
//!   tile by tile as the terrain mesh draws it (a quad per flat tile, four triangles around the centre of a sloped
//!   one), in front of it in the terrain's z band ([`FOG_Z`]): objects stand on it. Each texel is black, with [`UNEXPLORED_ALPHA`] for an Unexplored tile, [`EXPLORED_ALPHA`] for an
//!   Explored one and none for a Visible one, sampled linearly so the edges soften across a tile
//!   ([`FOG_SOFT_EDGES`]).
//!   The texture is written only when what it shows changes: the tiles of the visions the local player watches
//!   (`Sight::revision`), the local player, or the sim (`SimRes::generation`); the mesh and the texture's size only
//!   with the map.
//! - **Off** ([`FogOfWar`]): `--no-fog` and the debug toggle (F3, a placeholder key, only while debug commands are
//!   on, or in a replay's playback) turn it off for QA: then the client shows and lets the player select
//!   everything, as if every tile were Visible. The map editor never shows fog; its play-tests do.
//!
//! Placeholders, the design being silent on how fog looks: the Explored dimming and the soft edges.

mod layer;
mod sight;
mod targets;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::Sim;

use crate::input::{ActionState, InputAction};
use crate::object_render::ObjectVisual;
use crate::replay::Playback;
use crate::sim_driver::{LocalPlayer, SimRes};
use crate::{ClientMode, ClientSystems};
use layer::{spawn_fog, update_fog_layer};

pub use layer::{
    EXPLORED_ALPHA, FOG_SOFT_EDGES, FOG_Z, FogAssets, FogLayer, FogView, UNEXPLORED_ALPHA, fog_alpha, fog_mesh,
    fog_texels,
};
pub use sight::{Revision, Sight, Watched};

/// Whether the fog of war hides what the local player doesn't see. Display only.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FogOfWar {
    /// On unless `--no-fog`; [`InputAction::DebugToggleFog`] switches it while debug commands are on.
    pub enabled: bool,
}

impl FogOfWar {
    /// Whether the fog applies in `mode`: in a game and in a play-test while it is enabled, never in the editor.
    pub fn applies(self, mode: ClientMode) -> bool {
        self.enabled && mode.is_game()
    }
}

/// The local player's [`Sight`], for systems.
#[derive(SystemParam)]
pub struct LocalSight<'w> {
    sim: Res<'w, SimRes>,
    player: Res<'w, LocalPlayer>,
    fog: Res<'w, FogOfWar>,
    mode: Res<'w, ClientMode>,
}

impl LocalSight<'_> {
    pub fn get(&self) -> Sight<'_> {
        Sight::new(&self.sim, self.player.0, self.fog.applies(*self.mode))
    }

    /// The sim it looks at.
    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    /// Whether what it shows may have changed since the system last ran: the sim, the local player, the fog or the
    /// mode changed.
    pub fn is_changed(&self) -> bool {
        self.sim.is_changed() || self.player.is_changed() || self.fog.is_changed() || self.mode.is_changed()
    }
}

pub struct FogPlugin {
    /// The fog is on from the start (no `--no-fog`).
    pub enabled: bool,
}

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FogOfWar { enabled: self.enabled })
            .init_resource::<FogView>()
            .add_systems(Startup, spawn_fog)
            .add_systems(Update, toggle_fog.in_set(ClientSystems::Actions))
            .add_systems(
                Update,
                (update_fog_layer, hide_unseen_objects).in_set(ClientSystems::Present),
            );
    }
}

/// [`InputAction::DebugToggleFog`]: only while the game's debug commands are on (`SimConfig::debug`), since it
/// shows what the player doesn't see; and in a replay's playback, which shows a finished game.
fn toggle_fog(actions: Res<ActionState>, sim: Res<SimRes>, playback: Option<Res<Playback>>, mut fog: ResMut<FogOfWar>) {
    if actions.just_pressed(InputAction::DebugToggleFog) && (sim.config().debug || playback.is_some()) {
        fog.enabled = !fog.enabled;
    }
}

/// Hides the visual of every object the local player doesn't see ([`Sight::sees`]): on Explored tiles too, where
/// the records show instead (`object_render::remembered`). Visuals of a headless app have no `Visibility` until this
/// gives them one.
fn hide_unseen_objects(
    mut commands: Commands,
    sight: LocalSight,
    mut visuals: Query<(Entity, &ObjectVisual, Option<&mut Visibility>)>,
) {
    let seen = sight.get();
    for (entity, visual, visibility) in &mut visuals {
        let shown = sight.sim.object(visual.0).is_some_and(|o| seen.sees(o));
        let wanted = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        match visibility {
            Some(mut visibility) => {
                visibility.set_if_neq(wanted);
            }
            None => {
                commands.entity(entity).insert(wanted);
            }
        }
    }
}
