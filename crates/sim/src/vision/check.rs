//! The vision's own invariant, V1 ([`Vision::check`]): the incremental state against a vision built from scratch out
//! of the objects and the same history. Split off `vision.rs` in slice M6-GR.

use std::collections::BTreeMap;

use glam::Vec2;

use super::{Vision, memory};
use crate::ids::ObjectId;
use crate::map::Map;
use crate::object::Object;
use crate::ruleset::rules;
use crate::units::{Tick, TilePos};

impl Vision {
    /// Checks the incremental state against a vision built from scratch out of `objects` (all of the sim's, in id
    /// order) at the end of `tick`, from the same history (records, tiles that linger, reveals, (M6-GR) scans): the
    /// same sources, the
    /// same Visible tiles seen by as many of them, and the same tiles lingering until the same ticks, for every player;
    /// every Visible tile Explored; no tile lingering while a source sees it, and no delay or reveal that ends in the
    /// past or beyond its length (`Vision::fog_delay_ticks`, `Vision::reveal_ticks`); the same reveals, so none was due
    /// to end; and the memory a fresh build makes of the same records: none dropped or refreshed, none of an object the
    /// player owns, and the same objects in sight; (M6-GR) no scan cast and left unstamped, and none that ends by
    /// `tick` (`super::scans`). The test harness runs it after every tick (`testing::invariants`, V1). The first
    /// difference, and where on the map (space units).
    pub(crate) fn check<'a>(
        &self,
        map: &Map,
        objects: impl IntoIterator<Item = &'a Object>,
        tick: Tick,
    ) -> Result<(), (Vec2, String)> {
        let tuning = &rules().tuning.vision;
        let objects: BTreeMap<ObjectId, &Object> = objects.into_iter().map(|o| (o.id, o)).collect();
        let mut fresh = Vision::new(map, self.players.iter().map(|p| p.id));
        for (fresh, have) in fresh.players.iter_mut().zip(&self.players) {
            fresh.set_records(have.records().cloned());
            fresh.restore(have.lingering(), have.revealed());
            fresh.scans.restore(have.scans().copied());
        }
        fresh.update(map, objects.values().copied(), tick);
        fresh.remember(tick, |id| objects.get(&id).copied());
        for have in &self.players {
            if let Some(scan) = have.scans.unstamped().next() {
                return Err((scan.at, format!("player {}'s scan is left unstamped", have.id.0)));
            }
            if let Some(scan) = have.scans().find(|scan| scan.until <= tick) {
                let message = format!(
                    "player {}'s scan ends at tick {}, by tick {tick}",
                    have.id.0, scan.until
                );
                return Err((scan.at, message));
            }
        }
        let eyes = |v: &Vision| v.sources.iter().map(|s| (s.id, s.eye)).collect::<Vec<_>>();
        if eyes(self) != eyes(&fresh) {
            let at = self.sources.first().map_or(Vec2::ZERO, |s| s.eye.at);
            return Err((at, "the sight sources differ from a fresh build".into()));
        }
        for (have, want) in self.players.iter().zip(&fresh.players) {
            for (i, (&n, &m)) in have.seers.iter().zip(&want.seers).enumerate() {
                let tile = TilePos::new((i % self.width as usize) as i32, (i / self.width as usize) as i32);
                let fail =
                    |message: String| Err((tile.center(), format!("player {}: tile {tile:?} {message}", have.id.0)));
                if n != m {
                    return fail(format!("is seen by {n} sources, a fresh count says {m}"));
                }
                if have.visible_at(i) && !have.explored.get(i) {
                    return fail("is Visible but not Explored".into());
                }
                let end = have.linger.end(i);
                match end {
                    Some(_) if n > 0 => return fail(format!("lingers, though {n} sources see it")),
                    Some(end) if end <= tick || end > tick + tuning.fog_delay_ticks => {
                        return fail(format!("lingers until tick {end}, outside the delay after tick {tick}"));
                    }
                    _ => {}
                }
                if end != want.linger.end(i) {
                    return fail(format!(
                        "lingers until {end:?}, a fresh build's {:?}",
                        want.linger.end(i)
                    ));
                }
            }
            let at = |id: ObjectId| objects.get(&id).map_or(Vec2::ZERO, |o| o.pos);
            let player = have.id.0;
            if let Some((id, end)) = have
                .revealed()
                .find(|&(_, end)| end <= tick || end > tick + tuning.reveal_ticks)
            {
                let message = format!(
                    "player {player}: #{} is revealed until tick {end}, outside the reveal after tick {tick}",
                    id.0
                );
                return Err((at(id), message));
            }
            if have.reveals != want.reveals {
                let id = have.revealed().map(|(id, _)| id).next().unwrap_or(ObjectId(0));
                let message = format!(
                    "player {player}: the reveals are {:?}, a fresh build's {:?}",
                    have.reveals, want.reveals
                );
                return Err((at(id), message));
            }
            memory::check(have, want)?;
        }
        if !self.footprints.matches(&fresh.footprints) {
            return Err((
                Vec2::ZERO,
                "the footprints of the tile-based objects differ from a fresh build".into(),
            ));
        }
        Ok(())
    }
}
