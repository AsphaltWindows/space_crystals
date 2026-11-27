//! (M6-GR) Cooldowns (plan decision T92: "a cooldown instead of energy"): the wait after a cast of an ability with one
//! (`AbilityDef::cooldown`), each caster's own (`Object::cooldown`, a [`Progress`]): the Radar Station's Scan,
//! "**cooldown 60 seconds** (960 frames) [...] Each Radar Station has its own cooldown"
//! (`factions/gdo/structures/radar_station.md > Placeholder stats`).
//!
//! - **The start**: the cast sets it (`Sim::cast`, in the commands phase), and the gate refuses the caster's next cast
//!   while it is set (`Sim::can_cast`: `Reject::CoolingDown`).
//! - **The wait** ([`step`]): every tick's economy phase, the tick of the cast's included, takes each one tick on at
//!   its caster's speed, as the constructors' progress (`crate::construction`): design change 0423, "Each frame, these
//!   timers advance by the speed ratio (generated ÷ consumed, capped at 100%), as production does. [...] Producers
//!   (Deployment Center, Generators) are still never slowed" (`factions/gdo/overview.md > GDOResources > Power`:
//!   "**ability cooldowns** (the Radar Station's Scan)"; `economy::PowerSummary::speed`). Once done it is dropped, and
//!   the caster casts again from the next tick. So a cast in tick `t` at full Power waits ticks `t` to `t + 959` and
//!   may cast again in tick `t + 960`; at about 59% Power (30 generated, 51 consumed, a speed of 38551 of 65536) the
//!   wait takes 1632 ticks, 102 s; a wait under way goes on at the speed of each tick ("Durations already running
//!   stretch from that moment on"), so a Generator that brings the Power back up gives it full speed at once.

use std::collections::BTreeMap;

use crate::economy::{PowerSummary, RATIO_ONE};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::progress::Progress;
use crate::units::Tick;

/// The wait that a cast of an ability with a cooldown of `ticks` starts (module docs).
pub(crate) fn start(ticks: Tick) -> Progress {
    Progress::new(ticks)
}

/// Whether `o` may cast now: no wait under way (module docs).
pub fn ready(o: &Object) -> bool {
    o.cooldown.is_none()
}

/// The economy phase (module docs): every wait under way, one tick on at its caster's speed by its owner's Power in
/// `power` (`economy::power::summaries`), and dropped once done.
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, power: &BTreeMap<PlayerId, PowerSummary>) {
    for o in objects.values_mut() {
        let Some(cooldown) = &mut o.cooldown else {
            continue;
        };
        let speed = o.owner.map_or(RATIO_ONE, |owner| {
            power.get(&owner).copied().unwrap_or_default().speed(o.kind)
        });
        if cooldown.step(speed) {
            o.cooldown = None;
        }
    }
}
