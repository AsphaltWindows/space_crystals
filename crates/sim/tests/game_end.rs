//! Defeat and the end of the game (slice M2-F2): `core/game_rules.md > Defeat Condition`, `> On Defeat` and `> End
//! of Game`, plan decision T8, design changes 0101 and 0102.
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

/// T8, by debug removal: player 2 loses their Generator but still has a plate, which counts as a structure
/// (`> Defeat Condition`: "including GDO Extraction Plates"); they are defeated in the tick the plate goes, with one
/// event, and in a two-player game that ends it.
#[test]
fn losing_the_last_structure_to_a_removal_defeats_the_player_plates_included() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 1)) // 2
        .spawn(ObjectKind::CrystalPatch, None, tile(20, 8)) // 3
        .spawn(ObjectKind::ExtractionPlate, Some(P2), tile(20, 8)) // 4
        .start();
    t.debug_at(3, remove(2)).run_until(5);
    assert_eq!(defeated_at(&t, P2), None, "the plate is left");
    assert_eq!(t.sim().outcome(), None);
    t.debug_at(6, remove(4)).run_until(8);
    assert_eq!(defeated_at(&t, P2), Some(6));
    assert_eq!(defeated_at(&t, P1), None);
    assert!(t.sim().player(P2).unwrap().is_defeated());
    assert_eq!(game_events(&t), [(6, defeated(P2)), (6, game_over(&[P1]))]);
    // The patch under the plate had crystals left: it stays, nobody's.
    assert!(t.sim().object(ObjectId(3)).is_some());
}

/// T8, in combat: player 1's Peacekeepers destroy player 2's last structure, a plate, by themselves (design change
/// 0103: enemy structures are automatic targets). Player 2 is defeated in the tick it dies, and their Peacekeeper,
/// far away, becomes neutral in that same tick.
#[test]
fn losing_the_last_structure_in_combat_defeats_the_player() {
    let mut t = ScenarioBuilder::new(&flat(32, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(28, 1)) // 2
        .spawn(ObjectKind::CrystalPatch, None, tile(8, 6)) // 3
        .spawn(ObjectKind::ExtractionPlate, Some(P2), tile(8, 6)) // 4
        .spawn(PK, Some(P1), SpawnAt::Point(centre(5, 5))) // 5
        .spawn(PK, Some(P1), SpawnAt::Point(centre(5, 7))) // 6
        .spawn(PK, Some(P2), SpawnAt::Point(centre(28, 9))) // 7: far from it all
        .start();
    t.debug_at(1, remove(2)).run_until(400);
    let died = t.events().iter().find_map(|(tick, e)| match e {
        SimEvent::Died { id, .. } if *id == ObjectId(4) => Some(*tick),
        _ => None,
    });
    let died = died.expect("the plate is destroyed");
    assert_eq!(defeated_at(&t, P2), Some(died));
    assert_eq!(game_events(&t), [(died, defeated(P2)), (died, game_over(&[P1]))]);
    let unit = t.sim().object(ObjectId(7)).unwrap();
    assert_eq!(unit.owner, None);
    assert_eq!(unit.hp, Some(50.0), "nobody shot at it");
}

/// T8: a player who never owned a structure, as in the M1 sandboxes, is never defeated, even with no object left;
/// and a structure that comes and goes within one tick never counted (the count is taken between ticks).
#[test]
fn players_who_never_owned_a_structure_are_never_defeated() {
    let mut t = ScenarioBuilder::new(&flat(16, 8))
        .spawn(PK, Some(P1), SpawnAt::Point(centre(4, 4))) // 1
        .spawn(PK, Some(P1), SpawnAt::Point(centre(4, 2))) // 2
        .spawn(PK, Some(P2), SpawnAt::Point(centre(7, 4))) // 3
        .start();
    t.run_until(300);
    assert!(t.sim().object(ObjectId(3)).is_none(), "player 2 has nothing left");
    // A Generator spawned and removed in one tick (it gets id 4).
    t.debug_at(301, spawn(ObjectKind::Generator, P2, tile(12, 1)));
    t.debug_at(301, remove(4));
    t.run_until(310);
    assert!(t.sim().object(ObjectId(4)).is_none());
    assert_eq!(game_events(&t), []);
    assert!(t.sim().players().iter().all(|p| p.defeated_at.is_none()));
    assert_eq!(t.sim().outcome(), None);
    let order = Command::Debug(spawn(PK, P2, SpawnAt::Point(centre(12, 6))));
    for player in [P1, P2] {
        let command = sc_sim::PlayerCommand {
            player,
            command: order.clone(),
        };
        assert_eq!(t.sim().validate(&command), Ok(()));
    }
}

/// Design change 0101's verifications 1 and 2: player 2 is defeated while their Peacekeepers walk, with orders
/// queued. In that tick the Peacekeepers become neutral, without orders, standing where they are; player 2 sees
/// nothing any more and plans no path (`Mover::repath`: nothing was left to plan on their knowledge). Player 1's
/// units walk up to them without shooting, and an explicit Attack destroys one. Player 3 keeps the game going.
#[test]
fn a_defeated_players_units_become_neutral_and_idle() {
    let mut t = three_way();
    let (p1_units, p2_units) = ([ObjectId(4), ObjectId(5)], [ObjectId(6), ObjectId(7)]);
    let west = OrderKind::Move(Target::Point(centre(13, 9)));
    let back = OrderKind::Move(Target::Point(centre(24, 9)));
    t.order_at(1, P2, &p2_units, west, false);
    t.order_at(1, P2, &p2_units, back, true);
    t.debug_at(5, remove(2)).run_until(4);
    for id in p2_units {
        let unit = t.sim().object(id).unwrap();
        assert!(unit.mover.as_ref().unwrap().goal != MoveGoal::None, "walking");
        assert!(!unit.orders.as_ref().unwrap().queue.is_empty(), "with an order queued");
    }

    t.run_until(5);
    assert_eq!(defeated_at(&t, P2), Some(5));
    assert_eq!(t.sim().outcome(), None, "players 1 and 3 are still in the game");
    let mut stood = Vec::new();
    for id in p2_units {
        let unit = t.sim().object(id).unwrap();
        assert_eq!(unit.owner, None);
        assert_eq!(unit.orders.as_ref(), Some(&Orders::default()));
        let mover = unit.mover.as_ref().unwrap();
        assert_eq!((mover.goal, mover.path_state), (MoveGoal::None, PathState::None));
        assert!(mover.path.is_empty());
        assert_eq!((mover.velocity, mover.locomotion), (Vec2::ZERO, Locomotion::Stationary));
        assert_eq!(unit.attack.as_ref().unwrap().target, None);
        stood.push(unit.pos);
    }
    // Nobody's eyes: player 2 sees no tile, and keeps no planning grid (no unit of theirs walks).
    let vision = t.sim().vision().player(P2).unwrap();
    assert!(vision.states().all(|s| s != VisibilityState::Visible));
    assert!(
        vision.states().any(|s| s == VisibilityState::Explored),
        "what they explored stays"
    );
    assert!(t.sim().known_nav(P2).is_none());

    // Player 1 walks up to them and stands by; nobody shoots.
    t.order_at(6, P1, &p1_units, OrderKind::Move(Target::Point(centre(17, 9))), false);
    t.run_until(150);
    let positions: Vec<Vec2> = p2_units.iter().map(|&id| t.sim().object(id).unwrap().pos).collect();
    assert_eq!(positions, stood, "they stay where they are");
    let p1 = t.sim().object(ObjectId(4)).unwrap();
    assert!(p1.pos.distance(stood[0]) < 4.0 * 64.0, "within weapon range");
    let attacks = |t: &TestSim| {
        t.events()
            .iter()
            .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. }))
            .count()
    };
    assert_eq!(attacks(&t), 0);

    // An explicit Attack works.
    t.order_at(151, P1, &[ObjectId(4)], OrderKind::Attack(ObjectId(6)), false);
    t.run_until(400);
    assert!(t.sim().object(ObjectId(6)).is_none(), "destroyed");
    assert!(attacks(&t) > 0);
    assert!(t.sim().object(ObjectId(7)).is_some(), "the other one is left alone");
}

/// `> On Defeat`: "nothing they had can be commanded"; design change 0101: refuse every command from that player,
/// debug commands included. Nobody can give them an object either. The others play on.
#[test]
fn every_command_of_a_defeated_player_is_refused() {
    let mut t = three_way();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P2,
            crystals: 500,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t.debug_at(2, remove(2)).run_until(2);
    assert_eq!(defeated_at(&t, P2), Some(2));
    let command = |player, command| sc_sim::PlayerCommand { player, command };
    let refused = [
        Command::Order {
            units: vec![ObjectId(6)],
            kind: OrderKind::Stop,
            queue: false,
        },
        Command::Production {
            objects: vec![ObjectId(2)],
            command: ProductionCommand::Train(PK),
        },
        Command::Debug(spawn(PK, P1, SpawnAt::Point(centre(16, 4)))),
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    ];
    for c in refused {
        assert_eq!(
            t.sim().validate(&command(P2, c.clone())),
            Err(Reject::Defeated(P2)),
            "{c:?}"
        );
    }
    // No one gives them anything, and the others play on.
    let gift = Command::Debug(spawn(PK, P2, SpawnAt::Point(centre(16, 4))));
    assert_eq!(t.sim().validate(&command(P1, gift)), Err(Reject::Defeated(P2)));
    let stop = Command::Order {
        units: vec![ObjectId(4)],
        kind: OrderKind::Stop,
        queue: false,
    };
    assert_eq!(t.sim().validate(&command(P1, stop)), Ok(()));
    // Applied, a refused command changes nothing.
    let before = t.sim().objects().count();
    t.at(3, P2, Command::Debug(spawn(PK, P2, SpawnAt::Point(centre(16, 4)))));
    t.run_until(3);
    assert_eq!(t.sim().objects().count(), before);
}

/// Design change 0101's verification 3 (the sim's half): in a 1v1, the defeat of one player ends the game in that
/// tick, with the outcome in the hashed state, and from then on the sim accepts no command at all. It still steps if
/// asked; nobody is defeated after the end.
#[test]
fn a_1v1_ends_with_the_first_defeat() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 1)) // 2
        .spawn(PK, Some(P2), SpawnAt::Point(centre(20, 8))) // 3
        .start();
    let hash = t.run_until(3).sim().state_hash();
    t.debug_at(4, remove(2)).run_until(4);
    let outcome = Outcome {
        tick: 4,
        winners: vec![P1],
    };
    assert_eq!(t.sim().outcome(), Some(&outcome));
    assert!(outcome.is_winner(P1) && !outcome.is_winner(P2));
    // In order: the defeat, then the end.
    let tick_events: Vec<&SimEvent> = t
        .sim()
        .events()
        .iter()
        .filter(|e| matches!(e, SimEvent::PlayerDefeated { .. } | SimEvent::GameOver { .. }))
        .collect();
    assert_eq!(tick_events, [&defeated(P2), &game_over(&[P1])]);
    assert_ne!(t.sim().state_hash(), hash);

    let command = |player, command| sc_sim::PlayerCommand { player, command };
    for player in [P1, P2] {
        for c in [
            Command::Debug(remove(1)),
            Command::Order {
                units: vec![ObjectId(3)],
                kind: OrderKind::Stop,
                queue: false,
            },
        ] {
            assert_eq!(t.sim().validate(&command(player, c)), Err(Reject::GameOver));
        }
    }
    // Player 1's last structure goes after the end: refused, and no defeat would count anyway.
    t.debug_at(5, remove(1)).run_until(40);
    assert!(t.sim().object(ObjectId(1)).is_some());
    assert_eq!(t.sim().tick(), 40);
    assert_eq!(t.sim().outcome(), Some(&outcome));
    assert_eq!(game_events(&t), [(4, defeated(P2)), (4, game_over(&[P1]))]);
}

/// `> End of Game` with three players, every player their own side (no teams): the first defeat leaves two
/// standing and the game goes on; the second ends it, the last player standing winning. The defeated player's
/// commands are refused in between, the others' are not.
#[test]
fn three_players_end_with_the_second_defeat() {
    let mut t = three_way();
    t.debug_at(3, remove(3)).run_until(5);
    assert_eq!(defeated_at(&t, P3), Some(3));
    assert_eq!(t.sim().outcome(), None);
    assert!(t.sim().allies(P3).is_empty(), "no teams: no allies");
    t.debug_at(6, remove(2)).run_until(8);
    assert_eq!(defeated_at(&t, P2), Some(6));
    assert_eq!(
        t.sim().outcome(),
        Some(&Outcome {
            tick: 6,
            winners: vec![P1],
        })
    );
    assert_eq!(
        game_events(&t),
        [(3, defeated(P3)), (6, defeated(P2)), (6, game_over(&[P1]))]
    );
}

/// `> End of Game` doesn't say what happens when the last sides fall in the same tick. The placeholder reading: the
/// game ends with no winner, and everyone sees Defeat.
#[test]
fn players_defeated_in_the_same_tick_leave_no_winner() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 1)) // 2
        .start();
    t.debug_at(4, remove(2));
    t.debug_at(4, remove(1));
    t.run_until(4);
    assert_eq!(
        game_events(&t),
        [(4, defeated(P1)), (4, defeated(P2)), (4, game_over(&[]))]
    );
    let outcome = t.sim().outcome().unwrap();
    assert!(!outcome.is_winner(P1) && !outcome.is_winner(P2));
}

/// Plan §9 (the note from M2-D2): a producer handed over to neutral would keep its queue. By T8 that never happens:
/// a defeated player owns no structure, so a Barracks goes with its queue refunded (design change 0084) before its
/// owner is defeated, and nothing neutral is left that produces.
#[test]
fn a_defeat_hands_over_no_producer() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Barracks, Some(P2), tile(18, 4)) // 2
        .start();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P2,
            crystals: 100,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t.at(
        2,
        P2,
        Command::Production {
            objects: vec![ObjectId(2)],
            command: ProductionCommand::Train(PK),
        },
    );
    t.debug_at(10, remove(2)).run_until(10);
    let refunded = t
        .sim()
        .events()
        .iter()
        .position(|e| matches!(e, SimEvent::ProductionCancelled { .. }));
    let defeat = t.sim().events().iter().position(|e| *e == defeated(P2));
    assert!(refunded.unwrap() < defeat.unwrap());
    assert_eq!(t.sim().player(P2).unwrap().crystals, 100);
    assert!(
        t.sim()
            .objects()
            .all(|o| o.def().category != Category::Structure || o.owner.is_some())
    );
}

/// Determinism across a defeat and the end: player 1 attacks into player 2's corner and destroys their Peacekeepers
/// and Generator, while player 3 loses theirs to a removal. Saved and loaded around each defeat and after the end,
/// the game continues exactly as the original; the replay runs to the same state; and a save from before the end of
/// the game existed (no `outcome`, no `defeated_at`) loads as a game going on.
#[test]
fn a_save_across_a_defeat_loads_and_continues_identically() {
    let game = || {
        let mut t = three_way();
        // Four more Peacekeepers for player 1 (8 to 11), and all six attack.
        for y in [7, 9, 11, 13] {
            t.spawn_unit_at(1, PK, P1, centre(8, y));
        }
        let army: Vec<ObjectId> = [4, 5, 8, 9, 10, 11].map(ObjectId).to_vec();
        t.order_at(2, P1, &army, OrderKind::AttackMove(centre(27, 4)), false);
        t.debug_at(30, remove(3));
        t
    };
    let mut t = game();
    t.run_until(1500);
    let end = t.sim().outcome().cloned().expect("player 1 won");
    assert_eq!(end.winners, [P1]);
    let p2_fell = defeated_at(&t, P2).unwrap();
    assert!(30 < p2_fell && p2_fell == end.tick, "{p2_fell}");

    let mut t = game();
    for k in [0, 29, 30, 31, p2_fell - 1, p2_fell, p2_fell + 1, p2_fell + 50] {
        t.run_until(k);
        let json = serde_json::to_string(t.sim()).unwrap();
        let mut loaded: Sim = serde_json::from_str(&json).unwrap();
        assert_eq!(&loaded, t.sim(), "tick {k}");
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {k}");
        let mut twin = game();
        twin.run_until(k);
        for _ in 0..60 {
            twin.step();
            loaded.step(&twin.replay().inputs[twin.sim().tick() as usize - 1]);
            assert_eq!(
                loaded.state_hash(),
                twin.sim().state_hash(),
                "from tick {k}, at {}",
                twin.sim().tick()
            );
        }
    }
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());

    // Older saves: without the outcome and the defeats, a game going on.
    let mut t = game();
    t.run_until(10);
    let mut value = serde_json::to_value(t.sim()).unwrap();
    value.as_object_mut().unwrap().remove("outcome");
    for player in value["players"].as_array_mut().unwrap() {
        player.as_object_mut().unwrap().remove("defeated_at");
    }
    let loaded: Sim = serde_json::from_value(value).unwrap();
    assert_eq!(&loaded, t.sim());
    // A winner who isn't in the game is refused.
    let mut value = serde_json::to_value(t.sim()).unwrap();
    value["outcome"] = serde_json::json!({ "tick": 3, "winners": [7] });
    let error = serde_json::from_value::<Sim>(value).unwrap_err().to_string();
    assert!(error.contains("winner 7"), "{error}");
}
