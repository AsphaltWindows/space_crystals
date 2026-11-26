//! What the local player sees while placing a building (`factions/gdo/structures/deployment_center.md >
//! ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`, `factions/gdo/economy.md > ExtractionFacility >
//! ObjectInterfaceState[ExtractionFacility] > AwaitingPlacement`, `factions/colonists/units/drone.md > Drone
//! ObjectInterfaceState > A: Deploy`, `factions/colonists/economy.md > Warp In > Warp In Command`; plan decisions T10,
//! T23), and the buildings ordered but not started. The state itself, its keys and its clicks are the command flow's
//! (`crate::command_ui::InterfaceState::AwaitingPlacement`); this module only shows it:
//! - [`ghost`]: "A ghost preview of the building follows the cursor, snapped to the grid", green or red tile by tile
//!   as the check its builder faces has it now (design change 0093; the Drones' as far as the player knows, design
//!   change 0120), with the letters of its sides (design change 0079). It is the cursor over the map: no pointer
//!   shows there (`crate::command_ui::cursor`). A Warp In's ghost stands on its Beacon instead, whatever the cursor
//!   does (design change 0121), and the pointer shows;
//! - [`overlay`]: "The player's current GDO build area is highlighted as a semi-transparent overlay on the ground",
//!   the area only (design change 0093), redrawn only when the area or the map changes; while a GDO constructor
//!   places its building;
//! - [`ghost_area`]: the Tier 1 Tunnel Area of a Tunnel ghost, green or red as the check finds it (slice M4-J1,
//!   design change 0242), within the rings of its later tiers ([`tunnel_areas`]);
//! - [`lingering`]: the lingering build ghosts, the sites of the player's pending Deploys and Builds, a Tunnel's with
//!   its Tier 1 area (`core/entities.md > Placement Validation > Lingering build ghosts`, design change 0125),
//!   whatever the player does;
//! - [`tunnel_areas`]: the Tunnel Area display (slices M4-J2 and M4-J3; `factions/syndicate/tunnel_area_and_space.md >
//!   Tunnel Area display`, design changes 0299 and 0300): the areas of the player's own Tunnels they select, and while
//!   they place a Tunnel the Tier 2 and Tier 3 rings of its ghost, red where they would block an upgrade; never an
//!   enemy's area.
//!
//! The ghost and the overlay show only while the local player places a building ([`placing_now`]): in a game mode,
//! in AwaitingPlacement, with a selection they may command (never once the game is over or they are defeated:
//! `core/game_rules.md > End of Game`, design change 0101).

pub mod ghost;
pub mod ghost_area;
pub mod lingering;
pub mod overlay;
pub mod tunnel_areas;

use bevy::prelude::*;
use sc_sim::{PlayerId, Sim};

pub use ghost::{Ghost, PlacementGhost};
pub use ghost_area::GhostArea;
pub use lingering::LingeringGhosts;
pub use overlay::BuildAreaOverlay;
pub use tunnel_areas::TunnelAreaOverlay;

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
            .init_resource::<GhostArea>()
            .init_resource::<BuildAreaOverlay>()
            .init_resource::<LingeringGhosts>()
            .init_resource::<TunnelAreaOverlay>()
            .add_systems(
                Startup,
                (
                    ghost::spawn_ghost,
                    ghost_area::spawn_ghost_area,
                    overlay::spawn_overlay,
                    lingering::spawn_lingering,
                    tunnel_areas::spawn_overlay,
                ),
            )
            .add_systems(
                Update,
                (
                    (ghost::update_ghost, ghost::draw_ghost, ghost_area::draw_ghost_area).chain(),
                    overlay::draw_overlay,
                    lingering::draw_lingering,
                    tunnel_areas::draw_overlay.after(ghost::update_ghost),
                )
                    .in_set(ClientSystems::Present),
            );
    }
}
