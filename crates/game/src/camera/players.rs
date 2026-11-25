//! The players' cameras of the debug player switch (F2, `crate::selection`'s `switch_player`; slice M3-J3: M3-K2's
//! walkthrough found that F2 kept the camera where the player before had left it, so the other side saw black): each
//! player has a camera position of their own, which starts where `core/camera.md > Starting Position` puts it, at
//! their start ([`super::start_view`]), and which the camera comes back to when the switch comes back to them.
//! In a replay's playback the switch changes whose view the replay shows, and so whose camera too.

use std::collections::BTreeMap;

use bevy::prelude::*;
use sc_sim::PlayerId;

use super::{CameraView, start_view};
use crate::sim_driver::{LoadedScenario, LocalPlayer, SimRes};

/// Where each player's camera was when the local client switched away from them, and whom the camera shows now, in
/// which sim ([`SimRes::generation`]).
#[derive(Resource, Default, Debug)]
pub struct PlayerCameras {
    left: BTreeMap<PlayerId, Vec2>,
    shown: Option<(PlayerId, u32)>,
}

impl PlayerCameras {
    /// The view center the camera had when the client switched away from `player`, in this sim; `None` if it never
    /// did.
    pub fn left(&self, player: PlayerId) -> Option<Vec2> {
        self.left.get(&player).copied()
    }
}

/// When the local player changes within a sim (the debug switch): keeps the view of the player before, and moves the
/// camera to where the new one's was, or to their start the first time. A new sim (a play-test starting or ending, a
/// scenario loaded) forgets every player's, and leaves the camera where whatever replaced the sim put it.
pub(super) fn switch_cameras(
    sim: Res<SimRes>,
    loaded: Res<LoadedScenario>,
    player: Res<LocalPlayer>,
    mut cameras: ResMut<PlayerCameras>,
    mut view: ResMut<CameraView>,
) {
    let now = (player.0, sim.generation());
    if cameras.shown == Some(now) {
        return;
    }
    match cameras.shown {
        Some((before, generation)) if generation == now.1 => {
            let center = view.center;
            cameras.left.insert(before, center);
            view.center = (cameras.left(player.0)).unwrap_or_else(|| start_view(&loaded.0, player.0, sim.map()));
        }
        _ => cameras.left.clear(),
    }
    cameras.shown = Some(now);
}
