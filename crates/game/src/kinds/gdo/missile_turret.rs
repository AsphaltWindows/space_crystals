//! The Missile Turret (`factions/gdo/structures/missile_turret.md`, design change 0366, slice M6-GK): a structure,
//! labelled "MTT", the initials of its name and its last letter ("MT" being the Militant's), with no panel. (M6-GR) It
//! fires homing missiles at air and ground (`sc_sim::defs::StructureDef::attack`): a turret's look, a structure's
//! footprint with a barrel on it that turns to face what it fires at (`Look::Turret`, the Laser Turret's, always
//! raised), its missiles drawn as missiles ([`ShotStyle::Missile`], the Rocketman's), along the path the sim steers
//! them. A selected one shows its InfoPanel alone.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "MTT",
    look: Look::Turret,
    shot: Some(ShotStyle::Missile),
    template: PanelTemplate::NONE,
};
