//! The Altar's golden game (slice M5-N1b, `factions/cults/augmentation.md > Altar`): three Militants augmented in turn,
//! one order of each type, against Agents.

use sc_sim::{AugmentOrder, Augmentation, DebugCommand, RecruitKind, SimConfig};

use super::*;

/// The Altar (id 1), player 1's, on tiles (12, 5) to (14, 6): its entrance, side B, to the east, its exit, side C, to
/// the west.
const ALTAR: ObjectId = ObjectId(1);
/// Player 1's three Militants, east of the Altar.
const MILITANTS: [u32; 3] = [2, 3, 4];
/// Player 2's two Agents, at the map's western edge.
const AGENTS: [u32; 2] = [5, 6];
/// The three orders, queued in this tick with the Militants inside.
const ORDERS: Tick = 100;
/// The orders, in the order they are queued (`> Altar > Augmentation Orders`: "Orders are queueable and serial —
/// processed one at a time, in the order queued"), each with the tick its Militant comes out in, the last of its order's
/// frames (48, 48 and 96 since design change 0391, from the tick the one before it is out), and the Augmentations and
/// the MaxHP it has then (`factions/cults/augmentation.md > Constitution and Metabolism`: 40, 10 more or less per step
/// of Constitution).
const AUGMENTED: [(AugmentOrder, Tick, RecruitKind, f32); 3] = [
    (
        AugmentOrder::EnhancedConstitution,
        ORDERS + 47,
        RecruitKind::Constitution,
        60.0,
    ),
    (
        AugmentOrder::ImprovedMetabolism,
        ORDERS + 95,
        RecruitKind::Metabolism,
        30.0,
    ),
    (AugmentOrder::Both, ORDERS + 191, RecruitKind::Both, 50.0),
];
/// Player 2's Agents AttackMove onto the rally point from this tick on, the three Militants out and waiting there (the
/// last since tick 291).
const ATTACK: Tick = 760;
/// The game's last tick: the Agents fell by 0:58 (tick 931), the Militants stand.
const END: Tick = 960;

/// Player 1 plays the Cults, player 2 the Syndicate, with debug commands on: the Cults start with no Space Crystals
/// (`factions/cults/economy.md > Starting Conditions`), and a debug command gives player 1 the orders' 100, so that the
/// game is about the Altar, not about gathering.
fn cults_p1_syndicate_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.config(SimConfig::DEBUG).build();
    scenario.setup.players[0].faction = Faction::Cults;
    scenario.setup.players[1].faction = Faction::Syndicate;
    scenario
}

/// Player 1's three Militants (ids 2–4) go into its Altar (id 1) at tick 1, which rallies them west of it; at
/// [`ORDERS`] it queues one order of each type, Enhanced Constitution (25 Space Crystals, 48 frames), Improved
/// Metabolism (25, 48) and Both (50, 96), all its 100 Space Crystals, each of the next Militant in
/// (`> Augmentation Orders`: "the next eligible unit in StoredUnits is reserved"). They come out in turn, 48, 96 and
/// 192 frames later, with the Augmentations and the MaxHP of [`AUGMENTED`], at full HP (`> Augmentation Orders`: "the
/// unit's MaxHP is recomputed"), and make for the rally point. At [`ATTACK`] two Agents of player 2's (ids 5 and 6)
/// AttackMove onto it: the Militants outrange them, 4 grid units to 2, and kill both. The Agents go for the Enhanced
/// Constitution Militant, which comes through at 15 of its 60 HP; the two others are unhurt.
pub(crate) fn cults_altar() -> Game {
    let mut builder =
        ScenarioBuilder::new(&open(28, 12)).spawn(ObjectKind::Altar, Some(P1), SpawnAt::Tile(TilePos::new(12, 5)));
    for y in [4, 5, 6] {
        builder = builder.spawn(ObjectKind::Militant, Some(P1), at(18, y));
    }
    for y in [4, 6] {
        builder = builder.spawn(ObjectKind::Agent, Some(P2), at(1, y));
    }
    let bank = PlayerCommand {
        player: P1,
        command: Command::Debug(DebugCommand::SetResources {
            player: P1,
            crystals: 100,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        }),
    };
    let altar = |command| PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![ALTAR],
            command,
        },
    };
    let augment = |order| {
        altar(ProductionCommand::Augment {
            kind: ObjectKind::Militant,
            from: RecruitKind::Plain,
            order,
        })
    };
    let mut commands = vec![
        (1, bank),
        (1, order(P1, MILITANTS, OrderKind::Enter(ALTAR))),
        (1, altar(ProductionCommand::SetRally(Target::Point(point(9, 5))))),
    ];
    commands.extend(AUGMENTED.map(|(order, ..)| (ORDERS, augment(order))));
    commands.push((ATTACK, order(P2, AGENTS, OrderKind::AttackMove(point(9, 5)))));
    Game {
        scenario: cults_p1_syndicate_p2(builder),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == ORDERS {
                assert_eq!(sim.held_in(ALTAR).len(), 3, "the three went in");
                let queued = sim
                    .events()
                    .iter()
                    .filter(|e| matches!(e, SimEvent::AugmentQueued { .. }));
                assert_eq!(queued.count(), 3, "the three orders are queued");
                assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "paid");
            }
            let research = sim.research();
            for unit in units_of(sim, P1).into_iter().filter(|o| o.kind == ObjectKind::Militant) {
                let max = unit.max_hp_with(research);
                assert!(
                    unit.hp <= max,
                    "tick {tick}: #{} has {:?} of {max:?}",
                    unit.id.0,
                    unit.hp
                );
            }
            let out: Vec<ObjectId> = (sim.events().iter())
                .filter_map(|e| match *e {
                    SimEvent::UnitProduced { producer, unit } if producer == ALTAR => Some(unit),
                    _ => None,
                })
                .collect();
            match AUGMENTED.iter().find(|(_, at, ..)| *at == tick) {
                Some(&(_, _, kind, max_hp)) => {
                    let [unit] = out[..] else {
                        panic!("tick {tick}: one Militant out, not {out:?}");
                    };
                    let unit = sim.object(unit).expect("on the map");
                    assert_eq!(unit.augmentation, Augmentation::of(kind), "tick {tick}: #{}", unit.id.0);
                    let hp = (unit.hp, unit.max_hp_with(research));
                    assert_eq!(
                        hp,
                        (Some(max_hp), Some(max_hp)),
                        "tick {tick}: #{} at full HP",
                        unit.id.0
                    );
                }
                None => assert_eq!(out, [], "tick {tick}: none out"),
            }
            if tick == ATTACK {
                assert!(sim.held_in(ALTAR).is_empty(), "the three came out");
            }
        },
        expect: |sim| {
            assert!(units_of(sim, P2).is_empty(), "player 2's Agents are all dead");
            let militants = units_of(sim, P1);
            let militants: Vec<_> = militants.iter().filter(|o| o.kind == ObjectKind::Militant).collect();
            assert!(!militants.is_empty(), "a Militant survives");
            for militant in militants {
                assert!(!militant.augmentation.is_none(), "#{} is augmented", militant.id.0);
            }
            assert!(sim.object(ALTAR).is_some(), "the Altar stands");
        },
    }
}
