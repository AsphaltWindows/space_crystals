//! The Laser Turret (`factions/syndicate/structures/laser_turret.md`, design change 0260; slice M4-S1): upright, a
//! structure's footprint with a barrel on it that turns to face what it fires at (`crate::object_render`'s turret
//! look). Its laser is red: design change 0207 (`factions/syndicate/units/README.md`): "every Syndicate laser-shooting
//! unit and structure (Guard, Laser Battery, Laser Turret) fires a **red** laser". "The turret has no commands"
//! (`factions/syndicate/structures/laser_turret.md > Targeting`), so no panel: a selected one shows its InfoPanel
//! alone, and its site the site's Cancel (`crate::command_ui::interface::templates::SITE_GRID`). Its label is its
//! initials, "LT", as the Agent's D placeholder's was (slice M4-J1).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "LT",
    look: Look::Turret,
    shot: Some(ShotStyle::RedLaser),
    template: PanelTemplate::NONE,
};
