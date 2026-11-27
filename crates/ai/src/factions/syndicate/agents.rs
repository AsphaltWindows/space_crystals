//! The Syndicate's Agents (`factions/syndicate/units/agent.md > Gathering`; `super` module docs, step 1): out of the
//! Tunnel Network, to their jobs, and the Hideouts' rally point for the Agent each finishes next.
//!
//! - **Out of the network** ([`eject`]): an Agent free in the Tunnel Network (one that finished a Tunnel,
//!   `factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`: "the Agent is inside the Tunnel
//!   Network, available for redeployment from any Tunnel") comes out of the Tunnel of the field that has the fewest
//!   Agents ([`neediest`]), and a unit that fights out of the Tunnel nearest the staging point
//!   (`factions/syndicate/tunnel_menus.md > EjectMenu`).
//! - **Jobs** ([`Agents::jobs`]): an Agent standing idle on the map gathers Supplies at the station while the player
//!   has fewer runners than it wants and its fields have the Agents they want, else Space Crystals at the patch of the
//!   field that has the fewest Agents (`super::Field`). A runner no longer wanted goes to a field, and so does one
//!   Agent a tick of a field that has more than [`AGENTS_PER_HATCH`] while another has fewer (slice A2d: the Agents of
//!   a field whose patches ran out find the next patches nearest them, which may be another field's).
//! - **The rally point** ([`rally`]): while an Agent is at the front of a Hideout's queue, the Hideout rallies on its
//!   job's target, which the Agent gathers as it comes out (`factions/syndicate/structures/README.md > Tunnel
//!   Expansions > Rally Point Behavior`: "e.g. an Agent rallied on a crystal patch gathers"): the patch of the field
//!   that has the fewest Agents while the fields want Agents, else the station.

use std::collections::BTreeMap;

use sc_sim::{Command, ObjectId, ObjectKind, OrderKind, ProductionCommand, Rally, Target, Tick, TunnelCommand, Vec2};

use super::{Base, fighter, front, gather_of, hideouts, orders};
use crate::Turn;
use crate::tuning::syndicate::{AGENTS_PER_HATCH, JOB_RETRY_TICKS};

use ObjectKind::Agent;

/// The Agents' jobs (module docs): when each Agent given one may be given another.
#[derive(Debug, Default)]
pub(super) struct Agents {
    /// For each Agent given a job, the tick from which it may be given another if it stands idle.
    next: BTreeMap<ObjectId, Tick>,
}

impl Agents {
    /// Gives every Agent idle on the map, but `builder` (given a Build this tick), its job (module docs): the fields'
    /// first, then the runners'; and moves a runner no longer wanted, and one Agent of a field with too many, to the
    /// field that has the fewest.
    pub(super) fn jobs(&mut self, turn: &mut Turn<'_, '_>, base: &Base<'_>, builder: Option<ObjectId>) {
        let view = turn.view();
        let now = view.tick();
        self.next.retain(|_, &mut from| now < from);
        let (mut gatherers, mut runners) = (base.gatherers, base.runners);
        let mut staff: Vec<usize> = base.fields.iter().map(|f| f.staff).collect();
        let mut moved = false;
        let agents = (view.own()).filter(|o| o.kind == Agent && Some(o.id) != builder);
        for agent in agents {
            if self.next.contains_key(&agent.id) {
                continue;
            }
            let idle = agent.orders.as_ref().is_some_and(|o| o.is_idle());
            let station = |kind| matches!(kind, OrderKind::Gather(target) if Some(target) == base.station);
            let surplus = runners > base.runners_wanted && orders(agent).any(station);
            let from = orders(agent).find_map(gather_of).and_then(|patch| base.field_of(patch));
            let crowded = !moved
                && from.is_some_and(|i| staff[i] > AGENTS_PER_HATCH)
                && staff.iter().any(|&s| s < AGENTS_PER_HATCH);
            if !idle && !surplus && !crowded {
                continue;
            }
            let to_station = idle && gatherers >= base.gatherers_wanted() && runners < base.runners_wanted;
            let field = neediest(base, &staff, agent.pos);
            let target = match to_station {
                true => base.station,
                false => field.map(|i| base.fields[i].patch),
            };
            let Some(target) = target else {
                continue;
            };
            if !to_station && crowded && field == from {
                continue;
            }
            let command = Command::Order {
                units: vec![agent.id],
                kind: OrderKind::Gather(target),
                queue: false,
            };
            if !turn.give(command) {
                continue;
            }
            self.next.insert(agent.id, now + JOB_RETRY_TICKS);
            if to_station {
                runners += 1;
                continue;
            }
            if let Some(i) = field {
                staff[i] += 1;
            }
            match from.filter(|_| crowded) {
                Some(i) => {
                    staff[i] -= 1;
                    moved = true;
                }
                None => {
                    gatherers += 1;
                    runners -= usize::from(surplus);
                }
            }
        }
    }
}

/// The field that has the fewest Agents of `staff` (each field's, in `base`'s order), the one whose patch is nearest
/// `pos` among equals, then the first; `None` if the player has no field.
fn neediest(base: &Base<'_>, staff: &[usize], pos: Vec2) -> Option<usize> {
    (base.fields.iter().enumerate())
        .min_by(|(i, a), (j, b)| {
            (staff[*i].cmp(&staff[*j]))
                .then(a.at.distance_squared(pos).total_cmp(&b.at.distance_squared(pos)))
                .then(i.cmp(j))
        })
        .map(|(i, _)| i)
}

/// Every unit free in the Tunnel Network comes out (module docs): an Agent by the Tunnel of the field that has the
/// fewest Agents (else the first Tunnel in operation), a unit that fights by the Tunnel nearest the staging point. One
/// of each kind a tick.
pub(super) fn eject(turn: &mut Turn<'_, '_>, base: &Base<'_>) {
    let view = turn.view();
    let counts = view.network_counts();
    let first = base.tunnels.first().map(|t| t.id);
    let staff: Vec<usize> = base.fields.iter().map(|f| f.staff).collect();
    for (&kind, &free) in &counts {
        if free == 0 {
            continue;
        }
        let tunnel = if kind == Agent {
            let home = base.tunnels.first().map_or(Vec2::ZERO, |t| t.pos);
            neediest(base, &staff, home).map(|i| base.fields[i].tunnel).or(first)
        } else if fighter(kind) {
            match turn.staging() {
                Some(staging) => (base.tunnels.iter())
                    .min_by(|a, b| {
                        (a.pos.distance_squared(staging))
                            .total_cmp(&b.pos.distance_squared(staging))
                            .then(a.id.cmp(&b.id))
                    })
                    .map(|t| t.id),
                None => first,
            }
        } else {
            None
        };
        let Some(tunnel) = tunnel else {
            continue;
        };
        turn.give(Command::Tunnel {
            tunnel,
            command: TunnelCommand::Eject { kind },
        });
    }
}

/// Each Hideout whose next unit is an Agent rallies on that Agent's job's target (module docs): the patch of the field
/// that has the fewest Agents, the nearest the Hideout among equals, while the player's Agents but its runners (those
/// idle included, which go to the fields first) are fewer than its fields want, or it has the runners it wants; else
/// the station. Only if it rallies elsewhere.
pub(super) fn rally(turn: &mut Turn<'_, '_>, base: &Base<'_>) {
    let view = turn.view();
    let fields_want = base.agents.saturating_sub(base.runners) < base.gatherers_wanted();
    let staff: Vec<usize> = base.fields.iter().map(|f| f.staff).collect();
    let producers: Vec<_> = hideouts(view).filter(|o| front(o) == Some(Agent)).collect();
    for hideout in producers {
        let target = if fields_want || base.runners >= base.runners_wanted {
            neediest(base, &staff, hideout.pos).map(|i| base.fields[i].patch)
        } else {
            base.station
        };
        let Some(target) = target else {
            continue;
        };
        let rally = hideout.production.as_ref().and_then(|p| p.rally);
        if matches!(rally, Some(Rally::Object { id, .. }) if id == target) {
            continue;
        }
        turn.give(Command::Production {
            objects: vec![hideout.id],
            command: ProductionCommand::SetRally(Target::Object(target)),
        });
    }
}
