//! Selection (`systems/controls/selection_and_hud.md`): client-side UI state, never part of the sim.
//!
//! Each player has a ControlState ([`ControlStates`]): a selection grouped by type with an ActiveGroup, 10 control
//! groups and the ObjectInterfaceState of the command flow (`> ControlState`, `> Selection`, `> ControlGroups`;
//! the command flow is [`crate::command_ui`]). The local player's is the one in use; the
//! debug player switch ([`crate::input::InputAction::DebugSwitchPlayer`]) changes the local player, and with it
//! whose state that is and what counts as "own".
//!
//! - [`model`]: the selection, its groups and the control groups, as plain data;
//! - [`pick`]: what the local player can select and where it is drawn: click picking, the box's priority tiers;
//! - [`rules`]: what clicks, boxes, portrait clicks, control group keys and Tab do, under the Constraints;
//! - [`markers`]: the selection markers under selected objects and the box being dragged;
//! - the systems (`systems.rs`): the input layer's clicks, boxes and actions become those rules, and removed
//!   objects leave every selection and control group at once.
//!
//! The selection-related HUD (InfoPanel, SelectionPanel) lives in [`crate::hud`].

pub mod markers;
pub mod model;
pub mod pick;
pub mod rules;
mod systems;

use std::collections::BTreeMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{PlayerId, Sim};

pub use model::{ControlGroups, ControlState, GroupKey, GroupType, MAX_SELECTED, Selection, SelectionGroup};
pub use pick::{PICK_SLOP_SU, Relation, Selectable, Selectables};
pub use rules::ClickRoles;

use crate::command_ui::InterfaceState;
use crate::fog::{FogOfWar, Sight};
use crate::sim_driver::{LocalPlayer, RenderAlpha, SimRes};
use crate::{ClientMode, ClientSystems};

/// Every player's ControlState (`> ControlState`: per player). The local player's is the one the input changes
/// and the HUD shows ([`ControlStates::of`] with `LocalPlayer`).
#[derive(Resource, Debug, Default)]
pub struct ControlStates {
    by_player: BTreeMap<PlayerId, ControlState>,
    /// The `SimRes::generation` the states belong to: a replaced sim starts them over, since its ids name other
    /// objects.
    generation: Option<u32>,
}

/// The state of a player who hasn't selected anything yet.
static NO_STATE: ControlState = ControlState::EMPTY;

impl ControlStates {
    /// `player`'s ControlState.
    pub fn of(&self, player: PlayerId) -> &ControlState {
        self.by_player.get(&player).unwrap_or(&NO_STATE)
    }

    /// `player`'s selection.
    pub fn selection(&self, player: PlayerId) -> &Selection {
        &self.of(player).selection
    }

    /// `player`'s ControlState, to change it.
    pub fn of_mut(&mut self, player: PlayerId) -> &mut ControlState {
        self.by_player.entry(player).or_default()
    }
}

/// The local player, their ControlState, and what they can select this frame: what the selection's input systems
/// and the SelectionPanel's clicks work with.
#[derive(SystemParam)]
pub struct LocalControl<'w> {
    sim: Res<'w, SimRes>,
    alpha: Res<'w, RenderAlpha>,
    player: Res<'w, LocalPlayer>,
    states: ResMut<'w, ControlStates>,
    fog: Res<'w, FogOfWar>,
    mode: Res<'w, ClientMode>,
}

impl LocalControl<'_> {
    /// The local player.
    pub fn player(&self) -> PlayerId {
        self.player.0
    }

    /// The sim, as this frame shows it.
    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    /// The local player's ControlState.
    pub fn state(&self) -> &ControlState {
        self.states.of(self.player.0)
    }

    /// What the local player can select, as this frame draws it.
    pub fn objects(&self) -> Selectables {
        let sight = Sight::new(&self.sim, self.player.0, self.fog.applies(*self.mode));
        Selectables::new(&self.sim, &sight, self.alpha.0)
    }

    /// Applies `change` to the local player's ControlState. A change of the Selection or of its ActiveGroup resets
    /// the ObjectInterfaceState to DefaultState (`> ObjectInterfaceState[ObjectEnum]`): the player changed them, which
    /// cancels targeting (design change 0069; the game's removals go through `ControlState::retain`, which resets it
    /// only with the whole ActiveGroup gone). A control group's recall resets it even when the selection stays the
    /// same (`ControlState::recall_group`, design change 0078). [`ControlStates`] is marked changed only when the
    /// state did change, so that the HUD and the markers look again only then.
    pub fn update<R>(&mut self, change: impl FnOnce(&mut ControlState) -> R) -> R {
        let player = self.player.0;
        let before = self.states.of(player);
        let mut state = before.clone();
        let result = change(&mut state);
        if state.selection != before.selection {
            state.interface = InterfaceState::DefaultState;
        }
        if state != *before {
            *self.states.of_mut(player) = state;
        }
        result
    }
}

/// Order of the selection's `Update` systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SelectionSystems {
    /// Catch up with the sim: removed objects leave the selections and groups (in `ClientSystems::SyncSim`).
    Sync,
    /// The frame's input changes the local player's ControlState (in `ClientSystems::Actions`).
    Input,
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlStates>()
            .configure_sets(Update, SelectionSystems::Sync.in_set(ClientSystems::SyncSim))
            .configure_sets(Update, SelectionSystems::Input.in_set(ClientSystems::Actions))
            .add_systems(Update, systems::sync_control_states.in_set(SelectionSystems::Sync))
            .add_systems(
                Update,
                (
                    systems::switch_player,
                    systems::select_on_map,
                    systems::control_group_keys,
                )
                    .chain()
                    .in_set(SelectionSystems::Input),
            )
            .add_plugins(markers::SelectionMarkersPlugin);
    }
}
