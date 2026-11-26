//! `sc_sim::testing::syndicate`'s line (slice M4-L) beyond what its golden games play, through the gate, with both of
//! the player's banks balanced every tick: an upgrade (`Syndicate::upgrade`, `TunnelCommand::Upgrade`, slice M4-G)
//! paid by the Supplies its Agent gathers, on `dev_skirmish` from its real start; and its builders' eject
//! (`Syndicate::eject_builders`) by the Tunnel they finish.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::testing::build_order::{Books, BuildOrder, Order};
use sc_sim::testing::syndicate::skirmish::{self, AGENTS, BUILDER, PATCH, STATION, SYNDICATE, TUNNEL_SITE, TURNED};
use sc_sim::testing::syndicate::{Done, Job, SupplyBooks, Syndicate};
use sc_sim::tunnel::Resource;
use sc_sim::{
    BuildingOrientation, Faction, ObjectId, ObjectKind, PlayerId, Sim, SimEvent, SpawnAt, Target, Tick, TickInputs,
    TilePos,
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

/// `Syndicate::eject_builders`: an Agent that finishes a Tunnel goes into the network, and the line ejects it by that
/// Tunnel and gives it its job (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`; an Agent
/// builds nothing else since design change 0359). On a flat 40 × 20 map, player 1's Tunnel (1) on tiles (4, 8) to
/// (7, 11), an Agent (2) south-east of it and a crystal patch (3) north of it, with the Syndicate's starting Supply:
/// the line's Agent builds a Tunnel east of the first, for the Supply, and gathers the patch once ejected.
#[test]
fn the_line_ejects_a_tunnels_builder() {
    const P1: PlayerId = PlayerId(1);
    const TUNNEL_BUILDER: ObjectId = ObjectId(2);
    const CRYSTALS: ObjectId = ObjectId(3);
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let point = |x, y| SpawnAt::Point(TilePos::new(x, y).center());
    let scenario = ScenarioBuilder::new(&vec![".".repeat(40); 20].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(PlayerId(2), Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 8))
        .spawn(ObjectKind::Agent, Some(P1), point(12, 16))
        .spawn(ObjectKind::CrystalPatch, None, tile(12, 3))
        .build();
    let orientation = BuildingOrientation::default();
    // The Tunnel's site, its Tier 1 area (14, 6) to (21, 13) clear of the first's.
    let site = TilePos::new(16, 8);
    let mut line = Syndicate::new(BuildOrder::new(P1))
        .job(0, Job::Tunnel(site, orientation))
        .eject_builders(Job::Crystals(CRYSTALS));
    let mut sim = Sim::new(&scenario);
    let mut completed = Vec::new();
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
    while !gathers(&line) {
        assert!(sim.tick() < 1600, "the builder out by tick 1600: {:?}", line.done());
        let tick = sim.tick() + 1;
        let commands = line.commands(&sim);
        for c in &commands {
            assert_eq!(sim.validate(c), Ok(()), "tick {tick}: {c:?}");
        }
        let books = Books::open(&sim, P1);
        let supplies = SupplyBooks::open(&sim, P1);
        sim.step(&TickInputs { commands });
        books.close(&sim, line.spent_in(tick));
        supplies.close(&sim, line.supplies_spent_in(tick));
        for e in sim.events() {
            if let SimEvent::SiteCompleted { site, worker } = *e {
                completed.push((sim.object(site).expect("complete").kind, worker, site));
            }
        }
    }
    let [(ObjectKind::Tunnel, TUNNEL_BUILDER, tunnel)] = completed[..] else {
        panic!("the Tunnel completed: {completed:?}");
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
        "the Tunnel's builder ejected and sent to gather"
    );
}
