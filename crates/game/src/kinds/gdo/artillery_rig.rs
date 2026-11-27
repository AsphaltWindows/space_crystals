//! The Artillery Rig's two types (`factions/gdo/units/artillery_rig.md`, design change 0366, slice M6-GK): the Rig and
//! the Artillery Rig (Deployed), a type each (`> Modes`). They share a tracked hull ([`UnitShape::Tracked`], the Hand
//! of Heaven's), each with its label on it: "ARG", the mobile Rig's, the initials of its name and its last letter
//! ("AR" being the Altar's), and "ARD", the deployed one's, the D its Deployed's. The mobile Rig is unarmed; the
//! deployed one's shell is drawn as the Hand of Heaven's is, a tracer's lobbed ball ([`ShotStyle::Tracer`]), its blast
//! the ring of every blast. Their panels are the BasicCombatUnit template's, the mobile Rig's without its attack
//! commands, each with its R, Deploy or Undeploy (slice M6-GA: `crate::command_ui::interface::templates::gdo::
//! artillery_rig`); the Factory trains the mobile Rig in its R.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::gdo::artillery_rig;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

/// The Artillery Rig, mobile.
pub const KIND: ClientKind = ClientKind {
    label: "ARG",
    look: Look::Unit {
        shape: UnitShape::Tracked,
        tagged: true,
    },
    shot: None,
    template: artillery_rig::MOBILE,
};

/// The Artillery Rig (Deployed): the mobile Rig's look, with its gun and its attacks.
pub const DEPLOYED: ClientKind = ClientKind {
    label: "ARD",
    shot: Some(ShotStyle::Tracer),
    template: artillery_rig::DEPLOYED,
    ..KIND
};
