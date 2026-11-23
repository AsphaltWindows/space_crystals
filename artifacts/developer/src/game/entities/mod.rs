#![allow(dead_code)]
//! Declarative entity definition system.
//!
//! Provides traits (`EntityDefinition`, `UnitDefinition`, `StructureDefinition`)
//! and a runtime `EntityRegistry` for looking up entity metadata and spawn functions.

pub mod types;
pub mod utils;
pub mod registry;
pub mod factions;
pub mod neutral;

pub use registry::EntitiesPlugin;
pub use types::*;
