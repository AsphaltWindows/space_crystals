//! `sc_sim::testing::syndicate`'s line (slice M4-L) beyond what its golden games play: an upgrade
//! (`Syndicate::upgrade`, `TunnelCommand::Upgrade`, slice M4-G) paid by the Supplies its Agent gathers, on
//! `dev_skirmish` from its real start, through the gate, with both of the player's banks balanced every tick.

use sc_sim::testing::build_order::{Books, BuildOrder, Order};
use sc_sim::testing::syndicate::skirmish::{self, AGENTS, BUILDER, PATCH, STATION, SYNDICATE, TUNNEL_SITE, TURNED};
use sc_sim::testing::syndicate::{Done, Job, SupplyBooks, Syndicate};
use sc_sim::tunnel::Resource;
use sc_sim::{ObjectKind, Sim, SimEvent, Target, Tick, TickInputs};

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
