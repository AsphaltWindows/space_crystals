//! The Radar Station and its Scan (slice M6-GR, plan decision T92; `factions/gdo/structures/radar_station.md`),
//! through the gate, one test binary:
//! - `building`: the Deployment Center constructs a Radar Station only for a player with a Generator, and places it
//!   complete: it scans from its first tick;
//! - `scans`: a Scan at an unexplored point shows the disc of 6 grid units round it for exactly 128 ticks, which then
//!   turns Explored after the fog delay; at any point of the map, seen or not, and nowhere off it; the enemy's units and
//!   buildings in it are seen, an underground expansion stays hidden (D39's placeholder); two scans overlap;
//! - `cooldown`: the station's next Scan is refused for 960 ticks, while another station scans (design change 0422,
//!   the gate's part: each press names the ActiveGroup's one station);
//! - `power`: at 30 generated for 51 consumed the cooldown takes 1632 ticks, about 102 s, and Generators that bring the
//!   Power back to positive give it full speed at once (design change 0423);
//! - `saves`: a game with scans and cooldowns under way goes on alike after a save and in its replay, and a save with a
//!   scan off the map or over is refused.
//!
//! Every test runs under the invariants of `TestSim`, V1 among them, which rebuilds each player's vision from its
//! history, the scans included (`Vision::check`).

mod building;
mod cooldown;
mod power;
mod saves;
mod scans;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::vision::Scan;
use sc_sim::{
    AreaAbility, Command, ConstructionCommand, DebugCommand, Faction, ObjectId, ObjectKind, PlayerCommand, PlayerId,
    Progress, Reject, SimEvent, SpawnAt, Tick, TilePos, Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SCAN: AreaAbility = AreaAbility::Scan;
const RADAR: ObjectKind = ObjectKind::RadarStation;
const GENERATOR: ObjectKind = ObjectKind::Generator;

/// `> Placeholder stats`: "reveals a circle of radius **6 grid units** for **8 seconds** (128 frames); **cooldown 60
/// seconds** (960 frames)".
const RADIUS: i32 = 6;
const SCAN_TICKS: Tick = 128;
const COOLDOWN_TICKS: Tick = 960;
const FOG_DELAY_TICKS: Tick = sc_sim::tuning::vision::BUILTIN.fog_delay_ticks;

/// [`base`]'s Deployment Center, Generator and Radar Stations.
const DC: u32 = 1;
const S1: u32 = 3;
const S2: u32 = 4;
/// A tile far east of [`base`]'s buildings, which player 1 has never seen, and its centre, where the tests scan.
const FAR_TILE: TilePos = TilePos::new(36, 20);
const FAR: Vec2 = Vec2::new(36.0 * 64.0 + 32.0, 20.0 * 64.0 + 32.0);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Player 1's Deployment Center 1 at (2, 2) to (5, 5), its Generator 2 at (8, 2) and its Radar Stations 3 and 4 at
/// (2, 8) and (6, 8), on a flat map of 48 × 32 tiles: Power 50 generated for 40 consumed. The stations see 8 grid units
/// round them and the Deployment Center 6, so player 1 has seen nothing east of tile 16.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(48, 32))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(GENERATOR, Some(P1), tile(8, 2))
        .spawn(RADAR, Some(P1), tile(2, 8))
        .spawn(RADAR, Some(P1), tile(6, 8))
}

/// `scenario` started, player 1 with 1000 Space Crystals from tick 1, which has run.
fn started(scenario: ScenarioBuilder) -> TestSim {
    let mut t = scenario.start();
    t.debug_at(1, fund(P1, 1000));
    t.run_until(1);
    t
}

fn fund(player: PlayerId, crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// The Radar Station's Q (`Command::Cast`): a Scan at `at` from one of `casters`.
fn scan(casters: &[u32], at: Vec2) -> Command {
    Command::Cast {
        casters: casters.iter().map(|&n| id(n)).collect(),
        ability: SCAN,
        at,
    }
}

fn construction(constructor: u32, command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: vec![id(constructor)],
        command,
    }
}

fn place(origin: TilePos) -> ConstructionCommand {
    ConstructionCommand::Place {
        origin,
        orientation: Default::default(),
    }
}

fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// Schedules player 1's `command` for the next tick and runs it: the tick.
fn next(t: &mut TestSim, command: Command) -> Tick {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).run_until(tick);
    tick
}

/// The tiles of a scan's disc round the centre of `centre`: those whose centres lie within [`RADIUS`] tiles of it
/// (`core/vision.md`: a tile is seen when its centre is within the sight, the boundary included), on the map.
fn disc(centre: TilePos) -> Vec<TilePos> {
    let mut tiles = Vec::new();
    for y in centre.y - RADIUS..=centre.y + RADIUS {
        for x in centre.x - RADIUS..=centre.x + RADIUS {
            let (dx, dy) = (x - centre.x, y - centre.y);
            if dx * dx + dy * dy <= RADIUS * RADIUS {
                tiles.push(TilePos::new(x, y));
            }
        }
    }
    tiles
}

/// What `player` sees of `tile` now: its state, and whether it lingers for the fog delay.
fn seen(t: &TestSim, player: PlayerId, tile: TilePos) -> (VisibilityState, bool) {
    let vision = t.sim().vision().player(player).expect("in the game");
    (vision.state(tile), vision.lingers(tile))
}

/// Whether every tile of `tiles` is in `state` for `player`, lingering or not as `lingering` says.
fn all_seen(t: &TestSim, player: PlayerId, tiles: &[TilePos], state: VisibilityState, lingering: bool) -> bool {
    tiles.iter().all(|&tile| seen(t, player, tile) == (state, lingering))
}

/// The tiles Visible to `player` now, row-major.
fn visible(t: &TestSim, player: PlayerId) -> Vec<TilePos> {
    let map = t.sim().map();
    let mut tiles = Vec::new();
    for y in 0..map.height() as i32 {
        for x in 0..map.width() as i32 {
            let tile = TilePos::new(x, y);
            if t.sim().visibility(player, tile) == VisibilityState::Visible {
                tiles.push(tile);
            }
        }
    }
    tiles
}

/// `player`'s scans under way.
fn scans(t: &TestSim, player: PlayerId) -> Vec<Scan> {
    let vision = t.sim().vision().player(player).expect("in the game");
    vision.scans().copied().collect()
}

/// Station `n`'s cooldown under way (`Object::cooldown`).
fn cooldown(t: &TestSim, n: u32) -> Option<Progress> {
    t.sim().object(id(n)).expect("on the map").cooldown
}

/// The Scans cast so far: (tick, station, point).
fn casts(t: &TestSim) -> Vec<(Tick, u32, Vec2)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Cast {
                caster,
                ability: SCAN,
                at,
                ..
            } => Some((*tick, caster.0, at)),
            _ => None,
        })
        .collect()
}
