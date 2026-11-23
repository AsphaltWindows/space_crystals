//! Cults structure stat constants.

// Recruitment Center
pub const RC_MAX_HP: f32 = 1000.0; // Same as DC (4x4 primary structure)
pub const RC_POINT_ARMOR: u32 = 1;
pub const RC_FULL_ARMOR: u32 = 16;

// Armory
pub const ARMORY_MAX_HP: f32 = 300.0;
pub const ARMORY_POINT_ARMOR: u32 = 1;
pub const ARMORY_FULL_ARMOR: u32 = 4;
pub const ARMORY_SC_COST: u32 = 150; // TBD placeholder
pub const ARMORY_INTERNAL_RECRUIT_CAPACITY: usize = 10;
pub const SOLDIER_TRAINING_COST: u32 = 75; // TBD placeholder
pub const SOLDIER_TRAINING_FRAMES: u32 = 160; // TBD placeholder
pub const GUNNER_TRAINING_COST: u32 = 100; // TBD placeholder
pub const GUNNER_TRAINING_FRAMES: u32 = 200; // TBD placeholder

// Storage
pub const STORAGE_MAX_HP: f32 = 200.0;
pub const STORAGE_POINT_ARMOR: u32 = 1;
pub const STORAGE_FULL_ARMOR: u32 = 4;
/// Build frames for Cults Storage with 1 Recruit (~18.75 seconds at 16fps)
pub const STORAGE_BUILD_FRAMES: u32 = 300;
