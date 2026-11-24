//! Unit production (slice M2-D2; `factions/gdo/structures.md > Barracks`; the Colonists' portals, slice M3-D), one
//! test binary:
//! - `queue`: the queue and its costs, the commands and their gate, group production (design change 0083), the
//!   refund of a destroyed producer (0084), a game replayed and saved;
//! - `rally`: rally points (`> RallyPoint behavior`, design changes 0086 and 0088);
//! - `exit`: the exit doorway (`core/entities.md > Structure Type > Exit Doorway`, design change 0080): where a unit
//!   appears, in every orientation, and how it gets out;
//! - `stalls`: blocked doorways (production pauses, design change 0085's refund), saves, and a long random game;
//! - `portals`: the Robo Portal and the Bio Portal (`factions/colonists/structures.md`, plan decision T19): which of
//!   their doors a unit leaves by, their pause, rally points on enemies, refunds and Beacon Capacity.
//!
//! Every test runs under the invariants of `TestSim` (I1: no two units overlap; I2 with its doorway exception: only
//! the released unit, only its own building's tiles, only outwards).

mod doors;
mod exit;
mod portals;
mod queue;
mod rally;
mod stalls;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Command, ObjectId, ObjectKind, PlayerId, ProductionCommand, SpawnAt, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);

const P2: PlayerId = PlayerId(2);

const PK: ObjectKind = ObjectKind::Peacekeeper;

/// The Barracks of [`base`].
const BARRACKS: ObjectId = ObjectId(2);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn at(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

/// Player 1's Deployment Center (1; Power +30) and Barracks (2; Power −30, so at full power) on a 30 × 16 map. The
/// Barracks covers tiles (8, 4) to (10, 5): x 512..704, y 256..384, and its side B faces east, so its doorway is
/// at (704, 320).
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(30, 16))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 4))
}

fn production(objects: &[ObjectId], command: ProductionCommand) -> Command {
    Command::Production {
        objects: objects.to_vec(),
        command,
    }
}
