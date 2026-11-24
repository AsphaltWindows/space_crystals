//! The players' resources as the sim shows them (`factions/gdo/overview.md > GDOResources`, `crate::economy`): the
//! banks are part of each [`crate::Player`], power and Unit Control follow from the objects at the end of the latest
//! tick.

use super::Sim;
use crate::economy::{self, PowerSummary};
use crate::ids::PlayerId;

impl Sim {
    /// `player`'s power at the end of the latest tick (`> Power`): what their structures generate and consume, and
    /// the speed ratio of their consumers. Nothing generated and nothing consumed for a player who owns no
    /// structure, or isn't in the game.
    pub fn power(&self, player: PlayerId) -> PowerSummary {
        economy::power::summary(self.objects.values(), player)
    }

    /// The Unit Control `player` uses at the end of the latest tick (`> Unit Control`): the UnitControlCost of their
    /// units, out of [`economy::UNIT_CONTROL_CAP`].
    pub fn unit_control(&self, player: PlayerId) -> u32 {
        economy::unit_control::used(self.objects.values(), player)
    }

    /// `DebugCommand::SetResources`, checked: `player`'s banks become `crystals` and `supplies`.
    pub(super) fn set_resources(&mut self, player: PlayerId, crystals: u32, supplies: u32) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player) {
            p.crystals = crystals;
            p.supplies = supplies;
        }
    }
}
