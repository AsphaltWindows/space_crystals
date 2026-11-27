//! The client's feel tunables, kept apart from the logic that uses them (`ARCHITECTURE.md > Tunables`): the camera's
//! ([`camera`]), the input's ([`input`]), how long feedback shows ([`feedback`], the minimap alerts' throttle with it)
//! and (slice M5-X2) the looks a playtest tunes ([`look`]: the Shroud's clouds, the bars over the units, the air units'
//! lift and its shadow's offset). Colours and the other drawing sizes stay with the drawing code, and technical
//! constants where they are used.
//!
//! (M5-X2) Each module is one struct, declared by [`tunables!`] as the sim's tuning modules are: a field per tunable,
//! named as the constant it was, with its doc comment citing the design or saying that it is a placeholder and why; and
//! the module's `BUILTIN`, its values. The four are the [`ClientTuning`] resource, which the client's systems read and
//! the data files tune while the game runs (`client/<module>.json`, `crate::data`). A test reads a module's `BUILTIN`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Declares a module's tunables (module docs): the struct `$name`, one public field per tunable with its doc comment,
/// and the module's `pub const BUILTIN: $name`, each field at the value it is given here. The struct is data
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
        #[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )+
        }

        /// The built-in values (`crate::tuning::ClientTuning::BUILTIN`): each tunable as its doc gives it.
        pub const BUILTIN: $name = $name {
            $($field: $value,)+
        };
    };
}
pub(crate) use tunables;

pub mod camera;
pub mod feedback;
pub mod input;
pub mod look;

/// (M5-X2) The client's feel tunables (module docs), as the data files have tuned them: the built-in ones without
/// `--tuning`. Each plugin whose systems read it makes sure it is there.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientTuning {
    pub camera: camera::CameraTuning,
    pub feedback: feedback::FeedbackTuning,
    pub input: input::InputTuning,
    pub look: look::LookTuning,
}

impl ClientTuning {
    /// Every module's `BUILTIN`.
    pub const BUILTIN: ClientTuning = ClientTuning {
        camera: camera::BUILTIN,
        feedback: feedback::BUILTIN,
        input: input::BUILTIN,
        look: look::BUILTIN,
    };
}

impl Default for ClientTuning {
    fn default() -> Self {
        Self::BUILTIN
    }
}
