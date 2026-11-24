//! What the local player sees while placing a building ready to place (`factions/gdo/structures.md >
//! DeploymentCenter > ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`, `factions/gdo/economy.md >
//! ExtractionFacility > ObjectInterfaceState[ExtractionFacility] > AwaitingPlacement`; plan decision T10). The state
//! itself, its keys and its clicks are the command flow's (`crate::command_ui::InterfaceState::AwaitingPlacement`);
//! this module only shows it:
//! - [`ghost`]: "A ghost preview of the building follows the cursor, snapped to the grid", green or red tile by tile
//!   as the placement check has it now (design change 0093), with the letters of its sides (design change 0079). It
//!   is the cursor over the map: no pointer shows there (`crate::command_ui::cursor`);
//! - [`overlay`]: "The player's current GDO build area is highlighted as a semi-transparent overlay on the ground",
//!   the area only (design change 0093), redrawn only when the area or the map changes.
//!
//! Both show only while the local player places a building ([`placing_now`]): in a game mode, in AwaitingPlacement,
//! with a selection they may command (never once the game is over or they are defeated: `core/game_rules.md > End
//! of Game`, design change 0101).

pub mod ghost;
pub mod overlay;

use bevy::prelude::*;
use sc_sim::{PlayerId, Sim};

pub use ghost::{Ghost, PlacementGhost};
pub use overlay::BuildAreaOverlay;

use crate::command_ui::Placing;
use crate::command_ui::interface::commandable;
use crate::selection::ControlStates;
use crate::{ClientMode, ClientSystems};

/// The placement `player` makes now, as the client shows it: AwaitingPlacement in a game mode, while their selection
/// is commandable.
pub fn placing_now(mode: ClientMode, sim: &Sim, player: PlayerId, states: &ControlStates) -> Option<Placing> {
    let state = states.of(player);
    let placing = state.interface.placing()?;
    (mode.is_game() && commandable(sim, player, &state.selection)).then_some(placing)
}

pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlacementGhost>()
            .init_resource::<BuildAreaOverlay>()
            .add_systems(Startup, (ghost::spawn_ghost, overlay::spawn_overlay))
            .add_systems(
                Update,
                ((ghost::update_ghost, ghost::draw_ghost).chain(), overlay::draw_overlay)
                    .in_set(ClientSystems::Present),
            );
    }
}
