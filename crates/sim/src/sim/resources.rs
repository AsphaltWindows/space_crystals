//! The players' resources as the sim shows them (`factions/gdo/overview.md > GDOResources`,
//! `factions/colonists/overview.md > ColonistResources`, `crate::economy`): the banks are part of each
//! [`crate::Player`], power and the army caps follow from the objects at the end of the latest tick.

use super::Sim;
use crate::economy::{self, ArmyCap, PowerSummary};
use crate::ids::PlayerId;
use crate::player::Banks;

impl Sim {
    /// `player`'s power at the end of the latest tick (`> Power`): what their structures generate and consume, and
    /// the speed ratio of their consumers. Nothing generated and nothing consumed for a player who owns no
    /// structure, or isn't in the game.
    pub fn power(&self, player: PlayerId) -> PowerSummary {
        economy::power::summary(self.objects.values(), player)
    }

    /// `player`'s army cap at the end of the latest tick (plan decision T20, `economy::army`): what their units and
    /// their producers' queues take up of their faction's cap, and what is available (the GDO's Unit Control, the
    /// Colonists' Beacon Capacity). The HUD's figure, and what the gate checks a Train against. `None` for a faction
    /// without one yet, or a player who isn't in the game.
    pub fn army_cap(&self, player: PlayerId) -> Option<ArmyCap> {
        let faction = self.player(player)?.faction;
        economy::army::cap(self.objects.values(), player, faction)
    }

    /// The Unit Control `player` uses at the end of the latest tick (`> Unit Control`): the UnitControlCost of their
    /// units and of the units queued at their producers (design change 0082: reserved when queued), out of
    /// [`economy::UNIT_CONTROL_CAP`]. A GDO player's [`Sim::army_cap`] `used`.
    pub fn unit_control(&self, player: PlayerId) -> u32 {
        economy::army::unit_control_used(self.objects.values(), player)
    }

    /// `DebugCommand::SetResources`, checked: `player`'s banks become `banks`.
    pub(super) fn set_resources(&mut self, player: PlayerId, banks: Banks) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player) {
            p.set_banks(banks);
        }
    }
}
