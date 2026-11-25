//! Small golden games of units on test maps: three Peacekeepers against three Guards, an AttackMove battle of
//! twenty a side, a group move through a choke, a patrol that engages a Guard on its way, and a chase that
//! loses its target in the fog.

use super::*;

/// Three Peacekeepers (player 1, ids 1–3) against three Guards (player 2, ids 4–6), in two columns 4 tiles apart:
/// the Peacekeepers are ordered to attack the middle Guard, the Guards acquire their targets by themselves. It ends
/// with one side gone.
pub(crate) fn duel_3v3() -> Game {
    let mut builder = ScenarioBuilder::new(&open(16, 10));
    for y in [3, 5, 7] {
        builder = builder.spawn(PK, Some(P1), at(5, y));
    }
    for y in [3, 5, 7] {
        builder = builder.spawn(GUARD, Some(P2), at(9, y));
    }
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![(1, order(P1, 1..=3, OrderKind::Attack(ObjectId(5))))],
        script: None,
        end: 240,
        during: |_, _| {},
        expect: |sim| {
            let (p1, p2) = (units_of(sim, P1).len(), units_of(sim, P2).len());
            assert!(p1 == 0 || p2 == 0, "one side is gone: {p1} against {p2}");
        },
    }
}

/// Twenty units a side, five rows of Peacekeepers and Guards in turn, in blocks at opposite ends of the map, are
/// ordered to AttackMove to the other block's place: they meet in the middle and fight.
pub(crate) fn attack_move_battle() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 20));
    for (player, x0, y0) in [(P1, 3, 6), (P2, 33, 9)] {
        for y in 0..5 {
            for x in 0..4 {
                let kind = if y % 2 == 0 { PK } else { GUARD };
                builder = builder.spawn(kind, Some(player), at(x0 + x, y0 + y));
            }
        }
    }
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, order(P1, 1..=20, OrderKind::AttackMove(point(35, 11)))),
            (1, order(P2, 21..=40, OrderKind::AttackMove(point(4, 8)))),
        ],
        script: None,
        end: 480,
        during: |_, _| {},
        expect: |sim| {
            let left = sim.objects().count();
            assert!(left <= 30, "at least ten died: {left} are left");
        },
    }
}

/// Twelve Peacekeepers in a block west of a mountain wall, ordered as one group to a point east of it, cross it
/// through a gap two tiles wide.
pub(crate) fn choke_group_move() -> Game {
    let wall = format!("{}M{}\n", ".".repeat(15), ".".repeat(16));
    let gap = open(32, 1);
    let rows: Vec<&str> = (0..16)
        .map(|y| if y == 7 || y == 8 { &gap } else { &wall })
        .map(String::as_str)
        .collect();
    let mut builder = ScenarioBuilder::new(&rows.concat());
    for y in 6..=8 {
        for x in 3..=6 {
            builder = builder.spawn(PK, Some(P1), at(x, y));
        }
    }
    Game {
        scenario: builder.build(),
        commands: vec![(1, order(P1, 1..=12, OrderKind::Move(Target::Point(point(26, 8)))))],
        script: None,
        end: 480,
        during: |_, _| {},
        expect: |sim| {
            let wall_east = 16.0 * SU_PER_GU;
            for unit in units_of(sim, P1) {
                assert!(
                    unit.pos.x > wall_east,
                    "#{} is through the gap: {}",
                    unit.id.0,
                    unit.pos
                );
            }
        },
    }
}

/// Three Peacekeepers (ids 1–3) patrol between their place and a point 14 tiles east. A Guard of player 2 (id 4)
/// stands near the route: the patrol engages it, kills it and walks on.
pub(crate) fn patrol() -> Game {
    let mut builder = ScenarioBuilder::new(&open(32, 12));
    for y in [4, 5, 6] {
        builder = builder.spawn(PK, Some(P1), at(4, y));
    }
    builder = builder.spawn(GUARD, Some(P2), at(14, 9));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![(1, order(P1, 1..=3, OrderKind::Patrol(point(18, 5))))],
        script: None,
        end: 640,
        during: |_, _| {},
        expect: |sim| {
            assert!(sim.object(ObjectId(4)).is_none(), "the Guard is dead");
            let patrolling = units_of(sim, P1)
                .iter()
                .filter(|u| {
                    let current = u.orders.as_ref().and_then(|o| o.current.as_ref());
                    current.is_some_and(|c| matches!(c.order.kind, OrderKind::Patrol(_)))
                })
                .count();
            assert!(patrolling > 0, "the patrol goes on");
        },
    }
}

/// A Peacekeeper (player 1, id 1) attacks a Guard (player 2, id 2) that runs away east, faster than it: the
/// Peacekeeper shoots while it can, chases, loses the Guard in the fog, walks to where it last saw it and stops
/// there (design change 0034).
pub(crate) fn fog_chase() -> Game {
    let builder = ScenarioBuilder::new(&open(48, 10))
        .spawn(PK, Some(P1), at(4, 5))
        .spawn(GUARD, Some(P2), at(8, 5));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, order(P2, [2], OrderKind::Move(Target::Point(point(44, 5))))),
            (1, order(P1, [1], OrderKind::Attack(ObjectId(2)))),
        ],
        script: None,
        end: 400,
        during: |_, _| {},
        expect: |sim| {
            let (chaser, runner) = (sim.object(ObjectId(1)).unwrap(), sim.object(ObjectId(2)).unwrap());
            assert!(runner.hp < Some(80.0), "the chaser hit it before it got away");
            assert!(!sim.is_visible(P1, runner), "out of the chaser's sight");
            assert!(chaser.orders.as_ref().unwrap().is_idle(), "the chase ended");
            assert!(
                chaser.pos.x > 9.0 * SU_PER_GU,
                "it chased past the Guard's start: {}",
                chaser.pos
            );
        },
    }
}
