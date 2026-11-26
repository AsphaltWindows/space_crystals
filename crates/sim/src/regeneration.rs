//! HP regained over time, as a share of the MaxHP a second: one rule for every such regeneration. A unit inside the
//! Tunnel Network (`factions/syndicate/tunnels.md > Regeneration inside`, design change 0249: `network::Held::regenerate`)
//! and a completed Tunnel expansion while its Tunnel lives (slice M5-N8; `factions/syndicate/structures/README.md >
//! Tunnel Expansions`, design changes 0329 and 0330: `sim/expansions.rs`), each at 1% a second for now. The decay of a
//! neutral expansion is the same share the other way, counted out exactly so that its last tick is known
//! (`sim/tunnels.rs`); the Shields regenerate a fixed amount a tick (`crate::shields`).

use crate::augmentation::Research;
use crate::object::Object;
use crate::units::TICKS_PER_SECOND;

/// One tick of regeneration of `o` at `share_per_second` of its MaxHP a second, up to its MaxHP: (M5-N1a) its
/// Augmentations' included, at its owner's levels in `research` (`Object::max_hp_with`). Nothing for an object without
/// HP.
pub(crate) fn regenerate(o: &mut Object, share_per_second: f32, research: Research) {
    let (Some(hp), Some(max)) = (o.hp, o.max_hp_with(research)) else {
        return;
    };
    if hp < max {
        let gain = max * (share_per_second / TICKS_PER_SECOND as f32);
        o.hp = Some((hp + gain).min(max));
    }
}
