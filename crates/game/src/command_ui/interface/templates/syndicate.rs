//! The Syndicate's panels (slice M4-J2, design change 0257): the Tunnel's ([`tunnel`]), with its EjectMenu
//! ([`eject_menu`]) and its ExpandMenu ([`expand_menu`], slice M4-F), and the Hideout's ([`hideout`]) and (M6-SV) the
//! Vehicle Assembly's ([`vehicle_assembly`]), the Production template with their units not in the game yet in their
//! slots; and the Agent's ([`agent`], slice M4-J1), with its Build Tunnel and its Drop Off.

pub mod agent;
pub mod eject_menu;
pub mod expand_menu;
pub mod hideout;
pub mod tunnel;
pub mod vehicle_assembly;
