//! (M4-O) The projectile invariant P1 (plan decision T42: "every projectile lands by its arrival tick"): between
//! ticks, every projectile in flight is due after the tick just played (`crate::combat::projectiles`), and flies
//! between finite points; (M5-K) a homing one is at a finite point too (`crate::combat::projectiles::Homing`), with a
//! speed of its own that isn't negative.

use super::Violation;
use crate::sim::Sim;

/// P1: every projectile still in flight lands after the tick just played, between finite points.
pub(super) fn projectiles(sim: &Sim) -> Result<(), Violation> {
    let tick = sim.tick();
    for p in sim.projectiles() {
        if p.lands_at <= tick {
            return Err(Violation {
                invariant: "P1",
                message: format!(
                    "the projectile of #{} due in tick {} is still in flight at tick {tick}",
                    p.shooter.0, p.lands_at
                ),
                at: p.to,
            });
        }
        if !(p.from.is_finite() && p.to.is_finite()) {
            return Err(Violation {
                invariant: "P1",
                message: format!("the projectile of #{} flies from {} to {}", p.shooter.0, p.from, p.to),
                at: p.from,
            });
        }
        if let Some(homing) = p.homing
            && !(homing.pos.is_finite() && homing.prev.is_finite() && homing.speed.is_finite() && homing.speed >= 0.0)
        {
            return Err(Violation {
                invariant: "P1",
                message: format!("the homing projectile of #{} flies as {homing:?}", p.shooter.0),
                at: p.from,
            });
        }
    }
    Ok(())
}
