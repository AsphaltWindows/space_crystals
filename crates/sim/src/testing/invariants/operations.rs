//! (M4-F) The operation invariant O1 (`tunnel::Operation`): a Tunnel's operation and its site name each other. A
//! Tunnel that builds an expansion names a site of the game whose builder is that Tunnel, and an expansion's site
//! names a Tunnel of the game whose operation it is: one whose Tunnel leaves the game goes with it, in the same removal
//! (`sim/tunnels.rs`, design change 0254: "Its Tunnel destroyed mid-build: the unfinished expansion is destroyed").
//! (M4-G) A Tunnel that upgrades is in operation, and upgrades to the tier above its own, which its table has, over that
//! tier's upgrade frames (`sim/upgrades.rs`).

use super::Violation;
use crate::construction::site::Builder;
use crate::network;
use crate::sim::Sim;
use crate::tunnel::{self, Operation};

/// O1: every Tunnel's operation and the site it builds name each other, and every upgrade goes one tier up.
pub(super) fn operations(sim: &Sim) -> Result<(), Violation> {
    for o in sim.objects() {
        let fail = |message: String| {
            Err(Violation {
                invariant: "O1",
                message: format!("#{} ({:?}) {message}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        let operation = o.tunnel.as_ref().and_then(|tunnel| tunnel.operation);
        if let Some(Operation::Building(site)) = operation {
            let builder = sim.object(site).and_then(|site| site.construction).map(|c| c.builder);
            if builder != Some(Builder::Tunnel(o.id)) {
                return fail(format!("builds #{} whose builder is {builder:?}", site.0));
            }
        }
        if let (Some(Operation::Upgrading { progress, to }), Some(state)) = (operation, o.tunnel.as_ref()) {
            if !network::is_exit(o) {
                return fail("upgrades, but isn't a Tunnel in operation".to_string());
            }
            if state.tier.checked_add(1) != Some(to) {
                return fail(format!("at Tier {} upgrades to Tier {to}", state.tier));
            }
            let frames = (tunnel::def_of(o.kind)).and_then(|def| Some(def.tier(to)?.upgrade?.frames));
            if frames != Some(progress.total) {
                return fail(format!(
                    "upgrades over {} frames, Tier {to}'s upgrade {frames:?}",
                    progress.total
                ));
            }
        }
        if let Some(tunnel) = o.construction.and_then(|site| site.builder.tunnel()) {
            let Some(t) = sim.object(tunnel) else {
                return fail(format!("is built by #{}, which has left the game", tunnel.0));
            };
            let operation = t.tunnel.as_ref().and_then(|state| state.operation);
            if operation != Some(Operation::Building(o.id)) {
                return fail(format!("is built by #{} whose operation is {operation:?}", tunnel.0));
            }
        }
    }
    Ok(())
}
