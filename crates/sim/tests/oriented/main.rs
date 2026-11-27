//! Oriented Silhouettes (slice M5-V1, plan decision T78; design change 0033, `systems/units.md > UnitCollision >
//! Silhouette Rotation`: "A unit's Silhouette rotates with its facing [...] Collision, crushing, range measurement and
//! proximity all use the rotated shape"), played through the gate with the Seeker, 56 × 40 su (`factions/cults/units/
//! seeker.md`), one test binary:
//! - `range`: Range is measured to a turned target's edge: a Militant fires at a Seeker facing north from 4 grid units
//!   off its long side and off its short side, and not from 6 su further (`systems/combat.md > AttackAttributes >
//!   Range Measurement`);
//! - `soak`: Seekers among Guards and Peacekeepers on a map of walls and gaps, under seeded random orders: the
//!   invariants hold after every tick, the no-overlap invariant I1 between turned shapes among them, and every move
//!   ends.
//!
//! A turn's halving, the 48 su gap and a turn beside a wall are the sim's unit tests (`movement/collide/turn/tests.rs`,
//! on a Seeker given a TurnRate: its own turning isn't built), the geometry `shape/tests.rs`.

mod range;
mod soak;

use sc_sim::{ObjectId, PlayerId};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// The Seeker of each test: the first object it spawns.
const SEEKER: ObjectId = ObjectId(1);
