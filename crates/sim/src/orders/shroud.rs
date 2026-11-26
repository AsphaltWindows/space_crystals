//! (M5-N4b) The Shroud order's behaviour (`OrderKind::Shroud`, plan decision T76), `factions/cults/units/disciple.md >
//! Spells > Shroud`: "while the Disciple channels it, its energy **drains gradually**"; "The Disciple **can't move while
//! channeling**"; "**What ends the channel** (user, 2026-11-26, corrected): only the Disciple's **death**, a **cancel**
//! command (Stop or any other order), or running **out of energy**. Taking damage does **not** end it." The cloud of a
//! unit that channels is `crate::shroud`'s, from the units this phase leaves channelling (`super::update`).
//!
//! - **It stands**: the order's start stops the unit, as every order's but a Move to a point does (`super::run::start`),
//!   and nothing gives it a goal while it channels, so it stays Stationary where it stood. With an order it never yields
//!   (`movement::yielding::can_yield`: "no active command").
//! - **The drain** is the order's, each tick from its first ([`run`]): "drains **5 energy per second**", its type's
//!   Shroud's (`defs::ShroudDef::drain`), 20,480 of `Object::energy`'s units a tick (`energy::per_tick`), spent through
//!   `energy::spend`. The regeneration stays `energy::step`'s, later in the tick: "its **own regeneration continues while
//!   channelling** (user, 2026-11-26; Q0113), so the net drain is 4.5/s, about 44 s from full".
//! - **Running dry**: the order completes in the first tick whose whole drain its energy can't pay, and pays nothing
//!   then. From full, 200 points, it pays 711 ticks, 18,432 units net each, and completes in the 712th with 2,048 units
//!   left: 711 ticks, 44.4 s, after it started.
//! - **The start**: "needs at least **10 energy** to start": the gate refuses the order below that
//!   (`Reject::NotEnoughEnergy`), and one that starts below it, queued behind another order, completes in its first
//!   tick, before any drain. Placeholder (the design says only "needs at least 10 energy to start"): the channel's start
//!   is the order's, and the gate judges a queued one by the energy as it is given, as an order given at once.
//! - **A new Shroud order** while the unit channels is "any other order" to the channel under way: that one ends and
//!   the new one starts its own, with its own start check (`crate::shroud`: a member is a channel, the unit and its
//!   order's start).

use super::world::Unit;
use crate::defs::UnitDef;
use crate::energy;

/// One tick of a Shroud order (module docs), `first` in the tick it started: whether it is complete. A unit whose type
/// casts no Shroud or has no energy pool (only a save edited by hand gives it the order) completes it at once.
pub(super) fn run(unit: &mut Unit, first: bool) -> bool {
    let Some(shroud) = unit.unit_def().and_then(UnitDef::shroud) else {
        return true;
    };
    let Some(energy) = unit.energy.as_deref_mut() else {
        return true;
    };
    if first && !energy::enough(*energy, shroud.needs) {
        return true;
    }
    !energy::spend(energy, energy::per_tick(shroud.drain))
}
