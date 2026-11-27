//! The Syndicate's computer player (`factions/syndicate/overview.md`; `core/game_rules.md > Computer players (playtest
//! tool)`: its behavior "is the Director's placeholder"). From the Syndicate's Starting Conditions
//! (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`: "One pre-built Tier 1 Tunnel containing one pre-built
//! Hideout", "One Agent", "1 Supply (enough for the 2nd Tunnel, which costs 1 Supply)"), each tick:
//!
//! 1. **The Agents** ([`agents`]): every Agent free in the Tunnel Network comes out, and every Agent idle on the map
//!    gets a job: Space Crystals at the patch nearest a crystal hatch, [`AGENTS_PER_HATCH`] for each Tunnel on a field
//!    (design change 0294: "3 Agents saturate out to about 3 GU"), or Supplies at a Supply Delivery Station, the
//!    [`RUNNERS`]' job. An Agent the Hideout trains goes to its job by the Hideout's rally point.
//! 2. **A Tunnel** ([`tunnels`]), paid in Supplies: once the Supplies cover the next one, a runner builds it toward the
//!    staging point, up to [`TUNNELS`], for the Tunnel Space it adds (`factions/syndicate/overview.md > TheSyndicate`:
//!    "Army size is limited by Tunnel Space [...] building more Tunnels/upgrades grows it").
//!
//! Then, in this order of priority for the bank:
//!
//! 3. **Agents**: the first Hideout trains one while the player has fewer than it wants.
//! 4. **The expansions** of [`EXPANSIONS`]: a Hideout and an Optics Lab, one of each, built by the first Tunnel that may
//!    (`factions/syndicate/tunnel_menus.md > AwaitingPlacement (Expansion)`), the Optics Lab being the Guard's
//!    prerequisite (`factions/syndicate/structures/hideout.md > Produces`).
//! 5. **Guards** (`factions/syndicate/units/guard.md`): each Hideout trains them without pause, [`TRAIN_QUEUE`] queued,
//!    while Tunnel Space has room for them (`factions/syndicate/tunnel_area_and_space.md > Tunnel Space > Production
//!    and Tunnel Space`).
//!
//! A step that its bank can't pay for yet holds the bank: the steps after it wait. **Released units**: a Hideout makes
//! both the workers and the army, and has one rally point (`factions/syndicate/structures/README.md > Tunnel
//! Expansions > Rally Point Behavior`: "Rally point set on the surface: Unit auto-ejects from the parent Tunnel (Side
//! A) and moves to the rally point"). So the rally point follows the unit at the front of its queue, the one finished
//! next: an Agent's job's patch or station while an Agent is at the front ([`agents`]), the army's staging point while a
//! Guard is ([`FactionAi::rallies`]). The army layer (`crate::army`) leads the Guards, in waves of its own size,
//! [`WAVE_SIZE`] ([`FactionAi::waves`]). Every rule and number is a placeholder (Q0147 points 2 and 3,
//! `crate::tuning::syndicate`). Costs, times and Tunnel Space come from the defs.

mod agents;
mod tunnels;

use sc_sim::construction::site::expansion_def;
use sc_sim::economy::drop_off::{self, Resource};
use sc_sim::{
    BuildingOrientation, Command, Object, ObjectId, ObjectKind, OrderKind, ProductionCommand, TunnelCommand, Vec2, gu,
};

use super::{FactionAi, Hook};
use crate::army::Waves;
use crate::tuning::syndicate::{
    AGENTS_PER_HATCH, EXPANSIONS, FIELD_REACH_GU, RUNNERS, TRAIN_QUEUE, TUNNELS, WAVE_PATIENCE_TICKS, WAVE_SIZE,
};
use crate::{Profile, Turn, View};

use ObjectKind::{Agent, Grenadier, Guard, Hideout, SupplyDeliveryStation, Tunnel};

pub(super) const HOOK: Option<Hook> = Some(new);

fn new(_view: &View<'_>, _profile: Profile) -> Box<dyn FactionAi> {
    Box::new(Syndicate::default())
}

/// The Syndicate's computer player (module docs).
#[derive(Debug, Default)]
struct Syndicate {
    agents: agents::Agents,
    tunnels: tunnels::Tunnels,
}

impl FactionAi for Syndicate {
    fn think(&mut self, turn: &mut Turn<'_, '_>) {
        let base = Base::of(turn.view());
        agents::eject(turn, &base);
        let builder = self.tunnels.build(turn, &base);
        self.agents.jobs(turn, &base, builder);
        agents::rally(turn, &base);
        if train_agents(turn, &base) && expand(turn) {
            train_army(turn);
        }
    }

    fn fights(&self, unit: &Object) -> bool {
        fighter(unit.kind)
    }

    /// A Hideout whose next unit fights (module docs, "Released units").
    fn rallies(&self, building: &Object) -> bool {
        building.kind == Hideout && front(building).is_some_and(fighter)
    }

    /// Waves of [`WAVE_SIZE`], the first too, or however many have waited [`WAVE_PATIENCE_TICKS`] (module docs).
    fn waves(&self) -> Waves {
        Waves {
            first: WAVE_SIZE,
            later: WAVE_SIZE,
            patience: WAVE_PATIENCE_TICKS,
        }
    }
}

/// Whether a unit of `kind` fights: the Syndicate's infantry but its workers.
fn fighter(kind: ObjectKind) -> bool {
    matches!(kind, Guard | Grenadier)
}

/// The kind of unit at the front of `producer`'s queue, the one it finishes next.
fn front(producer: &Object) -> Option<ObjectKind> {
    producer.production.as_ref()?.queue.first()?.unit()
}

/// The player's base as the latest tick left it: its Tunnels and fields, and its Agents' jobs.
struct Base<'a> {
    /// Its Tunnels in operation, by id.
    tunnels: Vec<&'a Object>,
    /// Its fields, in its Tunnels' order.
    fields: Vec<Field>,
    /// The Supply Delivery Station nearest its first Tunnel (ties by id), if it knows one: all the map placed.
    station: Option<ObjectId>,
    /// Its Agents alive: on the map, in the Tunnel Network, or building a site.
    agents: usize,
    /// Its runners: the Agents gathering Supplies or building a Tunnel, ordered to, or coming out of the network to.
    runners: usize,
    /// Its other Agents but those idle on the map: gathering Space Crystals, or in the network.
    gatherers: usize,
    /// The runners it wants ([`RUNNERS`]): while it may still build a Tunnel ([`TUNNELS`]) and there are Supplies to
    /// gather, or to spend on one.
    runners_wanted: usize,
    /// Its Tunnels, sites and Builds ordered included.
    tunnels_owned: usize,
}

/// A Tunnel on a field: one in operation whose crystal hatch is the nearest hatch to a patch with crystals left within
/// [`FIELD_REACH_GU`] of it (module docs), centre to centre, so that the patch's Agents deliver to it.
struct Field {
    tunnel: ObjectId,
    /// The centre of its crystal hatch's square.
    hatch: Vec2,
    /// The nearest of those patches to the hatch, ties by id: where its Agents gather.
    patch: ObjectId,
}

impl<'a> Base<'a> {
    fn of(view: &View<'a>) -> Self {
        let mut tunnels: Vec<&'a Object> = (view.own())
            .filter(|o| o.kind == Tunnel && o.is_operational())
            .collect();
        tunnels.sort_by_key(|o| o.id);
        let hatches: Vec<(ObjectId, Vec2)> = (tunnels.iter())
            .filter_map(|t| Some((t.id, drop_off::hatch_square(t, Resource::Crystals)?.center())))
            .collect();
        // Each patch with crystals left, within reach of a hatch, goes to the nearest (ties by id): the one its Agents
        // deliver to (`factions/syndicate/units/agent.md > Gathering > Gathering rules`, "Which Tunnel": "the own
        // operational Tunnel whose crystal hatch is nearest").
        let reach = gu(FIELD_REACH_GU) * gu(FIELD_REACH_GU);
        let mut nearest: Vec<(ObjectId, f32, ObjectId)> = Vec::new();
        for k in view
            .patches()
            .filter(|k| k.crystals_left().is_some_and(|left| left > 0))
        {
            let hatch = (hatches.iter())
                .map(|&(tunnel, at)| (at.distance_squared(k.pos), tunnel))
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            if let Some((d, tunnel)) = hatch.filter(|&(d, _)| d <= reach) {
                nearest.push((tunnel, d, k.id));
            }
        }
        let fields = (hatches.iter())
            .filter_map(|&(tunnel, hatch)| {
                let (_, _, patch) = (nearest.iter())
                    .filter(|n| n.0 == tunnel)
                    .min_by(|a, b| a.1.total_cmp(&b.1).then(a.2.cmp(&b.2)))?;
                Some(Field {
                    tunnel,
                    hatch,
                    patch: *patch,
                })
            })
            .collect();
        let home = tunnels.first().map(|t| t.pos);
        let station = (view.known().iter())
            .filter(|k| k.kind == SupplyDeliveryStation)
            .min_by(|a, b| {
                let d = |p: Vec2| home.map_or(0.0, |h| p.distance_squared(h));
                d(a.pos).total_cmp(&d(b.pos)).then(a.id.cmp(&b.id))
            })
            .map(|k| k.id);
        let held = view.own_held().map(|h| &h.object);
        let agents = (view.own().chain(held)).filter(|o| o.kind == Agent).count();
        let stations: Vec<ObjectId> = (view.known().iter())
            .filter(|k| k.kind == SupplyDeliveryStation)
            .map(|k| k.id)
            .collect();
        let runs = |kind: OrderKind| match kind {
            OrderKind::Gather(target) => stations.contains(&target),
            OrderKind::Build { kind, .. } => kind == Tunnel,
            _ => false,
        };
        let on_map = (view.own()).filter(|o| o.kind == Agent && orders(o).any(runs)).count();
        let in_sites = (view.own_held())
            .filter(|h| h.object.kind == Agent && matches!(h.place, sc_sim::network::Place::Site(_)))
            .count();
        let coming_out = (view.own())
            .filter_map(|o| o.tunnel.as_ref())
            .flat_map(|t| t.eject.iter())
            .filter(|e| e.order.is_some_and(runs))
            .count();
        let ordered = (view.own())
            .filter(|o| o.kind == Agent)
            .flat_map(orders)
            .filter(|&k| matches!(k, OrderKind::Build { kind: Tunnel, .. }))
            .count();
        let tunnels_owned = view.own().filter(|o| o.kind == Tunnel).count() + ordered;
        let supplies = view.me().map_or(0, |p| p.supplies);
        let affordable = view.build_price(Tunnel).is_some_and(|price| price.amount <= supplies);
        let runners_wanted = if tunnels_owned < TUNNELS && (station.is_some() || affordable) {
            RUNNERS
        } else {
            0
        };
        let runners = on_map + in_sites + coming_out;
        let idle = (view.own())
            .filter(|o| o.kind == Agent && o.orders.as_ref().is_some_and(|o| o.is_idle()))
            .count();
        Self {
            tunnels,
            fields,
            station,
            agents,
            runners,
            gatherers: agents.saturating_sub(runners + idle),
            runners_wanted,
            tunnels_owned,
        }
    }

    /// The Agents its fields want: [`AGENTS_PER_HATCH`] each.
    fn gatherers_wanted(&self) -> usize {
        AGENTS_PER_HATCH * self.fields.len()
    }

    /// The Agents it wants: its fields', and its runners.
    fn agents_wanted(&self) -> usize {
        self.gatherers_wanted() + self.runners_wanted
    }

    /// The field nearest `at` (its hatch), ties by its Tunnel's id.
    fn field_near(&self, at: Vec2) -> Option<&Field> {
        (self.fields.iter()).min_by(|a, b| {
            (a.hatch.distance_squared(at))
                .total_cmp(&b.hatch.distance_squared(at))
                .then(a.tunnel.cmp(&b.tunnel))
        })
    }
}

/// The kinds of `o`'s orders, the current one first.
fn orders(o: &Object) -> impl Iterator<Item = OrderKind> + '_ {
    let orders = o.orders.as_ref();
    let current = orders.and_then(|o| o.current.as_ref()).map(|a| a.order.kind);
    let queued = orders.into_iter().flat_map(|o| o.queue.iter().map(|q| q.kind));
    current.into_iter().chain(queued)
}

/// 3: the first Hideout trains an Agent while the player has fewer, alive and queued, than it wants, and Tunnel Space
/// has room for one. Returns whether the steps after it may spend: `false` when it waits for the bank.
fn train_agents(turn: &mut Turn<'_, '_>, base: &Base<'_>) -> bool {
    let view = turn.view();
    let hideouts: Vec<&Object> = hideouts(view).collect();
    let queued: usize = (hideouts.iter())
        .filter_map(|o| o.production.as_ref())
        .map(|p| p.queue.iter().filter(|e| e.unit() == Some(Agent)).count())
        .sum();
    if base.agents + queued >= base.agents_wanted() {
        return true;
    }
    let Some(hideout) =
        (hideouts.into_iter()).find(|o| o.production.as_ref().is_some_and(|p| p.queue.len() < TRAIN_QUEUE))
    else {
        return true;
    };
    if !room(view, Agent) {
        return true;
    }
    let Some(price) = trains(hideout, Agent) else {
        return true;
    };
    if !turn.can_pay(price) {
        return false;
    }
    turn.buy(price, train(hideout.id, Agent));
    true
}

/// 4: the expansions of [`EXPANSIONS`] the player has none of, complete or under construction, each built by the first
/// Tunnel (by id) that may build it now (`View::can_expand`: in operation, with no operation under way, its tier and
/// prerequisites met), at the first spot of its area that the expansion check takes. Returns whether the steps after
/// it may spend: `false` when one waits for the bank.
fn expand(turn: &mut Turn<'_, '_>) -> bool {
    let view = turn.view();
    let mut busy: Vec<ObjectId> = Vec::new();
    for kind in EXPANSIONS {
        if view.own().any(|o| o.kind == kind) {
            continue;
        }
        let Some(cost) = expansion_def(kind).map(|def| def.cost) else {
            continue;
        };
        let mut tunnels: Vec<&Object> = (view.own())
            .filter(|o| o.kind == Tunnel && !busy.contains(&o.id) && view.can_expand(o.id, kind).is_ok())
            .collect();
        if tunnels.is_empty() {
            continue;
        }
        if !turn.can_pay(cost) {
            return false;
        }
        tunnels.sort_by_key(|o| o.id);
        let site = |t: &Object| Some((t.id, tunnels::expansion_site(view, t, kind)?));
        let Some((tunnel, origin)) = tunnels.into_iter().find_map(site) else {
            continue;
        };
        let command = Command::Tunnel {
            tunnel,
            command: TunnelCommand::Expand {
                kind,
                origin,
                orientation: BuildingOrientation::default(),
            },
        };
        if turn.buy(cost, command) {
            busy.push(tunnel);
        }
    }
    true
}

/// 5: keeps [`TRAIN_QUEUE`] Guards queued at each Hideout, while Tunnel Space has room for them.
fn train_army(turn: &mut Turn<'_, '_>) {
    let view = turn.view();
    let hideouts: Vec<&Object> = hideouts(view).collect();
    for hideout in hideouts {
        let Some(production) = &hideout.production else {
            continue;
        };
        if production.queue.len() >= TRAIN_QUEUE || !room(view, Guard) {
            continue;
        }
        let Some(price) = trains(hideout, Guard) else {
            continue;
        };
        turn.buy(price, train(hideout.id, Guard));
    }
}

/// The player's Hideouts that produce, complete, by id.
fn hideouts<'a>(view: &View<'a>) -> impl Iterator<Item = &'a Object> + 'a {
    (view.own()).filter(|o| o.kind == Hideout && o.is_operational() && o.production.is_some())
}

/// Whether Tunnel Space has room for one more `kind` once every unit queued has started (at most: the units started
/// are counted), as the GDO's Unit Control does (`crate::factions::gdo`).
fn room(view: &View<'_>, kind: ObjectKind) -> bool {
    let Some(cap) = view.army_cap() else {
        return true;
    };
    let queued: u32 = (view.own())
        .filter_map(|o| o.production.as_ref())
        .map(|p| p.queue.len() as u32)
        .sum();
    let cost = cap.resource.cost_of(kind);
    cap.used.saturating_add(queued).saturating_add(cost) <= cap.available
}

/// What `producer` pays for a `kind`, if it trains them.
fn trains(producer: &Object, kind: ObjectKind) -> Option<u32> {
    let production = producer.kind.def().structure.as_ref()?.production.as_ref()?;
    production.recipe(kind).map(|r| r.cost)
}

fn train(by: ObjectId, kind: ObjectKind) -> Command {
    Command::Production {
        objects: vec![by],
        command: ProductionCommand::Train(kind),
    }
}
