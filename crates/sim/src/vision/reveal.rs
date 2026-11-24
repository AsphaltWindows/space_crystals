//! Attackers revealed — `core/vision.md > Attacks from outside vision` (design change 0177, answering question Q0017
//! point 1): a unit or structure that attacks one of a player's objects while outside that player's vision becomes
//! Visible to the player for [`REVEAL_TICKS`] after each shot, so one that keeps firing stays Visible. Only the
//! attacker is revealed, not the area around it.
//!
//! - **Outside vision**: as the shot lands in phase 5e of tick `t`, none of the sources of the owner of the object
//!   it hits sees a tile under the attacker. That uses the vision of the end of tick `t − 1`, which every check of
//!   5e reads, and the attacker's box where it fires from. A reveal doesn't count as vision, so every shot from
//!   outside renews it.
//! - **The fog delay** doesn't count either: the two timers are independent (design change 0178). An attacker
//!   firing from a tile that is Visible only for the fog delay ([`super::linger`]) is revealed too. It is Visible
//!   anyway, but it would otherwise drop from view once the delay ran out, until its next shot, though it keeps
//!   firing.
//! - **Ticks**: the attacker is Visible at the end of ticks `t` to `t + REVEAL_TICKS − 1`. Phase 5f of tick `t +
//!   REVEAL_TICKS` ends the reveal, unless another shot renews it first.
//! - **Visible means visible**: while revealed, the attacker counts as seen through the one query every question
//!   "does the player see this object" asks ([`PlayerVision::object_box_state`]). So it shows and can be selected,
//!   ordered as a target, and picked by auto-targeting like any Visible unit (`orders::world`, `combat`, the
//!   command gate, the client's `Sight`). Its tiles keep their state.
//! - **Whose vision**: the reveal goes to the owner of the object hit. A defeated player who watches with their
//!   allies' vision (design change 0102, the client's `Sight::watching`) sees what those visions show, the attackers
//!   revealed to them included. No team shares vision in the sim.
//! - **State**: per player, each attacker revealed and the tick its reveal ends. History: hashed and serialized.
//! - **Every attacker**: whatever its range (artillery too, once it exists). Only units attack so far
//!   (`combat::step`), so no structure is ever revealed, and the memory's records of structures never change
//!   with a reveal.
//! - **Stealth**: TBD in the design (stealth and detection aren't designed), so nothing here.
//!
//! [`PlayerVision::object_box_state`]: super::PlayerVision::object_box_state

use super::Vision;
use crate::combat::object_box;
use crate::ids::PlayerId;
use crate::object::Object;
use crate::units::Tick;

/// `core/vision.md > Attacks from outside vision` (design change 0177): how many ticks an attacker that shot from
/// outside its victim's vision stays Visible to the victim's owner after each shot. "1.5 seconds (24 frames),
/// placeholder".
pub const REVEAL_TICKS: Tick = 24;

impl Vision {
    /// Phase 5e of `tick`: `attacker`'s shot landed on an object of `player`. If none of the player's sources sees
    /// the attacker's tiles, it is revealed to them until phase 5f of `tick + REVEAL_TICKS`. An attacker the player
    /// owns needs no reveal.
    pub(crate) fn reveal(&mut self, player: PlayerId, attacker: &Object, tick: Tick) {
        if attacker.owner == Some(player) {
            return;
        }
        let Some(vision) = self.players.iter_mut().find(|p| p.id == player) else {
            return;
        };
        let (lo, hi) = object_box(attacker);
        if !vision.sources_see_box(lo, hi) {
            vision.reveals.insert(attacker.id, tick + REVEAL_TICKS);
        }
    }
}
