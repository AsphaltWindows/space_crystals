//! (M6-GD) The drone invariant D1 (`crate::stock`, `crate::tether`), as the tick leaves the state:
//! - **A stock**: an object has one exactly when its type has one (`UnitDef::stock`), a held unit as one on the map; it
//!   holds at most its max, and buys only while it holds fewer, its purchase's frames not all done (a done one has
//!   added its item).
//! - **A drone and its operator name each other** (`Object::tether`, `Object::drone`), both on the map, the drone of a
//!   tethered type.
//! - **A drone is within its leash**, edge to edge, and its battery's tick is still to come: the tether's step, after
//!   phase 5d, the last move of the tick, made those whose end came fall.
//! - **An operator with its drone out stood still in the tick** ("The Drone Operator **cannot move** while it has a
//!   drone active"; `combat::may_move`), where the checker saw it a tick earlier, unless a debug command teleported it.

use std::collections::BTreeMap;

use super::{Seen, Violation};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::sim::Sim;
use crate::{stock, tether};

/// D1 (module docs): `seen` is what the checker saw a tick earlier, `teleported` the units a debug command moved in the
/// tick.
pub(super) fn tethers(sim: &Sim, seen: &BTreeMap<ObjectId, Seen>, teleported: &[ObjectId]) -> Result<(), Violation> {
    let held = sim.held().map(|held| &held.object);
    for o in sim.objects().chain(held) {
        stocked(o)?;
    }
    for o in sim.objects() {
        let fail = |message: String| {
            Err(Violation {
                invariant: "D1",
                message: format!("#{} ({:?}) {message}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        if let Some(drone) = o.drone {
            let names = sim.object(drone).and_then(|d| d.tether).map(|t| t.operator);
            if names != Some(o.id) {
                return fail(format!("has drone #{} out, which isn't tethered to it", drone.0));
            }
            let stood = seen.get(&o.id).is_none_or(|was| was.pos == o.pos);
            if !stood && !teleported.contains(&o.id) {
                return fail(format!("moved to {} with its drone #{} out", o.pos, drone.0));
            }
        }
        let Some(leash) = o.tether else {
            continue;
        };
        let operator = sim.object(leash.operator).filter(|op| op.drone == Some(o.id));
        let Some(operator) = operator else {
            return fail(format!("is tethered to #{}, whose drone it isn't", leash.operator.0));
        };
        if tether::def_of(o.kind).is_none() {
            return fail("is tethered, its type on no leash".to_string());
        }
        if tether::past_leash(o, operator) {
            return fail(format!("is past its leash from #{}", operator.id.0));
        }
        if sim.tick() >= leash.battery_until {
            return fail(format!("flies on past its battery's tick {}", leash.battery_until));
        }
    }
    Ok(())
}

/// D1's stock for one object (module docs).
fn stocked(o: &Object) -> Result<(), Violation> {
    let problem = match (o.stock, stock::def_of(o.kind)) {
        (None, None) => return Ok(()),
        (Some(_), None) => "has a stock, its type none".to_string(),
        (None, Some(_)) => "has no stock, its type one".to_string(),
        (Some(stock), Some(def)) if stock.count > def.max => {
            format!("holds {} items, above its max {}", stock.count, def.max)
        }
        (Some(stock), Some(def)) if stock.buying.is_some() && stock.count >= def.max => {
            format!("buys one more with its stock full ({})", stock.count)
        }
        (Some(stock), Some(_)) if stock.buying.is_some_and(|progress| progress.done()) => {
            "keeps a purchase whose frames are all done".to_string()
        }
        (Some(_), Some(_)) => return Ok(()),
    };
    Err(Violation {
        invariant: "D1",
        message: format!("#{} ({:?}) {problem}", o.id.0, o.kind),
        at: o.pos,
    })
}
