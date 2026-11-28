//! (M6-ST1) The carrier invariant U1 (`crate::carry`, `network::Place::Carrier`): a passenger is aboard a carrier in
//! the game, on the map or held in its owner's Tunnel Network, not itself, of a type that carries (`UnitDef::carries`),
//! its own owner's, and of a base the carrier's hold takes (`carry::takes`): it boarded so (`Sim::board`), its carrier
//! changes its type only into one that carries too (a carrier's two types, the Subterranean Transport's), and the
//! removal path takes it along as its carrier leaves the game (`Sim::passengers_of`). The places its passengers take
//! may pass the hold's capacity under rules that lowered it since (`DebugCommand::SetRules`), which the sim lets stand,
//! as it lets a building's pool: that is no breach.

use super::Violation;
use crate::carry;
use crate::network::Place;
use crate::sim::Sim;

/// U1 (module docs).
pub(super) fn carriers(sim: &Sim) -> Result<(), Violation> {
    for held in sim.held() {
        let Place::Carrier(carrier) = held.place else {
            continue;
        };
        let o = &held.object;
        let fail = |problem: String| {
            Err(Violation {
                invariant: "U1",
                message: format!("#{} ({:?}) {problem}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        let at = carrier.0;
        if carrier == o.id {
            return fail("is aboard itself".to_string());
        }
        let Some(c) = (sim.object(carrier)).or_else(|| sim.held_unit(carrier).map(|held| &held.object)) else {
            return fail(format!("is aboard #{at}, not in the game"));
        };
        let Some(def) = carry::def_of(c) else {
            return fail(format!("is aboard #{at} ({:?}), which carries no one", c.kind));
        };
        if c.owner != o.owner {
            return fail(format!("is aboard #{at}, {:?}'s", c.owner));
        }
        if !carry::takes(def, o.kind) {
            return fail(format!("is aboard #{at} ({:?}), which takes no {:?}", c.kind, o.kind));
        }
    }
    Ok(())
}
