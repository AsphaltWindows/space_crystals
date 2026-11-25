//! `sc_sim::testing::syndicate`'s line (slice M4-L) beyond what its golden games play, through the gate, with both of
//! the player's banks balanced every tick: an upgrade (`Syndicate::upgrade`, `TunnelCommand::Upgrade`, slice M4-G)
//! paid by the Supplies its Agent gathers, on `dev_skirmish` from its real start; and its builders' eject
//! (`Syndicate::eject_builders`), a Tunnel's only, beside a Laser Turret's Agent that comes out by itself.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::testing::build_order::{Books, BuildOrder, Order};
use sc_sim::testing::syndicate::skirmish::{self, AGENTS, BUILDER, PATCH, STATION, SYNDICATE, TUNNEL_SITE, TURNED};
use sc_sim::testing::syndicate::{Done, Job, SupplyBooks, Syndicate};
use sc_sim::tunnel::Resource;
use sc_sim::{
    BuildingOrientation, Command, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Sim, SimEvent,
    SpawnAt, Target, Tick, TickInputs, TilePos,
};

/// An upgrade's frames (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "**480 frames (30
/// seconds)**", design change 0229).
const FRAMES: Tick = 480;

/// The skirmish's opening line without the Optics Lab, its starting Tunnel upgrading once the Supplies cover it: the
/// starting Supply builds the second Tunnel, and its builder's first two Supplies pay for the upgrade, "2 + 2 x
/// (number of T2+ Tunnels owned)" = 2 (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`). The starting
/// Tunnel's Tier 2 area, (3, 3) to (12, 12), lies on the map, on Plane, clear of the second's.
#[test]
fn the_line_upgrades_its_starting_tunnel_with_the_supplies_its_agent_gathers() {
    let line = BuildOrder::new(SYNDICATE).track(vec![Order::new(ObjectKind::Hideout, ObjectKind::Agent); AGENTS]);
    let mut line = Syndicate::new(line)
        .gather(PATCH)
        .rally(Target::Object(PATCH))
        .job(BUILDER, Job::Tunnel(TUNNEL_SITE, TURNED))
        .eject_builders(Job::Supplies(STATION))
        .upgrade(0);
    let mut sim = Sim::new(&skirmish::scenario());
    let first = (sim.objects())
        .find(|o| o.owner == Some(SYNDICATE) && o.kind == ObjectKind::Tunnel)
        .map(|o| o.id)
        .expect("the starting Tunnel");
    let (mut supplied, mut started, mut completed) = (Vec::new(), None, None);
    while completed.is_none() && sim.tick() < 4 * 960 {
        let tick = sim.tick() + 1;
        let commands = line.commands(&sim);
        for c in &commands {
            assert_eq!(sim.validate(c), Ok(()), "tick {tick}: {c:?}");
        }
        let books = Books::open(&sim, SYNDICATE);
        let supplies = SupplyBooks::open(&sim, SYNDICATE);
        sim.step(&TickInputs { commands });
        books.close(&sim, line.spent_in(tick));
        supplies.close(&sim, line.supplies_spent_in(tick));
        for e in sim.events() {
            match *e {
                SimEvent::ResourcesDelivered {
                    resource: Resource::Supplies,
                    ..
                } => supplied.push(tick),
                SimEvent::UpgradeStarted { tunnel, to, price } => started = Some((tick, tunnel, to, price)),
                SimEvent::UpgradeCompleted { tunnel, tier } => completed = Some((tick, tunnel, tier)),
                _ => {}
            }
        }
    }
    let upgrades: Vec<(Tick, Done)> = (line.done().iter())
        .filter(|(_, done)| matches!(done, Done::Upgrade { .. }))
        .copied()
        .collect();
    assert!(supplied.len() >= 2, "the builder delivered two Supplies: {supplied:?}");
    let ordered = supplied[1] + 1;
    assert_eq!(
        upgrades,
        [(
            ordered,
            Done::Upgrade {
                tunnel: first,
                price: 2
            }
        )],
        "the line upgrades the starting Tunnel in the tick after the second Supply, for 2"
    );
    assert_eq!(
        started,
        Some((ordered, first, 2, 2)),
        "the upgrade to Tier 2 starts then"
    );
    assert_eq!(
        completed,
        Some((ordered + FRAMES - 1, first, 2)),
        "and completes 480 frames later"
    );
    let tunnel = sim.object(first).expect("the starting Tunnel");
    assert_eq!(tunnel.max_hp(), Some(800.0), "Tier 2: HP 800");
    let left = u32::try_from(supplied.len()).unwrap() - 2;
    assert_eq!(
        sim.player(SYNDICATE).map(|p| p.supplies),
        Some(left),
        "the starting Supply paid for the second Tunnel, two of those delivered for the upgrade"
    );
}

/// `Syndicate::eject_builders` takes only a Tunnel's builder: an Agent that finishes a Laser Turret comes out beside it
/// by itself (`factions/syndicate/structures/laser_turret.md > Construction`, design change 0298 point 7), so the line
/// orders no Eject by the turret, which the gate would refuse (`Reject::NotATunnel`), and gives its Agent no job. On a
/// flat 40 × 20 map, player 1's Tunnel (1) on tiles (4, 8) to (7, 11), an Optics Lab (2) in its area, two Agents (3
/// and 4) south-east of it and a crystal patch (5) north of them, with the Syndicate's starting 100 Space Crystals and
/// 1 Supply: the line's first Agent builds a Tunnel east of the first, for the Supply, and gathers the patch once
/// ejected; in the same tick player 1 orders the second to build a turret, for the Space Crystals.
#[test]
fn the_line_ejects_a_tunnels_builder_and_leaves_a_turrets_be() {
    const P1: PlayerId = PlayerId(1);
    const TUNNEL_BUILDER: ObjectId = ObjectId(3);
    const TURRET_BUILDER: ObjectId = ObjectId(4);
    const CRYSTALS: ObjectId = ObjectId(5);
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let point = |x, y| SpawnAt::Point(TilePos::new(x, y).center());
    let scenario = ScenarioBuilder::new(&vec![".".repeat(40); 20].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(PlayerId(2), Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 8))
        .spawn(ObjectKind::OpticsLab, Some(P1), tile(2, 6))
        .spawn(ObjectKind::Agent, Some(P1), point(12, 16))
        .spawn(ObjectKind::Agent, Some(P1), point(14, 16))
        .spawn(ObjectKind::CrystalPatch, None, tile(12, 3))
        .build();
    let orientation = BuildingOrientation::default();
    // The Tunnel's site, its Tier 1 area (14, 6) to (21, 13) clear of the first's; the turret's tile, outside both.
    let (site, turret) = (TilePos::new(16, 8), TilePos::new(12, 12));
    let mut line = Syndicate::new(BuildOrder::new(P1))
        .job(0, Job::Tunnel(site, orientation))
        .eject_builders(Job::Crystals(CRYSTALS));
    let mut sim = Sim::new(&scenario);
    let build_turret = Command::Order {
        units: vec![TURRET_BUILDER],
        kind: OrderKind::Build {
            kind: ObjectKind::LaserTurret,
            origin: turret,
            orientation,
        },
        queue: false,
    };
    let (mut completed, mut out) = (Vec::new(), None);
    let gathers = |line: &Syndicate| {
        (line.done().iter()).any(|(_, done)| {
            matches!(
                done,
                Done::Job {
                    job: Job::Crystals(_),
                    ..
                }
            )
        })
    };
    while !(gathers(&line) && out.is_some()) {
        assert!(sim.tick() < 1600, "both builders out by tick 1600: {:?}", line.done());
        let tick = sim.tick() + 1;
        let mut commands = line.commands(&sim);
        let mut crystals = line.spent_in(tick);
        if tick == 1 {
            commands.push(PlayerCommand {
                player: P1,
                command: build_turret.clone(),
            });
            crystals += 100;
        }
        for c in &commands {
            assert_eq!(sim.validate(c), Ok(()), "tick {tick}: {c:?}");
        }
        let books = Books::open(&sim, P1);
        let supplies = SupplyBooks::open(&sim, P1);
        sim.step(&TickInputs { commands });
        books.close(&sim, crystals);
        supplies.close(&sim, line.supplies_spent_in(tick));
        for e in sim.events() {
            match *e {
                SimEvent::SiteCompleted { site, worker } => {
                    completed.push((sim.object(site).expect("complete").kind, worker, site));
                }
                SimEvent::LeftSite { worker, .. } if worker == TURRET_BUILDER => out = Some(tick),
                _ => {}
            }
        }
    }
    let [
        (ObjectKind::LaserTurret, TURRET_BUILDER, _),
        (ObjectKind::Tunnel, TUNNEL_BUILDER, tunnel),
    ] = completed[..]
    else {
        panic!("the turret, then the Tunnel, completed: {completed:?}");
    };
    let done: Vec<Done> = line.done().iter().map(|&(_, done)| done).collect();
    assert_eq!(
        done,
        [
            Done::Job {
                n: 0,
                agent: TUNNEL_BUILDER,
                job: Job::Tunnel(site, orientation)
            },
            Done::Eject {
                tunnel,
                kind: ObjectKind::Agent
            },
            Done::Job {
                n: 0,
                agent: TUNNEL_BUILDER,
                job: Job::Crystals(CRYSTALS)
            },
        ],
        "the Tunnel's builder ejected and sent to gather; nothing for the turret's"
    );
    let agent = sim.object(TURRET_BUILDER).expect("the turret's Agent, out beside it");
    assert!(
        agent.orders.as_ref().is_some_and(|orders| orders.is_idle()),
        "the turret's Agent idles: {:?}",
        agent.orders
    );
}
