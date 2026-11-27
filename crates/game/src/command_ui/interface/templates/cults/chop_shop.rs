//! The Chop Shop's panel (slice M5-N6; `factions/cults/structures/chop_shop.md > Placeholder stats`: "Panel like the
//! Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Build
//! Seeker**, **A: Release** in the kind menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**"): the
//! pool buildings' ([`super::armory`]), as the Airstrip's is ([`super::airstrip`]), its KindMenus its one recipe's, the
//! Seeker's training named as the design names it, "Build Seeker" (`PanelCommand::name_on`). A kind's Q is greyed
//! while fewer than two Recruits of the kind are free inside (the gate's `Reject::NoRecruit`: a Seeker reserves 2
//! Recruits of the kind). Its right-click sets the rally point, as every pool building's.

use sc_sim::ObjectKind;
use sc_sim::RecruitKind::{self, Both, Constitution, Metabolism, Plain};

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Chop Shop's panel template: the pool buildings', with "Build Seeker" for its KindMenus' Q, of every kind.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    names: &[build(Plain), build(Constitution), build(Metabolism), build(Both)],
    ..PanelTemplate::of(InterfaceTemplate::PoolProducer)
};

/// "**Q: Build Seeker**", in KindMenu\[`from`\].
const fn build(from: RecruitKind) -> (PanelCommand, &'static str) {
    (PanelCommand::TrainFrom(ObjectKind::Seeker, from), "Build Seeker")
}
