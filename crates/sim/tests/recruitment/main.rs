//! The Recruitment Halls (slice M5-C, plan decisions T56 and T57; `factions/cults/economy.md > RecruitmentHall`):
//! the tiles they claim, the Recruits they make for nothing up to their capacity, the Unit Control of their lineages,
//! and the HUD's figures. One test binary, in modules by topic:
//! - `timing`: a Hall at 100% fills 6 → 20 in 2688 ticks and then makes none, its Recruit in production counted from
//!   the start (design change 0342); at 50 tiles, capacity 10 and 384 ticks a Recruit; at 33, 6 and no more (design
//!   change 0341); Water lowers the effectiveness, buildings don't;
//! - `claims`: overlapping Halls, the freed tiles taken in the tick the first Hall is gone, the units of a destroyed
//!   Hall still used and its capacity gone, a unit's death restarting production from 0, the HUD's figures;
//! - `commands`: no X (design change 0342), a `Train`, the rally point (a patch makes the new Recruits gather), a game
//!   replayed and saved;
//! - `exits`: the one exit side, B (design change 0342), and the pause while its doorway is occupied.
//!
//! Every test runs under the invariants of `TestSim`.

mod claims;
mod commands;
mod exits;
mod timing;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, Faction, HallState, ObjectId, ObjectKind, PlayerId, ProductionCommand, SimEvent, SpawnAt, Tick, TilePos,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HALL: ObjectKind = ObjectKind::RecruitmentHall;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
/// `factions/cults/economy.md > RecruitmentHall > BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`.
const FRAMES: Tick = 192;

/// A flat `w` × `h` map on which players 1 and 2 play the Cults.
fn flat(w: usize, h: usize) -> ScenarioBuilder {
    cults(&vec![".".repeat(w); h].join("\n"))
}

/// The Cults on `ascii`.
fn cults(ascii: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(ascii)
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

/// A `w` × `h` map of Plane but for the Water tiles `water` says, by tile.
fn with_water(w: i32, h: i32, water: impl Fn(i32, i32) -> bool) -> String {
    let row = |y| (0..w).map(|x| if water(x, y) { '~' } else { '.' }).collect::<String>();
    (0..h).map(row).collect::<Vec<_>>().join("\n")
}

/// The Recruits `hall` made so far: `(tick, unit)`.
fn made(t: &TestSim, hall: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::UnitProduced { producer, unit } if producer == hall => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The ticks `hall`'s Recruits came out in.
fn ticks(t: &TestSim, hall: ObjectId) -> Vec<Tick> {
    made(t, hall).iter().map(|&(tick, _)| tick).collect()
}

/// `player`'s Cults Unit Control as the HUD shows it: `(used, available)`.
fn unit_control(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("in the game");
    (cap.used, cap.available)
}

fn hall_state(t: &TestSim, hall: ObjectId) -> HallState {
    t.sim().hall_state(hall).expect("a Hall")
}

/// `hall`'s CurrentBuildProgress, in tiles (`production::total_of`), or `None` with nothing in production.
fn progress(t: &TestSim, hall: ObjectId) -> Option<u32> {
    t.sim().production_status(hall).map(|status| status.progress)
}

fn production(objects: &[ObjectId], command: ProductionCommand) -> Command {
    Command::Production {
        objects: objects.to_vec(),
        command,
    }
}
