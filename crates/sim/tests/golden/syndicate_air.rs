//! The air doors' golden game (slice M6-SD, plan decision T96; `factions/syndicate/structures/forge.md`,
//! `factions/syndicate/structures/aerospace_assembly.md`, `factions/syndicate/units/phantom.md`, design change 0431): a
//! Tier 3 Tunnel builds a Forge behind its Vehicle Assembly while its Aerospace Assembly trains three Phantoms into the
//! Tunnel Network: two ejected by hand out of the air exit door, 8 frames apart and rising at once, the third
//! auto-ejected by a surface rally point; they fight two Peacekeepers, which can't shoot at them, and a Sparrow, kill
//! all three, and fly back in by the air entry door, descending one after the other.

use std::cell::RefCell;

use sc_sim::{BuildingOrientation, DebugCommand, SimConfig, TunnelCommand, network, network::Paths};

use super::*;

/// Player 1's Tunnel (id 1), unturned on tiles (12, 10) to (15, 13): Side A faces north, its air exit door's centre at
/// (948, 768) and its air entry door's at (844, 768). Its Optics Lab (id 2) on (16, 14), the Phantom's
/// TechPrerequisite; its Vehicle Assembly (id 3) on (9, 7), the Forge's; its Aerospace Assembly (id 4) on (8, 14), in
/// the Tier 3 area.
const TUNNEL: ObjectId = ObjectId(1);
const ASSEMBLY: ObjectId = ObjectId(4);
/// Player 2's two Peacekeepers on Hold in the north-east, and its Sparrow circling south of them.
const PEACEKEEPERS: [ObjectId; 2] = [ObjectId(5), ObjectId(6)];
const SPARROW: ObjectId = ObjectId(7);
const GROUP_AT: Vec2 = Vec2::new(1984.0, 224.0);
/// The Forge (id 8), which the Tunnel expands on tiles (16, 8) to (17, 9), complete 480 frames after the click.
const FORGE: ObjectId = ObjectId(8);
const FORGE_AT: TilePos = TilePos::new(16, 8);
const EXPAND: Tick = 2;
const BUILT: Tick = EXPAND + 480;
/// The three Phantoms, in the order they are finished: queued in the Expand's tick, with no rally point, 320 frames
/// each.
const PHANTOMS: [ObjectId; 3] = [ObjectId(9), ObjectId(10), ObjectId(11)];
const DONE: [Tick; 3] = [EXPAND + 319, EXPAND + 639, EXPAND + 959];
/// In the tick after the second is finished, both are ejected and come out of the air exit door 8 frames apart, and
/// the rally point is set on the surface, north of the Tunnel: the third comes out in the tick it is finished.
const EJECT: Tick = DONE[1] + 1;
const RALLY: Vec2 = Vec2::new(948.0, 256.0);
/// The first two are sent on to the rally point once risen.
const SEND_ON: Tick = EJECT + 30;
/// The three AttackMove onto player 2's group from the rally point, and right-click the Tunnel once it is dead.
const ATTACK: Tick = DONE[2] + 60;
const HOME: Tick = 1300;
/// The game's last tick.
const END: Tick = 1500;

/// The game's events of interest so far.
#[derive(Default)]
struct Seen {
    exits: Vec<(Tick, ObjectId)>,
    entries: Vec<(Tick, ObjectId)>,
    /// The most Phantoms rising out of the exit door at once, and descending into the entry door at once.
    rising: usize,
    descending: usize,
    /// The ticks of the descents' starts, by Phantom.
    descents: Vec<(Tick, ObjectId)>,
    /// The Phantoms' hits on the Peacekeepers and on the Sparrow, and every other hit.
    on_ground: usize,
    in_air: usize,
    others: Vec<(Tick, ObjectId, ObjectId)>,
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

/// Player 1 (the Syndicate) has a Tunnel, set to Tier 3 and given 725 Space Crystals by debug commands in tick 1 (its
/// upgrades and its mining aren't what the game is about), an Optics Lab, a Vehicle Assembly and an Aerospace
/// Assembly; player 2 (the GDO), two Peacekeepers on Hold far in the north-east and a Cults Sparrow circling beside
/// them. At [`EXPAND`] the Tunnel expands a Forge (200, 480 frames), complete at [`BUILT`], and the Aerospace Assembly
/// queues three Phantoms (175, 320 frames each). The first two, finished with no rally point, stay in the network
/// until both are ejected at [`EJECT`]: they come out over the air exit door 8 frames apart, each rising there for 16
/// frames, the two at once for 8. The rally point set then on the surface sends the third out of the door in the tick
/// it is finished, and once risen it flies there, as the first two do when sent on. At [`ATTACK`] the three AttackMove
/// onto player 2's group: their lasers, Universal, kill the Sparrow and the Peacekeepers, whose attacks are Ground
/// only; the Sparrow's machine gun, Universal too, lands the only hits on them. At [`HOME`] they right-click the
/// Tunnel: each flies to the air entry door and descends into it for 16 frames, the door letting one down every 8 at
/// most, two descending at once, and all three end inside.
pub(crate) fn syndicate_air() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(ObjectKind::OpticsLab, Some(P1), SpawnAt::Tile(TilePos::new(16, 14)))
        .spawn(ObjectKind::VehicleAssembly, Some(P1), SpawnAt::Tile(TilePos::new(9, 7)))
        .spawn(
            ObjectKind::AerospaceAssembly,
            Some(P1),
            SpawnAt::Tile(TilePos::new(8, 14)),
        );
    for dx in [-20.0, 20.0] {
        builder = builder.spawn(PK, Some(P2), SpawnAt::Point(GROUP_AT + Vec2::new(dx, 0.0)));
    }
    builder = builder.spawn(
        ObjectKind::Sparrow,
        Some(P2),
        SpawnAt::Point(GROUP_AT + Vec2::new(0.0, 160.0)),
    );
    let bank = player_debug(DebugCommand::SetResources {
        player: P1,
        crystals: 200 + 3 * 175,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    });
    let tier = player_debug(DebugCommand::SetTier { id: TUNNEL, tier: 3 });
    let tunnel = |command: TunnelCommand| PlayerCommand {
        player: P1,
        command: Command::Tunnel {
            tunnel: TUNNEL,
            command,
        },
    };
    let expand = tunnel(TunnelCommand::Expand {
        kind: ObjectKind::Forge,
        origin: FORGE_AT,
        orientation: BuildingOrientation::default(),
    });
    let production = |command: ProductionCommand| PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![ASSEMBLY],
            command,
        },
    };
    let eject = || {
        tunnel(TunnelCommand::Eject {
            kind: ObjectKind::Phantom,
        })
    };
    let mut commands = vec![
        (1, bank),
        (1, tier),
        (1, order(P2, PEACEKEEPERS.map(|id| id.0), OrderKind::Hold)),
        (EXPAND, expand),
    ];
    for _ in PHANTOMS {
        commands.push((EXPAND, production(ProductionCommand::Train(ObjectKind::Phantom))));
    }
    let phantoms = || PHANTOMS.map(|id| id.0);
    commands.extend([
        (EJECT, eject()),
        (EJECT, eject()),
        (EJECT, production(ProductionCommand::SetRally(Target::Point(RALLY)))),
        (
            SEND_ON,
            order(
                P1,
                [PHANTOMS[0].0, PHANTOMS[1].0],
                OrderKind::Move(Target::Point(RALLY)),
            ),
        ),
        (ATTACK, order(P1, phantoms(), OrderKind::AttackMove(GROUP_AT))),
        (HOME, order(P1, phantoms(), OrderKind::Enter(TUNNEL))),
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
                        assert_eq!((tick, tunnel, site), (EXPAND, TUNNEL, FORGE));
                    }
                    SimEvent::ExpansionCompleted { tunnel, site } => {
                        assert_eq!((tick, tunnel, site), (BUILT, TUNNEL, FORGE));
                    }
                    SimEvent::UnitProduced { producer, unit } => {
                        let n = DONE.iter().position(|&done| done == tick).expect("a Phantom's tick");
                        assert_eq!((producer, unit), (ASSEMBLY, PHANTOMS[n]), "tick {tick}");
                    }
                    SimEvent::LeftNetwork { unit, tunnel } => {
                        assert_eq!(tunnel, TUNNEL);
                        let o = sim.object(unit).expect("out");
                        assert_eq!(o.pos, paths.air_exit, "tick {tick}: over the air exit door");
                        let rise = o.mover.as_ref().and_then(|mover| mover.rise);
                        assert_eq!(rise.map(|rise| rise.ticks_done()), Some(0), "tick {tick}: rising");
                        SEEN.with_borrow_mut(|seen| seen.exits.push((tick, unit)));
                    }
                    SimEvent::EnteredNetwork { unit, tunnel } => {
                        assert_eq!(tunnel, TUNNEL);
                        SEEN.with_borrow_mut(|seen| seen.entries.push((tick, unit)));
                    }
                    SimEvent::Hit { attacker, target, .. } => {
                        SEEN.with_borrow_mut(|seen| match (PHANTOMS.contains(&attacker), target) {
                            (true, target) if PEACEKEEPERS.contains(&target) => seen.on_ground += 1,
                            (true, SPARROW) => seen.in_air += 1,
                            _ => seen.others.push((tick, attacker, target)),
                        })
                    }
                    _ => {}
                }
            }
            let rising = (sim.objects())
                .filter(|o| o.mover.as_ref().is_some_and(|mover| mover.rise.is_some()))
                .count();
            let descending: Vec<ObjectId> = (sim.objects())
                .filter(|o| {
                    PHANTOMS.contains(&o.id)
                        && network::door_lift(o).is_some()
                        && o.mover.as_ref().is_some_and(|m| m.rise.is_none())
                })
                .map(|o| o.id)
                .collect();
            SEEN.with_borrow_mut(|seen| {
                seen.rising = seen.rising.max(rising);
                seen.descending = seen.descending.max(descending.len());
                for unit in descending {
                    if !seen.descents.iter().any(|&(_, seen)| seen == unit) {
                        seen.descents.push((tick, unit));
                    }
                }
            });
        },
        expect: |sim| {
            SEEN.with_borrow(|seen| {
                assert_eq!(
                    seen.exits,
                    [(EJECT, PHANTOMS[0]), (EJECT + 8, PHANTOMS[1]), (DONE[2], PHANTOMS[2])]
                );
                assert_eq!(seen.rising, 2, "two rising at once");
                let sparrows = |&(_, by, on): &(Tick, ObjectId, ObjectId)| by == SPARROW && PHANTOMS.contains(&on);
                assert!(
                    seen.others.iter().all(sparrows),
                    "only the Sparrow hits back: {:?}",
                    seen.others
                );
                assert_eq!(seen.descending, 2, "two descending at once");
                assert!(
                    seen.on_ground >= 2 && seen.in_air >= 1,
                    "{} on the ground, {} in the air",
                    seen.on_ground,
                    seen.in_air
                );
                assert_eq!(seen.entries.len(), 3, "{:?}", seen.entries);
                assert_eq!(seen.descents.len(), 3, "{:?}", seen.descents);
                for pair in seen.descents.windows(2) {
                    assert!(pair[1].0 >= pair[0].0 + 8, "the entry door's rate: {:?}", seen.descents);
                }
                for &(began, unit) in &seen.descents {
                    assert!(
                        seen.entries.contains(&(began + 16, unit)),
                        "16 frames down: {:?}",
                        seen.entries
                    );
                }
            });
            assert!(
                PEACEKEEPERS
                    .iter()
                    .chain([&SPARROW])
                    .all(|&id| sim.object(id).is_none()),
                "player 2's group is dead"
            );
            let inside: Vec<ObjectId> = sim.network(P1).map(|held| held.object.id).collect();
            assert_eq!(inside, PHANTOMS, "all three back inside");
            let forge = sim.object(FORGE).expect("the Forge");
            assert_eq!(
                (forge.kind, forge.hp, sim.parent_tunnel(FORGE)),
                (ObjectKind::Forge, Some(400.0), Some(TUNNEL))
            );
            assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "all spent");
        },
    }
}
