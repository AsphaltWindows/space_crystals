//! The Hand of Heaven's golden game (slice M5-N10, `factions/cults/units/hand_of_heaven.md`,
//! `factions/cults/structures/chop_shop.md`): a Chop Shop builds one of three Recruits, which shells an enemy building
//! from beyond its own sight, a Recruit spotting for it, and drives off as its last missile flies.

use std::cell::RefCell;

use sc_sim::{DebugCommand, RecruitKind, SimConfig, math};

use super::*;

/// Player 1's Chop Shop (id 1), on tiles (30, 8) to (32, 10), unturned: its exit, side C, to the west, the middle of
/// which is [`EXIT`].
const CHOP_SHOP: ObjectId = ObjectId(1);
/// Player 1's three Recruits, east of the Chop Shop, which go into it in tick 1; the first in, the southern one,
/// comes out the Hand of Heaven ([`HAND`]), the other two gone into it.
const RECRUITS: [u32; 3] = [2, 3, 4];
const HAND: ObjectId = ObjectId(4);
/// Player 1's Recruit by player 2's Deployment Center, on Hold: the spotter.
const SPOTTER: ObjectId = ObjectId(5);
/// Player 2's Deployment Center (4 × 4, 1000 HP, FullArmor 16), the target, and its Generator in the far corner, which
/// keeps it in the game.
const TARGET: ObjectId = ObjectId(6);
const GENERATOR: ObjectId = ObjectId(7);
/// The middle of side C, where the Hand of Heaven appears, half inside, facing west.
const EXIT: Vec2 = Vec2::new(1920.0, 608.0);
/// The Chop Shop's default rally point, 3 grid units out of side C (design change 0446), where the Hand of Heaven
/// drives on its way out, and back to at [`SCOOT`].
const RALLY: Vec2 = Vec2::new(1728.0, 608.0);
/// The Hand of Heaven is queued in this tick, the three Recruits inside.
const TRAIN: Tick = 64;
/// It comes out 480 frames after the queue, the queue's tick the first.
const OUT: Tick = TRAIN + 479;
/// It is ordered to attack the Deployment Center in this tick, from the rally point, 20 grid units east of it.
const ATTACK: Tick = 620;
/// Its missiles leave in these ticks, one cycle apart (Aim 48 + Firing 2 + Cooldown 4 + Reload 96 = 150), and land
/// in these, 131 ticks of flight later (1045.6 su at 8 su a tick), as the game played when it was made.
const LAUNCHES: [Tick; 2] = [720, 870];
const LANDINGS: [Tick; 2] = [851, 1001];
/// It is sent back to the rally point in the tick after its second launch: "shoot-and-scoot".
const SCOOT: Tick = LAUNCHES[1] + 1;
/// What each missile takes off the Deployment Center: all 600 of its blast, the circle of 96 su wholly on the
/// footprint, less its FullArmor 16 times the share of the footprint the circle covers (π × 96² / 256²).
const BLAST_ON_IT: f32 = 600.0 - 16.0 * std::f32::consts::PI * 96.0 * 96.0 / (256.0 * 256.0);
/// The game's last tick: the Deployment Center fell at 1:02.56 (tick 1001).
const END: Tick = 1100;

thread_local! {
    /// The Recruits inside the Chop Shop as the Hand of Heaven is queued, longest inside first.
    static INSIDE: RefCell<Vec<ObjectId>> = const { RefCell::new(Vec::new()) };
    /// The ticks of the game's launches and blasts so far.
    static FIRED: RefCell<(Vec<Tick>, Vec<Tick>)> = const { RefCell::new((Vec::new(), Vec::new())) };
}

/// Player 1 plays the Cults, player 2 the GDO, with debug commands on: a debug command gives player 1 the Hand of
/// Heaven's 150 Space Crystals, so that the game is about it, not about gathering.
fn cults_p1(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.config(SimConfig::DEBUG).build();
    scenario.setup.players[0].faction = Faction::Cults;
    scenario
}

/// The gap, edge to edge, between `hand`'s Silhouette and the Deployment Center's footprint, `target`.
fn gap(hand: &sc_sim::Object, target: &sc_sim::Object) -> f32 {
    sc_sim::shape::Shape::of(hand).gap(&sc_sim::shape::Shape::of(target))
}

/// Player 1's three Recruits (ids 2–4) go into its Chop Shop (id 1) in tick 1, and its Recruit by player 2's base
/// (id 5) holds there; at [`TRAIN`] the Chop Shop queues a Hand of Heaven ("**W: Build Hand of Heaven**"; 3 Recruits,
/// 150 Space Crystals, 480 frames), which comes out at [`OUT`] half inside the middle of side C, facing out, at full
/// HP, the Recruit inside the longest, the other two gone, and drives to the Chop Shop's default rally point. At
/// [`ATTACK`] it is ordered to attack player 2's Deployment Center (id 6), 20 grid units west: it drives into range,
/// 14 grid units, and stands there, well beyond its own sight of 7 and the Deployment Center's of 6, where only the
/// spotter sees the target, and shells it: two missiles 150 ticks apart, each of [`BLAST_ON_IT`]. Player 2 sees
/// nothing of it until the first lands, which reveals it (`core/vision.md > Attacks from outside vision`). In the tick
/// after the second launch it is sent back to the rally point, and it drives off while the missile flies, which then
/// brings the Deployment Center down. Every hit of the game is the Hand of Heaven's on it.
pub(crate) fn hand_of_heaven() -> Game {
    let mut builder =
        ScenarioBuilder::new(&open(40, 18)).spawn(ObjectKind::ChopShop, Some(P1), SpawnAt::Tile(TilePos::new(30, 8)));
    for y in [7, 8, 9] {
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), at(35, y));
    }
    builder = builder
        .spawn(ObjectKind::Recruit, Some(P1), at(8, 5))
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P2),
            SpawnAt::Tile(TilePos::new(5, 7)),
        )
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(TilePos::new(1, 15)));
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
    let train = PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![CHOP_SHOP],
            command: ProductionCommand::TrainFrom {
                kind: ObjectKind::HandOfHeaven,
                from: RecruitKind::Plain,
            },
        },
    };
    let commands = vec![
        (1, bank),
        (1, order(P1, RECRUITS, OrderKind::Enter(CHOP_SHOP))),
        (1, order(P1, [SPOTTER.0], OrderKind::Hold)),
        (TRAIN, train),
        (ATTACK, order(P1, [HAND.0], OrderKind::Attack(TARGET))),
        (SCOOT, order(P1, [HAND.0], OrderKind::Move(Target::Point(RALLY)))),
    ];
    Game {
        scenario: cults_p1(builder),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                FIRED.set((Vec::new(), Vec::new()));
            }
            if tick == TRAIN {
                let inside: Vec<ObjectId> = sim.held_in(CHOP_SHOP).iter().map(|held| held.object.id).collect();
                assert_eq!(inside.len(), RECRUITS.len(), "all went in");
                assert_eq!(inside[0], HAND, "the Recruit inside the longest");
                assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "paid");
                INSIDE.set(inside);
            }
            let produced: Vec<ObjectId> = (sim.events().iter())
                .filter_map(|e| match *e {
                    SimEvent::UnitProduced { producer, unit } if producer == CHOP_SHOP => Some(unit),
                    _ => None,
                })
                .collect();
            if tick == OUT {
                assert_eq!(produced, [HAND], "tick {tick}");
                let o = sim.object(HAND).expect("out");
                assert_eq!((o.kind, o.pos, o.hp), (ObjectKind::HandOfHeaven, EXIT, Some(200.0)));
                assert!(math::facing_vector(o.facing).x < -0.999, "facing out, west");
                INSIDE.with_borrow(|inside| {
                    for gone in &inside[1..] {
                        assert!(sim.object(*gone).is_none() && sim.held_unit(*gone).is_none());
                    }
                });
            } else {
                assert_eq!(produced, [], "tick {tick}");
            }
            for e in sim.events() {
                match *e {
                    SimEvent::ProjectileLaunched {
                        shooter,
                        target,
                        homing,
                        ..
                    } => {
                        assert_eq!((shooter, target, homing), (HAND, Some(TARGET), false));
                        let (hand, it) = (sim.object(HAND).unwrap(), sim.object(TARGET).unwrap());
                        assert!(
                            gap(hand, it) > 7.0 * SU_PER_GU,
                            "tick {tick}: beyond its own sight, {} su off",
                            gap(hand, it)
                        );
                        assert!(sim.is_visible(P1, it), "the spotter sees it");
                        FIRED.with_borrow_mut(|fired| fired.0.push(tick));
                    }
                    SimEvent::Blast { attacker, .. } => {
                        assert_eq!(attacker, Some(HAND));
                        FIRED.with_borrow_mut(|fired| fired.1.push(tick));
                    }
                    SimEvent::Hit {
                        attacker,
                        target,
                        damage,
                        ..
                    } => {
                        assert_eq!((attacker, target), (HAND, TARGET), "tick {tick}");
                        assert!((damage - BLAST_ON_IT).abs() < 1e-2, "tick {tick}: {damage}");
                    }
                    _ => {}
                }
            }
            if let Some(hand) = sim.object(HAND)
                && tick < LANDINGS[0]
            {
                assert!(!sim.is_visible(P2, hand), "tick {tick}: player 2 sees nothing of it");
            }
            if tick == LANDINGS[0] {
                let hand = sim.object(HAND).expect("the Hand of Heaven");
                assert!(sim.is_visible(P2, hand), "revealed as the missile lands");
            }
            if tick == LANDINGS[1] {
                let hand = sim.object(HAND).expect("the Hand of Heaven");
                let left = hand.pos.distance(sim.object(TARGET).map_or(Vec2::ZERO, |t| t.pos));
                assert!(left > 1100.0, "driven off before the last landing: {left} su off");
            }
        },
        expect: |sim| {
            FIRED.with_borrow(|fired| {
                assert_eq!((fired.0.as_slice(), fired.1.as_slice()), (&LAUNCHES[..], &LANDINGS[..]));
            });
            assert!(sim.object(TARGET).is_none(), "the Deployment Center fell");
            assert!(sim.object(GENERATOR).is_some() && sim.object(SPOTTER).is_some());
            let hand = sim.object(HAND).expect("the Hand of Heaven");
            assert_eq!(hand.hp, Some(200.0), "untouched");
            assert!(hand.pos.distance(RALLY) < 32.0, "back at the rally point: {}", hand.pos);
        },
    }
}
