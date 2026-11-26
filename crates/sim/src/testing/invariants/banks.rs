//! (M5-M1) The bank invariant B1, the M5 plan's "nor takes a bank below 0" (slice M): no player's bank goes below 0,
//! nor past its top. The banks are every resource a player stores: Space Crystals, Supplies, and the Colonists' Alloys,
//! Essence and Flux (`Player::banks`, `ARCHITECTURE.md > Economy`). Power is no bank: it is derived from the objects,
//! and what is used may exceed what is produced by design (`Sim::power`).
//!
//! A bank is a `u32`, so it can't hold a number below 0, and every debit is checked (`Player::spend_crystals`,
//! `Player::spend_supplies`: "a spend is refused, never taken below 0", plan decision T7; `sim/repairs.rs` pays what the
//! bank holds at most), as every credit saturates. A debit that isn't would go below 0 by an unchecked subtraction: in
//! a build with overflow checks (the dev and test profiles) it panics where it is made, and in one without (release) it
//! wraps to near `u32::MAX`, as a credit past the top wraps to near 0. B1 catches the wrap: a bank that changes by half
//! its range or more in one tick, which no game comes near (a tick's income and refunds are thousands at most), unless a
//! `DebugCommand::SetResources` of that tick set the player's banks.

use glam::Vec2;

use super::Violation;
use crate::command::{Command, DebugCommand, TickInputs};
use crate::ids::PlayerId;
use crate::player::Banks;
use crate::sim::Sim;

/// B1: the change of a bank in one tick from which on it wrapped (module docs): half the range of a `u32`.
const WRAPPED: u32 = 1 << 31;

/// Each player's banks as the last tick left them, for B1's check of the next one.
#[derive(Clone, Debug)]
pub(super) struct Books(Vec<(PlayerId, Banks)>);

impl Books {
    /// The banks of `sim`'s players now.
    pub(super) fn of(sim: &Sim) -> Self {
        Self(sim.players().iter().map(|p| (p.id, p.banks())).collect())
    }

    /// B1: no bank of `sim`'s players wrapped in the tick that applied `inputs`, since these books (module docs).
    pub(super) fn check(&self, sim: &Sim, inputs: &TickInputs) -> Result<(), Violation> {
        let set = |player: PlayerId| {
            (inputs.commands.iter()).any(
                |c| matches!(c.command, Command::Debug(DebugCommand::SetResources { player: p, .. }) if p == player),
            )
        };
        for p in sim.players() {
            let Some(&(_, before)) = self.0.iter().find(|(id, _)| *id == p.id) else {
                continue;
            };
            if set(p.id) {
                continue;
            }
            for (name, before, after) in pairs(before, p.banks()) {
                if before.abs_diff(after) >= WRAPPED {
                    return Err(Violation {
                        invariant: "B1",
                        message: format!(
                            "player {}'s {name} went from {before} to {after} in one tick: the bank wrapped, below 0 \
                             or past its top",
                            p.id.0
                        ),
                        at: Vec2::ZERO,
                    });
                }
            }
        }
        Ok(())
    }
}

/// Each bank by name, as it was and as it is (every field of [`Banks`]: a new bank is checked as soon as it is one).
fn pairs(before: Banks, after: Banks) -> [(&'static str, u32, u32); 5] {
    let Banks {
        crystals,
        supplies,
        alloys,
        essence,
        flux,
    } = before;
    [
        ("Space Crystals", crystals, after.crystals),
        ("Supplies", supplies, after.supplies),
        ("Alloys", alloys, after.alloys),
        ("Essence", essence, after.essence),
        ("Flux", flux, after.flux),
    ]
}
