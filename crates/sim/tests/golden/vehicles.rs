//! The Seekers' golden game (slice M5-N6, `factions/cults/structures/chop_shop.md`, `factions/cults/units/seeker.md`): a
//! Chop Shop trains two Seekers of four Recruits, which drive out of its doorway on side C to their rally point and then
//! fight two Militants and two Guards, their turrets firing on the move.

use std::cell::RefCell;

use sc_sim::{DebugCommand, RecruitKind, SimConfig};

use super::*;

/// The Chop Shop (id 1), player 1's, on tiles (30, 8) to (32, 10), unturned: its entrance, side B, to the east (x
/// 2112), its exit, side C, to the west (x 1920), the middle of which is [`EXIT`].
const CHOP_SHOP: ObjectId = ObjectId(1);
/// Player 1's four Recruits, east of the Chop Shop, which go into it in tick 1 and come out as the two Seekers.
const RECRUITS: [u32; 4] = [2, 3, 4, 5];
/// Player 2's two Militants, in the west, and its two Guards, behind them.
const MILITANTS: [ObjectId; 2] = [ObjectId(6), ObjectId(7)];
const GUARDS: [ObjectId; 2] = [ObjectId(8), ObjectId(9)];
/// The middle of side C, where a Seeker appears, half inside, facing west.
const EXIT: Vec2 = Vec2::new(1920.0, 608.0);
/// The two Seekers are queued in this tick, with the four Recruits inside, and the rally point set.
const TRAIN: Tick = 64;
/// The tick each Seeker comes out in: the first 320 frames after the queue (the queue's tick the first), the other
/// 320 after it.
const OUT: [Tick; 2] = [TRAIN + 319, TRAIN + 639];
/// The Seekers' rally point, on the ground west of the Chop Shop, where they see nothing of player 2's.
const RALLY: (i32, i32) = (24, 9);
/// The two Seekers AttackMove onto player 2's infantry from this tick on.
const ATTACK: Tick = 760;
/// The game's last tick: the last Guard fell at 1:13 (tick 1172).
const END: Tick = 1200;

/// The hits of the game so far: by the Seekers on the Militants and on the Guards, those of them a Seeker fired while
/// it drove, and on the Seekers by each.
#[derive(Default)]
struct Hits {
    on_militants: u32,
    on_guards: u32,
    on_the_move: u32,
    by_militants: u32,
    by_guards: u32,
}

thread_local! {
    /// The Recruits inside the Chop Shop as the Seekers are queued, longest inside first.
    static INSIDE: RefCell<Vec<ObjectId>> = const { RefCell::new(Vec::new()) };
    static HITS: RefCell<Hits> = RefCell::new(Hits::default());
}

/// Player 1 plays the Cults, player 2 the Syndicate, with debug commands on: a debug command gives player 1 the two
/// Seekers' 150 Space Crystals, so that the game is about the Seekers, not about gathering. Player 2's Militants, the
/// Cults' infantry, stand with its Guards, spawned for the game.
fn cults_p1_syndicate_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.config(SimConfig::DEBUG).build();
    scenario.setup.players[0].faction = Faction::Cults;
    scenario.setup.players[1].faction = Faction::Syndicate;
    scenario
}

/// Player 1's four Recruits (ids 2–5) go into its Chop Shop (id 1) in tick 1; at [`TRAIN`] it sets its rally point west
/// of it and queues two Seekers (`factions/cults/structures/chop_shop.md > Placeholder stats`: "**Seeker**: 2 Recruits,
/// 75 Space Crystals, 320 frames"), all its 150 Space Crystals, each of the two Recruits inside the longest of those
/// left (design change 0332). Each comes out at its tick of [`OUT`], the first of its two Recruits, half inside the
/// middle of side C and facing out of it, and drives to the rally point. At [`ATTACK`] the two AttackMove onto player
/// 2's Militants (ids 6, 7) and the Guards (ids 8, 9) behind them: they kill the two Militants, a Guard kills one of
/// them, and the other kills both Guards (as the game played when it was made). (M5-S8) Until [`TRAIN`] the Chop Shop
/// has the rally point it starts with, 3 grid units out of side C (design change 0446), which the game's own replaces:
/// re-blessed for it, the checkpoints of ticks 0 and 32 alone moved.
pub(crate) fn cults_vehicles() -> Game {
    let mut builder =
        ScenarioBuilder::new(&open(40, 18)).spawn(ObjectKind::ChopShop, Some(P1), SpawnAt::Tile(TilePos::new(30, 8)));
    for y in [7, 8, 9, 10] {
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), at(35, y));
    }
    builder = builder
        .spawn(ObjectKind::Militant, Some(P2), at(11, 8))
        .spawn(ObjectKind::Militant, Some(P2), at(11, 10))
        .spawn(GUARD, Some(P2), at(6, 8))
        .spawn(GUARD, Some(P2), at(6, 10));
    let production = |command| PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![CHOP_SHOP],
            command,
        },
    };
    let bank = PlayerCommand {
        player: P1,
        command: Command::Debug(DebugCommand::SetResources {
            player: P1,
            crystals: 150,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        }),
    };
    let train = ProductionCommand::TrainFrom {
        kind: ObjectKind::Seeker,
        from: RecruitKind::Plain,
    };
    let rally = ProductionCommand::SetRally(Target::Point(point(RALLY.0, RALLY.1)));
    let commands = vec![
        (1, bank),
        (1, order(P1, RECRUITS, OrderKind::Enter(CHOP_SHOP))),
        (TRAIN, production(rally)),
        (TRAIN, production(train)),
        (TRAIN, production(train)),
        (ATTACK, order(P1, RECRUITS, OrderKind::AttackMove(point(5, 9)))),
    ];
    Game {
        scenario: cults_p1_syndicate_p2(builder),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                HITS.set(Hits::default());
            }
            if tick == TRAIN {
                let inside: Vec<ObjectId> = sim.held_in(CHOP_SHOP).iter().map(|held| held.object.id).collect();
                assert_eq!(inside.len(), RECRUITS.len(), "all went in");
                assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "paid");
                INSIDE.set(inside);
            }
            let produced: Vec<ObjectId> = (sim.events().iter())
                .filter_map(|e| match *e {
                    SimEvent::UnitProduced { producer, unit } if producer == CHOP_SHOP => Some(unit),
                    _ => None,
                })
                .collect();
            match OUT.iter().position(|&at| at == tick) {
                Some(i) => {
                    let (seeker, consumed) = INSIDE.with_borrow(|inside| (inside[2 * i], inside[2 * i + 1]));
                    assert_eq!(
                        produced,
                        [seeker],
                        "tick {tick}: the Recruit inside the longest of those left"
                    );
                    let o = sim.object(seeker).expect("out");
                    assert_eq!((o.kind, o.pos), (ObjectKind::Seeker, EXIT), "tick {tick}");
                    assert!(
                        sim.object(consumed).is_none() && sim.held_unit(consumed).is_none(),
                        "tick {tick}"
                    );
                }
                None => assert_eq!(produced, [], "tick {tick}"),
            }
            let seeker = |id: ObjectId| RECRUITS.contains(&id.0);
            let driving = |id: ObjectId| {
                let mover = sim.object(id).and_then(|o| o.mover.as_ref());
                mover.is_some_and(|mover| mover.momentum.length() > 0.5)
            };
            HITS.with_borrow_mut(|hits| {
                for e in sim.events() {
                    let SimEvent::Hit { attacker, target, .. } = *e else {
                        continue;
                    };
                    if seeker(attacker) && driving(attacker) {
                        hits.on_the_move += 1;
                    }
                    match (attacker, target) {
                        (a, t) if seeker(a) && MILITANTS.contains(&t) => hits.on_militants += 1,
                        (a, t) if seeker(a) && GUARDS.contains(&t) => hits.on_guards += 1,
                        (a, t) if MILITANTS.contains(&a) && seeker(t) => hits.by_militants += 1,
                        (a, t) if GUARDS.contains(&a) && seeker(t) => hits.by_guards += 1,
                        _ => panic!("tick {tick}: {attacker:?} hit {target:?}"),
                    }
                }
            });
            if tick < ATTACK {
                assert_eq!(units_of(sim, P2).len(), 4, "tick {tick}: no fight before the attack");
            }
        },
        expect: |sim| {
            assert_eq!(
                units_of(sim, P2).len(),
                0,
                "the Seekers killed the Militants and the Guards"
            );
            let p1: Vec<ObjectKind> = units_of(sim, P1).iter().map(|o| o.kind).collect();
            assert_eq!(
                p1,
                [ObjectKind::ChopShop, ObjectKind::Seeker],
                "a Guard took one Seeker with it"
            );
            HITS.with_borrow(|hits| {
                assert!(hits.on_militants >= 28, "40 HP each, 3 a hit: {}", hits.on_militants);
                assert!(
                    hits.on_guards >= 80,
                    "80 HP each behind PointArmor 1, 2 a hit: {}",
                    hits.on_guards
                );
                assert!(hits.on_the_move > 0, "the turrets fire on the move");
                assert!(hits.by_militants > 0 && hits.by_guards > 0);
            });
        },
    }
}
