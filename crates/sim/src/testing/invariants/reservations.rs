//! (M5-F2) The reservation invariant R1 (`Production::reserved`, `sim/producers/pool.rs`): a building that trains the
//! units inside it reserves, entry by entry of its queue, as many of them as the entry takes
//! (`QueueEntry::units_reserved`), each held in that building and reserved by one entry alone, and of its reserved
//! units only the front entry's first may be on its way out (`Held::leaving`: its entry finished). Every other
//! producer reserves nothing.

use std::collections::BTreeSet;

use super::Violation;
use crate::holding;
use crate::network::Place;
use crate::sim::Sim;

/// R1: every producer's reservations are as its queue and its pool have them (module docs).
pub(super) fn reservations(sim: &Sim) -> Result<(), Violation> {
    let mut reserved = BTreeSet::new();
    for o in sim.objects() {
        let Some(production) = &o.production else {
            continue;
        };
        let fail = |message: String| {
            Err(Violation {
                invariant: "R1",
                message: format!("#{} ({:?}) {message}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        if !holding::trains(o.kind) {
            if !production.reserved.is_empty() {
                return fail(format!("holds no units but reserves {:?}", production.reserved));
            }
            continue;
        }
        if production.reserved.len() != production.queue.len() {
            return fail(format!(
                "reserves for {} entries of its {}",
                production.reserved.len(),
                production.queue.len()
            ));
        }
        for (i, (entry, units)) in production.queue.iter().zip(&production.reserved).enumerate() {
            if units.len() != entry.units_reserved() {
                return fail(format!("{entry:?} reserves {units:?}"));
            }
            for (j, &unit) in units.iter().enumerate() {
                let Some(held) = sim.held_unit(unit) else {
                    return fail(format!("{entry:?} reserves #{}, which isn't held", unit.0));
                };
                if held.place != Place::Building(o.id) {
                    return fail(format!("{entry:?} reserves #{}, held at {:?}", unit.0, held.place));
                }
                if !reserved.insert(unit) {
                    return fail(format!("#{} is reserved twice", unit.0));
                }
                if held.leaving.is_some() && (i, j) != (0, 0) {
                    return fail(format!("#{}, reserved by entry {i}, is on its way out", unit.0));
                }
            }
        }
    }
    Ok(())
}
