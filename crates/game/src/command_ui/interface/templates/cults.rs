//! The Cults' panels: the pool buildings' ([`armory`], slice M5-F2), the Armory's first, which choose the kind of
//! Recruit an order uses before the order (design change 0337), the Cathedral's next (slice M5-N4a); the Recruit's
//! ([`recruit`], slice M5-J1, design change 0322), with its two Construct menus ([`construct_menu`], design change
//! 0327); the Garrison's, its Eject All ([`garrison`], slice M5-N2); the Altar's, a unit type and a state before the
//! order ([`altar`], slice M5-N1b); the Disciple's, the BasicCombatUnit's with its spell in R ([`disciple`], slice
//! M5-N4a). The Recruitment Hall's is the Production template (slice M5-C).

pub mod altar;
pub mod armory;
pub mod construct_menu;
pub mod disciple;
pub mod garrison;
pub mod recruit;
