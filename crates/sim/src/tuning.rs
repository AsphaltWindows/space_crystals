//! The tunables of the simulation's systems (plan decision T54: "Modules now, data files at playtest"): the numbers
//! that tune how a system behaves and belong to no one object type, one module per system. A number of one object
//! type is in its def (`crate::defs`), a faction's own rule in its faction's defs module; a technical constant (units
//! of measure, hashing, file formats, the caps and margins that only bound a computation or absorb rounding) stays
//! with the code it serves.
//!
//! (M5-X1, plan decision T66) Each module is one struct of the ruleset (`crate::ruleset`), declared by [`tunables!`]:
//! a field per tunable, named as the constant it was, with its doc comment citing the design, or saying that it is a
//! placeholder and why it has its value; and the module's `BUILTIN`, its values, which the built-in rules hold. The
//! code reads the current rules, `rules().tuning.<system>.<field>`, once per step in a loop over the objects
//! (`ARCHITECTURE.md > Tunables`). A number that follows from others is a method, not a tunable of its own
//! (`Movement::give_up_ticks`). A slice adds its tunables at the end of its system's struct.

use serde::{Deserialize, Serialize};

/// Declares a system's tunables (module docs): the struct `$name`, one public field per tunable with its doc
/// comment, and the module's `pub const BUILTIN: $name`, each field at the value it is given here. The struct is data
/// (serialized by its fields' names, unknown fields refused), so that a data file maps onto it.
macro_rules! tunables {
    (
        $(#[$meta:meta])*
        $name:ident;
        $(
            $(#[$field_meta:meta])*
            $field:ident: $ty:ty = $value:expr,
        )+
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )+
        }

        /// The built-in values (`crate::ruleset::Ruleset::builtin`): each tunable as its doc gives it.
        pub const BUILTIN: $name = $name {
            $($field: $value,)+
        };
    };
}
pub(crate) use tunables;

pub mod air;
pub mod combat;
pub mod construction;
pub mod gathering;
pub mod holding;
pub mod movement;
pub mod orders;
pub mod paths;
pub mod production;
pub mod repair;
pub mod shields;
pub mod spells;
pub mod tunnel;
pub mod vision;

/// Every system's tunables (plan decision T66), one struct per module above: the ruleset's `tuning`
/// (`crate::ruleset::Ruleset`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tuning {
    pub air: air::Air,
    pub combat: combat::Combat,
    pub construction: construction::Construction,
    pub gathering: gathering::Gathering,
    pub holding: holding::Holding,
    pub movement: movement::Movement,
    pub orders: orders::Orders,
    pub paths: paths::Paths,
    pub production: production::Production,
    pub repair: repair::Repair,
    pub shields: shields::Shields,
    pub spells: spells::Spells,
    pub tunnel: tunnel::Tunnel,
    pub vision: vision::Vision,
}

/// Every system's built-in tunables.
pub const BUILTIN: Tuning = Tuning {
    air: air::BUILTIN,
    combat: combat::BUILTIN,
    construction: construction::BUILTIN,
    gathering: gathering::BUILTIN,
    holding: holding::BUILTIN,
    movement: movement::BUILTIN,
    orders: orders::BUILTIN,
    paths: paths::BUILTIN,
    production: production::BUILTIN,
    repair: repair::BUILTIN,
    shields: shields::BUILTIN,
    spells: spells::BUILTIN,
    tunnel: tunnel::BUILTIN,
    vision: vision::BUILTIN,
};
