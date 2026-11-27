//! The Airstrip's panel (slice M5-N7; `factions/cults/structures/airstrip.md > Placeholder stats`: "Panel like the
//! Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Build
//! Sparrow**, **A: Release** in the kind menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**"):
//! the pool buildings' ([`super::armory`]), its KindMenus its one recipe's, the Sparrow's training named as the design
//! names it, "Build Sparrow" (`PanelCommand::name_on`), the Armory's being "Train Militant". Its right-click sets the
//! rally point, as every pool building's.

use sc_sim::ObjectKind;
use sc_sim::RecruitKind::{self, Both, Constitution, Metabolism, Plain};

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Airstrip's panel template: the pool buildings', with "Build Sparrow" for its KindMenus' Q, of every kind.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    names: &[build(Plain), build(Constitution), build(Metabolism), build(Both)],
    ..PanelTemplate::of(InterfaceTemplate::PoolProducer)
};

/// "**Q: Build Sparrow**", in KindMenu\[`from`\].
const fn build(from: RecruitKind) -> (PanelCommand, &'static str) {
    (PanelCommand::TrainFrom(ObjectKind::Sparrow, from), "Build Sparrow")
}
