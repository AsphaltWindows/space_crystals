//! (M6-GR) Scans (plan decision T92: a timed sight source): `factions/gdo/structures/radar_station.md > Placeholder
//! stats`, "**Scan** (hotkey **Q**, targeted: left-click any point on the map): reveals a circle of radius **6 grid
//! units** for **8 seconds** (128 frames)". A Radar Station's cast (`Command::Cast`, `AreaAbility::Scan`, whose numbers
//! are its `AbilityDef`'s) gives its owner a [`Scan`]: sight of a disc round the point, for the ability's duration.
//!
//! - **The state**: each player's scans under way ([`Scans`]), the point, the radius and the tick that ends each.
//!   Nothing in the objects says when a scan was cast, so they are history, as the fog delay's tiles: hashed and saved,
//!   each only while the player has one, so that the games without a scan hash and save as they did.
//! - **Sight**: a scan sees as a sight source does (`super::sight`, the same disc of tiles, each counted in the
//!   player's per-tile counts), its radius the ability's, with no ElevationModifier, as for air units
//!   (`core/vision.md > Air exempt`). **Placeholder (not in the design: "high ground doesn't shrink it")**, the plan's
//!   D39. It is sight, not detection: an object hidden from the player (`super::hidden`, an enemy's underground
//!   expansion) stays hidden inside its circle, as inside any source's sight (D39's placeholder).
//! - **Ticks**: a scan cast in tick `t` (the commands phase) is stamped in phase 5f of `t`, so its tiles are Visible at
//!   the end of ticks `t` to `t + duration − 1`, the 128 ticks of "8 seconds". Phase 5f of `t + duration` takes it
//!   away, and its tiles that no other source sees linger for the fog delay (`super::linger`), then turn Explored.
//! - **Cost**: a scan is stamped once, as it starts, and taken away once, as it ends, each from a queue in the order
//!   the scans end ([`Scans::active`]), never a look at the map or at every scan in a tick.
//! - **Smoke** (design change 0433, slice M6-SC's rule): "smoke **blocks Scan** for what is **inside** the smoke; a
//!   Scan has no viewpoint, so it isn't blocked "behind" smoke". A scan's tiles are stamped in [`Vision::step_scans`],
//!   where smoke would leave out the tiles under it.

use std::collections::VecDeque;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::{Eye, Vision, stamp};
use crate::elevation::Elevation;
use crate::hash::{StateHash, StateHasher};
use crate::ids::PlayerId;
use crate::units::Tick;

/// A scan under way (module docs): sight of a disc for its player until phase 5f of `until`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    /// Its centre, space units: the point it was cast at.
    pub at: Vec2,
    /// Its radius, grid units: its ability's as it was cast (`AbilityDef::radius`).
    pub radius: f32,
    /// The tick whose phase 5f ends it: the tick of its cast and its ability's duration.
    pub until: Tick,
}

impl StateHash for Scan {
    fn hash(&self, h: &mut StateHasher) {
        let Scan { at, radius, until } = self;
        at.hash(h);
        radius.hash(h);
        until.hash(h);
    }
}

/// One player's scans under way (module docs).
#[derive(Clone, Debug, Default)]
pub(super) struct Scans {
    /// Those cast in the tick under way, which its phase 5f stamps, in cast order. History; empty between ticks.
    pending: Vec<Scan>,
    /// Those stamped, by ascending end (in cast order among equal ends), each with the tiles it stamped, ascending.
    /// The scans are history; their tiles are derived.
    active: VecDeque<(Scan, Vec<u32>)>,
}

impl Scans {
    /// Whether the player has a scan under way.
    pub(super) fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.active.is_empty()
    }

    /// The scans under way: those stamped, in the order they end, then those cast in the tick under way.
    pub(super) fn list(&self) -> impl Iterator<Item = &Scan> + '_ {
        self.active.iter().map(|(scan, _)| scan).chain(&self.pending)
    }

    /// Those cast and not stamped yet: none between ticks (`Vision::check`).
    pub(super) fn unstamped(&self) -> impl Iterator<Item = &Scan> + '_ {
        self.pending.iter()
    }

    /// The scans of a save, or of a copy to check against, to be stamped by the next phase 5f.
    pub(super) fn restore(&mut self, scans: impl IntoIterator<Item = Scan>) {
        self.pending.extend(scans);
    }
}

impl PartialEq for Scans {
    /// The same scans; their tiles are derived.
    fn eq(&self, other: &Self) -> bool {
        self.list().eq(other.list())
    }
}

impl StateHash for Scans {
    /// The scans, in [`Scans::list`]'s order.
    fn hash(&self, h: &mut StateHasher) {
        h.len(self.list().count());
        for scan in self.list() {
            scan.hash(h);
        }
    }
}

impl Vision {
    /// `player`'s Radar Station cast a Scan in the commands phase (`Sim::cast`): phase 5f of the tick stamps it (module
    /// docs). Nothing for a player who isn't in the game.
    pub(crate) fn scan(&mut self, player: PlayerId, scan: Scan) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player) {
            p.scans.pending.push(scan);
        }
    }

    /// Phase 5f of `tick`, after the sources (`Vision::update`): each player's scans that end now take their tiles
    /// away, which linger for the fog delay, and those cast in the tick stamp theirs, each joining the queue in the
    /// order of its end. One that ends by now already (a duration of none) stamps nothing.
    pub(super) fn step_scans(&mut self, tick: Tick) {
        let Vision {
            width,
            height,
            tile_levels,
            players,
            ..
        } = self;
        for (index, player) in players.iter_mut().enumerate() {
            while let Some((scan, _)) = player.scans.active.front()
                && scan.until <= tick
            {
                let (_, tiles) = player.scans.active.pop_front().expect("a front");
                for i in tiles {
                    player.unsee(i, tick);
                }
            }
            for scan in std::mem::take(&mut player.scans.pending) {
                if scan.until <= tick {
                    continue;
                }
                let eye = Eye {
                    player: index,
                    at: scan.at,
                    sight: scan.radius,
                    elevation: Elevation::Exempt,
                };
                let mut tiles = Vec::new();
                stamp(&eye, *width, *height, tile_levels, &mut tiles);
                for &i in &tiles {
                    player.see(i);
                }
                let active = &mut player.scans.active;
                let at = active.partition_point(|(other, _)| other.until <= scan.until);
                active.insert(at, (scan, tiles));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;
    use crate::object::Object;
    use crate::units::TilePos;
    use crate::vision::VisibilityState;

    const P1: PlayerId = PlayerId(1);
    const P2: PlayerId = PlayerId(2);
    const NONE: [&Object; 0] = [];

    fn hash(vision: &Vision) -> u64 {
        let mut h = StateHasher::new();
        vision.hash(&mut h);
        h.finish()
    }

    /// Player `player`'s state of the tile `(x, y)`, and whether it lingers.
    fn seen(vision: &Vision, player: PlayerId, x: i32, y: i32) -> (VisibilityState, bool) {
        let p = vision.player(player).unwrap();
        (p.state(TilePos::new(x, y)), p.lingers(TilePos::new(x, y)))
    }

    /// A scan cast in tick 2 until tick 7 (module docs): unstamped, it is caught by V1 between ticks
    /// (`Vision::check`); phase 5f of tick 2 stamps its disc, Visible to its player alone at the end of ticks 2 to 6,
    /// and V1 rebuilds the same tiles from it; a vision checked in tick 7 before its phase 5f has run still holds it,
    /// over, which V1 finds; phase 5f of tick 7 takes it away, its tiles lingering for the fog delay. The scans are
    /// history: a vision with one hashes apart from one without, and one without any hashes as it did.
    #[test]
    fn a_scan_is_stamped_taken_away_checked_and_hashed() {
        use VisibilityState::{Unexplored, Visible};
        let map = Map::new(30, 20);
        let mut vision = Vision::new(&map, [P1, P2]);
        vision.update(&map, NONE, 1);
        let mut without = vision.clone();
        let at = TilePos::new(15, 10).center();
        vision.scan(
            P1,
            Scan {
                at,
                radius: 3.0,
                until: 7,
            },
        );
        let (point, message) = vision.check(&map, NONE, 1).unwrap_err();
        assert_eq!(point, at);
        assert!(message.contains("unstamped"), "{message}");
        assert_eq!(seen(&vision, P1, 15, 10), (Unexplored, false));
        for tick in 2..7 {
            vision.update(&map, NONE, tick);
            without.update(&map, NONE, tick);
            vision.check(&map, NONE, tick).unwrap();
            for (x, y) in [(15, 10), (18, 10), (15, 7), (17, 12)] {
                assert_eq!(seen(&vision, P1, x, y), (Visible, false), "tick {tick}: ({x}, {y})");
            }
            assert_eq!(seen(&vision, P1, 19, 10), (Unexplored, false), "beyond 3 grid units");
            assert_eq!(seen(&vision, P2, 15, 10), (Unexplored, false), "player 1's alone");
            assert_ne!(hash(&vision), hash(&without), "tick {tick}");
        }
        let (point, message) = vision.check(&map, NONE, 7).unwrap_err();
        assert_eq!(point, at);
        assert!(message.contains("ends at tick 7"), "{message}");
        vision.update(&map, NONE, 7);
        vision.check(&map, NONE, 7).unwrap();
        assert_eq!(vision.player(P1).unwrap().scans().count(), 0);
        assert_eq!(seen(&vision, P1, 15, 10), (Visible, true), "lingering");
        let mut never = Vision::new(&map, [P1, P2]);
        never.update(&map, NONE, 1);
        let mut scanned = never.clone();
        scanned.scan(
            P2,
            Scan {
                at,
                radius: 3.0,
                until: 1,
            },
        );
        scanned.update(&map, NONE, 2);
        never.update(&map, NONE, 2);
        assert_eq!(
            hash(&scanned),
            hash(&never),
            "a scan over by its stamp stamps nothing, and is gone"
        );
    }
}
