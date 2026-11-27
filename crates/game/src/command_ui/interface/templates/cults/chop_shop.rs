//! The Chop Shop's panel (slice M5-N6; `factions/cults/structures/chop_shop.md > Placeholder stats`: "Panel like the
//! Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Build
//! Seeker**, **W: Build Hand of Heaven**, **E: Build Prayer-mobile** [...], **A: Release** in the kind menu; **A: Eject
//! All**, **C: Set Rally Point**, **X: Cancel Production**", design change 0444): the pool buildings'
//! ([`super::armory`]), as the Airstrip's is ([`super::airstrip`]), its KindMenus its recipes' in their order, Q the
//! Seeker's and (M5-N10) W the Hand of Heaven's, each training named as the design names it, "Build Seeker" and "Build
//! Hand of Heaven" (`PanelCommand::name_on`), its hint with its price (`Conditions::price`). A kind's training is
//! greyed while fewer Recruits of the kind are free inside than the vehicle reserves, two for a Seeker and three for a
//! Hand of Heaven (the gate's `Reject::NoRecruit`), or while the bank can't pay. Its right-click sets the rally point,
//! as every pool building's. E, the Prayer-mobile's, is slice M5-N11's.

use sc_sim::ObjectKind::{self, HandOfHeaven, Seeker};
use sc_sim::RecruitKind::{self, Both, Constitution, Metabolism, Plain};

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Chop Shop's panel template: the pool buildings', with "Build Seeker" for its KindMenus' Q and "Build Hand of
/// Heaven" for their W, of every kind.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    names: &[
        build(Seeker, Plain),
        build(Seeker, Constitution),
        build(Seeker, Metabolism),
        build(Seeker, Both),
        build(HandOfHeaven, Plain),
        build(HandOfHeaven, Constitution),
        build(HandOfHeaven, Metabolism),
        build(HandOfHeaven, Both),
    ],
    ..PanelTemplate::of(InterfaceTemplate::PoolProducer)
};

/// "**Q: Build Seeker**" and "**W: Build Hand of Heaven**", in KindMenu\[`from`\].
const fn build(kind: ObjectKind, from: RecruitKind) -> (PanelCommand, &'static str) {
    let name = match kind {
        HandOfHeaven => "Build Hand of Heaven",
        _ => "Build Seeker",
    };
    (PanelCommand::TrainFrom(kind, from), name)
}
