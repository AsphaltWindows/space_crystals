//! The Cults' kinds (`factions/cults/`): slice M5-A's six, the Recruit, the Militant and the Gunner, the Recruitment
//! Hall, the Storage and the Armory, and slice M5-G2's fifteen, every other kind of the design (plan decision T69): the
//! Gunworks, the Garrison, the Chop Shop, the Machine Shop, the Airstrip, the Scrap Hangar, the Altar, the Temple, the
//! Cathedral and the Shrine; the Disciple, the Seeker, the Sparrow, the Hand of Heaven and the Prayer-mobile. Their
//! labels follow [`Planned::label`]'s rules (a one-word name's first and last letters, a name of more words its
//! initials), but where those are another's: the Gunner's "GU" ("GR" is the Grenadier's, plan §3.3 A), the Garrison's
//! "GA" ("GN" the Generator's), the Shrine's "SR" ("SE" the Storage's), the Airstrip's "AP" and the Seeker's "SKR"
//! ("SR" the Shrine's, "SK" the Skylance's). A building of the Construct menus keeps the label its entry had (slice
//! M5-J1). Their panels come with the slices that build their commands: the Hall's with slice M5-C, the Recruit's
//! with M5-J1 and the Armory's with M5-F2; slice M5-G2's buildings have none (the Gunworks, the Machine Shop and the
//! Scrap Hangar "no panel commands", the others until their slices: the Garrison's came with M5-N2, the Altar's with
//! M5-N1b, the Cathedral's, the Armory's template, with M5-N4a, the Temple's with M5-N3), its armed units the
//! BasicCombatUnit template, and its Prayer-mobile none until its ability's slice; the Disciple's, the BasicCombatUnit
//! template with its Shroud held in R, came with slice M5-N4a.
//!
//! [`Planned::label`]: crate::command_ui::interface::templates::Planned::label

pub mod airstrip;
pub mod altar;
pub mod armory;
pub mod cathedral;
pub mod chop_shop;
pub mod disciple;
pub mod garrison;
pub mod gunner;
pub mod gunworks;
pub mod hand_of_heaven;
pub mod machine_shop;
pub mod militant;
pub mod prayer_mobile;
pub mod recruit;
pub mod recruitment_hall;
pub mod scrap_hangar;
pub mod seeker;
pub mod shrine;
pub mod sparrow;
pub mod storage;
pub mod temple;
