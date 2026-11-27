//! A unit type's movement numbers, one variant per movement model — `systems/units.md > Unit >
//! UnitBaseAttributes[UnitBase]` (moved out of `defs/units.rs` at slice M5-V3, as it neared 500 lines): what each
//! model reads of them, and how a unit backs up.

use serde::{Deserialize, Serialize};

use super::MovementModel;

/// `systems/units.md > Unit > UnitBaseAttributes[UnitBase]`: a unit type's movement numbers, one variant per
/// movement model. An acceleration or deceleration of `None` is infinite: the unit reaches its speed, or stops,
/// within the tick.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MovementAttrs {
    /// `systems/units.md > TurnRateMovement`.
    TurnRate {
        /// Radians per tick.
        turn_rate: f32,
        acceleration: Option<f32>,
        deceleration: Option<f32>,
        /// Space units per tick.
        max_speed: f32,
    },
    /// `systems/units.md > FixedTurnRadiusMovement`.
    FixedTurnRadius {
        /// Space units.
        minimum_turn_radius: f32,
        forward_acceleration: Option<f32>,
        forward_max_speed: f32,
        reverse_acceleration: Option<f32>,
        reverse_max_speed: f32,
        deceleration: Option<f32>,
    },
    /// `systems/units.md > SpeedTurnRadiusMovement` (design change 0417: the tread law, which TrackWidth and
    /// ReverseMaxSpeed replace SpeedToTurnRadiusRatio for).
    SpeedTurnRadius {
        /// TrackWidth: the distance between the two treads, space units.
        track_width: f32,
        /// Speeding up either way, forward or backing up.
        acceleration: Option<f32>,
        deceleration: Option<f32>,
        /// Space units per tick: each tread's top speed, and the forward speed's.
        max_speed: f32,
        /// ReverseMaxSpeed, space units per tick.
        reverse_max_speed: f32,
    },
    /// `systems/units.md > DragMovement`: top speed is where thrust and drag balance.
    Drag {
        forward_acceleration: f32,
        omni_directional_acceleration: f32,
        drag_ratio: f32,
        /// Radians per tick.
        turn_rate: f32,
    },
    /// `systems/units.md > GliderMovement`.
    Glider {
        idle_speed: f32,
        max_speed: f32,
        acceleration: Option<f32>,
        deceleration: Option<f32>,
        max_centripetal_acceleration: f32,
    },
}

impl MovementAttrs {
    pub fn model(&self) -> MovementModel {
        match self {
            MovementAttrs::TurnRate { .. } => MovementModel::TurnRate,
            MovementAttrs::FixedTurnRadius { .. } => MovementModel::FixedTurnRadius,
            MovementAttrs::SpeedTurnRadius { .. } => MovementModel::SpeedTurnRadius,
            MovementAttrs::Drag { .. } => MovementModel::Drag,
            MovementAttrs::Glider { .. } => MovementModel::Glider,
        }
    }

    /// The top forward speed, space units per tick. `DragMovement`: `(OmniDirectionalAcceleration +
    /// ForwardAcceleration) / DragRatio` (`systems/units.md > DragMovement`).
    pub fn max_speed(&self) -> f32 {
        match *self {
            MovementAttrs::TurnRate { max_speed, .. }
            | MovementAttrs::SpeedTurnRadius { max_speed, .. }
            | MovementAttrs::Glider { max_speed, .. } => max_speed,
            MovementAttrs::FixedTurnRadius { forward_max_speed, .. } => forward_max_speed,
            MovementAttrs::Drag {
                forward_acceleration,
                omni_directional_acceleration,
                drag_ratio,
                ..
            } => (omni_directional_acceleration + forward_acceleration) / drag_ratio,
        }
    }

    /// The most the unit turns per tick, radians, for the models with a fixed turn rate
    /// (`systems/combat.md > LocomotionOrientationConstraints`); `None` for the models whose turning depends on
    /// their speed.
    pub fn turn_rate(&self) -> Option<f32> {
        match *self {
            MovementAttrs::TurnRate { turn_rate, .. } | MovementAttrs::Drag { turn_rate, .. } => Some(turn_rate),
            MovementAttrs::FixedTurnRadius { .. }
            | MovementAttrs::SpeedTurnRadius { .. }
            | MovementAttrs::Glider { .. } => None,
        }
    }

    /// (M5-V2) How the unit backs up on the Reverse order (`systems/controls/command_behaviors.md >
    /// ReversingToLocation`), for the models that reverse: a FixedTurnRadiusMovement unit by its ReverseMaxSpeed and
    /// ReverseAcceleration, (M5-V3) a SpeedTurnRadiusMovement unit by its ReverseMaxSpeed and its one Acceleration
    /// (design change 0417: "reverse is capped at ReverseMaxSpeed"). `None` for the others.
    pub fn reverse_speed(&self) -> Option<ReverseSpeed> {
        match *self {
            MovementAttrs::FixedTurnRadius {
                reverse_max_speed,
                reverse_acceleration,
                ..
            } => Some(ReverseSpeed {
                max_speed: reverse_max_speed,
                acceleration: reverse_acceleration,
            }),
            MovementAttrs::SpeedTurnRadius {
                reverse_max_speed,
                acceleration,
                ..
            } => Some(ReverseSpeed {
                max_speed: reverse_max_speed,
                acceleration,
            }),
            MovementAttrs::TurnRate { .. } | MovementAttrs::Drag { .. } | MovementAttrs::Glider { .. } => None,
        }
    }

    /// (M5-V3) SpinRate, radians per tick, for SpeedTurnRadiusMovement: `2 × MaxSpeed ÷ TrackWidth`, the rate it turns
    /// at when stopped, treads in opposite directions (`systems/units.md > SpeedTurnRadiusMovement > The turning
    /// law`, design change 0417). `None` for the other models.
    pub fn spin_rate(&self) -> Option<f32> {
        match *self {
            MovementAttrs::SpeedTurnRadius {
                track_width, max_speed, ..
            } => Some(2.0 * max_speed / track_width),
            _ => None,
        }
    }
}

/// (M5-V2) A model's numbers for backing up ([`MovementAttrs::reverse_speed`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ReverseSpeed {
    /// The top speed backwards, space units per tick: ReverseMaxSpeed.
    pub max_speed: f32,
    /// Space units per tick per tick; `None` where the design gives none.
    pub acceleration: Option<f32>,
}
