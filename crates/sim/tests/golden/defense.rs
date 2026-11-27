//! The GDO's defence golden game (slice M6-GR; `factions/gdo/structures/radar_station.md`,
//! `factions/gdo/structures/defense_system.md`, `factions/gdo/structures/missile_turret.md`): a Deployment Center
//! constructs and places a Radar Station, which scans the Sparrows' holding point, then a Defense System, which builds
//! two Missile Turrets; the second turret takes player 1's Power below what it generates as three Sparrows raid the
//! base, so that the turrets fire slowed and the station's cooldown runs slow (design change 0423), until a Generator
//! placed mid-fight brings both back to full speed; the turrets' homing missiles bring the three down, and the station
//! scans again once its cooldown is over.

use std::cell::RefCell;

use sc_sim::economy::RATIO_ONE;
use sc_sim::{AreaAbility, BuildingOrientation, ConstructionCommand, DebugCommand, Reject, SimConfig};

use super::*;

// The objects, by id: 1 to 4 player 1's, below; 5 to 7 player 2's Sparrows, on Hold over (30, 8), (30, 10) and (30,
// 12) until the raid; 8 player 2's Airstrip, far off in the south-east, so that player 2 owns a structure all game
// (`core/game_rules.md > Defeat Condition`); 9 the Radar Station, 10 the Defense System, 11 and 12 the Missile
// Turrets and 13 the third Generator, as they are placed.
/// Player 1's Deployment Center, on tiles (2, 6) to (5, 9); its Generators (2, 3) on (8, 3) and (8, 12), and its
/// Arsenal (4) on (2, 12), which the Defense System requires.
const DC: ObjectId = ObjectId(1);
const SPARROWS: [u32; 3] = [5, 6, 7];
const STATION: ObjectId = ObjectId(9);
const DEFENSE: ObjectId = ObjectId(10);
const TURRETS: [u32; 2] = [11, 12];
/// The Radar Station is ready at the end of this tick, 320 frames from its Construct in tick 1, and placed in the next,
/// where the Defense System's construction starts, the station standing.
const STATION_READY: Tick = 320;
/// The station scans the Sparrows' holding point.
const FIRST_SCAN: Tick = STATION_READY + 2;
/// The Defense System is placed, 320 frames after its Construct, and the Deployment Center starts the third Generator.
const DEFENSE_PLACED: Tick = STATION_READY + 1 + 320;
/// The turrets are placed, 240 frames apart, the second's Construct in the tick the first is placed.
const TURRETS_PLACED: [Tick; 2] = [DEFENSE_PLACED + 1 + 240, DEFENSE_PLACED + 1 + 480];
/// The Sparrows AttackMove onto the base.
const RAID: Tick = TURRETS_PLACED[1] + 1;
/// The third Generator is placed, mid-fight.
const GENERATOR_PLACED: Tick = 1250;
/// The station's cooldown, from [`FIRST_SCAN`]: 800 ticks at full Power, 128 at 70/80 (57 344 of 65 536 a tick, 112
/// ticks' worth), and the 48 ticks left at full Power from [`GENERATOR_PLACED`], over at the end of tick 1297, 16
/// ticks later than at full Power all along.
const COOLDOWN_OVER: Tick = 1297;
const SECOND_SCAN: Tick = 1300;
/// The game's last tick, a few seconds after the last Sparrow fell.
const END: Tick = 1500;

/// Where the station scans first: the Sparrows' holding point; and where it scans again, the way they came.
const HOLD_AT: (i32, i32) = (30, 10);
const SECOND_AT: (i32, i32) = (24, 10);

/// What the game's events showed: the turrets' missiles, their hits and the hits on the turrets; the casts; whether a
/// turret banked a share of a tick at low Power (`AttackState::power_carry`).
#[derive(Default)]
struct Seen {
    launched: u32,
    hits: u32,
    on_turrets: u32,
    casts: Vec<Tick>,
    banked: bool,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
}

/// Player 1 plays the GDO, player 2 the Cults, whose Sparrows they are, with debug commands on: a debug command gives
/// player 1 the buildings' Space Crystals, so that the game is about the defence, not about mining.
fn gdo_p1_cults_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.config(SimConfig::DEBUG).build();
    scenario.setup.players[0].faction = Faction::Gdo;
    scenario.setup.players[1].faction = Faction::Cults;
    scenario
}

/// The Radar Station's Q at the centre of tile `(x, y)`.
fn scan((x, y): (i32, i32)) -> Command {
    Command::Cast {
        casters: vec![STATION],
        ability: AreaAbility::Scan,
        at: point(x, y),
    }
}

/// Player 2's Sparrows hold over their spots from tick 1, circling. Player 1's Deployment Center constructs a Radar
/// Station from tick 1 and places it at [`STATION_READY`] + 1, starting the Defense System in that tick (it requires
/// the station, just placed, and the Arsenal); the station scans the Sparrows at [`FIRST_SCAN`], and player 1 sees them
/// for 128 ticks. The Defense System, placed at [`DEFENSE_PLACED`], builds a Missile Turret and then another, placed at
/// [`TURRETS_PLACED`]: the second takes player 1 to 70 generated for 80 consumed. At [`RAID`] the Sparrows
/// AttackMove onto the base; the turrets fire at them as they come in range, at 70/80, banking a share of each tick,
/// until the Deployment Center places a third Generator at [`GENERATOR_PLACED`], 90/80, and then at full speed. Every
/// missile is a homing one, at a Sparrow, and none misses; the three fall, and the turrets stand. The station's
/// cooldown, slowed for 128 ticks, is over at the end of [`COOLDOWN_OVER`], and it scans again at [`SECOND_SCAN`].
pub(crate) fn gdo_defense() -> Game {
    let builder = ScenarioBuilder::new(&open(40, 20))
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 6)),
        )
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(8, 3)))
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(8, 12)))
        .spawn(ObjectKind::Arsenal, Some(P1), SpawnAt::Tile(TilePos::new(2, 12)))
        .spawn(ObjectKind::Sparrow, Some(P2), at(30, 8))
        .spawn(ObjectKind::Sparrow, Some(P2), at(30, 10))
        .spawn(ObjectKind::Sparrow, Some(P2), at(30, 12))
        .spawn(ObjectKind::Airstrip, Some(P2), SpawnAt::Tile(TilePos::new(34, 17)));
    let p1 = |command| PlayerCommand { player: P1, command };
    let bank = p1(Command::Debug(DebugCommand::SetResources {
        player: P1,
        crystals: 700,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }));
    let construction = |by: ObjectId, command| {
        p1(Command::Construction {
            objects: vec![by],
            command,
        })
    };
    let place = |x, y| ConstructionCommand::Place {
        origin: TilePos::new(x, y),
        orientation: BuildingOrientation::default(),
    };
    let construct = ConstructionCommand::Construct;
    let commands = vec![
        (1, bank),
        (1, construction(DC, construct(ObjectKind::RadarStation))),
        (1, order(P2, SPARROWS, OrderKind::Hold)),
        (STATION_READY + 1, construction(DC, place(8, 7))),
        (
            STATION_READY + 1,
            construction(DC, construct(ObjectKind::DefenseSystem)),
        ),
        (FIRST_SCAN, p1(scan(HOLD_AT))),
        (DEFENSE_PLACED, construction(DC, place(11, 8))),
        (DEFENSE_PLACED, construction(DC, construct(ObjectKind::Generator))),
        (
            DEFENSE_PLACED + 1,
            construction(DEFENSE, construct(ObjectKind::MissileTurret)),
        ),
        (TURRETS_PLACED[0], construction(DEFENSE, place(14, 7))),
        (
            TURRETS_PLACED[0],
            construction(DEFENSE, construct(ObjectKind::MissileTurret)),
        ),
        (TURRETS_PLACED[1], construction(DEFENSE, place(14, 11))),
        (RAID, order(P2, SPARROWS, OrderKind::AttackMove(point(12, 10)))),
        (GENERATOR_PLACED, construction(DC, place(5, 12))),
        (SECOND_SCAN, p1(scan(SECOND_AT))),
    ];
    Game {
        scenario: gdo_p1_cults_p2(builder),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                SEEN.set(Seen::default());
            }
            let placed = |id: ObjectId, kind: ObjectKind| {
                let o = sim.object(id).unwrap_or_else(|| panic!("tick {tick}: {kind:?} placed"));
                assert_eq!((o.kind, o.owner), (kind, Some(P1)), "tick {tick}");
            };
            match tick {
                STATION_READY => {
                    let ready = sim.object(DC).and_then(|o| o.constructor).and_then(|c| c.ready());
                    assert_eq!(ready, Some(ObjectKind::RadarStation));
                }
                DEFENSE_PLACED => placed(DEFENSE, ObjectKind::DefenseSystem),
                FIRST_SCAN => {
                    placed(STATION, ObjectKind::RadarStation);
                    let sparrow = sim.object(ObjectId(SPARROWS[1])).unwrap();
                    assert!(sim.is_visible(P1, sparrow), "the scan shows the Sparrows");
                }
                _ => {}
            }
            if tick == TURRETS_PLACED[1] {
                placed(ObjectId(TURRETS[1]), ObjectKind::MissileTurret);
                let power = sim.power(P1);
                assert_eq!((power.generated, power.consumed, power.ratio()), (70, 80, 57_344));
            }
            if tick == GENERATOR_PLACED {
                assert_eq!(sim.power(P1).ratio(), RATIO_ONE, "a third Generator: 90/80");
            }
            let again = PlayerCommand {
                player: P1,
                command: scan(SECOND_AT),
            };
            if tick == COOLDOWN_OVER - 1 {
                assert_eq!(sim.validate(&again), Err(Reject::CoolingDown), "slowed by the Power");
            }
            if tick == COOLDOWN_OVER {
                assert_eq!(sim.validate(&again), Ok(()));
            }
            let turret = |id: ObjectId| TURRETS.contains(&id.0);
            let sparrow = |id: ObjectId| SPARROWS.contains(&id.0);
            SEEN.with_borrow_mut(|seen| {
                for e in sim.events() {
                    match *e {
                        SimEvent::ProjectileLaunched {
                            shooter,
                            target,
                            homing,
                            ..
                        } => {
                            assert!(
                                turret(shooter) && homing && target.is_some_and(sparrow),
                                "tick {tick}: {e:?}"
                            );
                            seen.launched += 1;
                        }
                        SimEvent::Hit { attacker, target, .. } if turret(attacker) && sparrow(target) => seen.hits += 1,
                        SimEvent::Hit { attacker, target, .. } if sparrow(attacker) && turret(target) => {
                            seen.on_turrets += 1;
                        }
                        SimEvent::Missed { .. } => panic!("tick {tick}: {e:?}"),
                        SimEvent::Cast { caster, .. } => {
                            assert_eq!(caster, STATION);
                            seen.casts.push(tick);
                        }
                        _ => {}
                    }
                }
                for n in TURRETS {
                    let carry = sim
                        .object(ObjectId(n))
                        .and_then(|o| o.attack.as_ref())
                        .map_or(0, |a| a.power_carry);
                    match tick < GENERATOR_PLACED {
                        true => seen.banked |= carry > 0,
                        false => assert_eq!(carry, 0, "tick {tick}: full speed"),
                    }
                }
            });
        },
        expect: |sim| {
            for n in SPARROWS {
                assert!(sim.object(ObjectId(n)).is_none(), "Sparrow {n} fell");
            }
            for n in TURRETS {
                assert!(sim.object(ObjectId(n)).is_some(), "turret {n} stands");
            }
            SEEN.with_borrow(|seen| {
                assert_eq!(seen.casts, [FIRST_SCAN, SECOND_SCAN]);
                assert!(seen.hits >= 9, "three missiles a Sparrow: {}", seen.hits);
                assert!(seen.launched >= seen.hits, "{} launched", seen.launched);
                assert!(seen.on_turrets > 0, "the Sparrows strafed the turrets");
                assert!(seen.banked, "the turrets fired at 70/80 for a while");
            });
        },
    }
}
