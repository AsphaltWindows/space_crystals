//! (M5-N3) The research invariant T1 (`Player::techs`, `QueueEntry::Research`; `core/tech_and_research.md > Research >
//! Shared Rules`), the M5 plan's "techs come only from completed research":
//! - a player holds a technology only with the one it requires, its line's previous level (5: "each requiring the
//!   previous level");
//! - no technology of a player's is both completed and queued, nor queued at more than one structure, or twice at one
//!   (4: "a technology cannot be queued or in progress at more than one structure at the same time. Once completed, it
//!   no longer appears in any research menu");
//! - every technology queued is one its structure's type offers (`StructureDef::research`), at a structure with an
//!   owner, who has completed its previous level (`core/tech_and_research.md > Tech Prerequisites > Research
//!   Prerequisites`: "A next level can only be queued once the previous level has **completed**", and nothing takes a
//!   completed one away).

use std::collections::BTreeSet;

use glam::Vec2;

use super::Violation;
use crate::sim::Sim;

/// T1: every player's research is as their completed and queued technologies have it (module docs).
pub(super) fn research(sim: &Sim) -> Result<(), Violation> {
    let fail = |message: String, at: Vec2| {
        Err(Violation {
            invariant: "T1",
            message,
            at,
        })
    };
    for p in sim.players() {
        for &tech in &p.techs {
            if let Some(previous) = tech.def().requires.filter(|&previous| !p.has(previous)) {
                return fail(
                    format!("player {} has {tech:?} without {previous:?}", p.id.0),
                    Vec2::ZERO,
                );
            }
        }
    }
    let mut queued = BTreeSet::new();
    for o in sim.objects() {
        let Some(production) = &o.production else {
            continue;
        };
        for tech in production.queue.iter().filter_map(|entry| entry.tech()) {
            let at = |message: String| fail(format!("#{} ({:?}) {message}", o.id.0, o.kind), o.pos);
            let research = o.def().structure.as_ref().and_then(|s| s.research.as_ref());
            if !research.is_some_and(|research| research.offers(tech)) {
                return at(format!("queues {tech:?}, which its type doesn't research"));
            }
            let Some(p) = o.owner.and_then(|owner| sim.player(owner)) else {
                return at(format!("queues {tech:?} for no player"));
            };
            if p.has(tech) {
                return at(format!("queues {tech:?}, which player {} has completed", p.id.0));
            }
            if let Some(previous) = tech.def().requires.filter(|&previous| !p.has(previous)) {
                return at(format!("queues {tech:?} before {previous:?} is completed"));
            }
            if !queued.insert((p.id, tech)) {
                return at(format!("queues {tech:?}, queued already for player {}", p.id.0));
            }
        }
    }
    Ok(())
}
