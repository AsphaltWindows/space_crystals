//! The client's feel tunables, kept apart from the logic that uses them (`ARCHITECTURE.md > Tunables`): the camera's
//! ([`camera`]), the input's ([`input`]) and how long feedback shows ([`feedback`], the minimap alerts' throttle
//! with it). Each keeps the name it had where it is used, which still names it there, and has a design citation or
//! "Placeholder (not in the design)". Colours and drawing sizes stay with the drawing code, and technical constants
//! where they are used.

pub mod camera;
pub mod feedback;
pub mod input;
