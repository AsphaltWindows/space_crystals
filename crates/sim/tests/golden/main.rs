//! Golden hashes (`ARCHITECTURE.md > Replays and golden hashes`): fifteen small games, defined below and in
//! `colonists.rs`, `syndicate.rs`, `cults.rs` and the others and recorded as replays in `tests/golden/`, each with the
//! state hash at tick 0, every 32 ticks and at its end (`harness.rs`). A run must give the blessed hashes: a change
//! that alters how any of these games plays out fails here, at the first checkpoint that differs. (M5-M1) Each game is
//! checked by the invariants after every step, as a `TestSim` game is (`sc_sim::testing::invariants`). The replays
//! play in the client too (`cargo run -- --replay crates/sim/tests/golden/<name>.json`).
//!
//! When the change is intended, re-bless the files and review the diff (the checkpoint hashes) before committing:
//!
//! ```sh
//! SC_BLESS=1 cargo test -p sc_sim golden
//! ```
//!
//! The games are defined in `units.rs` (small games of units on test maps: a duel, an AttackMove battle, a
//! group move through a choke, a patrol and a chase into the fog), `gdo.rs` (the GDO opening on `dev_skirmish`),
//! `colonists.rs` (the Colonists' two), `syndicate.rs` (the Syndicate's two), `cults.rs` (the Cults' two),
//! `garrison.rs` (a Garrison's four against Agents and Grenadiers), `altar.rs` (three Militants augmented at an Altar,
//! against Agents) and below (the Riot Shield game), and every test is below.

mod altar;
mod colonists;
mod cults;
mod garrison;
mod gdo;
mod harness;
mod syndicate;
mod units;

use std::collections::BTreeMap;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::testing::build_order::{Books, BuildOrder, Order};
use sc_sim::{
    Ability, Command, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ProductionCommand, SU_PER_GU,
    Scenario, ShieldMode, Sim, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2, riot_shield, scenarios,
};

use gdo::gdo_opening;
use harness::{Game, Script, golden};
use units::{attack_move_battle, choke_group_move, duel_3v3, fog_chase, patrol};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
const GUARD: ObjectKind = ObjectKind::Guard;

/// Open Plane ground, `w` × `h` tiles.
fn open(w: usize, h: usize) -> String {
    format!("{}\n", ".".repeat(w)).repeat(h)
}

/// A unit's spawn at the centre of tile `(x, y)`.
fn at(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

fn point(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn order(player: PlayerId, units: impl IntoIterator<Item = u32>, kind: OrderKind) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: units.into_iter().map(ObjectId).collect(),
            kind,
            queue: false,
        },
    }
}

/// Player 2 plays the Syndicate, whose unit the Guard is.
fn syndicate_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.build();
    scenario.setup.players[1].faction = Faction::Syndicate;
    scenario
}

/// Player 1's `units` raise (`raise`) or lower their Riot Shields.
fn shield(units: impl IntoIterator<Item = u32>, raise: bool) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Ability {
            units: units.into_iter().map(ObjectId).collect(),
            ability: Ability::RiotShield { raise },
        },
    }
}

fn units_of(sim: &Sim, player: PlayerId) -> Vec<&sc_sim::Object> {
    sim.objects().filter(|o| o.owner == Some(player)).collect()
}

/// Player 1 raises the Riot Shields of three Peacekeepers (ids 1–3, `Command::Ability`) and walks them, at half
/// speed, up to a Guard (player 2, id 4), which shoots at them: behind their shields they don't shoot back. On the
/// way #3 lowers its shield and raises it again before it is down. At tick [`SHIELDS_DOWN`] all three lower their
/// shields, and then they kill the Guard; with its laser rifle (design change 0186) it takes #2 with it.
fn riot_shield() -> Game {
    let mut builder = ScenarioBuilder::new(&open(16, 10));
    for y in [4, 5, 6] {
        builder = builder.spawn(PK, Some(P1), at(3, y));
    }
    builder = builder.spawn(GUARD, Some(P2), at(11, 5));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, shield(1..=3, true)),
            (1, order(P1, 1..=3, OrderKind::Move(Target::Point(point(8, 5))))),
            (30, shield([3], false)),
            (32, shield([3], true)),
            (SHIELDS_DOWN, shield(1..=3, false)),
        ],
        script: None,
        end: 240,
        during: |tick, sim| {
            if tick == SHIELDS_DOWN - 1 {
                let guard = sim.object(ObjectId(4)).expect("the Guard lives");
                assert_eq!(guard.hp, Some(80.0), "the shielded Peacekeepers didn't shoot");
                let units = units_of(sim, P1);
                assert!(units.iter().any(|u| u.hp < Some(50.0)), "the Guard shot at them");
                for unit in units {
                    assert_eq!(unit.kind, ObjectKind::PeacekeeperShielded, "#{}", unit.id.0);
                    assert_eq!(riot_shield::mode(unit), Some(ShieldMode::Raised), "#{}", unit.id.0);
                }
            }
        },
        expect: |sim| {
            assert!(sim.object(ObjectId(4)).is_none(), "the Guard is dead");
            let units = units_of(sim, P1);
            let ids: Vec<u32> = units.iter().map(|unit| unit.id.0).collect();
            assert_eq!(ids, [1, 3], "the Guard killed #2");
            for unit in units {
                assert_eq!(unit.kind, PK, "#{}", unit.id.0);
                assert_eq!(riot_shield::mode(unit), Some(ShieldMode::Lowered), "#{}", unit.id.0);
            }
        },
    }
}

/// [`riot_shield`]: the Peacekeepers lower their shields in this tick.
const SHIELDS_DOWN: Tick = 120;

#[test]
fn golden_gdo_opening() {
    golden("gdo_opening", gdo_opening);
}

#[test]
fn golden_duel_3v3() {
    golden("duel_3v3", duel_3v3);
}

#[test]
fn golden_attack_move_battle() {
    golden("attack_move_battle", attack_move_battle);
}

#[test]
fn golden_choke_group_move() {
    golden("choke_group_move", choke_group_move);
}

#[test]
fn golden_patrol() {
    golden("patrol", patrol);
}

#[test]
fn golden_fog_chase() {
    golden("fog_chase", fog_chase);
}

#[test]
fn golden_riot_shield() {
    golden("riot_shield", riot_shield);
}

#[test]
fn golden_colonists_opening() {
    golden("colonists_opening", colonists::opening);
}

#[test]
fn golden_colonists_vs_gdo() {
    golden("colonists_vs_gdo", colonists::versus_gdo);
}

#[test]
fn golden_syndicate_opening() {
    golden("syndicate_opening", syndicate::opening);
}

#[test]
fn golden_syndicate_vs_gdo() {
    golden("syndicate_vs_gdo", syndicate::versus_gdo);
}

#[test]
fn golden_cults_opening() {
    golden("cults_opening", cults::opening);
}

#[test]
fn golden_cults_vs_syndicate() {
    golden("cults_vs_syndicate", cults::versus_syndicate);
}

#[test]
fn golden_cults_garrison() {
    golden("cults_garrison", garrison::cults_garrison);
}

#[test]
fn golden_cults_altar() {
    golden("cults_altar", altar::cults_altar);
}
