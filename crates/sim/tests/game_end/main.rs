//! Defeat and the end of the game (slice M2-F2): `core/game_rules.md > Defeat Condition`, `> On Defeat` and `> End
//! of Game`, plan decision T8, design changes 0101 and 0102; the draw, design change 0163 (slice M3-V).
//!
//! - A player is defeated in the tick they lose their last structure, plates included, whether to a debug removal
//!   or in combat; a player who never owned one (an M1 sandbox) never is.
//! - Their units become neutral: no orders, standing where they are, nobody's eyes, never shot at by themselves,
//!   but open to an explicit Attack. The gate refuses every command of theirs.
//! - The defeat that leaves one side standing ends the game, recorded in the hashed state; from then on the gate
//!   refuses every command.
//! - A save loads and continues identically across a defeat and the end.
//!
//! `TestSim` checks the invariants after every tick: the movement invariants, the vision (V1) and the path
//! knowledge (K1), so the tick of a defeat keeps them all.
//!
//! One test binary, a module a point: `defeat`; `on_defeat`, with a defeat that hands over no producer;
//! `end_of_game`, with the draws of design change 0163; and `saves`.

mod defeat;
mod end_of_game;
mod on_defeat;
mod saves;

use sc_sim::mover::{Locomotion, PathState};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Category, Command, DebugCommand, GameSetup, MoveGoal, ObjectId, ObjectKind, OrderKind, Orders, Outcome, PlayerId,
    ProductionCommand, Reject, Sim, SimEvent, Slot, SpawnAt, Target, Tick, TilePos, Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);
const PK: ObjectKind = ObjectKind::Peacekeeper;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn remove(id: u32) -> DebugCommand {
    DebugCommand::Remove { id: ObjectId(id) }
}

fn spawn(kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation: Default::default(),
    }
}

/// `scenario` with a third slot, and a GDO player in each slot: player `n` in slot `n`.
fn three_players(builder: ScenarioBuilder) -> TestSim {
    let mut scenario = builder.build();
    scenario.map.slots.push(Slot::new(TilePos::new(16, 8)));
    scenario.setup = GameSetup::default_for(&scenario.map);
    TestSim::new(scenario)
}

/// Every `PlayerDefeated` and `GameOver` so far, with its tick.
fn game_events(t: &TestSim) -> Vec<(Tick, SimEvent)> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::PlayerDefeated { .. } | SimEvent::GameOver { .. }))
        .cloned()
        .collect()
}

fn defeated(player: PlayerId) -> SimEvent {
    SimEvent::PlayerDefeated { player }
}

fn game_over(winners: &[PlayerId]) -> SimEvent {
    SimEvent::GameOver {
        winners: winners.to_vec(),
    }
}

fn defeated_at(t: &TestSim, player: PlayerId) -> Option<Tick> {
    t.sim().player(player).unwrap().defeated_at
}

/// A three-player game on a 32 × 16 map, each player with a Generator in a corner and players 1 and 2 with two
/// Peacekeepers each, out of each other's sight:
/// 1. player 1's Generator, (1, 1) to (2, 2);
/// 2. player 2's Generator, (28, 1) to (29, 2);
/// 3. player 3's Generator, (1, 13) to (2, 14);
/// 4. and 5. player 1's Peacekeepers, at the centres of tiles (10, 8) and (10, 10);
/// 6. and 7. player 2's, at the centres of tiles (20, 8) and (20, 10).
fn three_way() -> TestSim {
    three_players(
        ScenarioBuilder::new(&flat(32, 16))
            .spawn(ObjectKind::Generator, Some(P1), tile(1, 1))
            .spawn(ObjectKind::Generator, Some(P2), tile(28, 1))
            .spawn(ObjectKind::Generator, Some(P3), tile(1, 13))
            .spawn(PK, Some(P1), SpawnAt::Point(centre(10, 8)))
            .spawn(PK, Some(P1), SpawnAt::Point(centre(10, 10)))
            .spawn(PK, Some(P2), SpawnAt::Point(centre(20, 8)))
            .spawn(PK, Some(P2), SpawnAt::Point(centre(20, 10))),
    )
}
