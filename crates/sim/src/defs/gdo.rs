//! The GDO — `factions/gdo/`: its own rules, and one module per unit (`units/`, `factions/gdo/units/*.md`) and per
//! structure (`structures/`, `factions/gdo/structures/*.md`, the economy buildings in `factions/gdo/economy.md`), and
//! (M6-GK) the Twister's mine (`mine`, `factions/gdo/units/twister.md > Mines`). Since slice M6-GK every kind of the
//! GDO's design is in the registry (plan decision T84), and the entries that make the ones no player makes yet are
//! held ([`PLANNED_RECIPES`]).

pub(crate) mod mine;
pub(crate) mod structures;
pub(crate) mod units;

use crate::defs::{ObjectKind, Recipe};

crate::tuning::tunables! {
    /// The GDO's own rules (`crate::ruleset::Ruleset::factions`).
    GdoRules;

    /// `factions/gdo/overview.md > GDOResources > Unit Control`: "Hard cap of 200". A unit that would take the used
    /// Unit Control past it doesn't start production until there is room (design change 0143; `economy::army`).
    unit_control_cap: u32 = 200,

    /// (M6-GR) The Radar Station's Scan's numbers (`defs/gdo/structures/radar_station.rs`, plan decision T92), which
    /// `AreaAbility::def` reads: an area ability's, as the Cults' are theirs (`CultsRules::abilities`).
    scan: crate::defs::AbilityDef = structures::radar_station::SCAN,
}

/// (M6-GK) The entries that make the GDO's kinds of design change 0366, each with the kind that will make it, held out
/// of every list a player's command reads until the slice that makes it live (plan decision T84, after T69's rule: no
/// `constructs`, `production`, `builds` or menu entry goes live with the kinds). Each is its kind's module's `RECIPE`,
/// its numbers the design's: the Deployment Center's buildings, in its `Constructs` order
/// (`factions/gdo/structures/deployment_center.md > Constructs`), each behind its own TechPrerequisite
/// (`StructureDef::requires`), but the Radar Station and the Defense System, live since slice M6-GR with the Defense
/// System's Missile Turret, and the Factory, Advanced Mechanics and the Industrial Powerplant, live since slice M6-GF;
/// the Field Engineer's Bunker and First Aid Tent; the Factory's two vehicles behind Advanced Mechanics and the
/// Airfield's Dragonfly, in their `Produces` order, and the Supply Tower's Supply Chopper, each with its Unit
/// Prerequisite and its Supplies (`Recipe::supplies`, design changes 0435 and 0441). The slice that makes an entry live
/// moves it into its maker's list: slice M6-GD the Barracks' Drone Operator; slice M6-GF the Factory's Raptor and
/// Twister; slice M6-GH1 the Airfield's Hornet (the Airfield's own entry stays held until slice M6-GH2).
pub const PLANNED_RECIPES: &[(ObjectKind, Recipe)] = &[
    (ObjectKind::DeploymentCenter, structures::supply_tower::RECIPE),
    (ObjectKind::DeploymentCenter, structures::airfield::RECIPE),
    (ObjectKind::DeploymentCenter, structures::propulsion_labs::RECIPE),
    (ObjectKind::FieldEngineer, structures::bunker::RECIPE),
    (ObjectKind::FieldEngineer, structures::first_aid_tent::RECIPE),
    (ObjectKind::Factory, units::prospector::RECIPE),
    (ObjectKind::Factory, units::artillery_rig::RECIPE),
    (ObjectKind::Airfield, units::dragonfly::RECIPE),
    (ObjectKind::SupplyTower, units::supply_chopper::RECIPE),
];

#[cfg(test)]
mod tests;
