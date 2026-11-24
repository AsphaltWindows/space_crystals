//! Defeat and the end of the game (slice M2-F2): `core/game_rules.md > Defeat Condition`, `> On Defeat` and `> End
//! of Game`, plan decision T8, design changes 0101 and 0102.
//!
//! - **Defeat** (T8): a player is defeated in the tick their owned-structure count drops to 0. Every structure
//!   counts, plates included (`> Defeat Condition`). A player who never owned a structure is never defeated, which
//!   keeps the M1 sandboxes, the tests and the goldens alive. The count is compared between ticks: the players who
//!   own a structure as a tick starts ([`structure_owners`], before its commands) against those who still own one
//!   at the end of its economy phase ([`step`], T5 step 5), after the debug removals of the commands and the deaths
//!   of phase 5e. So a structure that comes and goes within one tick (a debug Spawn and Remove in the same tick)
//!   never counted, and a player who loses their last structure and gains one in the same tick isn't defeated. The
//!   defeat is recorded in hashed state (`Player::defeated_at`) with a [`SimEvent::PlayerDefeated`], and the gate
//!   refuses every command from that player from then on (`Reject::Defeated`).
//! - **On defeat** (0101): the player's units become neutral ([`to_neutral`]).
//! - **Sides** ([`allied`]): the sim has no team or alliance data, so every player is their own side.
//! - **End of game** (0101): the defeat that leaves at most one side standing ends the game ([`Outcome`]).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::Category;
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::Orders;
use crate::player::Player;
use crate::units::Tick;

/// How the game ended (`core/game_rules.md > End of Game`, design change 0101): recorded in the sim's hashed state
/// in the tick it ends (`Sim::outcome`), and final. From then on the gate refuses every command
/// (`Reject::GameOver`), and the client stops the game, shows each player Victory or Defeat and saves the replay.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    /// The tick in which the game ended: that of the defeat that left at most one side standing.
    pub tick: Tick,
    /// The players of the side left standing, in id order: they see Victory, everyone else Defeat. A side wins as
    /// one ("it wins"), so a team's players defeated before the end win with it. Empty when the last sides were
    /// defeated in the same tick: nobody wins, a placeholder reading (the design doesn't say what a draw is).
    pub winners: Vec<PlayerId>,
}

impl Outcome {
    /// Whether `player` won: Victory for them, Defeat otherwise.
    pub fn is_winner(&self, player: PlayerId) -> bool {
        self.winners.contains(&player)
    }
}

impl StateHash for Outcome {
    fn hash(&self, h: &mut StateHasher) {
        let Outcome { tick, winners } = self;
        tick.hash(h);
        winners.hash(h);
    }
}

/// Whether players `a` and `b` play on one side (`core/game_rules.md > End of Game`: "one player or team"; design
/// change 0102: "their allies"). The sim has no team or alliance data: fixed teams are deferred with custom maps
/// (`core/map_editor.md > Map Types > Custom`). So **every player is their own side**, with no allies. This is the
/// one place to change when teams come; the end of the game ([`step`]), `Sim::allies` and the client's spectator
/// fog follow it.
pub fn allied(a: PlayerId, b: PlayerId) -> bool {
    a == b
}

/// The players still in the game (not defeated) who own at least one structure, in id order: what [`step`]
/// compares with at the end of the tick. Taken as a tick starts, before its commands. Nobody, once the game is over:
/// no one is defeated after the end.
pub(crate) fn structure_owners(
    objects: &BTreeMap<ObjectId, Object>,
    players: &[Player],
    outcome: Option<&Outcome>,
) -> Vec<PlayerId> {
    if outcome.is_some() {
        return Vec::new();
    }
    let mut owners = owners_now(objects);
    owners.retain(|&id| players.iter().any(|p| p.id == id && !p.is_defeated()));
    owners
}

/// Every player who owns at least one structure, in id order.
fn owners_now(objects: &BTreeMap<ObjectId, Object>) -> Vec<PlayerId> {
    let mut owners: Vec<PlayerId> = objects
        .values()
        .filter(|o| o.def().category == Category::Structure)
        .filter_map(|o| o.owner)
        .collect();
    owners.sort_unstable();
    owners.dedup();
    owners
}

/// The defeat check, the last step of the economy phase (plan decisions T5 and T8): each player of `owned_before`
/// (who owned a structure as the tick started, [`structure_owners`]) who owns none now is defeated in this tick,
/// in id order: `Player::defeated_at`, a [`SimEvent::PlayerDefeated`], and their units become neutral
/// ([`to_neutral`]). Then, if at most one side is left with a player who isn't defeated, the game ends in this tick
/// ([`Outcome`], with a [`SimEvent::GameOver`] after the defeats). Only a tick with a defeat can end the game, so a
/// game that starts with a single side (a sandbox with one player) goes on.
///
/// Before phase 5f, so that the transferred units' vision is gone in the tick of the defeat; before the path
/// knowledge, so that the grids of a player left without walkers go in the same tick (`paths::sync_known`).
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    players: &mut [Player],
    outcome: &mut Option<Outcome>,
    owned_before: &[PlayerId],
    tick: Tick,
    events: &mut Vec<SimEvent>,
) {
    if owned_before.is_empty() || outcome.is_some() {
        return;
    }
    let owned_now = owners_now(objects);
    let defeated: Vec<PlayerId> = owned_before
        .iter()
        .copied()
        .filter(|p| owned_now.binary_search(p).is_err())
        .collect();
    if defeated.is_empty() {
        return;
    }
    for &player in &defeated {
        if let Some(p) = players.iter_mut().find(|p| p.id == player) {
            p.defeated_at = Some(tick);
        }
        events.push(SimEvent::PlayerDefeated { player });
    }
    to_neutral(objects, &defeated, tick);
    if let Some(end) = game_end(players, tick) {
        events.push(SimEvent::GameOver {
            winners: end.winners.clone(),
        });
        *outcome = Some(end);
    }
}

/// `core/game_rules.md > On Defeat` (design change 0101): the objects of the `defeated` players stay on the map and
/// belong to no one from now on. By T8 they are units only: a defeated player owns no structure. So no producer is
/// ever handed over with its queue (the note from M2-D2 in plan §9: nothing to decide), and a unit leaving the
/// doorway of its producer was let go when the building was removed.
///
/// Each unit:
/// - loses its orders, current and queued, and whatever it was doing by itself ([`Orders::default`]), and stands
///   where it is: "their orders are cleared and they stay idle where they are". A raised Riot Shield stays raised
///   (it is a mode, not an order);
/// - disengages: an attack being aimed is cancelled, while one already being fired, and its cooldown, run on
///   (`systems/combat.md > AttackPhase`: not interruptible);
/// - plans its path again (`Mover::repath`; the note from M2-E in plan §9): what it had was planned on its old
///   owner's knowledge grids, and a neutral unit plans on the true grids. A unit standing still has no goal to plan
///   for, so today this asks for nothing; it stays for whatever may keep a transferred unit moving.
///
/// The rest follows from having no owner: from phase 5f of this tick on the units are no one's eyes, so their vision
/// counts for no one; they are never auto-targeted and never acquire a target themselves (`systems/combat.md >
/// ValidTarget`: auto-targeting never picks neutral objects); any player can destroy them with an explicit Attack
/// order; and they block movement and never step aside (only own idle units yield).
fn to_neutral(objects: &mut BTreeMap<ObjectId, Object>, defeated: &[PlayerId], tick: Tick) {
    for o in objects.values_mut() {
        if !o.owner.is_some_and(|owner| defeated.contains(&owner)) {
            continue;
        }
        debug_assert!(
            o.def().category != Category::Structure,
            "a defeated player owns no structure (plan decision T8)"
        );
        o.owner = None;
        if let Some(orders) = &mut o.orders {
            *orders = Orders::default();
        }
        if let Some(attack) = &mut o.attack {
            attack.engage(None);
        }
        if let Some(mover) = &mut o.mover {
            mover.stop();
            mover.repath(tick);
        }
    }
}

/// `core/game_rules.md > End of Game` (design change 0101): the game ends once at most one side ([`allied`]) has a
/// player who isn't defeated. The winners are that side's players, all of them; nobody if no side is left.
fn game_end(players: &[Player], tick: Tick) -> Option<Outcome> {
    // One player of each side still standing.
    let mut standing: Vec<PlayerId> = Vec::new();
    for p in players.iter().filter(|p| !p.is_defeated()) {
        if !standing.iter().any(|&side| allied(side, p.id)) {
            standing.push(p.id);
        }
    }
    if standing.len() > 1 {
        return None;
    }
    let mut winners: Vec<PlayerId> = players
        .iter()
        .map(|p| p.id)
        .filter(|&id| standing.first().is_some_and(|&side| allied(side, id)))
        .collect();
    winners.sort_unstable();
    Some(Outcome { tick, winners })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Faction;

    fn players(defeated: &[u8]) -> Vec<Player> {
        (1..=3)
            .map(|i| Player {
                defeated_at: defeated.contains(&i).then_some(5),
                ..Player::starting(PlayerId(i), format!("P{i}"), Faction::Gdo)
            })
            .collect()
    }

    /// Every player is their own side (no teams): the game goes on while two players stand, and the last one wins;
    /// with nobody left, nobody wins.
    #[test]
    fn the_last_player_standing_wins() {
        assert_eq!(game_end(&players(&[]), 9), None);
        assert_eq!(game_end(&players(&[2]), 9), None);
        let end = game_end(&players(&[2, 3]), 9).unwrap();
        assert_eq!(end.winners, [PlayerId(1)]);
        assert_eq!(end.tick, 9);
        assert!(end.is_winner(PlayerId(1)) && !end.is_winner(PlayerId(2)));
        assert_eq!(game_end(&players(&[1, 2, 3]), 9).unwrap().winners, []);
        assert!(allied(PlayerId(1), PlayerId(1)) && !allied(PlayerId(1), PlayerId(2)));
    }
}
