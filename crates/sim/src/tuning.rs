//! The tunables of the simulation's systems (plan decision T54: "Modules now, data files at playtest"): the numbers
//! that tune how a system behaves and belong to no one object type, one module per system. A number of one object
//! type is in its def (`crate::defs`), a faction's own rule in its faction's defs module; a technical constant (units
//! of measure, hashing, file formats, the caps and margins that only bound a computation or absorb rounding) stays
//! with the code it serves.
//!
//! Each constant keeps the name it had in its system's code, and its doc comment cites the design, or says that it is
//! a placeholder and why it has its value. A system's module maps one to one onto the struct that a data file will
//! load at playtest prep. A slice adds its constants at the end of its system's module.

pub mod combat;
pub mod construction;
pub mod gathering;
pub mod movement;
pub mod orders;
pub mod paths;
pub mod production;
pub mod repair;
pub mod shields;
pub mod tunnel;
pub mod vision;
