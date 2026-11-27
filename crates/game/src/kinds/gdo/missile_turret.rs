//! The Missile Turret (`factions/gdo/structures/missile_turret.md`, design change 0366, slice M6-GK): a structure,
//! labelled "MTT", the initials of its name and its last letter ("MT" being the Militant's), with no panel. Its
//! missiles, the barrel a turret's look draws (`Look::Turret`) and its Defense System's entry are slice M6-GR's: until
//! then it has no attack, and the look of a structure that doesn't attack.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "MTT",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
