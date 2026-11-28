//! The Syndicate's panels (slice M4-J2, design change 0257): the Tunnel's ([`tunnel`]), with its EjectMenu
//! ([`eject_menu`]) and its ExpandMenu ([`expand_menu`], slice M4-F), and the Hideout's ([`hideout`]), (M6-SV) the
//! Vehicle Assembly's ([`vehicle_assembly`]) and (M6-SD) the Aerospace Assembly's ([`aerospace_assembly`]), the
//! Production template with their units not in the game yet in their slots; and the Agent's ([`agent`], slice M4-J1),
//! with its Build Tunnel and its Drop Off, and (M6-SB) the Shadebringer's ([`shadebringer`]), with its toggles.

pub mod aerospace_assembly;
pub mod agent;
pub mod eject_menu;
pub mod expand_menu;
pub mod hideout;
pub mod shadebringer;
pub mod tunnel;
pub mod vehicle_assembly;
