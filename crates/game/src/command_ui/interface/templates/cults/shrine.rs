//! The Shrine's panel (`factions/cults/structures/shrine.md > Placeholder stats`, slice M5-N5): "**Health
//! regeneration** (hotkey Q)", "**Energy regeneration** (hotkey W)"; "No cooldowns. Targeting: AwaitingTarget,
//! left-click a point in vision. **A/S/D/F** release one Recruit of a kind, **V: Eject All** (participants leave;
//! README > Choosing Recruits, Release and Eject All)."
//! - Q and W await their point (`PanelCommand::Cast`), where a left-click on a point the player sees now casts the
//!   ability there from this Shrine (`sc_sim::Command::Cast`); a click out of vision is an invalid target
//!   (`crate::command_ui::resolution::target_resolution`). Each is greyed while the Shrine hasn't the energy it costs
//!   (`sc_sim::Sim::can_cast`: "Reject::NotEnoughEnergy"), its cost in the InfoPanel's hint.
//! - A/S/D/F each send one participant of their kind out of the exit at once (`PanelCommand::ReleaseOne`;
//!   `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "pressing one sends **one**
//!   Recruit of that kind out of the exit at once (no queue)"), in the kind buttons' order (Plain, Constitution,
//!   Metabolism, Both), each with its count inside and greyed while there are none, as the Temple's
//!   (`super::temple`).
//! - V lets the participants out one at a time (`PanelCommand::EjectAll`).
//!
//! No rally point, so no C and no right-click (`crate::command_ui::resolution::object_right_click`): **Placeholder
//! (not in the design: the Shrine's panel has no C)**, as the Temple's. It is Ungroupable ("Groupable **false**"): the
//! ActiveGroup is the one Shrine Tabbed to, whose energy a cast spends.

use sc_sim::{AreaAbility, RecruitKind};

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Shrine's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Shrine);

/// Its DefaultState, slot by slot (`CommandSlot::all` order): its two casts in Q and W; the releases of one Recruit of
/// each kind in A, S, D and F; V Eject All.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use AreaAbility::{EnergyRegeneration, HealthRegeneration};
    use PanelCommand::{Cast, EjectAll, ReleaseOne};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    [
        Some(Cast(HealthRegeneration)), Some(Cast(EnergyRegeneration)), None,                     None,
        Some(ReleaseOne(Plain)),        Some(ReleaseOne(Constitution)), Some(ReleaseOne(Metabolism)), Some(ReleaseOne(Both)),
        None,                           None,                           None,                     Some(EjectAll),
    ]
};
