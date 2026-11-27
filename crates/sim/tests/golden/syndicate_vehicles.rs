//! The Vehicle Assembly's golden game (slice M6-SV, `factions/syndicate/structures/vehicle_assembly.md`,
//! `factions/syndicate/units/assault_bike.md`): a Tier 2 Tunnel builds a Vehicle Assembly behind a Chemical Plant,
//! which trains three Assault Bikes into the Tunnel Network: the first ejected by hand, the other two auto-ejected by a
//! surface rally point, each out of Side A's exit path; then they shell a group of four Guards on Hold with their
//! mortars, whose blasts hit the group by the area rule.

use std::cell::RefCell;

use sc_sim::{BuildingOrientation, DebugCommand, SimConfig, TunnelCommand, math, network::Paths};

use super::*;

/// Player 1's Tunnel (id 1), unturned on tiles (12, 10) to (15, 13): Side A faces north, its exit path's spot at
/// (926, 640). Its Chemical Plant (id 2), on (16, 14), the Vehicle Assembly's TechPrerequisite.
const TUNNEL: ObjectId = ObjectId(1);
const PLANT: ObjectId = ObjectId(2);
/// Player 2's four Guards, on Hold in a block in the north-east, 4 su apart.
const GUARDS: [ObjectId; 4] = [ObjectId(3), ObjectId(4), ObjectId(5), ObjectId(6)];
const GUARDS_AT: Vec2 = Vec2::new(1984.0, 224.0);
/// The Vehicle Assembly (id 7), which the Tunnel expands on tiles (9, 7) to (11, 9), in its Tier 2 area.
const ASSEMBLY: ObjectId = ObjectId(7);
const ASSEMBLY_AT: TilePos = TilePos::new(9, 7);
/// The three Bikes, in the order they are finished.
const BIKES: [ObjectId; 3] = [ObjectId(8), ObjectId(9), ObjectId(10)];
/// The Expand is clicked in this tick, and the Assembly completes 480 frames later.
const EXPAND: Tick = 2;
const BUILT: Tick = EXPAND + 480;
/// The three Bikes are queued in the tick after, with no rally point: each takes 240 frames, the first finished at
/// [`DONE`]`[0]`, inside the network.
const TRAIN: Tick = BUILT + 1;
const DONE: [Tick; 3] = [TRAIN + 239, TRAIN + 479, TRAIN + 719];
/// In the tick after the first is finished, it is ejected and comes out, and the rally point is set on the surface,
/// north of the Tunnel: the other two come out of the Tunnel in the tick each is finished.
const EJECT: Tick = DONE[0] + 1;
const RALLY: Vec2 = Vec2::new(928.0, 288.0);
/// The first is sent on to the rally point from where the EjectMenu left it, 1 grid unit out.
const SEND_ON: Tick = EJECT + 8;
/// The three AttackMove onto the Guards from the rally point.
const ATTACK: Tick = DONE[2] + 60;
/// The first round lands in this tick, as the game played when it was made, and hits all four Guards.
const FIRST_BLAST: Tick = 1375;
/// The game's last tick: the last two Guards fell at 1:32.25 (tick 1476).
const END: Tick = 1500;

/// The game's events of interest so far: the Bikes' exits, and the ticks with blasts, with how many landed and how
/// many hits on Guards they made.
#[derive(Default)]
struct Seen {
    exits: Vec<(Tick, ObjectId)>,
    blasts: Vec<(Tick, usize, usize)>,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
}

fn player_debug(command: DebugCommand) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Debug(command),
    }
}

/// Player 1 (the Syndicate) has a Tunnel, set to Tier 2 and given 625 Space Crystals by debug commands in tick 1 (its
/// upgrades and its mining aren't what the game is about), and a Chemical Plant; player 2 (the Syndicate), four Guards
/// on Hold far in the north-east. At [`EXPAND`] the Tunnel expands a Vehicle Assembly (250, 480 frames), complete at
/// [`BUILT`]; it queues three Assault Bikes (125, 240 frames each) at [`TRAIN`]. The first, finished with no rally
/// point, stays in the network until it is ejected at [`EJECT`] onto the exit path's spot, facing north, from where it
/// drives 1 grid unit out, and on to [`RALLY`]; the rally point set then on the surface sends the other two out of the
/// Tunnel in the tick each is finished, and they drive there. At [`ATTACK`] the three AttackMove onto the Guards, out
/// of their sight until then: they shell them from beyond the Guards' range of 3, and the blasts, landing on one Guard,
/// hit others of the block too, until all four are dead. No Guard's shot reaches a Bike.
pub(crate) fn syndicate_vehicles() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(ObjectKind::ChemicalPlant, Some(P1), SpawnAt::Tile(TilePos::new(16, 14)));
    for (dx, dy) in [(-20.0, -20.0), (20.0, -20.0), (-20.0, 20.0), (20.0, 20.0)] {
        builder = builder.spawn(GUARD, Some(P2), SpawnAt::Point(GUARDS_AT + Vec2::new(dx, dy)));
    }
    let bank = player_debug(DebugCommand::SetResources {
        player: P1,
        crystals: 250 + 3 * 125,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    });
    let tier = player_debug(DebugCommand::SetTier { id: TUNNEL, tier: 2 });
    let tunnel = |command: TunnelCommand| PlayerCommand {
        player: P1,
        command: Command::Tunnel {
            tunnel: TUNNEL,
            command,
        },
    };
    let expand = tunnel(TunnelCommand::Expand {
        kind: ObjectKind::VehicleAssembly,
        origin: ASSEMBLY_AT,
        orientation: BuildingOrientation::default(),
    });
    let production = |command: ProductionCommand| PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![ASSEMBLY],
            command,
        },
    };
    let mut commands = vec![
        (1, bank),
        (1, tier),
        (1, order(P2, GUARDS.map(|id| id.0), OrderKind::Hold)),
        (EXPAND, expand),
    ];
    for _ in BIKES {
        commands.push((TRAIN, production(ProductionCommand::Train(ObjectKind::AssaultBike))));
    }
    commands.extend([
        (
            EJECT,
            tunnel(TunnelCommand::Eject {
                kind: ObjectKind::AssaultBike,
            }),
        ),
        (EJECT, production(ProductionCommand::SetRally(Target::Point(RALLY)))),
        (SEND_ON, order(P1, [BIKES[0].0], OrderKind::Move(Target::Point(RALLY)))),
        (
            ATTACK,
            order(P1, BIKES.map(|id| id.0), OrderKind::AttackMove(GUARDS_AT)),
        ),
    ]);
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                SEEN.set(Seen::default());
            }
            let paths = Paths::of_object(sim.object(TUNNEL).expect("the Tunnel")).unwrap();
            for e in sim.events() {
                match *e {
                    SimEvent::ExpansionStarted { tunnel, site } => {
                        assert_eq!((tick, tunnel, site), (EXPAND, TUNNEL, ASSEMBLY));
                    }
                    SimEvent::ExpansionCompleted { tunnel, site } => {
                        assert_eq!((tick, tunnel, site), (BUILT, TUNNEL, ASSEMBLY));
                    }
                    SimEvent::UnitProduced { producer, unit } => {
                        let n = DONE.iter().position(|&done| done == tick).expect("a Bike's tick");
                        assert_eq!((producer, unit), (ASSEMBLY, BIKES[n]), "tick {tick}");
                        assert_eq!(sim.object(unit).is_none(), n == 0, "the first inside, the others out");
                    }
                    SimEvent::LeftNetwork { unit, tunnel } => {
                        assert_eq!(tunnel, TUNNEL);
                        let o = sim.object(unit).expect("out");
                        assert_eq!(o.pos, paths.exit + paths.outward * (28.0 + 1.0 / 64.0), "tick {tick}");
                        assert!(
                            math::facing_vector(o.facing).distance(paths.outward) < 1e-6,
                            "facing out"
                        );
                        SEEN.with_borrow_mut(|seen| seen.exits.push((tick, unit)));
                    }
                    SimEvent::Blast { attacker, .. } => {
                        assert!(attacker.is_some_and(|a| BIKES.contains(&a)), "tick {tick}");
                    }
                    SimEvent::Hit { attacker, target, .. } => {
                        assert!(
                            BIKES.contains(&attacker) && GUARDS.contains(&target),
                            "tick {tick}: {e:?}"
                        );
                    }
                    _ => {}
                }
            }
            let blasts = (sim.events().iter())
                .filter(|e| matches!(e, SimEvent::Blast { .. }))
                .count();
            let hits = (sim.events().iter())
                .filter(|e| matches!(e, SimEvent::Hit { .. }))
                .count();
            if blasts > 0 {
                SEEN.with_borrow_mut(|seen| seen.blasts.push((tick, blasts, hits)));
            }
        },
        expect: |sim| {
            SEEN.with_borrow(|seen| {
                assert_eq!(
                    seen.exits,
                    [(EJECT, BIKES[0]), (DONE[1], BIKES[1]), (DONE[2], BIKES[2])]
                );
                assert_eq!(
                    seen.blasts[0],
                    (FIRST_BLAST, 1, 4),
                    "one round on the block hits all four Guards"
                );
                assert!(seen.blasts.len() >= 4, "{:?}", seen.blasts);
            });
            assert!(
                GUARDS.iter().all(|&guard| sim.object(guard).is_none()),
                "the Guards are dead"
            );
            for bike in BIKES {
                assert_eq!(sim.object(bike).and_then(|o| o.hp), Some(100.0), "{bike:?} untouched");
            }
            let assembly = sim.object(ASSEMBLY).expect("the Vehicle Assembly");
            assert_eq!((assembly.hp, sim.parent_tunnel(ASSEMBLY)), (Some(600.0), Some(TUNNEL)));
            assert!(sim.object(PLANT).is_some());
            assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "all spent");
        },
    }
}
