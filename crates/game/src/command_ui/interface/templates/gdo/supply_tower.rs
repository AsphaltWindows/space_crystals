//! The Supply Tower's panel (`factions/gdo/economy.md > SupplyTower > ObjectInterfaceState[SupplyTower]`): the
//! Production template (plan decision T23, `super::super::production_grid`), Q its recipe, the Supply Chopper's, for
//! 100 Space Crystals, its right-click the rally point, and S, which the grid adds for a Supply Tower
//! ([`super::super::grid`], `sc_sim::supply_run::tower_def`): Schedule Deliveries (`PanelCommand::ScheduleDeliveries`)
//! awaits a Supply Delivery Station, where a left-click sets the schedule of every Tower of the ActiveGroup
//! (`sc_sim::Command::Schedule`; `factions/gdo/economy.md > SupplyTower > AwaitingTarget[ScheduleDeliveries]
//! resolution`; anything else is an invalid target), the pointer GatherResources over it (`systems/controls/hud.md >
//! Resolution Rules > AwaitingTarget[CommandType]`). S is greyed while none of its Towers has a chopper attached
//! (`sc_sim::Reject::NoChopperAttached`). While a Tower is selected, the station it is scheduled on shows, the Gather
//! marker on it and a line from the Tower to it (`crate::command_ui::indicators::schedule`), as a rally point does.

use super::super::{InterfaceTemplate, PanelTemplate};

/// The Supply Tower's panel template: the Production template, S added by the grid.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Production);

/// S: Schedule Deliveries, the index of its slot in `CommandSlot::all` order.
pub const SCHEDULE_SLOT: usize = 5;
