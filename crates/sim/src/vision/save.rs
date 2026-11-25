//! The serde form of [`Vision`]: each player's history, its Explored tiles, its records, the tiles still Visible for
//! the fog delay and the attackers revealed to it. The rest is derived from the objects when a save is loaded.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize, Serializer};

use super::{LastKnown, TileBits, Vision};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::object::Object;
use crate::tuning::vision::{FOG_DELAY_TICKS, REVEAL_TICKS};
use crate::units::{Tick, TilePos};

/// Tile symbols of the Explored rows in a save: Explored (Visible tiles are Explored too), and not.
pub(super) const EXPLORED_SYMBOL: char = '#';
pub(super) const UNEXPLORED_SYMBOL: char = '.';

/// One player's vision in a save (the serde form of [`Vision`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedVision {
    pub(super) player: PlayerId,
    /// One string per tile row, north to south: [`EXPLORED_SYMBOL`] for a tile the player has seen,
    /// [`UNEXPLORED_SYMBOL`] for one it hasn't.
    pub(super) explored: Vec<String>,
    /// Its records ([`super::PlayerVision::records`]), by ascending id.
    pub(super) last_known: Vec<LastKnown>,
    /// The tiles still Visible for the fog delay ([`super::linger`]), row by row. Optional, so that saves from before
    /// the fog delay load, and left out when there are none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) lingering: Vec<SavedLinger>,
    /// The attackers revealed to the player ([`super::reveal`]), by ascending id. Optional like `lingering`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) revealed: Vec<SavedReveal>,
}

/// A tile still Visible for the fog delay, in a save: phase 5f of tick `ends` turns it Explored.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedLinger {
    pub(super) tile: TilePos,
    pub(super) ends: Tick,
}

/// An attacker revealed, in a save: phase 5f of tick `ends` ends its reveal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedReveal {
    pub(super) id: ObjectId,
    pub(super) ends: Tick,
}

impl Vision {
    /// The serde form of each player's history ([`SavedVision`]).
    pub(super) fn saved(&self) -> Vec<SavedVision> {
        let (width, height) = (self.width as usize, self.height as usize);
        self.players
            .iter()
            .map(|p| SavedVision {
                player: p.id,
                explored: (0..height)
                    .map(|y| {
                        (0..width)
                            .map(|x| {
                                if p.explored.get(y * width + x) {
                                    EXPLORED_SYMBOL
                                } else {
                                    UNEXPLORED_SYMBOL
                                }
                            })
                            .collect()
                    })
                    .collect(),
                last_known: p.records().cloned().collect(),
                lingering: p
                    .lingering()
                    .map(|(i, ends)| SavedLinger {
                        tile: TilePos::new((i % self.width) as i32, (i / self.width) as i32),
                        ends,
                    })
                    .collect(),
                revealed: p.revealed().map(|(id, ends)| SavedReveal { id, ends }).collect(),
            })
            .collect()
    }
}

impl Serialize for Vision {
    /// Only the history: each player's Explored rows, records, tiles that linger and reveals ([`SavedVision`]). The
    /// rest comes from the objects on load.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.saved().serialize(serializer)
    }
}

/// The vision of a save loaded at the end of `tick` ([`crate::Sim`]'s serde form). `saved` must hold the history of
/// exactly `players`, in order: Explored rows of `map`'s size; records of objects the player doesn't own, each once,
/// on the map and seen by `tick` at the latest; tiles that linger, each once, in order, on the map, their delays
/// ending within the [`FOG_DELAY_TICKS`] after `tick`; and reveals, each once, in id order, ending within the
/// [`REVEAL_TICKS`] after `tick`. The rest is derived from `objects` (all of the sim's), as at the end of the tick:
/// tiles Visible now count as Explored even if the save forgot them, a tile a source sees no longer lingers, and the
/// records of the objects in sight take their current state.
pub(crate) fn load<'a>(
    map: &Map,
    players: &[PlayerId],
    saved: Vec<SavedVision>,
    tick: Tick,
    objects: impl IntoIterator<Item = &'a Object>,
) -> Result<Vision, String> {
    let saved_ids: Vec<PlayerId> = saved.iter().map(|s| s.player).collect();
    if saved_ids != players {
        return Err(format!(
            "the vision is saved for players {saved_ids:?}, the game has {players:?}"
        ));
    }
    let mut vision = Vision::new(map, players.iter().copied());
    for (player, s) in vision.players.iter_mut().zip(saved) {
        player.explored = explored(map, &s)?;
        check_records(map, &s, tick)?;
        let lingering = lingering(map, &s, tick)?;
        let revealed = revealed(&s, tick)?;
        player.set_records(s.last_known);
        player.restore(lingering, revealed);
    }
    let objects: BTreeMap<ObjectId, &Object> = objects.into_iter().map(|o| (o.id, o)).collect();
    vision.update(map, objects.values().copied(), tick);
    vision.remember(tick, |id| objects.get(&id).copied());
    Ok(vision)
}

/// A player's saved tiles that linger, as `(tile index, end)` pairs: each on the map, after the one before in row
/// order, its delay ending after `tick` and no later than [`FOG_DELAY_TICKS`] after it.
fn lingering(map: &Map, saved: &SavedVision, tick: Tick) -> Result<Vec<(u32, Tick)>, String> {
    let (width, height) = (map.width(), map.height());
    let player = saved.player.0;
    let mut tiles = Vec::with_capacity(saved.lingering.len());
    for &SavedLinger { tile, ends } in &saved.lingering {
        let fail = |problem: &str| Err(format!("player {player}'s tile {tile:?} lingers {problem}"));
        if tile.x < 0 || tile.y < 0 || tile.x as u32 >= width || tile.y as u32 >= height {
            return fail("off the map");
        }
        let i = tile.y as u32 * width + tile.x as u32;
        if tiles.last().is_some_and(|&(last, _)| last >= i) {
            return fail("out of order, or twice");
        }
        if ends <= tick || ends > tick + FOG_DELAY_TICKS {
            return fail(&format!(
                "until tick {ends}, outside the {FOG_DELAY_TICKS} ticks after tick {tick}"
            ));
        }
        tiles.push((i, ends));
    }
    Ok(tiles)
}

/// A player's saved reveals, as `(id, end)` pairs: by ascending id, each ending after `tick` and no later than
/// [`REVEAL_TICKS`] after it.
fn revealed(saved: &SavedVision, tick: Tick) -> Result<Vec<(ObjectId, Tick)>, String> {
    let player = saved.player.0;
    let mut reveals: Vec<(ObjectId, Tick)> = Vec::with_capacity(saved.revealed.len());
    for &SavedReveal { id, ends } in &saved.revealed {
        let fail = |problem: &str| Err(format!("player {player}'s reveal of #{} {problem}", id.0));
        if reveals.last().is_some_and(|&(last, _)| last >= id) {
            return fail("is out of order, or twice");
        }
        if ends <= tick || ends > tick + REVEAL_TICKS {
            return fail(&format!(
                "ends at tick {ends}, outside the {REVEAL_TICKS} ticks after tick {tick}"
            ));
        }
        reveals.push((id, ends));
    }
    Ok(reveals)
}

/// The Explored tiles of a player's saved rows.
fn explored(map: &Map, saved: &SavedVision) -> Result<TileBits, String> {
    let (width, height) = (map.width() as usize, map.height() as usize);
    let player = saved.player.0;
    let rows = &saved.explored;
    if rows.len() != height || rows.iter().any(|r| r.chars().count() != width) {
        return Err(format!(
            "player {player}'s explored tiles don't match the {width} x {height} map"
        ));
    }
    let mut bits = TileBits::new(width * height);
    for (y, row) in rows.iter().enumerate() {
        for (x, symbol) in row.chars().enumerate() {
            match symbol {
                EXPLORED_SYMBOL => bits.set(y * width + x),
                UNEXPLORED_SYMBOL => {}
                other => {
                    return Err(format!(
                        "player {player}'s explored tiles: {other:?} is neither {EXPLORED_SYMBOL:?} nor \
                         {UNEXPLORED_SYMBOL:?}"
                    ));
                }
            }
        }
    }
    Ok(bits)
}

/// A player's saved records: each object once, not one the player owns, on the map and seen by `tick`.
fn check_records(map: &Map, saved: &SavedVision, tick: Tick) -> Result<(), String> {
    let player = saved.player;
    let mut ids = BTreeSet::new();
    for record in &saved.last_known {
        let fail = |problem: &str| Err(format!("player {}'s record of #{} {problem}", player.0, record.id.0));
        if !ids.insert(record.id) {
            return fail("appears more than once");
        }
        if record.owner == Some(player) {
            return fail("is of its own object");
        }
        let p = record.placement;
        let on_map = p.w > 0
            && p.h > 0
            && p.origin.x >= 0
            && p.origin.y >= 0
            && i64::from(p.origin.x) + i64::from(p.w) <= i64::from(map.width())
            && i64::from(p.origin.y) + i64::from(p.h) <= i64::from(map.height());
        if !on_map {
            return fail("lies off the map");
        }
        if record.seen > tick {
            return fail(&format!("was seen at tick {}, after tick {tick}", record.seen));
        }
    }
    Ok(())
}
