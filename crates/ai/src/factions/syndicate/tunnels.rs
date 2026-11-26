//! The Syndicate's Tunnels (`factions/syndicate/tunnels.md`; `super` module docs, step 2): a new one, paid in Supplies,
//! for the Tunnel Space it adds (`factions/syndicate/tunnel_area_and_space.md > Tunnel Space`: "The only way to grow
//! both production capacity and army capacity simultaneously is building more Tunnels"), and where a Tunnel's
//! expansions go.
//!
//! - **When** ([`Tunnels::build`]): once the Supplies cover the next Tunnel's price (`View::build_price`:
//!   `factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, "Cost = current Tunnels owned"), one at a
//!   time, up to [`TUNNELS`].
//! - **Who**: a runner, the Agent nearest the site among those gathering Supplies; on a map without a station, any
//!   Agent once the player has more than its fields want (`super::Base::gatherers_wanted`).
//! - **Where** ([`site`]): the nearest spot the build check takes (`core/entities.md > Placement Validation >
//!   Worker-Built Structures`: the ghost "judges by what the player knows") to the point [`PRODUCER_SHARE`] of the way
//!   from the player's first Tunnel toward the staging point, unturned, with its Tier 1 Tunnel Area on the map and
//!   clear of the player's own Tunnels' (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap
//!   Rule`). Placeholder (not in the design: the design has no computer players' bases): the anchor, the order, and
//!   the search's budget, [`SITE_CHECKS`] build checks within [`SITE_REACH_GU`].
//! - **An expansion's spot** ([`expansion_site`]): the first origin of its Tunnel's area, row by row, that the
//!   expansion check takes, as the scripted openings' (`tests/pairings/openings.rs::expansion_site`). An underground
//!   expansion blocks no movement, and no unit blocks it (design change 0290), so any spot of the area serves.

use sc_sim::nav::TileRect;
use sc_sim::network::Place;
use sc_sim::placement::footprint_size;
use sc_sim::tunnel::area;
use sc_sim::{
    BuildingOrientation, Command, Object, ObjectId, ObjectKind, OrderKind, SU_PER_GU, Tick, TilePos, Vec2, gu,
};

use super::{Base, orders};
use crate::tuning::army::STAGING_CLEAR_GU;
use crate::tuning::build::PRODUCER_SHARE;
use crate::tuning::syndicate::{SITE_CHECKS, SITE_REACH_GU, SITE_RETRY_TICKS, TUNNELS};
use crate::{Turn, View};

use ObjectKind::{Agent, Tunnel};

/// The player's new Tunnels (module docs).
#[derive(Debug, Default)]
pub(super) struct Tunnels {
    /// The tick from which a search for a site may run again, after one found none.
    retry: Tick,
}

impl Tunnels {
    /// Has a runner build the player's next Tunnel, if it is due (module docs). Returns the builder.
    pub(super) fn build(&mut self, turn: &mut Turn<'_, '_>, base: &Base<'_>) -> Option<ObjectId> {
        let view = turn.view();
        let now = view.tick();
        if base.tunnels_owned >= TUNNELS || now < self.retry {
            return None;
        }
        let building = |kind| matches!(kind, OrderKind::Build { kind: Tunnel, .. });
        let agents: Vec<&Object> = view.own().filter(|o| o.kind == Agent).collect();
        let pending = view.own().any(|o| o.kind == Tunnel && o.is_site())
            || view.own_held().any(|h| matches!(h.place, Place::Site(_)))
            || agents.iter().any(|o| orders(o).any(building));
        if pending {
            return None;
        }
        let price = view.build_price(Tunnel)?;
        if price.amount > view.me()?.supplies {
            return None;
        }
        let first = base.tunnels.first()?;
        let anchor = turn
            .staging()
            .map_or(first.pos, |s| first.pos + (s - first.pos) * PRODUCER_SHARE);
        let runs = |o: &&Object| orders(o).any(|k| matches!(k, OrderKind::Gather(t) if Some(t) == base.station));
        let mut candidates: Vec<&Object> = agents.iter().copied().filter(runs).collect();
        if candidates.is_empty() && base.station.is_none() && base.agents > base.gatherers_wanted() {
            candidates = agents;
        }
        let builder = (candidates.into_iter()).min_by(|a, b| {
            (a.pos.distance_squared(anchor))
                .total_cmp(&b.pos.distance_squared(anchor))
                .then(a.id.cmp(&b.id))
        })?;
        let Some(origin) = site(turn, builder.id, anchor) else {
            self.retry = now + SITE_RETRY_TICKS;
            return None;
        };
        let command = Command::Order {
            units: vec![builder.id],
            kind: OrderKind::Build {
                kind: Tunnel,
                origin,
                orientation: BuildingOrientation::default(),
            },
            queue: false,
        };
        turn.give(command).then_some(builder.id)
    }
}

/// Where `agent` builds the player's next Tunnel, unturned (module docs): of the origins whose footprint's centre lies
/// within [`SITE_REACH_GU`] of `anchor`, nearest it first (ties to the north, then the west), whose Tier 1 Tunnel Area
/// lies on the map, clear of the areas of the player's own Tunnels, and whose footprint's centre lies at least
/// [`STAGING_CLEAR_GU`] from the staging point, the first the Agent's build check takes now. `None` if none is found
/// within [`SITE_CHECKS`] checks.
pub(super) fn site(turn: &Turn<'_, '_>, agent: ObjectId, anchor: Vec2) -> Option<TilePos> {
    let view = turn.view();
    let orientation = BuildingOrientation::default();
    let (w, h) = footprint_size(Tunnel, orientation)?;
    let radius = area::base_radius(Tunnel)?;
    let map = view.map();
    let bounds = TileRect::new(TilePos::new(0, 0), map.width(), map.height());
    let taken: Vec<TileRect> = view.own().filter_map(area::reserved).collect();
    let reach = gu(SITE_REACH_GU);
    let span = SITE_REACH_GU.ceil() as i32 + w.max(h) as i32;
    let clear = turn.staging().map(|s| (s, gu(STAGING_CLEAR_GU) * gu(STAGING_CLEAR_GU)));
    let at = TilePos::containing(anchor);
    let mut candidates: Vec<(f32, TilePos)> = Vec::new();
    for y in at.y - span..=at.y + span {
        for x in at.x - span..=at.x + span {
            let origin = TilePos::new(x, y);
            let centre = origin.corner() + Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0);
            let d2 = centre.distance_squared(anchor);
            if d2 > reach * reach || clear.is_some_and(|(staging, r2)| centre.distance_squared(staging) < r2) {
                continue;
            }
            let area = area::area(TileRect::new(origin, w, h), radius);
            if !area::encloses(bounds, area) || taken.iter().any(|&t| area::overlaps(t, area)) {
                continue;
            }
            candidates.push((d2, origin));
        }
    }
    candidates.sort_by(|a, b| (a.0.total_cmp(&b.0)).then(a.1.y.cmp(&b.1.y)).then(a.1.x.cmp(&b.1.x)));
    let takes =
        |origin| (view.build_check(agent, Tunnel, origin, orientation, false)).is_some_and(|check| check.is_valid());
    (candidates.into_iter().take(SITE_CHECKS))
        .map(|(_, origin)| origin)
        .find(|&origin| takes(origin))
}

/// Where `tunnel` builds an expansion of `kind`, unturned (module docs): the first origin of its Tunnel Area, row by
/// row (`tunnel::area::cells`), that the expansion check takes now.
pub(super) fn expansion_site(view: &View<'_>, tunnel: &Object, kind: ObjectKind) -> Option<TilePos> {
    let orientation = BuildingOrientation::default();
    let check = |origin| view.expansion_check(tunnel.id, kind, origin, orientation);
    area::cells(area::of(tunnel)?).find(|&origin| check(origin).is_some_and(|check| check.is_valid()))
}
