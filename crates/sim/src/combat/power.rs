//! (M6-GR) A consumer's weapon at low Power (design change 0423, `factions/gdo/overview.md > GDOResources > Power`):
//! "**Everything a consumer does is slowed** [...] **weapons** (the Missile Turret's whole attack cycle: aim, firing,
//! cooldown and reload take longer by the ratio) [...] Durations already running stretch from that moment on (they
//! progress at the ratio each frame)". A structure of negative Power that attacks runs its attack sequence
//! (`super::phases`) at its owner's speed, the Power ratio as production reads it (`economy::PowerSummary::speed`); a
//! unit's, and a producer's (none attacks), always at full speed.
//!
//! - **The share**: each tick gives the sequence `speed` [`RATIO_ONE`]s of a tick, which it banks
//!   (`AttackState::power_carry`) until they make a whole tick: a tick of its phase, which the phases count in whole
//!   ticks as ever. At full speed every tick is one, and nothing is banked, so every game at full Power plays as it
//!   did.
//! - **A whole cycle**: the share banked carries from phase to phase of a sequence ("the whole attack cycle"), and is
//!   dropped when the sequence ends or is cancelled, so a sequence starts with none: the Missile Turret's 24 ticks of
//!   Aim, Firing, Cooldown and Reload take `⌈24 × 65536 / speed⌉` ticks, 41 at about 59% Power (30 generated, 51
//!   consumed: 38551), "a turret fires about every 41 frames instead of 24". **Placeholder (not in the design: how a
//!   share of a tick rounds)**: production's rule, a tick's progress at the ratio (`crate::progress`).
//! - **When**: the Power as phase 5e finds it, from the structures in the game then (`economy::power::summaries`), so
//!   that a Generator placed by the tick's commands brings the turrets back to full speed in that tick ("Build a
//!   Generator back to positive: all return to full speed at once").

use std::collections::BTreeMap;

use crate::economy::{PowerSummary, RATIO_ONE, power};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;

/// Every owner's Power, if one of the objects before `first_new` is a structure of negative Power with an attack: the
/// weapons it slows (module docs). `None` otherwise, which costs no look at the structures' Power.
pub(super) fn summaries(
    objects: &BTreeMap<ObjectId, Object>,
    first_new: ObjectId,
) -> Option<BTreeMap<PlayerId, PowerSummary>> {
    let consumer = |o: &Object| {
        o.attack.is_some() && (o.def().structure.as_ref()).is_some_and(|s| s.power < 0 && s.attack.is_some())
    };
    (objects.range(..first_new).any(|(_, o)| consumer(o))).then(|| power::summaries(objects.values()))
}

/// The share of each tick `o`'s attack sequence gets, in [`RATIO_ONE`]s (module docs): its owner's Power ratio in
/// `power` for a consumer structure, a whole tick for anything else.
pub(super) fn speed(power: Option<&BTreeMap<PlayerId, PowerSummary>>, o: &Object) -> u32 {
    match (power, o.owner) {
        (Some(power), Some(owner)) if o.def().unit.is_none() => {
            power.get(&owner).copied().unwrap_or_default().speed(o.kind)
        }
        _ => RATIO_ONE,
    }
}

/// Whether the tick has a whole tick for the sequence's phase, at `speed` with `banked` saved up (module docs): it adds
/// its share to the bank and takes a tick out of it if it holds one. At full speed, always, with nothing banked.
pub(super) fn spend(banked: &mut u32, speed: u32) -> bool {
    if speed >= RATIO_ONE {
        *banked = 0;
        return true;
    }
    let total = banked.saturating_add(speed);
    if total >= RATIO_ONE {
        *banked = total - RATIO_ONE;
        true
    } else {
        *banked = total;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ticks a sequence of `ticks` whole ticks takes at `speed`, from nothing banked.
    fn ticks_taken(ticks: u32, speed: u32) -> u32 {
        let (mut banked, mut done, mut taken) = (0, 0, 0);
        while done < ticks {
            taken += 1;
            if spend(&mut banked, speed) {
                done += 1;
            }
        }
        taken
    }

    /// At full speed each tick is one, and nothing is banked; at the ratio of 30 generated for 51 consumed a 24-tick
    /// cycle takes 41 ticks, `⌈24 × 65536 / 38551⌉`; at half speed twice as long; at no Power never.
    #[test]
    fn a_share_of_each_tick_makes_whole_ticks() {
        let mut banked = 0;
        assert!(spend(&mut banked, RATIO_ONE) && banked == 0);
        banked = 100;
        assert!(spend(&mut banked, RATIO_ONE) && banked == 0, "full speed at once");
        let short = PowerSummary {
            generated: 30,
            consumed: 51,
        };
        assert_eq!(short.ratio(), 38551);
        assert_eq!(ticks_taken(24, short.ratio()), 41);
        assert_eq!((24 * u64::from(RATIO_ONE)).div_ceil(38551), 41);
        assert_eq!(ticks_taken(24, RATIO_ONE), 24);
        assert_eq!(ticks_taken(24, RATIO_ONE / 2), 48);
        let mut banked = 0;
        assert!((0..1000).all(|_| !spend(&mut banked, 0)), "no Power, no tick");
    }
}
