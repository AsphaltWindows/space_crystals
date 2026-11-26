//! The Temple's panel (`factions/cults/structures/temple.md > Placeholder stats`, slice M5-N3): "Panel: **Q**
//! Enhanced Constitution (next level), **W** Improved Metabolism (next level), **A/S/D/F** release one Recruit of a
//! kind (README > Choosing Recruits, Release and Eject All), **V: Eject All** (staff leave), **X: Cancel** (last
//! research). Recruits enter by right-click."
//! - Q and W research their line's next level (`PanelCommand::Research`), at once, paid as it is queued: the grid
//!   holds each line's Level 1, which the panel shows as the line's next level, and as "In progress" while that level
//!   is queued or in progress at any of the player's structures ("its key shows "In progress" meanwhile":
//!   `PanelCommand::Researching`, `super::super::super::Conditions::key`). A line with both levels completed leaves
//!   its key empty (`core/tech_and_research.md > Research > Shared Rules` 4: "Once completed, it no longer appears in
//!   any research menu"). Level 2 is greyed by the gate while its player owns no Cathedral (`sc_sim::Reject::Requires`,
//!   "Requires a Cathedral"), and both while the bank can't pay.
//! - A/S/D/F each send one Recruit of their kind out of the exit at once (`PanelCommand::ReleaseOne`;
//!   `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "pressing one sends **one**
//!   Recruit of that kind out of the exit at once (no queue)"), in the kind buttons' order (Plain, Constitution,
//!   Metabolism, Both), each with its count inside and greyed while there are none.
//! - V lets the staff out one at a time (`PanelCommand::EjectAll`), X takes the last research off the queue with its
//!   full refund (`PanelCommand::CancelProduction`; `core/tech_and_research.md > Research > Shared Rules` 6).
//!
//! No rally point, so no C and no right-click (`crate::command_ui::resolution::object_right_click`): **Placeholder
//! (not in the design: the Temple's panel has no C)**. It is Groupable ("Groupable true"): its commands go to every
//! Temple of the ActiveGroup, a Research to one of them, the one with the least work queued
//! (`sc_sim::ProductionCommand::Research`), as a Train to the Barracks (design change 0083).

use sc_sim::{RecruitKind, Tech};

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Temple's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Temple);

/// Its DefaultState, slot by slot (`CommandSlot::all` order): its two lines' keys in Q and W, each holding the line's
/// Level 1; the releases of one Recruit of each kind in A, S, D and F; X Cancel Production and V Eject All.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{CancelProduction, EjectAll, ReleaseOne, Research};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    [
        Some(Research(Tech::EnhancedConstitutionL1)), Some(Research(Tech::ImprovedMetabolismL1)), None, None,
        Some(ReleaseOne(Plain)), Some(ReleaseOne(Constitution)), Some(ReleaseOne(Metabolism)), Some(ReleaseOne(Both)),
        None,                                         Some(CancelProduction),                     None, Some(EjectAll),
    ]
};
