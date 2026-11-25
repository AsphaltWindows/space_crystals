//! The Recruitment Hall (`factions/cults/economy.md > RecruitmentHall`): a structure, labelled with its initials, "RH".
//! No panel yet: its Set Rally Point and its Cancel Production (`> RecruitmentHall >
//! ObjectInterfaceState[RecruitmentHall]`) come with its production, slice M5-C.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "RH",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
