//! (M4-F) The operation invariant O1 (`tunnel::Operation`): a Tunnel's operation and its site name each other. A
//! Tunnel that builds an expansion names a site of the game whose builder is that Tunnel, and an expansion's site
//! names a Tunnel of the game whose operation it is: one whose Tunnel leaves the game goes with it, in the same removal
//! (`sim/tunnels.rs`, design change 0254: "Its Tunnel destroyed mid-build: the unfinished expansion is destroyed").

use super::Violation;
use crate::construction::site::Builder;
use crate::sim::Sim;
use crate::tunnel::Operation;

/// O1: every Tunnel's operation and the site it builds name each other.
pub(super) fn operations(sim: &Sim) -> Result<(), Violation> {
    for o in sim.objects() {
        let fail = |message: String| {
            Err(Violation {
                invariant: "O1",
                message: format!("#{} ({:?}) {message}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        if let Some(Operation::Building(site)) = o.tunnel.as_ref().and_then(|tunnel| tunnel.operation) {
            let builder = sim.object(site).and_then(|site| site.construction).map(|c| c.builder);
            if builder != Some(Builder::Tunnel(o.id)) {
                return fail(format!("builds #{} whose builder is {builder:?}", site.0));
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
