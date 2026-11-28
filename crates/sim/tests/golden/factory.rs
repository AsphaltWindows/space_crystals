//! The GDO's vehicles golden game (slice M6-GF; `factions/gdo/structures/factory.md`, `factions/gdo/units/raptor.md`,
//! `factions/gdo/units/twister.md`): a Factory trains two Raptors and two Twisters, which drive out of its doorway on
//! side B and park round its default rally point (design changes 0446 and 0451), then attack-move onto player 2's
//! Recruits and Sparrows: the Raptors' homing missiles, which hit ground and air, kill the Recruits, and the Twisters'
//! machine guns, which hit air alone, the Sparrows.

use std::cell::RefCell;

use sc_sim::shape::Shape;
use sc_sim::{DebugCommand, Object, SimConfig};

use super::*;

// The objects, by id: 1 player 1's Deployment Center, on tiles (2, 7) to (5, 10), whose Power runs the Factory; 2 the
// Factory; 3 player 2's Storage, far off in the south-east, so that player 2 owns a structure all game
// (`core/game_rules.md > Defeat Condition`); 4 to 7 player 2's Recruits and 8 and 9 its Sparrows, on Hold, circling
// over them; 10 to 13 the vehicles, as they come out.
/// Player 1's Factory, on tiles (8, 8) to (10, 10), unturned: its exit, side B, to the east (x 704), the middle of
/// which is [`EXIT`].
const FACTORY: ObjectId = ObjectId(2);
const RECRUITS: [u32; 4] = [4, 5, 6, 7];
const SPARROWS: [u32; 2] = [8, 9];
const VEHICLES: [u32; 4] = [10, 11, 12, 13];
/// The middle of side B, where a vehicle appears, half inside, facing east.
const EXIT: Vec2 = Vec2::new(704.0, 608.0);
/// The Factory's default rally point, 3 grid units out of side B (design change 0446).
const RALLY: Vec2 = Vec2::new(EXIT.x + 3.0 * SU_PER_GU, EXIT.y);
/// What the Factory makes, in its queue's order, all four queued in tick 1: a Raptor, a Twister, a Raptor and a
/// Twister.
const KINDS: [ObjectKind; 4] = [
    ObjectKind::Raptor,
    ObjectKind::Twister,
    ObjectKind::Raptor,
    ObjectKind::Twister,
];
/// The tick each comes out in: a Raptor 240 frames after the start of its production, a Twister 320
/// (`factions/gdo/structures/factory.md > Produces`), each started in the tick after the one before came out.
const OUT: [Tick; 4] = [240, 560, 800, 1120];
/// The four attack-move onto player 2's units from this tick on, parked by then.
const ATTACK: Tick = 1160;
/// Where they attack-move to: past player 2's Recruits.
const ONTO: (i32, i32) = (33, 9);
/// The game's last tick.
const END: Tick = 1600;

/// The hits of the game so far: by the Raptors on the Recruits and on the Sparrows, by the Twisters on the Sparrows,
/// and on the vehicles by each of player 2's kinds (as the game played when it was made: the Raptors' twelve missiles
/// killed the four Recruits, one of which reached a vehicle first and struck it, and the Twisters' 26 rounds the two
/// Sparrows, which never came within their Range of 3 grid units of a vehicle; the Raptors hit neither Sparrow).
#[derive(Default)]
struct Hits {
    raptor_on_recruits: u32,
    raptor_on_sparrows: u32,
    twister_on_sparrows: u32,
    by_recruits: u32,
    by_sparrows: u32,
}

thread_local! {
    static HITS: RefCell<Hits> = RefCell::new(Hits::default());
}

/// Player 1's `command`.
fn p1(command: Command) -> PlayerCommand {
    PlayerCommand { player: P1, command }
}

/// Player 1 plays the GDO with a Deployment Center and a Factory, whose Power -30 the Deployment Center's +30 covers,
/// and gets by a debug command the four vehicles' 450 Space Crystals and the Twisters' 2 Supplies (set over the GDO's
/// starting 1). Player 2 plays the Cults: four Recruits stand 18 grid units east of the Factory,
/// and two Sparrows hold over them. At tick 1 the Factory queues its four vehicles ([`KINDS`]): each comes out at its
/// tick of [`OUT`], half inside the middle of side B and facing east, and drives to the default rally point; the first
/// stops on it, and each of the others, the point taken, parks at the free spot nearest it (design change 0451). At
/// [`ATTACK`] the four attack-move east: the Raptors' missiles kill the Recruits, the Twisters' machine guns, which hit
/// air alone, the Sparrows, and all six of player 2's units fall, the last at 1:33 (tick 1496; tick 1499 before design
/// change 0454: a Raptor whose Recruit runs in inside the circles it turns on drives straight on to come round onto it,
/// `orders::aim`).
pub(crate) fn gdo_vehicles() -> Game {
    let builder = ScenarioBuilder::new(&open(40, 20))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Cults)
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 7)),
        )
        .spawn(ObjectKind::Factory, Some(P1), SpawnAt::Tile(TilePos::new(8, 8)))
        .spawn(ObjectKind::Storage, Some(P2), SpawnAt::Tile(TilePos::new(36, 17)))
        .spawn(ObjectKind::Recruit, Some(P2), at(28, 8))
        .spawn(ObjectKind::Recruit, Some(P2), at(28, 10))
        .spawn(ObjectKind::Recruit, Some(P2), at(29, 9))
        .spawn(ObjectKind::Recruit, Some(P2), at(30, 10))
        .spawn(ObjectKind::Sparrow, Some(P2), at(29, 6))
        .spawn(ObjectKind::Sparrow, Some(P2), at(29, 12));
    let bank = p1(Command::Debug(DebugCommand::SetResources {
        player: P1,
        crystals: 450,
        supplies: 2,
        alloys: 0,
        essence: 0,
        flux: 0,
    }));
    let train = |kind| {
        p1(Command::Production {
            objects: vec![FACTORY],
            command: ProductionCommand::Train(kind),
        })
    };
    let mut commands = vec![(1, bank)];
    commands.extend(KINDS.map(|kind| (1, train(kind))));
    commands.push((1, order(P2, SPARROWS, OrderKind::Hold)));
    commands.push((
        ATTACK,
        order(P1, VEHICLES, OrderKind::AttackMove(point(ONTO.0, ONTO.1))),
    ));
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                HITS.set(Hits::default());
                let bank = sim.player(P1).map(|p| (p.crystals, p.supplies));
                assert_eq!(bank, Some((0, 0)), "the four paid");
            }
            let produced: Vec<ObjectId> = (sim.events().iter())
                .filter_map(|e| match *e {
                    SimEvent::UnitProduced { producer, unit } if producer == FACTORY => Some(unit),
                    _ => None,
                })
                .collect();
            match OUT.iter().position(|&at| at == tick) {
                Some(i) => {
                    let id = ObjectId(VEHICLES[i]);
                    assert_eq!(produced, [id], "tick {tick}");
                    let o = sim.object(id).expect("out");
                    assert_eq!((o.kind, o.pos, o.facing), (KINDS[i], EXIT, 0.0), "tick {tick}");
                }
                None => assert_eq!(produced, [], "tick {tick}"),
            }
            if tick == ATTACK - 1 {
                let parked: Vec<&Object> = VEHICLES.iter().map(|&n| sim.object(ObjectId(n)).unwrap()).collect();
                assert!(parked[0].pos.distance(RALLY) < 1.0, "the first on the point");
                for (i, o) in parked.iter().enumerate() {
                    let mover = o.mover.as_ref().expect("a vehicle");
                    assert_eq!((mover.momentum, mover.exit), (Vec2::ZERO, None), "{i}: parked");
                    let reach = o.def().half_extents().unwrap().length();
                    assert!(
                        o.pos.distance(RALLY) <= 3.0 * SU_PER_GU + reach,
                        "{i} round it: {}",
                        o.pos
                    );
                    for other in &parked[i + 1..] {
                        assert!(
                            !Shape::of(o).overlaps(&Shape::of(other)),
                            "{:?} and {:?}",
                            o.id,
                            other.id
                        );
                    }
                }
            }
            let kind_of = |id: ObjectId| sim.object(id).map(|o| o.kind);
            HITS.with_borrow_mut(|hits| {
                for e in sim.events() {
                    let SimEvent::Hit { attacker, target, .. } = *e else {
                        continue;
                    };
                    assert!(tick > ATTACK, "tick {tick}: no fight before the attack: {e:?}");
                    let by_vehicle = VEHICLES.contains(&attacker.0);
                    match (kind_of(attacker), target.0) {
                        (Some(ObjectKind::Raptor), n) if RECRUITS.contains(&n) => hits.raptor_on_recruits += 1,
                        (Some(ObjectKind::Raptor), n) if SPARROWS.contains(&n) => hits.raptor_on_sparrows += 1,
                        (Some(ObjectKind::Twister), n) if SPARROWS.contains(&n) => hits.twister_on_sparrows += 1,
                        (_, n) if !by_vehicle && VEHICLES.contains(&n) && RECRUITS.contains(&attacker.0) => {
                            hits.by_recruits += 1
                        }
                        (_, n) if !by_vehicle && VEHICLES.contains(&n) && SPARROWS.contains(&attacker.0) => {
                            hits.by_sparrows += 1
                        }
                        _ => panic!("tick {tick}: {attacker:?} hit {target:?}"),
                    }
                }
            });
        },
        expect: |sim| {
            let alive = |ids: &[u32]| ids.iter().filter(|&&n| sim.object(ObjectId(n)).is_some()).count();
            assert_eq!(alive(&RECRUITS) + alive(&SPARROWS), 0, "player 2's units fell");
            assert_eq!(alive(&VEHICLES), 4, "the vehicles stand");
            HITS.with_borrow(|hits| {
                assert!(
                    hits.raptor_on_recruits >= 12,
                    "40 HP each, 14 a hit: {}",
                    hits.raptor_on_recruits
                );
                assert_eq!(hits.twister_on_sparrows, 26, "50 HP each, 4 a hit");
                assert!(hits.by_recruits > 0, "a Recruit reached a vehicle");
            });
        },
    }
}
