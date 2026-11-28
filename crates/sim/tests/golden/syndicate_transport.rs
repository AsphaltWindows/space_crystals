//! The Subterranean Transport's golden game (slice M6-ST1, plan decision T94;
//! `factions/syndicate/units/subterranean_transport.md`, design changes 0367's placeholders, 0419 and 0432): the
//! Vehicle Assembly trains a Transport behind a Forge into the Tunnel Network; ejected, it takes four Guards aboard,
//! drives through a line of Peacekeepers, crushing them, and unloads the Guards in a ring where its move ends; two
//! board it again and it drives into the Tunnel with them, loaded, and out again; then six Peacekeepers kill it, and
//! its two passengers die with it. Auto-targeting is off: nobody fights but those ordered to.

use std::cell::RefCell;
use std::collections::BTreeMap;

use sc_sim::network::Place;
use sc_sim::{DebugCommand, SimConfig, TunnelCommand};

use super::*;

/// Player 1's Tunnel (id 1), unturned on tiles (12, 10) to (15, 13): Side A faces north, its exit path's spot at
/// (926, 640). Its Vehicle Assembly (id 2), finished, on (9, 7) to (11, 9), and its Forge (id 3), finished, on (16, 8)
/// to (17, 9), the Transport's TechPrerequisite.
const TUNNEL: ObjectId = ObjectId(1);
const ASSEMBLY: ObjectId = ObjectId(2);
/// Player 1's four Guards, west of the Tunnel's exit path, north of the Vehicle Assembly.
const GUARDS: [ObjectId; 4] = [ObjectId(4), ObjectId(5), ObjectId(6), ObjectId(7)];
const GUARDS_X: [f32; 4] = [560.0, 620.0, 680.0, 740.0];
const GUARDS_Y: f32 = 380.0;
/// Player 2's three Peacekeepers on Hold in a line across the Transport's way east, along y = 300.
const LINE: [ObjectId; 3] = [ObjectId(8), ObjectId(9), ObjectId(10)];
const LINE_X: [f32; 3] = [1300.0, 1380.0, 1460.0];
const LINE_Y: f32 = 300.0;
/// Player 2's six Peacekeepers in a row east of the Tunnel, until they are sent onto the Transport.
const SIX: [ObjectId; 6] = [
    ObjectId(11),
    ObjectId(12),
    ObjectId(13),
    ObjectId(14),
    ObjectId(15),
    ObjectId(16),
];
/// The Transport, trained.
const TRANSPORT: ObjectId = ObjectId(17);
/// The Train is clicked in this tick, and the Transport is finished 400 frames on, inside the network.
const TRAIN: Tick = 2;
const DONE: Tick = TRAIN + 399;
/// It is ejected in the tick after, and the Guards are sent onto it once it is out.
const EJECT: Tick = DONE + 1;
const BOARD: Tick = EJECT + 8;
/// It drives to the line's west end, then through the line to [`UNLOAD_AT`], an Unload All queued after the move.
const GO: Tick = 500;
const LINE_START: Vec2 = Vec2::new(1100.0, LINE_Y);
const THROUGH: Tick = 600;
const UNLOAD_AT: Vec2 = Vec2::new(1700.0, LINE_Y);
/// Two of the Guards board it again, and it drives into the Tunnel with them.
const REBOARD: Tick = 800;
const BACK: Tick = 860;
/// It is ejected again, loaded, its transit delay over (it went in at 1112), and the six Peacekeepers are sent onto it.
const OUT_AGAIN: Tick = 1180;
const ATTACK: Tick = 1200;
/// It dies in this tick, as the game played when it was made; the game ends 162 ticks after.
const DIES: Tick = 1338;
const END: Tick = DIES + 162;
/// The Tunnel Space each takes (`> Placeholder stats`: "TunnelSpaceCost 3"; the Guard's 1).
const TRANSPORT_SPACE: u32 = 3;

/// The game's events of interest so far.
#[derive(Default)]
struct Seen {
    /// The ticks each Guard boarded, in order.
    boarded: Vec<(Tick, ObjectId)>,
    /// The ticks each came out, in order.
    unloaded: Vec<(Tick, ObjectId)>,
    /// The Peacekeepers the Transport crushed, with the ticks.
    crushed: Vec<(Tick, ObjectId)>,
    /// The ticks the Transport went into the network and came out of it.
    entered: Vec<Tick>,
    left: Vec<Tick>,
    /// The ticks it was held in the network with passengers aboard.
    inside_loaded: usize,
    /// The deaths of the Transport and of player 1's Guards, with their ticks and where.
    died: Vec<(Tick, ObjectId, Vec2)>,
    /// Player 1's Tunnel Space in use, by tick, as it changed.
    space: BTreeMap<Tick, u32>,
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

fn queued(player: PlayerId, units: impl IntoIterator<Item = u32>, kind: OrderKind) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: units.into_iter().map(ObjectId).collect(),
            kind,
            queue: true,
        },
    }
}

/// Player 1 (the Syndicate) has a Tunnel, set to Tier 2 and given 200 Space Crystals by debug commands in tick 1 (its
/// upgrades and its mining aren't what the game is about), a Vehicle Assembly and a Forge, and four Guards; player 2
/// (the GDO), three Peacekeepers holding in a line and six more east of the Tunnel. At [`TRAIN`] the Vehicle Assembly
/// trains a Subterranean Transport (200, 400 frames), finished at [`DONE`] inside the network and ejected at [`EJECT`]
/// out of Side A's exit path. The four Guards, sent onto it at [`BOARD`], board it one by one. At [`GO`] it drives to
/// the line's west end, and at [`THROUGH`] through the line, crushing each Peacekeeper as it reaches it, to
/// [`UNLOAD_AT`], where its queued Unload All puts the four out round it in one tick. At [`REBOARD`] two of them board
/// it again, and at [`BACK`] it drives into the Tunnel with them, held in its carrier there, the EjectMenu listing it
/// alone, and is ejected at [`OUT_AGAIN`], still loaded. At [`ATTACK`] the six Peacekeepers attack it; it dies at
/// [`DIES`], and its two passengers die with it, in its tick, where it was. Player 1's Tunnel Space in use is the
/// Guards' 4, then 7 from the Train on, the Transport's 3 reserved, whoever is aboard and wherever, until it dies: then
/// the two Guards left on the map's 2.
pub(crate) fn syndicate_transport() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(ObjectKind::VehicleAssembly, Some(P1), SpawnAt::Tile(TilePos::new(9, 7)))
        .spawn(ObjectKind::Forge, Some(P1), SpawnAt::Tile(TilePos::new(16, 8)));
    for x in GUARDS_X {
        builder = builder.spawn(GUARD, Some(P1), SpawnAt::Point(Vec2::new(x, GUARDS_Y)));
    }
    for x in LINE_X {
        builder = builder.spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(x, LINE_Y)));
    }
    for k in 0..6 {
        let at = Vec2::new(1100.0 + 50.0 * k as f32, 700.0);
        builder = builder.spawn(PK, Some(P2), SpawnAt::Point(at));
    }
    let bank = player_debug(DebugCommand::SetResources {
        player: P1,
        crystals: 200,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    });
    let eject = || PlayerCommand {
        player: P1,
        command: Command::Tunnel {
            tunnel: TUNNEL,
            command: TunnelCommand::Eject {
                kind: ObjectKind::SubterraneanTransport,
            },
        },
    };
    let train = PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![ASSEMBLY],
            command: ProductionCommand::Train(ObjectKind::SubterraneanTransport),
        },
    };
    let move_to = |at: Vec2| OrderKind::Move(Target::Point(at));
    let commands = vec![
        (1, player_debug(DebugCommand::SetAutoTargeting { on: false })),
        (1, player_debug(DebugCommand::SetTier { id: TUNNEL, tier: 2 })),
        (1, bank),
        (1, order(P2, LINE.map(|id| id.0), OrderKind::Hold)),
        (TRAIN, train),
        (EJECT, eject()),
        (BOARD, order(P1, GUARDS.map(|id| id.0), OrderKind::Enter(TRANSPORT))),
        (GO, order(P1, [TRANSPORT.0], move_to(LINE_START))),
        (THROUGH, order(P1, [TRANSPORT.0], move_to(UNLOAD_AT))),
        (THROUGH, queued(P1, [TRANSPORT.0], OrderKind::UnloadAll)),
        (
            REBOARD,
            order(P1, [GUARDS[0].0, GUARDS[1].0], OrderKind::Enter(TRANSPORT)),
        ),
        (BACK, order(P1, [TRANSPORT.0], OrderKind::Enter(TUNNEL))),
        (OUT_AGAIN, eject()),
        (ATTACK, order(P2, SIX.map(|id| id.0), OrderKind::Attack(TRANSPORT))),
    ];
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                SEEN.set(Seen::default());
            }
            SEEN.with_borrow_mut(|seen| during(seen, tick, sim));
        },
        expect: |sim| {
            SEEN.with_borrow(|seen| {
                let boarded: Vec<ObjectId> = seen.boarded.iter().map(|&(_, unit)| unit).collect();
                assert_eq!(boarded.len(), 6, "{:?}", seen.boarded);
                let mut first: Vec<ObjectId> = boarded[..4].to_vec();
                first.sort_unstable();
                assert_eq!(first, GUARDS, "all four first");
                assert!(seen.boarded[..4].iter().all(|&(tick, _)| (BOARD..GO).contains(&tick)));
                let mut again: Vec<ObjectId> = boarded[4..].to_vec();
                again.sort_unstable();
                assert_eq!(again, GUARDS[..2], "two again");
                let crushed: Vec<ObjectId> = seen.crushed.iter().map(|&(_, unit)| unit).collect();
                assert_eq!(crushed, LINE, "the line, in its order");
                assert_eq!(seen.unloaded.len(), 4, "{:?}", seen.unloaded);
                let out = seen.unloaded[0].0;
                assert!(seen.unloaded.iter().all(|&(tick, _)| tick == out), "in one tick");
                assert!((THROUGH..REBOARD).contains(&out), "where its move ended: {out}");
                assert_eq!(seen.entered.len(), 1, "back in once");
                assert_eq!(seen.left.len(), 2, "ejected twice: {:?}", seen.left);
                assert!(seen.inside_loaded > 30, "{} ticks inside loaded", seen.inside_loaded);
                let [(fell, first_died, at), passengers @ ..] = &seen.died[..] else {
                    panic!("deaths: {:?}", seen.died);
                };
                assert_eq!((*fell, *first_died), (DIES, TRANSPORT), "{:?}", seen.died);
                let lost: Vec<(Tick, ObjectId, Vec2)> = GUARDS[..2].iter().map(|&g| (*fell, g, *at)).collect();
                assert_eq!(passengers, lost, "its two passengers with it");
                let space: Vec<u32> = seen.space.values().copied().collect();
                assert_eq!(space, [4, 4 + TRANSPORT_SPACE, 2], "{:?}", seen.space);
            });
            assert!(sim.object(TRANSPORT).is_none() && sim.held_unit(TRANSPORT).is_none());
            for guard in GUARDS[2..].iter() {
                assert!(sim.object(*guard).is_some(), "{guard:?} on the map");
            }
            assert!(sim.held().next().is_none(), "no one held");
        },
    }
}

/// What the game checks after each tick: who boarded, came out, was crushed and died, the Transport's ins and outs of
/// the network, its passengers held aboard it there, and player 1's Tunnel Space (the game's doc comment).
fn during(seen: &mut Seen, tick: Tick, sim: &Sim) {
    for e in sim.events() {
        match *e {
            SimEvent::Boarded { unit, carrier } => {
                assert_eq!(carrier, TRANSPORT, "tick {tick}");
                seen.boarded.push((tick, unit));
            }
            SimEvent::Unloaded { unit, carrier } => {
                assert_eq!(carrier, TRANSPORT, "tick {tick}");
                let o = sim.object(unit).expect("out");
                let near = o.pos.distance(sim.object(TRANSPORT).expect("the Transport").pos);
                assert!(near < 100.0, "tick {tick}: {unit:?} round it, {near} su off");
                seen.unloaded.push((tick, unit));
            }
            SimEvent::Crushed { id, crusher, .. } => {
                assert_eq!(crusher, TRANSPORT, "tick {tick}");
                seen.crushed.push((tick, id));
            }
            SimEvent::EnteredNetwork { unit, .. } => {
                assert_eq!(unit, TRANSPORT, "tick {tick}");
                seen.entered.push(tick);
            }
            SimEvent::LeftNetwork { unit, .. } => {
                assert_eq!(unit, TRANSPORT, "tick {tick}");
                seen.left.push(tick);
            }
            SimEvent::Died { id, owner, pos, .. } if owner == Some(P1) => {
                seen.died.push((tick, id, pos));
            }
            _ => {}
        }
    }
    let held_loaded = sim
        .held_unit(TRANSPORT)
        .is_some_and(|held| held.place == Place::Network)
        && sim.passengers(TRANSPORT).len() == 2;
    if held_loaded {
        seen.inside_loaded += 1;
        let inside: Vec<ObjectId> = sim.network(P1).map(|held| held.object.id).collect();
        assert_eq!(inside, [TRANSPORT], "tick {tick}: the EjectMenu lists it alone");
    }
    let used = sim.army_cap(P1).expect("a Syndicate player").used;
    if seen.space.values().next_back() != Some(&used) {
        seen.space.insert(tick, used);
    }
}
