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
}

/// (M6-GK) The entries that make the GDO's kinds of design change 0366, each with the kind that will make it, held out
/// of every list a player's command reads until the slice that makes it live (plan decision T84, after T69's rule: no
/// `constructs`, `production`, `builds` or menu entry goes live with the kinds). Each is its kind's module's `RECIPE`,
/// its numbers the design's: the Deployment Center's eight buildings, in its `Constructs` order
/// (`factions/gdo/structures/deployment_center.md > Constructs`), each behind its own TechPrerequisite
/// (`StructureDef::requires`); the Defense System's Missile Turret; the Field Engineer's Bunker and First Aid Tent; the
/// Factory's four vehicles and the Airfield's two aircraft, in their `Produces` order, and the Supply Tower's Supply
/// Chopper, each with its Unit Prerequisite and its Supplies (`Recipe::supplies`, design changes 0435 and 0441). The
/// slice that makes an entry live moves it into its maker's list: slice M6-GD the Barracks' Drone Operator.
pub const PLANNED_RECIPES: &[(ObjectKind, Recipe)] = &[
    (ObjectKind::DeploymentCenter, structures::supply_tower::RECIPE),
    (ObjectKind::DeploymentCenter, structures::factory::RECIPE),
    (ObjectKind::DeploymentCenter, structures::radar_station::RECIPE),
    (ObjectKind::DeploymentCenter, structures::advanced_mechanics::RECIPE),
    (ObjectKind::DeploymentCenter, structures::airfield::RECIPE),
    (ObjectKind::DeploymentCenter, structures::propulsion_labs::RECIPE),
    (ObjectKind::DeploymentCenter, structures::industrial_powerplant::RECIPE),
    (ObjectKind::DeploymentCenter, structures::defense_system::RECIPE),
    (ObjectKind::DefenseSystem, structures::missile_turret::RECIPE),
    (ObjectKind::FieldEngineer, structures::bunker::RECIPE),
    (ObjectKind::FieldEngineer, structures::first_aid_tent::RECIPE),
    (ObjectKind::Factory, units::raptor::RECIPE),
    (ObjectKind::Factory, units::twister::RECIPE),
    (ObjectKind::Factory, units::prospector::RECIPE),
    (ObjectKind::Factory, units::artillery_rig::RECIPE),
    (ObjectKind::Airfield, units::hornet::RECIPE),
    (ObjectKind::Airfield, units::dragonfly::RECIPE),
    (ObjectKind::SupplyTower, units::supply_chopper::RECIPE),
];

#[cfg(test)]
mod tests;
