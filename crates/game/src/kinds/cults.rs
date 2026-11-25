//! The Cults' kinds (`factions/cults/`, slice M5-A): the Recruit, the Militant and the Gunner, the Recruitment Hall,
//! the Storage and the Armory. Their labels follow [`Planned::label`]'s rules (a one-word name's first and last
//! letters, a two-word name's initials), but for the Gunner's: "GR" is the Grenadier's, so it is "GU" (plan §3.3 A).
//! Their panels come with the slices that build their commands: the Hall's with slice M5-C, the Armory's with M5-F2
//! and the Recruit's with M5-J1.
//!
//! [`Planned::label`]: crate::command_ui::interface::templates::Planned::label

pub mod armory;
pub mod gunner;
pub mod militant;
pub mod recruit;
pub mod recruitment_hall;
pub mod storage;
