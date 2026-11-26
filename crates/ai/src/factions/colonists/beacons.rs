//! The Colonists' Beacons (`super` module docs, step 4; `factions/colonists/economy.md > Beacon Deployment`): when a
//! Drone deploys, where, and which one.
//!
//! - **When** ([`wanted`]): Beacon Capacity is kept ahead of the army (`factions/colonists/units/drone.md >
//!   BeaconCapacityCost`: "roughly every 6th Drone must deploy as a Beacon to keep Drone production going"). The room
//!   under it is what is available less what is used, plus what the Beacons on their way will bring: 5 for each under
//!   deployment, which provides its 5 only at completion, and 6 for each Drone walking to its site, whose own 1 is
//!   freed as it starts (`factions/colonists/economy.md > Beacon Deployment > Beacon Capacity during deployment`, as
//!   `sc_sim::testing::colonists::room` counts it). A Drone deploys once the room falls short of what the queued
//!   units will take as they start plus [`BEACON_LEAD`] of each portal's next units, or when a Warp In finds no
//!   Beacon at all, and a Drone the field can't seat deploys too ([`spare`]); never past the cap
//!   (`factions/colonists/overview.md > ColonistResources > Beacon Capacity`: "Total Beacon Capacity caps at 200").
//!   One Drone a tick: the next tick counts it on its way.
//! - **Where** ([`sites`]): a Beacon's site holds what a Bio Portal warped in on it would cover
//!   (`sc_sim::transform::warp_in_footprint`) while the Bio Portals to come lack Beacons, else the Beacon's own 2 x 2,
//!   with a free ring of open ground round it (`crate::build`'s ring) that no footprint the player knows covers, nor
//!   the Bio Portals to come on the Beacons kept for them ([`hosts`]), nor the site of a pending Deploy or of one that
//!   failed. Its tiles are Buildable and flat, its centre is beyond [`STAGING_RADIUS_GU`] of the staging point, where
//!   the army waits, and its ring is out of the field's box, where the Drones work. Of the sites within [`SITE_REACH`]
//!   tiles of where the Robo Portal stood as the computer player took over, those beyond it from the field come first,
//!   then the nearest it, ties row by row.
//! - **Which Drone** ([`deployers`]): one the Robo Portal releases, as the scripted lines deploy
//!   (`sc_sim::testing::colonists`): an idle Drone, the Drones the player took over all gathering in its first turn.
//!   A Drone on a Gather job collides with no other one, but one sent to deploy is off its job and collides with every
//!   gathering Drone in its way (`factions/colonists/units/drone.md > Gathering`), which never yields to it. So a
//!   gathering Drone deploys only when no Robo Portal will release one (none can train, or Beacon Capacity has no
//!   room for its next Drone) and no Beacon is on its way to make room. The Robo Portal trains Drones for the
//!   Beacons as for the field, so the Beacons cost the field nothing. For each site in turn, within [`SITE_CHECKS`],
//!   the Drone nearest it, then one carrying nothing (a load is lost when the deployment completes,
//!   `factions/colonists/economy.md > Deploying with a load`), deploys there if the deploy check takes it
//!   (`View::deploy_check`, the gate's own: "The ghost judges by what the player knows").
//! - **A Deploy that fails** ([`Deploys`]): the Drone stands idle, still a Drone (`factions/colonists/economy.md >
//!   Placement`: "the order is cancelled and the Drone stops and idles"); one that hasn't begun its deployment within
//!   [`DEPLOY_PATIENCE_TICKS`] of its order is stopped, as when gathering Drones keep it from getting through. That
//!   Drone and that site are left out of the next Deploys for [`FAILED_DEPLOY_TICKS`].

use sc_sim::deploy;
use sc_sim::nav::TileRect;
use sc_sim::transform::warp_in_footprint;
use sc_sim::{
    ArmyResource, BEACON_CAPACITY_CAP, BEACON_CAPACITY_PER_BUILDING, BuildingOrientation, Command, Object, ObjectId,
    ObjectKind, OrderKind, SU_PER_GU, Tick, TilePos, Vec2, gu,
};

use super::Base;
use crate::build::Occupied;
use crate::tuning::army::STAGING_RADIUS_GU;
use crate::tuning::build::SPOT_RETRY_TICKS;
use crate::tuning::colonists::{
    BEACON_LEAD, DEPLOY_PATIENCE_TICKS, FAILED_DEPLOY_TICKS, SITE_CHECKS, SITE_REACH, WARP_INS,
};
use crate::{Turn, View};

use ObjectKind::{Beacon, BioPortal, Drone, RoboPortal, SabreLegionnaire};

/// What the player remembers of the Deploys it gave (module docs): those on their way, and those that failed.
#[derive(Debug, Default)]
pub(super) struct Deploys {
    /// The Drones it sent to deploy, with their sites and the ticks it sent them, until they deploy, fail or are lost.
    sent: Vec<(ObjectId, TilePos, Tick)>,
    /// The Drones whose Deploy failed, with their sites, each left out until its tick.
    failed: Vec<(ObjectId, TilePos, Tick)>,
    /// After a search found no site, the tick from which it looks again.
    retry: Tick,
    /// Whether it has had its first turn, in which the Drones it took over all gather.
    started: bool,
}

impl Deploys {
    /// Sees what became of the Deploys it sent: a Drone still a Drone without its Deploy order failed, and one still
    /// on its way after [`DEPLOY_PATIENCE_TICKS`] is stopped and fails.
    pub(super) fn review(&mut self, turn: &mut Turn<'_, '_>) {
        let view = turn.view();
        let now = view.tick();
        self.failed.retain(|&(_, _, until)| now < until);
        let (failed, mut late) = (&mut self.failed, Vec::new());
        self.sent.retain(|&(drone, origin, sent)| {
            let Some(o) = view.own_object(drone).filter(|o| o.kind == Drone) else {
                // Deployed (a Beacon now), or lost.
                return false;
            };
            let on_its_way = deploying_at(o) == Some(origin);
            if on_its_way && now < sent + DEPLOY_PATIENCE_TICKS {
                return true;
            }
            if on_its_way {
                late.push(drone);
            }
            failed.push((drone, origin, now + FAILED_DEPLOY_TICKS));
            false
        });
        if !late.is_empty() {
            turn.give(Command::Order {
                units: late,
                kind: OrderKind::Stop,
                queue: false,
            });
        }
    }

    /// 4: sends a Drone to deploy, if Beacon Capacity or a Warp In wants a Beacon ([`wanted`]), at the first of the
    /// Beacon sites ([`sites`]) the deploy check takes for the nearest of its [`deployers`]. Returns the Drone sent.
    pub(super) fn send(&mut self, turn: &mut Turn<'_, '_>, base: &Base<'_>, anchor: Vec2) -> Option<ObjectId> {
        let view = turn.view();
        let now = view.tick();
        if !self.started {
            // The Drones it took over gather first: the first Beacon comes from a Drone the Robo Portal releases.
            self.started = true;
            return None;
        }
        if now < self.retry || !(wanted(turn, base) || spare(view, base)) {
            return None;
        }
        let deployers = deployers(view, base, &self.failed);
        if deployers.is_empty() {
            return None;
        }
        let candidates = sites(turn, base, anchor, &self.failed);
        let site = (candidates.into_iter().take(SITE_CHECKS)).find_map(|(origin, rect)| {
            let drone = nearest(&deployers, centre(beacon_footprint(origin)))?;
            let valid = view.deploy_check(drone, origin, false).is_some_and(|c| c.is_valid());
            valid.then_some((drone, origin, rect))
        });
        let Some((drone, origin, rect)) = site else {
            self.retry = now + SPOT_RETRY_TICKS;
            return None;
        };
        let order = Command::Order {
            units: vec![drone],
            kind: OrderKind::Deploy { origin },
            queue: false,
        };
        if !turn.give(order) {
            return None;
        }
        turn.take(rect);
        self.sent.push((drone, origin, now));
        Some(drone)
    }
}

/// Whether the Drone `o` is on its way to deploy: it has a Deploy order, current or queued.
pub(super) fn walking(o: &Object) -> bool {
    deploying_at(o).is_some()
}

/// The site of `o`'s Deploy order, current or queued (the first), by the top-left tile of its Beacon's footprint.
fn deploying_at(o: &Object) -> Option<TilePos> {
    let orders = o.orders.as_ref()?;
    let current = orders.current.as_ref().map(|active| active.order.kind);
    let mut kinds = current.into_iter().chain(orders.queue.iter().map(|order| order.kind));
    kinds.find_map(|kind| match kind {
        OrderKind::Deploy { origin } => Some(origin),
        _ => None,
    })
}

/// Whether a Drone should deploy this tick (module docs): the room under Beacon Capacity falls short of what the
/// portals will take next, or a Warp In finds no Beacon; never once the Beacons standing and on their way reach the
/// cap.
pub(super) fn wanted(turn: &Turn<'_, '_>, base: &Base<'_>) -> bool {
    let view = turn.view();
    let Some(cap) = view.army_cap() else {
        return false;
    };
    let per = BEACON_CAPACITY_PER_BUILDING;
    if cap.available + per * (base.deploying + base.walking) >= BEACON_CAPACITY_CAP {
        return false;
    }
    if base.beacon_wanted {
        return true;
    }
    let coming = per * base.deploying + (per + 1) * base.walking;
    let room = i64::from(cap.available) + i64::from(coming) - i64::from(cap.used);
    room < i64::from(base.waiting + lead(view, base))
}

/// Whether the player has a Drone its field can't seat, which deploys whether a Beacon is wanted or not: one the Robo
/// Portal trained for a Beacon that was no longer wanted when it came out. Never once the Beacons standing and on
/// their way reach the cap.
fn spare(view: &View<'_>, base: &Base<'_>) -> bool {
    let per = BEACON_CAPACITY_PER_BUILDING;
    let below_cap = (view.army_cap())
        .is_some_and(|cap| cap.available + per * (base.deploying + base.walking) < BEACON_CAPACITY_CAP);
    below_cap && base.workers.len() + base.drones_queued > base.seats()
}

/// What [`BEACON_LEAD`] of each portal's next units take of Beacon Capacity: a Drone for each Robo Portal while the
/// field wants Drones, a Sabre Legionnaire for each Bio Portal, those warping in included.
fn lead(view: &View<'_>, base: &Base<'_>) -> u32 {
    let resource = ArmyResource::BeaconCapacity;
    let next = |o: &Object| match o.kind {
        RoboPortal if base.wants_drones() => resource.cost_of(Drone),
        BioPortal => resource.cost_of(SabreLegionnaire),
        _ => 0,
    };
    BEACON_LEAD * view.own().map(next).sum::<u32>()
}

/// The Drones that may deploy (module docs), but those whose Deploy failed lately: the idle ones, which the Robo
/// Portal released (the Drones the player took over gather in its first turn); every Drone that gathers or may when
/// no Robo Portal will release one and no Beacon is on its way to make room for one.
fn deployers<'v>(view: &View<'_>, base: &Base<'v>, failed: &[(ObjectId, TilePos, Tick)]) -> Vec<&'v Object> {
    let room = view
        .army_cap()
        .is_some_and(|cap| cap.used + cap.resource.cost_of(Drone) <= cap.available);
    let releases = |o: &Object| {
        let training = o.production.as_ref().is_some_and(|p| p.in_production() == Some(Drone));
        o.kind == RoboPortal && o.is_operational() && (room || training)
    };
    let any = !view.own().any(releases) && base.walking + base.deploying == 0;
    (base.workers.iter().copied())
        .filter(|o| !failed.iter().any(|&(drone, _, _)| drone == o.id))
        .filter(|o| any || o.orders.as_ref().is_none_or(|orders| orders.is_idle()))
        .collect()
}

/// Of `deployers`, the one nearest `site` (module docs), then one carrying nothing, ties by id; `None` without one.
fn nearest(deployers: &[&Object], site: Vec2) -> Option<ObjectId> {
    let key = |o: &Object| {
        let load = o.worker.as_ref().is_some_and(|w| w.load > 0);
        (o.pos.distance_squared(site), load)
    };
    (deployers.iter())
        .min_by(|a, b| {
            let (ka, kb) = (key(a), key(b));
            (ka.0.total_cmp(&kb.0)).then(ka.1.cmp(&kb.1)).then(a.id.cmp(&b.id))
        })
        .map(|o| o.id)
}

/// The Beacon sites (module docs) in their order, each by the top-left tile of its Beacon's footprint, with what it
/// keeps clear: a Bio Portal's footprint on it while the Bio Portals to come lack Beacons ([`hosts`]), else its own.
fn sites(
    turn: &Turn<'_, '_>,
    base: &Base<'_>,
    anchor: Vec2,
    failed: &[(ObjectId, TilePos, Tick)],
) -> Vec<(TilePos, TileRect)> {
    let view = turn.view();
    let map = view.map();
    let occupied = Occupied::of(turn);
    let failed = failed.iter().map(|&(_, origin, _)| origin);
    let pending: Vec<TileRect> = (view.own().filter_map(deploying_at).chain(failed))
        .map(beacon_footprint)
        .collect();
    let (reserved, short) = hosts(turn, base, &occupied);
    let away = super::field::centre(&base.field).map(|field| anchor - field);
    let clear = turn
        .staging()
        .map(|s| (s, gu(STAGING_RADIUS_GU) * gu(STAGING_RADIUS_GU)));
    let field_box = field_box(base);
    let at = TilePos::containing(anchor);
    let mut sites: Vec<(bool, f32, TilePos, TileRect)> = Vec::new();
    for y in at.y - SITE_REACH..=at.y + SITE_REACH {
        for x in at.x - SITE_REACH..=at.x + SITE_REACH {
            let origin = TilePos::new(x, y);
            let rect = match short {
                true => portal_footprint(origin),
                false => Some(beacon_footprint(origin)),
            };
            let Some(rect) = rect else {
                continue;
            };
            let ring = grown(rect, 1);
            let centre = centre(rect);
            if clear.is_some_and(|(staging, r2)| centre.distance_squared(staging) < r2)
                || field_box.is_some_and(|field_box| overlaps(ring, field_box))
                || (reserved.iter().chain(&pending)).any(|&r| overlaps(ring, r))
                || !occupied.free(map, rect, true)
                || !tiles(rect).all(|tile| buildable(view, tile))
            {
                continue;
            }
            let beyond = away.is_some_and(|away| (centre - anchor).dot(away) > 0.0);
            sites.push((!beyond, centre.distance_squared(anchor), origin, rect));
        }
    }
    sites.sort_by(|a, b| {
        (a.0.cmp(&b.0))
            .then(a.1.total_cmp(&b.1))
            .then((a.2.y, a.2.x).cmp(&(b.2.y, b.2.x)))
    });
    sites.into_iter().map(|(_, _, origin, rect)| (origin, rect)).collect()
}

/// The Beacons kept for the Bio Portals to come (module docs), and whether they are fewer than those: as many as the
/// Bio Portals of [`WARP_INS`] the player lacks, of its Beacons (complete, under deployment, or the sites of its
/// pending Deploys) on which a Bio Portal still fits as far as the footprints and the terrain go, those nearest the
/// staging point first (as a Warp In picks them), ties by id and then row by row. Each is given by what a Bio Portal
/// on it would cover, which the next sites keep clear.
fn hosts(turn: &Turn<'_, '_>, base: &Base<'_>, occupied: &Occupied) -> (Vec<TileRect>, bool) {
    let view = turn.view();
    let map = view.map();
    let planned = WARP_INS.iter().filter(|&&(kind, _)| kind == BioPortal).count();
    let to_come = planned.saturating_sub(view.own().filter(|o| o.kind == BioPortal).count());
    if to_come == 0 {
        return (Vec::new(), false);
    }
    let toward = turn.staging().unwrap_or(base.node);
    let mut hosts: Vec<(f32, ObjectId, TilePos, TileRect)> = (view.own())
        .filter_map(|o| match o.kind {
            Beacon => o.placement.map(|p| (o.id, p.origin)),
            Drone => deploying_at(o).map(|origin| (o.id, origin)),
            _ => None,
        })
        .filter_map(|(id, origin)| {
            let portal = portal_footprint(origin)?;
            let beacon = beacon_footprint(origin);
            // The tiles the Bio Portal adds to the Beacon's: free of every footprint, and buildable.
            let fits = (tiles(portal).filter(|&tile| !contains(beacon, tile)))
                .all(|tile| occupied.free(map, TileRect::new(tile, 1, 1), false) && buildable(view, tile));
            fits.then(|| (centre(portal).distance_squared(toward), id, origin, portal))
        })
        .collect();
    hosts.sort_by(|a, b| {
        (a.0.total_cmp(&b.0))
            .then(a.1.cmp(&b.1))
            .then((a.2.y, a.2.x).cmp(&(b.2.y, b.2.x)))
    });
    let short = hosts.len() < to_come;
    (
        hosts
            .into_iter()
            .take(to_come)
            .map(|(_, _, _, portal)| portal)
            .collect(),
        short,
    )
}

/// What a Bio Portal warped in, unturned, on a Beacon whose footprint's top-left tile is `origin` would cover
/// (`sc_sim::transform::warp_in_footprint`: the Beacon's 2 x 2 and a column and a row to the bottom-right).
fn portal_footprint(origin: TilePos) -> Option<TileRect> {
    let p = warp_in_footprint(&deploy::site(origin), BioPortal, BuildingOrientation::default())?;
    Some(TileRect::new(p.origin, p.w, p.h))
}

/// The footprint of a Beacon whose top-left tile is `origin` (`sc_sim::deploy::site`).
fn beacon_footprint(origin: TilePos) -> TileRect {
    let p = deploy::site(origin);
    TileRect::new(p.origin, p.w, p.h)
}

/// Whether `rect` covers `tile`.
fn contains(rect: TileRect, tile: TilePos) -> bool {
    (rect.origin.x..rect.origin.x + rect.w as i32).contains(&tile.x)
        && (rect.origin.y..rect.origin.y + rect.h as i32).contains(&tile.y)
}

/// The tiles the field's patches and the Egress Node span, where the Drones work; `None` without a field.
fn field_box(base: &Base<'_>) -> Option<TileRect> {
    if base.field.is_empty() {
        return None;
    }
    let node = TilePos::containing(base.node);
    let tiles = (base.field.iter().map(|k| TilePos::containing(k.pos))).chain(std::iter::once(node));
    let (mut x0, mut y0, mut x1, mut y1) = (node.x, node.y, node.x, node.y);
    for tile in tiles {
        (x0, y0) = (x0.min(tile.x), y0.min(tile.y));
        (x1, y1) = (x1.max(tile.x), y1.max(tile.y));
    }
    let span = |from: i32, to: i32| u32::try_from(to - from + 1).unwrap_or(0);
    Some(TileRect::new(TilePos::new(x0, y0), span(x0, x1), span(y0, y1)))
}

/// Whether a building may stand on `tile` as far as the terrain goes: Buildable and flat (`core/map.md > Structures
/// only on flat terrain`).
fn buildable(view: &View<'_>, tile: TilePos) -> bool {
    let map = view.map();
    map.tile_preset(tile).is_some_and(|p| p.buildable) && map.is_flat(tile)
}

/// `rect` grown by `margin` tiles on every side.
fn grown(rect: TileRect, margin: i32) -> TileRect {
    let origin = TilePos::new(rect.origin.x - margin, rect.origin.y - margin);
    let grow = |len: u32| u32::try_from(len as i32 + 2 * margin).unwrap_or(0);
    TileRect::new(origin, grow(rect.w), grow(rect.h))
}

/// Whether two rectangles share a tile.
fn overlaps(a: TileRect, b: TileRect) -> bool {
    let span = |o: i32, len: u32| (o, o + len as i32);
    let ((ax0, ax1), (ay0, ay1)) = (span(a.origin.x, a.w), span(a.origin.y, a.h));
    let ((bx0, bx1), (by0, by1)) = (span(b.origin.x, b.w), span(b.origin.y, b.h));
    ax0 < bx1 && bx0 < ax1 && ay0 < by1 && by0 < ay1
}

/// The tiles of `rect`.
fn tiles(rect: TileRect) -> impl Iterator<Item = TilePos> {
    let (x0, y0) = (rect.origin.x, rect.origin.y);
    let (x1, y1) = (x0 + rect.w as i32, y0 + rect.h as i32);
    (y0..y1).flat_map(move |y| (x0..x1).map(move |x| TilePos::new(x, y)))
}

/// The centre of `rect`, space units.
fn centre(rect: TileRect) -> Vec2 {
    rect.origin.corner() + Vec2::new(rect.w as f32, rect.h as f32) * (SU_PER_GU / 2.0)
}
