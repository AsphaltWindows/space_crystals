//! Path knowledge — `systems/controls/behaviors.md > PathKnowledge` (design changes 0040, 0041 and 0042): paths are
//! planned only from what the unit's owner knows, so pathing never leaks hidden information.
//!
//! What a player knows to stand in the way, besides the terrain ([`known_blockers`]):
//!
//! 1. **Its own** structures (every blocking footprint it owns), as they are: "own structures are always known".
//! 2. **What it has seen** of the others': the footprint of every record of its vision's memory
//!    (`core/vision.md > Explored`, `vision::LastKnown`): a structure or resource it sees, or saw and hasn't seen
//!    gone. So a seen structure in the fog is a wall at its last-known position, destroyed or not, until the player
//!    sees the place without it; one it never saw is open ground.
//! 3. **The map's neutral objects** (crystal patches, Supply Delivery Stations, every neutral object the map maker
//!    placed), known from the start like terrain (0042; design change 0159: "pathing and memory"): each player has a
//!    record of every one of them from the start (`Vision::seed`), so part 2 takes them in. A record goes once the
//!    player sees the place without its object, so a patch mined out in the fog is routed around until a unit sees
//!    its tile. A player's starting layout is not among them, and nothing of it is known until seen.
//!
//!    [`PlayerKnowledge::assumed`] is what an older save, from before design change 0159, has instead: the map's
//!    resources whose place the player hadn't seen yet, without records. Each is taken on trust until a tile under it
//!    is Visible, and then the player's record says what it knows (the resource as it last saw it, or nothing if it
//!    was already gone). A game started now assumes nothing.
//!
//! **State.** [`PathKnowledge`] is history, like the Explored tiles: each player's assumed resources and its
//! [`PlayerKnowledge::version`], which counts the changes of what the player knows to stand in the way. Hashed and
//! serialized. The grids planned on are derived from it, the objects and the vision (`paths::NavState`), and
//! rebuilt on load.
//!
//! **Timing.** A player's knowledge is brought up to date once per tick, at its very end, after the vision
//! (`Sim::step`): the tick's plans (phase 5c) and the command phase before them use what the player knew at the end
//! of the tick before, as the behaviours and the combat use that tick's vision. So each player's grids are rebuilt
//! at most once per tick, and between ticks they follow from the state alone, which is what a loaded save rebuilds.
//!
//! Which map-placed objects count as "known from the start" is design change 0159's answer: the neutral ones. A
//! structure a Custom map pre-places for a slot belongs to whoever takes the slot, and is known to the others only
//! once seen, like any structure.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::blocking_footprint;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::nav::{NavRules, TileRect};
use crate::object::Object;
use crate::units::TilePos;
use crate::vision::{PlayerVision, VisibilityState, Vision};

/// A map resource the game started with, which a player of an older save takes to be where the map put it until it
/// sees the place (design change 0042; since 0159 the players' records do it, module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assumed {
    /// The resource. It may be gone by now: the player hasn't seen that.
    pub id: ObjectId,
    /// Its footprint when the game started.
    pub footprint: TileRect,
}

/// One player's path knowledge beyond its vision's records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerKnowledge {
    player: PlayerId,
    /// How often the set of footprints the player knows to stand in the way has changed (wrapping). A path notes the version it was
    /// planned on (`Mover::nav_version`); a different one sends the unit's staggered check along its remaining
    /// path (`docs/movement.md > 5.6`), which is how a unit re-routes around what it discovers.
    version: u32,
    /// The map's resources whose place the player hasn't seen yet, by ascending id: only an older save has any
    /// (module docs). Optional in files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    assumed: Vec<Assumed>,
}

impl PlayerKnowledge {
    pub fn player(&self) -> PlayerId {
        self.player
    }

    /// Changes whenever the footprints the player knows of change: one it owns, one it remembers, or a map
    /// resource it assumed.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The map's resources the player still takes on trust (design change 0042), by ascending id: an older save's
    /// (module docs).
    pub fn assumed(&self) -> &[Assumed] {
        &self.assumed
    }
}

/// Every player's path knowledge, in the sim's player order (`systems/controls/behaviors.md > PathKnowledge`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PathKnowledge {
    players: Vec<PlayerKnowledge>,
}

impl PathKnowledge {
    /// The knowledge a game starts with: version 0 for each of `players`, and nothing assumed. What each knows of
    /// the map's neutral objects is in its records from the start (design change 0159, module docs).
    pub(crate) fn start(players: impl IntoIterator<Item = PlayerId>) -> Self {
        let players = players
            .into_iter()
            .map(|player| PlayerKnowledge {
                player,
                version: 0,
                assumed: Vec::new(),
            })
            .collect();
        Self { players }
    }

    /// Each player's knowledge, in the sim's player order.
    pub fn players(&self) -> &[PlayerKnowledge] {
        &self.players
    }

    /// `player`'s knowledge; `None` for a player who isn't in the game.
    pub fn player(&self, player: PlayerId) -> Option<&PlayerKnowledge> {
        self.players.iter().find(|p| p.player == player)
    }

    /// The version of `player`'s knowledge ([`PlayerKnowledge::version`]).
    pub fn version(&self, player: PlayerId) -> Option<u32> {
        self.player(player).map(PlayerKnowledge::version)
    }

    /// The end of a tick, after the vision: a player who now sees a tile under a resource it assumed stops
    /// assuming it. Its vision's record says what is there (`Vision::remember` wrote it this tick if the resource
    /// still stands), and nothing if it is gone.
    pub(crate) fn observe(&mut self, vision: &Vision) {
        for known in &mut self.players {
            let Some(sight) = vision.player(known.player) else {
                continue;
            };
            known.assumed.retain(|a| !sees_rect(sight, a.footprint));
        }
    }

    /// What `player` knows to stand in the way changed.
    pub(crate) fn bump(&mut self, player: PlayerId) {
        if let Some(known) = self.players.iter_mut().find(|p| p.player == player) {
            known.version = known.version.wrapping_add(1);
        }
    }

    /// A save's knowledge fits the game: one entry for each of `players`, in order; each assumed resource once,
    /// by ascending id, on the map. Loading never panics on a bad save.
    pub(crate) fn check(&self, players: &[PlayerId], map: &Map) -> Result<(), String> {
        let saved: Vec<PlayerId> = self.players.iter().map(|p| p.player).collect();
        if saved != players {
            return Err(format!(
                "the path knowledge is saved for players {saved:?}, the game has {players:?}"
            ));
        }
        for known in &self.players {
            let player = known.player.0;
            if !known.assumed.windows(2).all(|pair| pair[0].id < pair[1].id) {
                return Err(format!(
                    "player {player}'s assumed resources aren't in ascending id order"
                ));
            }
            for a in &known.assumed {
                let r = a.footprint;
                let on_map = r.w > 0
                    && r.h > 0
                    && r.origin.x >= 0
                    && r.origin.y >= 0
                    && i64::from(r.origin.x) + i64::from(r.w) <= i64::from(map.width())
                    && i64::from(r.origin.y) + i64::from(r.h) <= i64::from(map.height());
                if !on_map {
                    return Err(format!(
                        "player {player}'s assumed resource #{} lies off the map",
                        a.id.0
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Whether a tile under `rect` is Visible to the player.
fn sees_rect(sight: &PlayerVision, rect: TileRect) -> bool {
    (0..rect.h as i32).any(|dy| {
        (0..rect.w as i32)
            .any(|dx| sight.state(TilePos::new(rect.origin.x + dx, rect.origin.y + dy)) == VisibilityState::Visible)
    })
}

/// The blocking footprints the players own, by owner, each in object id order: part 1 of what a player knows. One
/// walk over the objects serves every player.
pub(super) fn own_blockers(objects: &BTreeMap<ObjectId, Object>, rules: NavRules) -> Vec<(PlayerId, TileRect)> {
    let mut own: Vec<(PlayerId, TileRect)> = objects
        .values()
        .filter_map(|o| Some((o.owner?, blocking_footprint(o.kind, o.placement.as_ref()?, rules)?)))
        .collect();
    // Stable: within an owner the id order stays.
    own.sort_by_key(|&(owner, _)| owner);
    own
}

/// `systems/controls/behaviors.md > PathKnowledge`: the footprints `known`'s player knows to block the ground,
/// into `out`: its own (from `own`, [`own_blockers`]), those of its vision's records (the map's neutral objects among
/// them from the start, design change 0159), and the map resources an older save has it assume, each part in id
/// order. An unchanged list means unchanged knowledge; a changed one may still cover the same footprints (an assumed
/// resource seen for the first time moves from the assumed to the records).
pub(super) fn known_blockers(
    known: &PlayerKnowledge,
    own: &[(PlayerId, TileRect)],
    vision: &Vision,
    rules: NavRules,
    out: &mut Vec<TileRect>,
) {
    out.clear();
    let from = own.partition_point(|&(owner, _)| owner < known.player);
    out.extend(
        own[from..]
            .iter()
            .take_while(|&&(owner, _)| owner == known.player)
            .map(|&(_, rect)| rect),
    );
    if let Some(sight) = vision.player(known.player) {
        out.extend(
            sight
                .records()
                .filter_map(|record| blocking_footprint(record.kind, &record.placement, rules)),
        );
    }
    out.extend(known.assumed.iter().map(|a| a.footprint));
}

impl StateHash for PathKnowledge {
    fn hash(&self, h: &mut StateHasher) {
        let Self { players } = self;
        h.len(players.len());
        for known in players {
            let PlayerKnowledge {
                player,
                version,
                assumed,
            } = known;
            player.hash(h);
            version.hash(h);
            h.len(assumed.len());
            for a in assumed {
                let Assumed {
                    id,
                    footprint: TileRect { origin, w, h: height },
                } = a;
                id.hash(h);
                origin.hash(h);
                w.hash(h);
                height.hash(h);
            }
        }
    }
}
