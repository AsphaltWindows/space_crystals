//! What the Syndicate's stat blocks share, for their modules' tests (`units/`, `structures/`). The rules every kind
//! follows are in `defs_are_consistent` (`defs/tests.rs`).

use crate::defs::AttackAttrs;

/// A unit's attack timings, `(AimDuration, FiringDuration, CooldownDuration, ReloadDuration)`, and the frames from one
/// shot to the next, their sum.
pub(super) fn timings(attack: &AttackAttrs) -> ((u32, u32, u32, u32), u32) {
    let t = (
        attack.aim_ticks,
        attack.firing_ticks,
        attack.cooldown_ticks,
        attack.reload_ticks,
    );
    (t, t.0 + t.1 + t.2 + t.3)
}
