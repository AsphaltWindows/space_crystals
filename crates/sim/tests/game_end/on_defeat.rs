//! What a defeat does (`core/game_rules.md > On Defeat`, design change 0101): the player's units neutral and
//! idle, every command of theirs refused, and no producer handed over.

use super::*;

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
    // Nobody's eyes: no source of player 2's sees a tile, so their tiles are Visible only for the fog delay (design
    // change 0178), and they keep no planning grid (no unit of theirs walks).
    let vision = t.sim().vision().player(P2).unwrap();
    let map = t.sim().map();
    let tiles = (0..map.height() as i32).flat_map(|y| (0..map.width() as i32).map(move |x| TilePos::new(x, y)));
    assert!(
        tiles
            .filter(|&tile| vision.state(tile) == VisibilityState::Visible)
            .all(|tile| vision.lingers(tile))
    );
    assert!(t.sim().known_nav(P2).is_none());

    // Player 1 walks up to them and stands by; nobody shoots.
    t.order_at(6, P1, &p1_units, OrderKind::Move(Target::Point(centre(17, 9))), false);
    t.run_until(150);
    let vision = t.sim().vision().player(P2).unwrap();
    assert!(vision.states().all(|s| s != VisibilityState::Visible));
    assert!(
        vision.states().any(|s| s == VisibilityState::Explored),
        "what they explored stays"
    );
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
