//! The faction seam: each faction's computer player is one module of this one, behind [`FactionAi`]
//! (`core/game_rules.md > Computer players (playtest tool)`: "Their behavior (profiles such as standard, rush and
//! passive; build orders) is the Director's placeholder"). A faction's module decides what its player builds and
//! trains, and which of its units fight; the shared layers do the rest: the View (what it knows), the turn (what the
//! bank still pays in the tick, and the gate's check of every command), the build layer (where a building goes) and the
//! army layer (the staging point, waves and defense).
//!
//! **Adding a faction's computer player** (slices A2a–c): write its module, `factions/<faction>.rs` (its own files
//! under `factions/<faction>/`), and set its [`Hook`] there, `HOOK`. Nothing else changes: [`hook`] already names
//! every faction's module, and the command line refuses a computer player for a faction whose `HOOK` is `None`.

mod colonists;
mod cults;
mod gdo;
mod syndicate;

use std::fmt;

use sc_sim::{Faction, Object};

use crate::army::Waves;
use crate::{Profile, Turn, View};

/// What makes a faction's computer player for the player whose View it is given (the game as it starts, or as it is
/// when the computer player takes over), playing `profile`.
pub type Hook = fn(&View<'_>, Profile) -> Box<dyn FactionAi>;

/// One faction's computer player (module docs). It must never panic, whatever the game's state: an assumption that
/// fails skips the action. It reads the game only through the View its turn gives it, and its every choice is
/// deterministic: no hash containers, no clock, ties broken by `ObjectId` (`ARCHITECTURE.md > Computer players`).
pub trait FactionAi: fmt::Debug + Send + Sync {
    /// Its commands for the next tick: what it builds and trains, given into `turn` (`Turn::give`, `Turn::buy`). The
    /// army layer then moves the units that fight.
    fn think(&mut self, turn: &mut Turn<'_, '_>);

    /// Whether `unit`, one of its player's own, fights: the army layer gathers it at the staging point, sends it in
    /// waves and defends with it.
    fn fights(&self, unit: &Object) -> bool;

    /// Whether `building`, one of its player's own, rallies the units it makes to the staging point.
    fn rallies(&self, building: &Object) -> bool;

    /// The waves its army sends: their sizes and its patience (`crate::tuning::army`'s by default).
    fn waves(&self) -> Waves {
        Waves::default()
    }
}

/// Each faction's computer player, from its module; `None` for a faction that has none yet.
pub fn hook(faction: Faction) -> Option<Hook> {
    match faction {
        Faction::Gdo => gdo::HOOK,
        Faction::Syndicate => syndicate::HOOK,
        Faction::Cults => cults::HOOK,
        Faction::Colonists => colonists::HOOK,
    }
}

/// Whether `faction` has a computer player.
pub fn has_ai(faction: Faction) -> bool {
    hook(faction).is_some()
}

/// The factions that have a computer player, in `Faction::ALL`'s order.
pub fn with_ai() -> Vec<Faction> {
    Faction::ALL.iter().copied().filter(|&f| has_ai(f)).collect()
}

/// A faction's name as the command line spells it (`--factions cults,gdo`): `gdo`, `syndicate`, `cults`, `colonists`.
pub fn name(faction: Faction) -> String {
    format!("{faction:?}").to_lowercase()
}
