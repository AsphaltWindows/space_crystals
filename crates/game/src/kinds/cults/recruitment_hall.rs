//! The Recruitment Hall (`factions/cults/structures/recruitment_hall.md`): a structure, labelled with its initials,
//! "RH". It has the Production template (slice M5-C; plan decision T23: "every type that produces units has the
//! Production template"), which for a Hall that recruits by itself is C Set Rally Point alone, with no X (design change
//! 0342: "Recruits can't be cancelled"), its right-click setting the rally point (`> RecruitmentHall >
//! ObjectInterfaceState[RecruitmentHall]`; `crate::command_ui::interface::templates::production`).

use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "RH",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::of(InterfaceTemplate::Production),
};
