mod shared;
pub use shared::types;
pub use shared::utils;
pub mod game;
pub mod simulation;
pub mod ui;
#[cfg(any(test, feature = "testing"))]
pub mod testing {
    pub use crate::shared::testing::*;
    pub use crate::shared::testing::assertions;
}
