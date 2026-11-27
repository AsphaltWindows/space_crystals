//! The Chop Shop's panel (slice M5-N6; `factions/cults/structures/chop_shop.md > Placeholder stats`: "Panel like the
//! Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Build
//! Seeker**, **W: Build Hand of Heaven**, **E: Build Prayer-mobile** (greyed "Requires a Temple" until one stands),
//! **A: Release** in the kind menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**", design change
//! 0444): the pool buildings' ([`super::armory`]), as the Airstrip's is ([`super::airstrip`]), its KindMenus its
//! recipes' in their order, Q the Seeker's, (M5-N10) W the Hand of Heaven's and (M5-N11) E the Prayer-mobile's (placed
//! there by [`PanelTemplate::trains`]), each training named as the design names it, "Build Seeker", "Build Hand of
//! Heaven" and "Build Prayer-mobile" (`PanelCommand::name_on`), its hint with its price (`Conditions::price`). A kind's
//! training is greyed while fewer Recruits of the kind are free inside than the vehicle reserves, two for a Seeker or a
//! Prayer-mobile and three for a Hand of Heaven (the gate's `Reject::NoRecruit`), while the bank can't pay, and the
//! Prayer-mobile's while its owner has no Temple (`Reject::Requires`). Its right-click sets the rally point, as every
//! pool building's.

use sc_sim::ObjectKind::{self, HandOfHeaven, PrayerMobile, Seeker};
use sc_sim::RecruitKind::{self, Both, Constitution, Metabolism, Plain};

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Chop Shop's panel template: the pool buildings', with "Build Seeker" for its KindMenus' Q, "Build Hand of
/// Heaven" for their W and "Build Prayer-mobile" for their E, of every kind.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    trains: &[(2, PrayerMobile)],
    names: &[
        build(Seeker, Plain),
        build(Seeker, Constitution),
        build(Seeker, Metabolism),
        build(Seeker, Both),
        build(HandOfHeaven, Plain),
        build(HandOfHeaven, Constitution),
        build(HandOfHeaven, Metabolism),
        build(HandOfHeaven, Both),
        build(PrayerMobile, Plain),
        build(PrayerMobile, Constitution),
        build(PrayerMobile, Metabolism),
        build(PrayerMobile, Both),
    ],
    ..PanelTemplate::of(InterfaceTemplate::PoolProducer)
};

/// "**Q: Build Seeker**", "**W: Build Hand of Heaven**" and "**E: Build Prayer-mobile**", in KindMenu\[`from`\].
const fn build(kind: ObjectKind, from: RecruitKind) -> (PanelCommand, &'static str) {
    let name = match kind {
        HandOfHeaven => "Build Hand of Heaven",
        PrayerMobile => "Build Prayer-mobile",
        _ => "Build Seeker",
    };
    (PanelCommand::TrainFrom(kind, from), name)
}
