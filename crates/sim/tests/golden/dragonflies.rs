//! The GDO's Dragonfly golden game (slice M6-GL; `factions/gdo/structures/propulsion_labs.md`,
//! `factions/gdo/units/dragonfly.md`, `systems/mechanics/aircraft_landing.md > Assigned base`; design changes
//! 0366, 0428 and 0435): the Deployment Center constructs Propulsion Labs, which unlocks the Airfield's Dragonfly;
//! three Dragonflies rebased onto the Airfield land on its slots by their landing approach, stopping over each in the
//! air and landing vertically, and the one it trains appears landed on the last. A raid on player 2's Sparrows: each
//! Dragonfly's one homing missile, fired along its heading, shoots at one, and the Sparrows' guns hit back; out of
//! ammo, each flies home by itself, lands and rearms in 64 frames, and stays landed. A sortie lifts the four off
//! straight up to circle a point together; sent home (F), they land once more, from another way.

use std::cell::RefCell;

use sc_sim::landing;
use sc_sim::{BuildingOrientation, ConstructionCommand, DebugCommand, Object, SimConfig};

use super::*;

// The objects, by id: 1 player 1's Deployment Center, on tiles (2, 7) to (5, 10), and 2 its Generator, on tiles (2,
// 12) to (3, 13), whose Power +50 runs the Airfield's -30 and Propulsion Labs' -20; 3 the Airfield; 4 player 2's
// Storage, far off in the south-east, so that player 2 owns a structure all game (`core/game_rules.md > Defeat
// Condition`); 5 to 8 player 2's Sparrows; 9 to 11 player 1's Dragonflies in the air; 12 Propulsion Labs, once placed;
// 13 the Dragonfly the Airfield trains.
/// Player 1's Airfield, on tiles (8, 7) to (10, 9), its centre at (608, 544): its four landing slots are the centres
/// of its quarters, north-west, north-east, south-west and south-east (`sc_sim::landing::slots`).
const AIRFIELD: ObjectId = ObjectId(3);
const SLOTS: [Vec2; 4] = [
    Vec2::new(560.0, 496.0),
    Vec2::new(656.0, 496.0),
    Vec2::new(560.0, 592.0),
    Vec2::new(656.0, 592.0),
];
/// Player 2's four Sparrows, each circling where it was spawned (its idle circle), in a line 17 grid units east of the
/// Airfield, 160 su apart. Their machine guns hit air and ground ("TargetDomain: **Universal**",
/// `factions/cults/units/sparrow.md`), and nothing of player 1's is within their sight until the raid.
const SPARROWS: [u32; 4] = [5, 6, 7, 8];
const SPARROWS_AT: [Vec2; 4] = [
    Vec2::new(1696.0, 304.0),
    Vec2::new(1696.0, 464.0),
    Vec2::new(1696.0, 624.0),
    Vec2::new(1696.0, 784.0),
];
/// Player 1's Dragonflies: three in the air south of the Airfield, unassigned, and the one the Airfield trains.
const DRAGONFLIES: [u32; 4] = [9, 10, 11, 13];
const TRAINED: u32 = 13;
/// Propulsion Labs, constructed from tick 1 (400 frames), placed at [`PLACED`] on tiles (5, 12) to (6, 13); the
/// Airfield's W in the tick after, the Dragonfly out 320 frames on, at [`OUT`].
const LABS: ObjectId = ObjectId(12);
const PLACED: Tick = 401;
const OUT: Tick = PLACED + 320;
/// The raid, an AttackMove of the four onto a point past the Sparrows.
const RAID: Tick = 760;
const ONTO: Vec2 = Vec2::new(1856.0, 544.0);
/// The sortie, a Move of the four onto a point north-west, away from the Sparrows left.
const SORTIE: Tick = 1300;
const SORTIE_AT: Vec2 = Vec2::new(320.0, 192.0);
/// The four are sent home (F).
const HOME: Tick = 1400;
/// The game's last tick.
const END: Tick = 1700;

/// The game's events of interest so far.
#[derive(Default)]
struct Seen {
    /// Whether each Dragonfly was landing or down on a slot at the end of the tick before.
    on_slot: BTreeMap<u32, bool>,
    /// The landings begun (an aircraft coming onto a slot from the air), the take-offs and the missiles, by tick and
    /// Dragonfly; the Sparrows' deaths; and the Sparrows' hits on the Dragonflies, by tick and Dragonfly.
    landings: Vec<(Tick, u32)>,
    take_offs: Vec<(Tick, u32)>,
    launches: Vec<(Tick, u32)>,
    deaths: Vec<(Tick, ObjectId)>,
    gunned: Vec<(Tick, u32)>,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
}

/// Player 1's `command`.
fn p1(command: Command) -> PlayerCommand {
    PlayerCommand { player: P1, command }
}

/// Dragonfly `n`, which is in the game from its tick on.
fn dragonfly(sim: &Sim, n: u32) -> &Object {
    sim.object(ObjectId(n))
        .unwrap_or_else(|| panic!("tick {}: Dragonfly {n} gone", sim.tick()))
}

/// The four are home: each down on its own slot of the Airfield, assigned to it, full and idle.
fn all_home(sim: &Sim) {
    let tick = sim.tick();
    let mut slots = Vec::new();
    for n in DRAGONFLIES {
        let o = dragonfly(sim, n);
        let state = o.aircraft.as_ref().expect("an aircraft");
        assert!(landing::down(o), "tick {tick}: {n} down");
        assert_eq!((state.base, state.ammo), (Some(AIRFIELD), 1), "tick {tick}: {n}");
        let doing = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
        assert_eq!(doing.map(|active| active.order.kind), None, "tick {tick}: {n} idle");
        let slot = SLOTS.iter().position(|&slot| slot.distance(o.pos) <= 8.0);
        slots.push(slot.unwrap_or_else(|| panic!("tick {tick}: {n} at {}, on no slot", o.pos)));
    }
    slots.sort();
    slots.dedup();
    assert_eq!(slots.len(), 4, "tick {tick}: a slot each");
}

/// Player 1 plays the GDO with a Deployment Center, a Generator and an Airfield, and gets by a debug command
/// Propulsion Labs' 200 Space Crystals and a Dragonfly's 175 and 4 Supplies; player 2 the Cults, whose four Sparrows
/// circle 17 grid units east. In tick 1 the Deployment Center begins Propulsion Labs (R), and the three Dragonflies in
/// the air are rebased onto the Airfield (`OrderKind::Rebase`, a right-click on it): each flies its landing approach to
/// the first slot free in id order, the last of it at IdleSpeed, stops over the slot in the air, and lands there in 16
/// frames (`movement::approach`). Propulsion Labs is placed at [`PLACED`], and the Airfield's W, refused until then,
/// trains a Dragonfly, which appears at [`OUT`] landed on the last slot, south-east, and with no rally point stays
/// there (design change 0428 point 2). At [`RAID`] the four lift off straight up and AttackMove east, head on at the
/// line of Sparrows: each fires its one missile along its heading at a Sparrow as it comes within its Range, and the
/// missile homes in (Damage 90 on MaxHP 50). As the game played when it was made, the four missiles killed four
/// Sparrows, one each, and the Sparrows' guns hit two Dragonflies in passing. Out of ammo, each flies
/// home by itself (the automatic return), lands on the first slot free in id order and rearms in 64 frames, and stays
/// landed. At [`SORTIE`] a Move lifts the four off again, to circle a point in the north-west together; at [`HOME`]
/// they are sent home (F) and land once more, coming in from the north-west. No Dragonfly is lost.
pub(crate) fn gdo_dragonflies() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 20))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Cults)
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 7)),
        )
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(2, 12)))
        .spawn(ObjectKind::Airfield, Some(P1), SpawnAt::Tile(TilePos::new(8, 7)))
        .spawn(ObjectKind::Storage, Some(P2), SpawnAt::Tile(TilePos::new(36, 17)));
    for at in SPARROWS_AT {
        builder = builder.spawn(ObjectKind::Sparrow, Some(P2), SpawnAt::Point(at));
    }
    for y in [900.0, 1020.0, 1140.0] {
        builder = builder.spawn(ObjectKind::Dragonfly, Some(P1), SpawnAt::Point(Vec2::new(608.0, y)));
    }
    let bank = p1(Command::Debug(DebugCommand::SetResources {
        player: P1,
        crystals: 200 + 175,
        supplies: 4,
        alloys: 0,
        essence: 0,
        flux: 0,
    }));
    let construction = |command| {
        p1(Command::Construction {
            objects: vec![ObjectId(1)],
            command,
        })
    };
    let labs = construction(ConstructionCommand::Construct(ObjectKind::PropulsionLabs));
    let place = construction(ConstructionCommand::Place {
        origin: TilePos::new(5, 12),
        orientation: BuildingOrientation::default(),
    });
    let train = p1(Command::Production {
        objects: vec![AIRFIELD],
        command: ProductionCommand::Train(ObjectKind::Dragonfly),
    });
    let commands = vec![
        (1, bank),
        (1, labs),
        (1, order(P1, [9, 10, 11], OrderKind::Rebase(AIRFIELD))),
        (PLACED, place),
        (PLACED + 1, train),
        (RAID, order(P1, DRAGONFLIES, OrderKind::AttackMove(ONTO))),
        (
            SORTIE,
            order(P1, DRAGONFLIES, OrderKind::Move(Target::Point(SORTIE_AT))),
        ),
        (HOME, order(P1, DRAGONFLIES, OrderKind::ReturnToBase)),
    ];
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                SEEN.set(Seen::default());
                let bank = sim.player(P1).map(|p| (p.crystals, p.supplies));
                assert_eq!(bank, Some((175, 4)), "Propulsion Labs paid");
            }
            if tick == PLACED {
                let labs = sim.object(LABS).map(|o| (o.kind, o.owner));
                assert_eq!(labs, Some((ObjectKind::PropulsionLabs, Some(P1))), "placed");
            }
            if tick == PLACED + 1 {
                let bank = sim.player(P1).map(|p| (p.crystals, p.supplies));
                assert_eq!(bank, Some((0, 0)), "the Dragonfly paid");
            }
            for e in sim.events() {
                match *e {
                    SimEvent::UnitProduced { producer, unit } => {
                        assert_eq!((tick, producer, unit), (OUT, AIRFIELD, ObjectId(TRAINED)));
                        let o = dragonfly(sim, TRAINED);
                        assert!(landing::down(o), "landed");
                        assert_eq!(o.pos, SLOTS[3], "on the last free slot");
                    }
                    SimEvent::ProjectileLaunched { shooter, target, .. } => {
                        assert!(DRAGONFLIES.contains(&shooter.0), "tick {tick}: {e:?}");
                        let at_sparrow = target.is_some_and(|target| SPARROWS.contains(&target.0));
                        assert!(at_sparrow, "tick {tick}: {e:?}");
                        SEEN.with_borrow_mut(|seen| seen.launches.push((tick, shooter.0)));
                    }
                    SimEvent::Hit { attacker, target, .. } => {
                        let missile = DRAGONFLIES.contains(&attacker.0) && SPARROWS.contains(&target.0);
                        let gun = SPARROWS.contains(&attacker.0) && DRAGONFLIES.contains(&target.0);
                        assert!(missile || gun, "tick {tick}: {e:?}");
                        if gun {
                            SEEN.with_borrow_mut(|seen| seen.gunned.push((tick, target.0)));
                        }
                    }
                    SimEvent::Died { id, .. } => {
                        assert!(SPARROWS.contains(&id.0), "tick {tick}: {id:?} died");
                        SEEN.with_borrow_mut(|seen| seen.deaths.push((tick, id)));
                    }
                    _ => {}
                }
            }
            SEEN.with_borrow_mut(|seen| {
                for n in DRAGONFLIES {
                    if n == TRAINED && tick < OUT {
                        continue;
                    }
                    let o = dragonfly(sim, n);
                    let on_slot = o.aircraft.is_some() && o.landing.is_some();
                    match (seen.on_slot.insert(n, on_slot), on_slot) {
                        (Some(false), true) => seen.landings.push((tick, n)),
                        (Some(true), false) => seen.take_offs.push((tick, n)),
                        _ => {}
                    }
                }
            });
            if tick == RAID - 1 || tick == SORTIE - 1 {
                all_home(sim);
            }
            if tick == HOME - 1 {
                // Each circles its own point of the group's destinations round the sortie's (`orders::destinations`).
                for n in DRAGONFLIES {
                    let o = dragonfly(sim, n);
                    let circles = o.mover.as_ref().and_then(|mover| mover.orbit);
                    let near = circles.is_some_and(|at| at.distance(SORTIE_AT) <= 2.0 * SU_PER_GU);
                    assert!(near, "tick {tick}: {n} circles {circles:?}");
                }
            }
        },
        expect: |sim| {
            all_home(sim);
            assert!(sim.object(ObjectId(4)).is_some(), "player 2's Storage stands");
            SEEN.with_borrow(|seen| {
                // The raid: a missile each, four Sparrows down, and the Sparrows' guns hitting back (on two
                // Dragonflies, 13 and 9, as the game played when it was made), the hit ones' HP short of their MaxHP.
                let mut fired: Vec<u32> = seen.launches.iter().map(|&(_, n)| n).collect();
                fired.sort();
                assert_eq!(fired, DRAGONFLIES, "{:?}", seen.launches);
                assert!(seen.launches.iter().all(|&(tick, _)| tick > RAID && tick < SORTIE));
                assert_eq!(seen.deaths.len(), 4, "{:?}", seen.deaths);
                let mut gunned: Vec<u32> = seen.gunned.iter().map(|&(_, n)| n).collect();
                gunned.dedup();
                assert!(!gunned.is_empty(), "the Sparrows hit back");
                for n in DRAGONFLIES {
                    let o = dragonfly(sim, n);
                    let hit = seen.gunned.iter().any(|&(_, m)| m == n);
                    assert_eq!(o.hp < o.def().max_hp, hit, "{n}: {:?}", o.hp);
                }
                // Each order lifts all four off in its tick, and each lands once after it: by itself after the raid,
                // by the F after the sortie; the three rebased landed once before.
                for lift in [RAID, SORTIE] {
                    let lifted: Vec<u32> = (seen.take_offs.iter())
                        .filter(|&&(tick, _)| tick == lift)
                        .map(|&(_, n)| n)
                        .collect();
                    assert_eq!(lifted, DRAGONFLIES, "tick {lift}");
                }
                assert_eq!(seen.take_offs.len(), 8, "{:?}", seen.take_offs);
                let landed = |n: u32, from: Tick, to: Tick| {
                    (seen.landings.iter())
                        .filter(|&&(tick, m)| m == n && (from..to).contains(&tick))
                        .count()
                };
                for n in DRAGONFLIES {
                    let before = if n == TRAINED { 0 } else { 1 };
                    let counts = [
                        landed(n, 0, RAID),
                        landed(n, RAID, SORTIE),
                        landed(n, SORTIE, HOME),
                        landed(n, HOME, END + 1),
                    ];
                    assert_eq!(counts, [before, 1, 0, 1], "{n}: {:?}", seen.landings);
                }
            });
        },
    }
}
