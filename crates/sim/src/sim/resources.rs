//! The players' resources as the sim shows them (`factions/gdo/overview.md > GDOResources`,
//! `factions/colonists/overview.md > ColonistResources`, `crate::economy`): the banks are part of each
//! [`crate::Player`], power and the army caps follow from the objects at the end of the latest tick.

use super::Sim;
use crate::economy::{self, ArmyCap, PowerSummary};
use crate::ids::PlayerId;
use crate::player::Banks;

impl Sim {
    /// `player`'s power at the end of the latest tick (`factions/gdo/overview.md > Power`): what their structures
    /// generate and consume, and the speed ratio of their consumers. Nothing generated and nothing consumed for a
    /// player who owns no structure, or isn't in the game.
    pub fn power(&self, player: PlayerId) -> PowerSummary {
        economy::power::summary(self.objects.values(), player)
    }

    /// `player`'s army cap at the end of the latest tick (plan decision T20, `economy::army`): what their units and
    /// their producers' units in production take up of their faction's cap (design change 0143), and what is
    /// available (the GDO's Unit Control, the Colonists' Beacon Capacity, the Syndicate's Tunnel Space, the Cults' Unit
    /// Control by their Recruitment Halls, slice M5-C). The HUD's figure, and what a unit at the front of a queue is
    /// checked against when its production would start (`sim/producers/step.rs`), but for the Cults', which is for
    /// display only (`ArmyResource::reserves`). `None` for a player who isn't in the game. The units inside the Tunnel
    /// Network take up theirs as those on the map do (`crate::network`): they are alive, and use Tunnel Space
    /// (`factions/syndicate/tunnel_area_and_space.md > Tunnel Space`, design change 0265). Tunnel Space's providers and
    /// the cells it loses are structures, all on the map, and so are the Halls.
    pub fn army_cap(&self, player: PlayerId) -> Option<ArmyCap> {
        let faction = self.player(player)?.faction;
        // The units held off the map count as those on it (plan decision T30: a unit takes its cost "for as long as
        // the unit lives", inside the Tunnel Network too).
        let held = self.held.values().map(|held| &held.object);
        Some(economy::army::cap(
            self.objects.values().chain(held),
            &self.map,
            player,
            faction,
        ))
    }

    /// The GDO's Unit Control `player` uses at the end of the latest tick (`factions/gdo/overview.md > Unit Control`):
    /// the UnitControlCost of their GDO units and of their producers' GDO units in production (design change 0143: the
    /// HUD's "used" count "includes in-production units, not merely queued ones"), out of [`crate::UNIT_CONTROL_CAP`].
    /// A GDO player's [`Sim::army_cap`] `used`. Another faction's unit costs none of it (`economy::army`).
    pub fn unit_control(&self, player: PlayerId) -> u32 {
        let held = self.held.values().map(|held| &held.object);
        economy::army::unit_control_used(self.objects.values().chain(held), player)
    }

    /// `DebugCommand::SetResources`, checked: `player`'s banks become `banks`.
    pub(super) fn set_resources(&mut self, player: PlayerId, banks: Banks) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player) {
            p.set_banks(banks);
        }
    }
}
