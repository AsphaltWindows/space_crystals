//! The GDO's air golden game (slice M6-GH2, plan decision T88; `factions/gdo/structures/airfield.md`,
//! `factions/gdo/units/hornet.md`, design changes 0428 and 0458): an Airfield's four Hornets, three rebased onto it and
//! one it trains, which appears landed on the last free slot, raid a block of Recruits, which can't hit back: their
//! missiles' bursts hit several Recruits at once. Out of ammo, each flies home by itself, lands on a free slot and
//! rearms, then stays landed until the second raid lifts them off and kills the rest; sent home (F), the four land
//! once more and rearm what they fired.

use std::cell::RefCell;

use sc_sim::landing;
use sc_sim::{DebugCommand, Object, SimConfig};

use super::*;

// The objects, by id: 1 player 1's Deployment Center, on tiles (2, 7) to (5, 10), whose Power +30 runs the Airfield's
// -30; 2 the Airfield; 3 player 2's Storage, far off in the south-east, so that player 2 owns a structure all game
// (`core/game_rules.md > Defeat Condition`); 4 to 15 player 2's Recruits; 16 to 19 player 1's Hornets.
/// Player 1's Airfield, on tiles (8, 7) to (10, 9), its centre at (608, 544): its four landing slots are the centres
/// of its quarters, north-west, north-east, south-west and south-east (`sc_sim::landing::slots`).
const AIRFIELD: ObjectId = ObjectId(2);
const SLOTS: [Vec2; 4] = [
    Vec2::new(560.0, 496.0),
    Vec2::new(656.0, 496.0),
    Vec2::new(560.0, 592.0),
    Vec2::new(656.0, 592.0),
];
/// Player 2's twelve Recruits, on Hold in a block of 4 by 3, 28 su apart, round [`CLUMP`], 17 grid units east of the
/// Airfield: their melee hits the ground alone (`factions/cults/units/recruit.md`).
const RECRUITS: [u32; 12] = [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
const CLUMP: Vec2 = Vec2::new(1696.0, 544.0);
/// Player 1's Hornets: three in the air south of the Airfield, unassigned, and the one the Airfield trains, queued in
/// tick 1 and finished 320 frames on, in [`OUT`].
const HORNETS: [u32; 4] = [16, 17, 18, 19];
const TRAINED: u32 = 19;
const OUT: Tick = 320;
/// The two raids, each an AttackMove of the four onto [`ONTO`], past the Recruits.
const RAIDS: [Tick; 2] = [360, 960];
const ONTO: Vec2 = Vec2::new(2016.0, 544.0);
/// The four are sent home (F), the Recruits dead.
const HOME: Tick = 1240;
/// The game's last tick.
const END: Tick = 1600;

/// The game's events of interest so far.
#[derive(Default)]
struct Seen {
    /// Whether each Hornet was landing or down on a slot at the end of the tick before.
    on_slot: BTreeMap<u32, bool>,
    /// The landings begun (an aircraft coming onto a slot from the air) and the take-offs, by tick and Hornet.
    landings: Vec<(Tick, u32)>,
    take_offs: Vec<(Tick, u32)>,
    /// The Hornets emptied in the first raid, the most Recruits one missile's burst hit, the Hornets' hits by raid,
    /// and the Recruits' deaths.
    emptied: Vec<u32>,
    most_in_a_burst: usize,
    hits: [usize; 2],
    deaths: Vec<(Tick, ObjectId)>,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
}

/// Player 1's `command`.
fn p1(command: Command) -> PlayerCommand {
    PlayerCommand { player: P1, command }
}

/// Hornet `n`, which is in the game all along.
fn hornet(sim: &Sim, n: u32) -> &Object {
    sim.object(ObjectId(n))
        .unwrap_or_else(|| panic!("tick {}: Hornet {n} gone", sim.tick()))
}

/// The four are home: each down on its own slot of the Airfield, assigned to it, full and idle; in `order`, on the
/// slots of [`SLOTS`] by id.
fn all_home(sim: &Sim, order: bool) {
    let tick = sim.tick();
    let mut slots = Vec::new();
    for n in HORNETS {
        let o = hornet(sim, n);
        let state = o.aircraft.as_ref().expect("an aircraft");
        assert!(landing::down(o), "tick {tick}: {n} down");
        assert_eq!((state.base, state.ammo), (Some(AIRFIELD), 5), "tick {tick}: {n}");
        let doing = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
        assert_eq!(doing.map(|active| active.order.kind), None, "tick {tick}: {n} idle");
        let slot = SLOTS.iter().position(|&slot| slot == o.pos);
        slots.push(slot.unwrap_or_else(|| panic!("tick {tick}: {n} at {}, on no slot", o.pos)));
    }
    if order {
        assert_eq!(slots, [0, 1, 2, 3], "tick {tick}");
    }
    slots.sort();
    slots.dedup();
    assert_eq!(slots.len(), 4, "tick {tick}: a slot each");
}

/// Player 1 plays the GDO with a Deployment Center and an Airfield, and gets by a debug command a Hornet's 150 Space
/// Crystals and 2 Supplies (set over the GDO's starting 1); player 2 the Cults, whose twelve
/// Recruits stand on Hold 17 grid units east. In tick 1 the Airfield queues a Hornet, and the three in the air are
/// rebased onto it (`OrderKind::Rebase`, a right-click on it): each flies to the first slot free in id order and lands
/// there in 16 frames. The trained one appears at [`OUT`] landed on the last, south-east, and with no rally point stays
/// there (design change 0428 point 2). At the first of [`RAIDS`] the four lift off and AttackMove east: their missiles
/// burst over the block (`TargetType::AoE`, 32 su, design change 0458), each hitting the Recruits within its circle,
/// each by its share. As the game played when it was made, a burst hit up to six at once, and the first raid's 20
/// missiles killed six; out of ammo, each Hornet flies home by itself (the automatic return), lands on the first slot
/// free in id order and rearms its five missiles in 160 frames, and stays landed. The second raid lifts them off again
/// and kills the other six; the two out of ammo fly home by themselves, the two with a missile left fly on to [`ONTO`].
/// At [`HOME`] all four are sent home (F) and end landed, rearmed. No Hornet is hit.
pub(crate) fn gdo_air() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 20))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Cults)
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 7)),
        )
        .spawn(ObjectKind::Airfield, Some(P1), SpawnAt::Tile(TilePos::new(8, 7)))
        .spawn(ObjectKind::Storage, Some(P2), SpawnAt::Tile(TilePos::new(36, 17)));
    for dy in [-28.0, 0.0, 28.0] {
        for dx in [-42.0, -14.0, 14.0, 42.0] {
            let at = SpawnAt::Point(CLUMP + Vec2::new(dx, dy));
            builder = builder.spawn(ObjectKind::Recruit, Some(P2), at);
        }
    }
    for y in [900.0, 1020.0, 1140.0] {
        builder = builder.spawn(ObjectKind::Hornet, Some(P1), SpawnAt::Point(Vec2::new(608.0, y)));
    }
    let bank = p1(Command::Debug(DebugCommand::SetResources {
        player: P1,
        crystals: 150,
        supplies: 2,
        alloys: 0,
        essence: 0,
        flux: 0,
    }));
    let train = p1(Command::Production {
        objects: vec![AIRFIELD],
        command: ProductionCommand::Train(ObjectKind::Hornet),
    });
    let raid = || order(P1, HORNETS, OrderKind::AttackMove(ONTO));
    let commands = vec![
        (1, bank),
        (1, train),
        (1, order(P1, [16, 17, 18], OrderKind::Rebase(AIRFIELD))),
        (1, order(P2, RECRUITS, OrderKind::Hold)),
        (RAIDS[0], raid()),
        (RAIDS[1], raid()),
        (HOME, order(P1, HORNETS, OrderKind::ReturnToBase)),
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
                assert_eq!(bank, Some((0, 0)), "the Hornet paid");
            }
            let mut bursts: BTreeMap<ObjectId, usize> = BTreeMap::new();
            for e in sim.events() {
                match *e {
                    SimEvent::UnitProduced { producer, unit } => {
                        assert_eq!((tick, producer, unit), (OUT, AIRFIELD, ObjectId(TRAINED)));
                        let o = hornet(sim, TRAINED);
                        assert!(landing::down(o), "landed");
                        assert_eq!(o.pos, SLOTS[3], "on the last free slot");
                    }
                    SimEvent::Hit { attacker, target, .. } => {
                        let raid = RAIDS.iter().rposition(|&raid| tick > raid);
                        let raid = raid.unwrap_or_else(|| panic!("tick {tick}: no hit before the raid: {e:?}"));
                        assert!(
                            HORNETS.contains(&attacker.0) && RECRUITS.contains(&target.0),
                            "tick {tick}: {e:?}"
                        );
                        SEEN.with_borrow_mut(|seen| seen.hits[raid] += 1);
                        *bursts.entry(attacker).or_default() += 1;
                    }
                    SimEvent::Died { id, .. } => {
                        assert!(RECRUITS.contains(&id.0), "tick {tick}: {id:?} died");
                        SEEN.with_borrow_mut(|seen| seen.deaths.push((tick, id)));
                    }
                    _ => {}
                }
            }
            SEEN.with_borrow_mut(|seen| {
                let most = bursts.values().copied().max().unwrap_or(0);
                seen.most_in_a_burst = seen.most_in_a_burst.max(most);
                for n in HORNETS {
                    if n == TRAINED && tick < OUT {
                        continue;
                    }
                    let o = hornet(sim, n);
                    let on_slot = o.aircraft.is_some() && o.landing.is_some();
                    match (seen.on_slot.insert(n, on_slot), on_slot) {
                        (Some(false), true) => seen.landings.push((tick, n)),
                        (Some(true), false) => seen.take_offs.push((tick, n)),
                        _ => {}
                    }
                    let ammo = o.aircraft.as_ref().map_or(0, |state| state.ammo);
                    if (RAIDS[0]..RAIDS[1]).contains(&tick) && ammo == 0 && !seen.emptied.contains(&n) {
                        seen.emptied.push(n);
                    }
                }
            });
            if tick == RAIDS[0] - 1 {
                all_home(sim, true);
            }
            if tick == RAIDS[1] - 1 {
                all_home(sim, false);
                SEEN.with_borrow(|seen| {
                    assert_eq!(seen.emptied.len(), 4, "all four ran out: {:?}", seen.emptied);
                    let killed = seen.deaths.len();
                    assert!((1..RECRUITS.len()).contains(&killed), "the first raid killed {killed}");
                });
            }
            if tick == HOME - 1 {
                let alive = RECRUITS.iter().filter(|&&n| sim.object(ObjectId(n)).is_some()).count();
                assert_eq!(alive, 0, "the Recruits are dead");
            }
        },
        expect: |sim| {
            all_home(sim, false);
            assert!(sim.object(ObjectId(3)).is_some(), "player 2's Storage stands");
            for n in HORNETS {
                let o = hornet(sim, n);
                assert_eq!(o.hp, o.def().max_hp, "{n} untouched");
            }
            SEEN.with_borrow(|seen| {
                assert!(
                    seen.most_in_a_burst >= 4,
                    "a burst over the block: {}",
                    seen.most_in_a_burst
                );
                assert!(
                    seen.hits.iter().all(|&hits| hits > 0),
                    "both raids hit: {:?}",
                    seen.hits
                );
                // Each raid's order lifts all four off in its tick, and each lands once after it: by itself after
                // the first, by itself or by the F after the second; the three rebased landed once before.
                for raid in RAIDS {
                    let lifted: Vec<u32> = (seen.take_offs.iter())
                        .filter(|&&(tick, _)| tick == raid)
                        .map(|&(_, n)| n)
                        .collect();
                    assert_eq!(lifted, HORNETS, "tick {raid}");
                }
                assert_eq!(seen.take_offs.len(), 8, "{:?}", seen.take_offs);
                let landed = |n: u32, from: Tick, to: Tick| {
                    (seen.landings.iter())
                        .filter(|&&(tick, m)| m == n && (from..to).contains(&tick))
                        .count()
                };
                for n in HORNETS {
                    let before = if n == TRAINED { 0 } else { 1 };
                    let counts = [
                        landed(n, 0, RAIDS[0]),
                        landed(n, RAIDS[0], RAIDS[1]),
                        landed(n, RAIDS[1], END + 1),
                    ];
                    assert_eq!(counts, [before, 1, 1], "{n}: {:?}", seen.landings);
                }
            });
        },
    }
}
