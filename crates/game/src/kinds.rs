//! What the client knows of each object kind, beside its def (`ObjectKind::def`: its name and its numbers are the
//! sim's): its label, the look of its placeholder, the style of its shots and its panel template. One module per
//! kind, `kinds/<faction>/<object>.rs`, named as the design's file of the object is, behind one registry ([`of`]): a
//! new kind adds its line there and its own module (`ARCHITECTURE.md > Adding a unit or structure`). The drawing code
//! stays in `crate::object_render` and `crate::combat_effects`; a panel of the kind's own is in
//! `crate::command_ui::interface::templates`, by faction and object too.

mod colonists;
mod gdo;
mod map;
mod syndicate;

use sc_sim::ObjectKind;
use sc_sim::tunnel::TunnelState;

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::PanelTemplate;
use crate::object_render::Look;

pub use syndicate::tunnel::tier_label;

/// What the client knows of a kind ([`of`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClientKind {
    /// Its label (placeholders until the art, Q0001): on its portrait, on the map (`object_render`'s tags) and on the
    /// button that makes it. Each its own (slice M3-J3): none is another type's, nor a planned unit's or building's
    /// button label (`Planned::label`: the Scout's had been the patch's "SC").
    pub label: &'static str,
    /// The look of its placeholder, on the map and on its portrait.
    pub look: Look,
    /// How its shots are drawn ([`ShotStyle::of`]); none for a kind without a weapon.
    pub shot: Option<ShotStyle>,
    /// Its panel template: its ObjectInterfaceState template, and what its own panel adds to it.
    pub template: PanelTemplate,
}

/// The registry: what the client knows of `kind`, from its module.
pub fn of(kind: ObjectKind) -> &'static ClientKind {
    match kind {
        ObjectKind::CrystalPatch => &map::crystal_patch::KIND,
        ObjectKind::SupplyDeliveryStation => &map::supply_delivery_station::KIND,
        ObjectKind::Peacekeeper => &gdo::peacekeeper::PEACEKEEPER,
        ObjectKind::Guard => &syndicate::guard::KIND,
        ObjectKind::DeploymentCenter => &gdo::deployment_center::KIND,
        ObjectKind::Generator => &gdo::generator::KIND,
        ObjectKind::Barracks => &gdo::barracks::KIND,
        ObjectKind::ExtractionFacility => &gdo::extraction_facility::KIND,
        ObjectKind::ExtractionPlate => &gdo::extraction_plate::KIND,
        ObjectKind::Drone => &colonists::drone::KIND,
        ObjectKind::Beacon => &colonists::beacon::KIND,
        ObjectKind::EgressNode => &colonists::egress_node::KIND,
        ObjectKind::RoboPortal => &colonists::robo_portal::KIND,
        ObjectKind::BioPortal => &colonists::bio_portal::KIND,
        ObjectKind::SabreLegionnaire => &colonists::sabre_legionnaire::KIND,
        ObjectKind::RifleLegionnaire => &colonists::rifle_legionnaire::KIND,
        ObjectKind::PeacekeeperShielded => &gdo::peacekeeper::SHIELDED,
        ObjectKind::Agent => &syndicate::agent::KIND,
        ObjectKind::Tunnel => &syndicate::tunnel::KIND,
        ObjectKind::Hideout => &syndicate::hideout::KIND,
        ObjectKind::OpticsLab => &syndicate::optics_lab::KIND,
        ObjectKind::ChemicalPlant => &syndicate::chemical_plant::KIND,
        ObjectKind::Grenadier => &syndicate::grenadier::KIND,
        ObjectKind::LaserTurret => &syndicate::laser_turret::KIND,
    }
}

/// The short label of a type ([`ClientKind::label`]). The Peacekeeper's two types have two letters each, "PK" and
/// "PS", short enough for the Peacekeeper's 24-su body.
pub fn short_label(kind: ObjectKind) -> &'static str {
    of(kind).label
}

/// The label an object shows, on the map (a record's too) and on its portrait: its type's short label
/// ([`short_label`]), a Tunnel's its tier's ([`tier_label`], from `Object::tunnel` or the record's), to every player
/// who sees it (design change 0258: "Enemies see a Tunnel's tier").
pub fn object_label(kind: ObjectKind, tunnel: Option<TunnelState>) -> &'static str {
    match tunnel {
        Some(state) => tier_label(state.tier),
        None => short_label(kind),
    }
}

#[cfg(test)]
mod tests;
