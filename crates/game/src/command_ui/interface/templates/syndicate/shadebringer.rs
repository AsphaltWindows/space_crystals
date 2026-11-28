//! The Shadebringer's panel (`factions/syndicate/units/shadebringer.md > ObjectInterfaceState`: "Move-type commands
//! like a combat unit without an attack, plus the two toggles"; `> Placeholder stats`: "Toggles follow the mode-switch
//! rule (keep orders, not queueable). Hotkeys: **R** Sludge, **F** Cover"; slice M6-SB): the BasicCombatUnit template
//! without its attack commands, the Prayer-mobile's and the Drone Operator's (the Shadebringer has "**No attack and no
//! turret**"), its A the selection's ([`super::super::attack_slot`], design change 0399), and a toggle for each of its
//! tanks, which the grid adds for a unit with tanks (`sc_sim::defs::UnitDef::tanks`; [`super::super::grid`]), as it adds
//! the units' other abilities:
//! - **R, Sludge** (`super::super::PanelCommand::Toggle`, [`slot`]): switches the Sludge on, or off while every one of
//!   its recipients has it on (`crate::command_ui::interface::Conditions::key`), at once, to every Shadebringer of the
//!   ActiveGroup, or of the selection while it is all Shadebringers (the Shadebringer is Groupable, "Groupable - true
//!   (assumed)"), each keeping its orders. Switching on is greyed while none of them has any charge left
//!   (`sc_sim::Reject::TankEmpty`: "a toggle can be switched on with **any charge** left in its tank").
//! - **F, Cover**: greyed, "Not available yet" ([`Planned::Cover`]), until slice M6-SC gives the Shadebringer its smoke
//!   tank, and so its toggle in F.
//!
//! Each tank's charge shows as a bar under the unit's health (`crate::combat_effects`), and its trail on the ground
//! (`crate::patches_view`). Its right-click is the sim's one rule, as every unit's
//! (`crate::command_ui::resolution::unit_right_click`).

use sc_sim::defs::PatchKind;

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// R: the Sludge's toggle ("Hotkeys: **R** Sludge").
const SLUDGE_SLOT: usize = 3;
/// F: the Cover's ("**F** Cover").
const COVER_SLOT: usize = 7;

/// The slot of the toggle of a tank that lays `lays`' patches: R for sludge, F for (slice M6-SC) smoke.
pub const fn slot(lays: PatchKind) -> usize {
    match lays {
        PatchKind::Sludge => SLUDGE_SLOT,
        PatchKind::Smoke => COVER_SLOT,
    }
}

/// The Shadebringer's panel template: the BasicCombatUnit's without its attack commands, R added by the grid, and F
/// holding Cover's slot until slice M6-SC.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    without_attacks: true,
    planned: &[(COVER_SLOT, Planned::Cover)],
    ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
};
