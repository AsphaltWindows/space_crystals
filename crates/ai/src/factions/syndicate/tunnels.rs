//! The Syndicate's Tunnels (`factions/syndicate/tunnels.md`; `super` module docs, step 2): a new one, paid in Supplies,
//! where its crystal hatch adds income (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule >
//! Crystal intake and stacked Tunnels`: the Syndicate "grows by adding Tunnels (paid in Supplies) rather than by
//! saturating one field"), else for the Tunnel Space it adds (`factions/syndicate/tunnel_area_and_space.md > Tunnel
//! Space`: "The only way to grow both production capacity and army capacity simultaneously is building more
//! Tunnels"), and where a Tunnel's expansions go.
//!
//! - **When** ([`Tunnels::build`]): once the Supplies cover the next Tunnel's price (`View::build_price`:
//!   `factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, "Cost = current Tunnels owned"), one at a
//!   time, up to [`TUNNELS`].
//! - **Who**: a runner, the Agent nearest the site among those gathering Supplies; on a map without a station, any
//!   Agent once the player has more than its fields want (`super::Base::gatherers_wanted`).
//! - **Where, for income** ([`income_sites`], slice A2d): a spot whose crystal hatch is nearer, by [`INCOME_MARGIN_GU`]
//!   at least, than every crystal hatch of the player's to patches with crystals left within [`INCOME_REACH_GU`] of
//!   it, so that the Agents mining them deliver there (`factions/syndicate/units/agent.md > Gathering > Gathering
//!   rules`: "the own operational Tunnel whose crystal hatch is **nearest to the Agent itself**"): patches of its own
//!   field, which its first hatch's Agents don't mine, or of another field on its side of the map (nearer its start
//!   than any other player's), "as a Syndicate player spreads" (the A2d brief; `factions/syndicate/
//!   tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule > Crystal intake and stacked Tunnels`: "Two Tier 1
//!   Tunnels may share a rich crystal field (doubling its intake)"). Each crystal hatch, the new one's and those of
//!   the Tunnels on a field, would take in the crystals left in the patches within [`FIELD_REACH_GU`] that are nearer
//!   it than any other: the spots that leave the poorest hatch the most first (a hatch that takes a whole field from
//!   another adds no income), then those nearest their patch, turned any of the four ways, with the square in front of
//!   the hatch open (Traversible, and no patch nor structure on it: an Agent unloads standing there, "within 8 su of
//!   the crystal hatch"), the first the build check takes. Placeholder (not in the design: the design has no computer
//!   players' bases).
//! - **Else, for Tunnel Space** ([`site`]): the nearest spot the build check takes (`core/entities.md > Placement
//!   Validation > Worker-Built Structures`: the ghost "judges by what the player knows") to the point
//!   [`PRODUCER_SHARE`] of the way from the player's first Tunnel toward the staging point, unturned. Placeholder
//!   (not in the design): the anchor, the order, and the search's budget, [`SITE_CHECKS`] build checks within
//!   [`SITE_REACH_GU`].
//!
//! Every spot has its Tier 1 Tunnel Area on the map and clear of the player's own Tunnels'
//! (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`), and its footprint's centre
//! [`STAGING_CLEAR_GU`] from the staging point at least.
//!
//! - **An expansion's spot** ([`expansion_site`]): the first origin of its Tunnel's area, row by row, that the
//!   expansion check takes, as the scripted openings' (`tests/pairings/openings.rs::expansion_site`). An underground
//!   expansion blocks no movement, and no unit blocks it (design change 0290), so any spot of the area serves.

use std::collections::BTreeSet;

use sc_sim::economy::drop_off::{self, Resource};
use sc_sim::nav::TileRect;
use sc_sim::network::Place;
use sc_sim::placement::footprint_size;
use sc_sim::sides::{DIRECTIONS, sides};
use sc_sim::tunnel::area;
use sc_sim::{
    BuildingOrientation, Command, Object, ObjectId, ObjectKind, OrderKind, Placement, Rotation, SU_PER_GU, Tick,
    TilePos, Vec2, gu,
};

use super::{Base, orders};
use crate::tuning::army::STAGING_CLEAR_GU;
use crate::tuning::build::PRODUCER_SHARE;
use crate::tuning::syndicate::{
    FIELD_REACH_GU, INCOME_MARGIN_GU, INCOME_REACH_GU, SITE_CHECKS, SITE_REACH_GU, SITE_RETRY_TICKS, TUNNELS,
};
use crate::{Turn, View};

use ObjectKind::{Agent, Tunnel};

/// The four ways a new Tunnel may be turned, for income ([`income_sites`]).
const TURNS: [Rotation; 4] = [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270];

/// The player's new Tunnels (module docs).
#[derive(Debug, Default)]
pub(super) struct Tunnels {
    /// The tick from which a search for a site may run again, after one found none.
    retry: Tick,
}

/// A spot for a new Tunnel: its footprint's top-left tile and how it is turned.
type Spot = (TilePos, BuildingOrientation);

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
        let toward_staging = turn
            .staging()
            .map_or(first.pos, |s| first.pos + (s - first.pos) * PRODUCER_SHARE);
        let income = income_sites(turn, base);
        let anchor = income.first().map_or(toward_staging, |&(spot, _)| spot);
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
        let takes = |&(origin, orientation): &Spot| {
            (view.build_check(builder.id, Tunnel, origin, orientation, false)).is_some_and(|check| check.is_valid())
        };
        let spot = (income.iter().map(|&(_, spot)| spot).take(SITE_CHECKS))
            .find(takes)
            .or_else(|| Some((site(turn, builder.id, toward_staging)?, BuildingOrientation::default())));
        let Some((origin, orientation)) = spot else {
            self.retry = now + SITE_RETRY_TICKS;
            return None;
        };
        let command = Command::Order {
            units: vec![builder.id],
            kind: OrderKind::Build {
                kind: Tunnel,
                origin,
                orientation,
            },
            queue: false,
        };
        turn.give(command).then_some(builder.id)
    }
}

/// The spots where a new Tunnel adds crystal income (module docs), best first, each with its footprint's centre;
/// none passed through a build check yet. A spot wins the patches its crystal hatch is nearer than the player's hatch
/// nearest them now, within [`FIELD_REACH_GU`] (`super::Field`), and must win one of them by the margin; each field
/// keeps the rest of its own. The spots whose poorest hatch, of the fields' and its own, is left the most crystals
/// first, so that no field is left without patches, then the nearest patch won by the margin, then to the north, then
/// the west, then by [`TURNS`]' order.
fn income_sites(turn: &Turn<'_, '_>, base: &Base<'_>) -> Vec<(Vec2, Spot)> {
    let view = turn.view();
    let Some(letter) = drop_off::rule(Tunnel).and_then(|rule| rule.hatch(Resource::Crystals)) else {
        return Vec::new();
    };
    let Some(radius) = area::base_radius(Tunnel) else {
        return Vec::new();
    };
    let map = view.map();
    let bounds = TileRect::new(TilePos::new(0, 0), map.width(), map.height());
    let taken: Vec<TileRect> = view.own().filter_map(area::reserved).collect();
    let clear = turn.staging().map(|s| (s, gu(STAGING_CLEAR_GU) * gu(STAGING_CLEAR_GU)));
    let (reach, margin, field) = (gu(INCOME_REACH_GU), gu(INCOME_MARGIN_GU), gu(FIELD_REACH_GU));
    // The patches on the player's side of the map, with crystals left, each with the hatch of the player's nearest it
    // (by its Tunnel), and how near.
    let mine = |pos: Vec2| {
        let d = |tile: TilePos| tile.center().distance_squared(pos);
        let own = view.my_start().map(|s| d(s.tile));
        own.is_some_and(|own| (view.starts().iter()).all(|s| s.player == view.player() || own < d(s.tile)))
    };
    let patches: Vec<(Vec2, u32, Option<ObjectId>, f32)> = (view.patches())
        .filter_map(|k| Some((k.pos, k.crystals_left().filter(|&left| left > 0)?)))
        .filter(|&(pos, _)| mine(pos))
        .map(|(pos, left)| {
            let nearest = (base.hatches.iter())
                .map(|&(tunnel, hatch)| (hatch.distance(pos), tunnel))
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            (pos, left, nearest.map(|n| n.1), nearest.map_or(f32::INFINITY, |n| n.0))
        })
        .collect();
    let blocked: BTreeSet<(i32, i32)> = (view.known().iter())
        .filter(|k| k.crystals_left().is_some() || k.is_structure())
        .filter_map(|k| k.placement)
        .chain(view.own().filter_map(|o| o.placement))
        .flat_map(|p| p.tiles().map(|t| (t.x, t.y)).collect::<Vec<_>>())
        .collect();
    let open = |tile: TilePos| {
        !blocked.contains(&(tile.x, tile.y)) && map.tile_preset(tile).is_some_and(|preset| preset.traversible)
    };
    let mut origins: BTreeSet<(i32, i32)> = BTreeSet::new();
    let span = INCOME_REACH_GU.ceil() as i32 + 4;
    for &(pos, ..) in &patches {
        let at = TilePos::containing(pos);
        for y in at.y - span..=at.y + span {
            for x in at.x - span..=at.x + span {
                origins.insert((y, x));
            }
        }
    }
    let mut spots: Vec<(u32, f32, i32, i32, usize, Vec2, Spot)> = Vec::new();
    for &(y, x) in &origins {
        for (turned, &rotation) in TURNS.iter().enumerate() {
            let orientation = BuildingOrientation {
                rotation,
                flip_h: false,
                flip_v: false,
            };
            let Some((w, h)) = footprint_size(Tunnel, orientation) else {
                continue;
            };
            let origin = TilePos::new(x, y);
            let footprint = TileRect::new(origin, w, h);
            let reserved = area::area(footprint, radius);
            if !area::encloses(bounds, reserved) || taken.iter().any(|&t| area::overlaps(t, reserved)) {
                continue;
            }
            let centre = origin.corner() + Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0);
            if clear.is_some_and(|(staging, r2)| centre.distance_squared(staging) < r2) {
                continue;
            }
            let placement = Placement {
                origin,
                w,
                h,
                rotation,
                flip_h: false,
                flip_v: false,
            };
            let Some(hatch) = drop_off::hatch_tile(Tunnel, &placement, letter) else {
                continue;
            };
            let facing = sides(Tunnel, orientation).and_then(|letters| letters.iter().position(|&l| l == letter));
            let Some(&(dx, dy)) = facing.and_then(|i| DIRECTIONS.get(i)) else {
                continue;
            };
            if !open(TilePos::new(hatch.x + dx as i32, hatch.y + dy as i32)) {
                continue;
            }
            // What it wins: the patches it is nearer than the hatch nearest them now, within a field's reach, and
            // the nearest of those it wins by the margin, within reach of income. What each field keeps: the rest.
            let at = hatch.center();
            let (mut won, mut best) = (0u32, f32::INFINITY);
            let mut kept: Vec<u32> = vec![0; base.fields.len()];
            for &(pos, left, owner, nearest) in &patches {
                let d = at.distance(pos);
                if d < nearest && d <= field {
                    won = won.saturating_add(left);
                    if d <= reach && d + margin <= nearest {
                        best = best.min(d);
                    }
                } else if nearest <= field
                    && let Some(i) = owner.and_then(|t| base.fields.iter().position(|f| f.tunnel == t))
                {
                    kept[i] = kept[i].saturating_add(left);
                }
            }
            let poorest = kept.iter().copied().fold(won, u32::min);
            if poorest == 0 || !best.is_finite() {
                continue;
            }
            spots.push((poorest, best, y, x, turned, centre, (origin, orientation)));
        }
    }
    spots.sort_by(|a, b| {
        (b.0.cmp(&a.0))
            .then(a.1.total_cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
            .then(a.4.cmp(&b.4))
    });
    (spots.into_iter()).map(|s| (s.5, s.6)).collect()
}

/// Where `agent` builds the player's next Tunnel for Tunnel Space, unturned (module docs): of the origins whose
/// footprint's centre lies within [`SITE_REACH_GU`] of `anchor`, nearest it first (ties to the north, then the west),
/// whose Tier 1 Tunnel Area lies on the map, clear of the areas of the player's own Tunnels, and whose footprint's
/// centre lies at least [`STAGING_CLEAR_GU`] from the staging point, the first the Agent's build check takes now.
/// `None` if none is found within [`SITE_CHECKS`] checks.
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
