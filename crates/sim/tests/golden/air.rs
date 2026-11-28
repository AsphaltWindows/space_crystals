//! The Sparrows' golden game (slice M5-N7, `factions/cults/structures/airstrip.md`, `factions/cults/units/sparrow.md`):
//! an Airstrip trains three Sparrows, which take off along its runway and strafe a Militant and a Guard, the infantry
//! that hit air.

use std::cell::RefCell;

use sc_sim::{DebugCommand, RecruitKind, SimConfig};

use super::*;

/// The Airstrip (id 1), player 1's, on tiles (30, 8) to (33, 9), unturned: its entrance and the runway's start, side
/// B, to the east (x 2176), the end its Sparrows take off toward, side C, to the west (x 1920).
const AIRSTRIP: ObjectId = ObjectId(1);
/// Player 1's three Recruits, east of the Airstrip, which go into it in tick 1 and come out as the Sparrows.
const RECRUITS: [u32; 3] = [2, 3, 4];
/// Player 2's Militant, in the west, and its Guard, behind it.
const MILITANT: ObjectId = ObjectId(5);
const GUARD_ID: ObjectId = ObjectId(6);
/// The Guard stands this far east of its tile's centre.
const GUARD_EAST: Vec2 = Vec2::new(10.0, 0.0);
/// The three Sparrows are queued in this tick, with the three Recruits inside, and the rally point set.
const TRAIN: Tick = 48;
/// The tick each Sparrow takes off in: the first 320 frames after the queue (the queue's tick the first), the others
/// 320 after the one before.
const TAKEOFFS: [Tick; 3] = [TRAIN + 319, TRAIN + 639, TRAIN + 959];
/// Where a Sparrow appears (40 × 40): wholly over the runway, against its B end, centred across it.
const START: Vec2 = Vec2::new(2156.0, 576.0);
/// The Sparrows' rally point, on the ground west of the Airstrip, where they see nothing of player 2's.
const RALLY: (i32, i32) = (25, 9);
/// The three Sparrows AttackMove onto player 2's infantry from this tick on.
const ATTACK: Tick = 1080;
/// The game's last tick: the last Sparrow fell at 1:29 (tick 1430).
const END: Tick = 1440;

/// The hits of the game so far: by the Sparrows on the Militant and on the Guard, and on the Sparrows by each.
#[derive(Default)]
struct Hits {
    on_militant: u32,
    on_guard: u32,
    by_militant: u32,
    by_guard: u32,
}

thread_local! {
    /// The Recruits inside the Airstrip as the Sparrows are queued, longest inside first.
    static INSIDE: RefCell<Vec<ObjectId>> = const { RefCell::new(Vec::new()) };
    static HITS: RefCell<Hits> = RefCell::new(Hits::default());
}

/// Player 1 plays the Cults, player 2 the Syndicate, with debug commands on: the Cults start with no Space Crystals
/// (`factions/cults/economy.md > Starting Conditions`), and a debug command gives player 1 the three Sparrows' 225, so
/// that the game is about the Sparrows, not about gathering. Player 2's Militant, the Cults' infantry, stands with its
/// Guard, spawned for the game: the two infantry whose attacks are Universal (`factions/cults/units/militant.md`,
/// `factions/syndicate/units/guard.md`).
fn cults_p1_syndicate_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.config(SimConfig::DEBUG).build();
    scenario.setup.players[0].faction = Faction::Cults;
    scenario.setup.players[1].faction = Faction::Syndicate;
    scenario
}

/// Player 1's three Recruits (ids 2–4) go into its Airstrip (id 1) in tick 1; at [`TRAIN`] it sets its rally point
/// west of it and queues three Sparrows (`factions/cults/structures/airstrip.md > Placeholder stats`: "**Sparrow**: 1
/// Recruit, 75 Space Crystals, 320 frames"), all its 225 Space Crystals. Each takes off at its tick of [`TAKEOFFS`],
/// the Recruit inside the longest first (design change 0332), over the runway's B end at IdleSpeed, on its takeoff run
/// (design change 0354), and flies along the runway, then to the rally point, where it circles. At [`ATTACK`] the
/// three AttackMove onto player 2's Militant (id 5) and Guard (id 6) behind it, and strafe them: they kill the Militant,
/// which outranges them (4 grid units to 3) and takes one of them with it, and hit the Guard, 80 HP behind PointArmor
/// 1, which kills the other two (as the game played when it was made).
pub(crate) fn cults_air() -> Game {
    let mut builder =
        ScenarioBuilder::new(&open(40, 18)).spawn(ObjectKind::Airstrip, Some(P1), SpawnAt::Tile(TilePos::new(30, 8)));
    for y in [7, 8, 9] {
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), at(35, y));
    }
    let guard_at = SpawnAt::Point(point(5, 9) + GUARD_EAST);
    builder = builder
        .spawn(ObjectKind::Militant, Some(P2), at(11, 9))
        .spawn(GUARD, Some(P2), guard_at);
    let production = |command| PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![AIRSTRIP],
            command,
        },
    };
    let bank = PlayerCommand {
        player: P1,
        command: Command::Debug(DebugCommand::SetResources {
            player: P1,
            crystals: 225,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        }),
    };
    let train = ProductionCommand::TrainFrom {
        kind: ObjectKind::Sparrow,
        from: RecruitKind::Plain,
    };
    let rally = ProductionCommand::SetRally(Target::Point(point(RALLY.0, RALLY.1)));
    let commands = vec![
        (1, bank),
        (1, order(P1, RECRUITS, OrderKind::Enter(AIRSTRIP))),
        (TRAIN, production(rally)),
        (TRAIN, production(train)),
        (TRAIN, production(train)),
        (TRAIN, production(train)),
        (ATTACK, order(P1, RECRUITS, OrderKind::AttackMove(point(8, 9)))),
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
                let inside: Vec<ObjectId> = sim.held_in(AIRSTRIP).iter().map(|held| held.object.id).collect();
                assert_eq!(inside.len(), RECRUITS.len(), "all went in");
                assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "paid");
                INSIDE.set(inside);
            }
            let produced: Vec<ObjectId> = (sim.events().iter())
                .filter_map(|e| match *e {
                    SimEvent::UnitProduced { producer, unit } if producer == AIRSTRIP => Some(unit),
                    _ => None,
                })
                .collect();
            match TAKEOFFS.iter().position(|&at| at == tick) {
                Some(i) => {
                    let sparrow = INSIDE.with_borrow(|inside| inside[i]);
                    assert_eq!(produced, [sparrow], "tick {tick}: the Recruit inside the longest");
                    let o = sim.object(sparrow).expect("out");
                    assert_eq!((o.kind, o.pos), (ObjectKind::Sparrow, START), "tick {tick}");
                    let mover = o.mover.as_ref().expect("a mover");
                    assert!(mover.takeoff.is_some(), "tick {tick}: on its takeoff run");
                }
                None => assert_eq!(produced, [], "tick {tick}"),
            }
            let sparrow = |id: ObjectId| RECRUITS.contains(&id.0);
            HITS.with_borrow_mut(|hits| {
                for e in sim.events() {
                    let SimEvent::Hit { attacker, target, .. } = *e else {
                        continue;
                    };
                    match (attacker, target) {
                        (a, MILITANT) if sparrow(a) => hits.on_militant += 1,
                        (a, GUARD_ID) if sparrow(a) => hits.on_guard += 1,
                        (MILITANT, t) if sparrow(t) => hits.by_militant += 1,
                        (GUARD_ID, t) if sparrow(t) => hits.by_guard += 1,
                        _ => panic!("tick {tick}: {attacker:?} hit {target:?}"),
                    }
                }
            });
            if tick < ATTACK {
                assert_eq!(units_of(sim, P2).len(), 2, "tick {tick}: no fight before the attack");
            }
        },
        expect: |sim| {
            assert!(sim.object(MILITANT).is_none(), "the Sparrows killed the Militant");
            let guard = sim.object(GUARD_ID).expect("the Guard lives");
            assert!(guard.hp < Some(80.0), "the Sparrows hit the Guard");
            let p1: Vec<ObjectKind> = units_of(sim, P1).iter().map(|o| o.kind).collect();
            assert_eq!(p1, [ObjectKind::Airstrip], "the three Sparrows fell");
            HITS.with_borrow(|hits| {
                assert!(hits.on_militant >= 14, "40 HP, 3 a hit: {}", hits.on_militant);
                assert!(hits.on_guard > 0 && hits.by_militant > 0 && hits.by_guard > 0);
            });
        },
    }
}
