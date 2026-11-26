//! One tick's thinking (`core/game_rules.md > Computer players (playtest tool)`: a computer player "issues the same
//! commands a player can, under the same rules"): what the computer player sees, what its bank still pays in the tick,
//! the footprints it has placed in it, and the commands it gives.
//!
//! Every command goes through the gate's check before it is kept ([`Turn::give`]), as the client checks a player's
//! before it sends them. The check sees the game as the latest tick left it, so what the tick's earlier commands
//! change is the turn's to count: the Space Crystals they pay ([`Turn::buy`]) and the footprints they place
//! ([`Turn::take`]). Mining and deliveries pay only after the commands, so nothing else draws on the bank in a tick.

use sc_sim::nav::TileRect;
use sc_sim::{Command, PlayerCommand, Vec2};

use crate::{Profile, View};

/// One tick's thinking (module docs).
pub struct Turn<'v, 'a> {
    view: &'v View<'a>,
    profile: Profile,
    staging: Option<Vec2>,
    /// The Space Crystals the bank still holds for the tick's commands.
    crystals: u32,
    taken: Vec<TileRect>,
    commands: Vec<PlayerCommand>,
}

impl<'v, 'a> Turn<'v, 'a> {
    /// A turn of `view`'s player, playing `profile`, with its army gathering at `staging` (if it has a place to).
    pub fn new(view: &'v View<'a>, profile: Profile, staging: Option<Vec2>) -> Self {
        Self {
            view,
            profile,
            staging,
            crystals: view.me().map_or(0, |p| p.crystals),
            taken: Vec::new(),
            commands: Vec::new(),
        }
    }

    /// What the player knows of the game.
    pub fn view(&self) -> &'v View<'a> {
        self.view
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }

    /// Where the army gathers, and the units the player's buildings make rally to.
    pub fn staging(&self) -> Option<Vec2> {
        self.staging
    }

    /// The Space Crystals the bank still holds for the tick's commands: the bank less what the turn has bought.
    pub fn crystals(&self) -> u32 {
        self.crystals
    }

    /// Whether what is left of the bank covers `cost`.
    pub fn can_pay(&self, cost: u32) -> bool {
        cost <= self.crystals
    }

    /// Gives `command` if the gate takes it now (`View::validate`); returns whether it did.
    pub fn give(&mut self, command: Command) -> bool {
        if self.view.validate(&command).is_err() {
            return false;
        }
        self.commands.push(PlayerCommand {
            player: self.view.player(),
            command,
        });
        true
    }

    /// Gives `command`, which pays `cost` Space Crystals, if what is left of the bank covers it and the gate takes it;
    /// it then pays from the bank. Returns whether it did.
    pub fn buy(&mut self, cost: u32, command: Command) -> bool {
        let Some(left) = self.crystals.checked_sub(cost) else {
            return false;
        };
        if !self.give(command) {
            return false;
        }
        self.crystals = left;
        true
    }

    /// Notes a footprint that a Place of the tick takes, so that no later Place of the tick goes on it.
    pub fn take(&mut self, rect: TileRect) {
        self.taken.push(rect);
    }

    /// The footprints the tick's Places take so far.
    pub fn taken(&self) -> &[TileRect] {
        &self.taken
    }

    /// The commands given so far, in order.
    pub fn commands(&self) -> &[PlayerCommand] {
        &self.commands
    }

    /// The tick's commands, in the order given.
    pub fn into_commands(self) -> Vec<PlayerCommand> {
        self.commands
    }
}
