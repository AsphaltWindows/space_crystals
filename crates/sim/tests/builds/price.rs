//! The price and its refunds (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, design change 0233):
//! paid as the order is given, a lingering ghost counting as a Tunnel, refused when the player can't pay, refunded at
//! the current top price whenever an order goes before its site starts; and the books, over random orders.

use super::*;
use sc_sim::construction::site::expansion_def;
use sc_sim::defs::Resource;
use sc_sim::testing::build_order::Books;
use sc_sim::testing::syndicate::SupplyBooks;
use sc_sim::{Rng, Sim, Target};

/// Design change 0233's verification: "With 3 Supplies and 1 Tunnel, order two Tunnels with Shift. Both ghosts appear
/// and 1 + 2 Supplies are deducted at once. Cancel one order: 2 come back." With two Agents, one placement each, as the
/// client sends Shift-placements ("each Agent takes exactly one placement", `systems/controls/interface.md >
/// AwaitingTarget > Shift while placing`): a second Build Shift-queued to the same Agent is refused, and so is one
/// naming both.
#[test]
fn two_orders_cost_1_and_2_and_cancelling_one_refunds_2() {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(12, 16)) // 2
        .spawn(AGENT, Some(P1), point(13, 17)) // 3
        .start();
    t.debug_at(1, supplies_of(P1, 3));
    step(&mut t);
    let other = at(24, 8);
    t.at(2, P1, build(&[2], SITE, true)).at(2, P1, build(&[3], other, true));
    step(&mut t);
    assert_eq!(supplies(&t, P1), 0);
    assert_eq!(ghosts(&t, P1), [(id(2), SITE), (id(3), other)]);
    assert_eq!(price(&t, P1), 3);
    assert_eq!(
        gate(&t, P1, build(&[3], at(32, 8), true)),
        Err(Reject::ConstructorBusy(id(3)))
    );
    assert_eq!(
        gate(&t, P1, build(&[2, 3], at(32, 8), false)),
        Err(Reject::SeveralConstructors)
    );

    next(&mut t, P1, order(&[2], OrderKind::Stop, false));
    assert_eq!(supplies(&t, P1), 2);
    assert_eq!(refunds(&t), [(id(2), 2)]);
    assert_eq!(ghosts(&t, P1), [(id(3), other)]);
    assert_eq!(price(&t, P1), 2);
}

/// Design change 0233: "The order is refused, with feedback, if the player can't pay": with 0 Supplies and 1 Tunnel the
/// gate says `Reject::InsufficientSupplies` (the client's "Not enough Supplies"), and the order changes nothing. A
/// Build that replaces the Agent's own pays out of its refund: with the 1 Supply it had paid, a new Build without
/// Shift goes through at 0 Supplies.
#[test]
fn an_order_the_player_cannot_pay_for_is_refused() {
    let mut t = base().spawn(AGENT, Some(P1), point(12, 16)).start();
    t.debug_at(1, supplies_of(P1, 0));
    step(&mut t);
    assert_eq!(
        gate(&t, P1, build(&[2], SITE, false)),
        Err(Reject::InsufficientSupplies)
    );
    next(&mut t, P1, build(&[2], SITE, false));
    assert_eq!((supplies(&t, P1), ghosts(&t, P1), price(&t, P1)), (0, vec![], 1));
    assert!(idle(&t, 2));

    next_debug(&mut t, supplies_of(P1, 1));
    next(&mut t, P1, build(&[2], SITE, false));
    assert_eq!((supplies(&t, P1), ghosts(&t, P1)), (0, vec![(id(2), SITE)]));
    let elsewhere = at(24, 8);
    assert_eq!(
        gate(&t, P1, build(&[2], elsewhere, true)),
        Err(Reject::ConstructorBusy(id(2)))
    );
    next(&mut t, P1, build(&[2], elsewhere, false));
    assert_eq!((supplies(&t, P1), ghosts(&t, P1)), (0, vec![(id(2), elsewhere)]));
    assert_eq!(refunds(&t), [(id(2), 1)]);
}

/// Design change 0233: "If the order is cancelled, replaced, or fails before construction starts, it is refunded at the
/// current top price": replaced by a Move (a Move Shift-queued after it leaves it be); gone with its Agent's orders as
/// the Agent walks into Tunnel 1 (an Enter before it); and its Agent gone before the start (design change 0289 point
/// 1: "If an Agent is killed before construction starts, its order is refunded at the current top price"; removed here,
/// the removal path every death takes, and killed in
/// [`an_agent_killed_walking_to_build_gets_its_supplies_back`]). Each gives back 1, the price of a Tunnel for the player
/// who has Tunnel 1 only. Failing on arrival is `rules`'.
#[test]
fn orders_gone_before_their_site_starts_refund_the_top_price() {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(12, 16)) // 2
        .spawn(AGENT, Some(P1), point(10, 15)) // 3
        .spawn(AGENT, Some(P1), point(14, 17)) // 4
        .start();
    t.debug_at(1, supplies_of(P1, 10));
    step(&mut t);
    let away = OrderKind::Move(Target::Point(at(30, 16).center()));
    next(&mut t, P1, build(&[2], SITE, false));
    next(&mut t, P1, order(&[2], away, true));
    assert_eq!((supplies(&t, P1), refunds(&t)), (9, vec![]));
    next(&mut t, P1, order(&[2], away, false));
    assert_eq!(
        (supplies(&t, P1), refunds(&t), ghosts(&t, P1)),
        (10, vec![(id(2), 1)], vec![])
    );

    next(&mut t, P1, order(&[3], OrderKind::Enter(T1), false));
    next(&mut t, P1, build(&[3], SITE, true));
    assert_eq!((supplies(&t, P1), ghosts(&t, P1)), (9, vec![(id(3), SITE)]));
    step_until(&mut t, 300, "Agent 3 walks into Tunnel 1", |t| inside(t, P1) == [id(3)]);
    assert_eq!((supplies(&t, P1), refunds(&t).last()), (10, Some(&(id(3), 1))));
    assert_eq!(ghosts(&t, P1), []);

    next(&mut t, P1, build(&[4], SITE, false));
    assert_eq!(supplies(&t, P1), 9);
    next_debug(&mut t, DebugCommand::Remove { id: id(4) });
    assert_eq!((supplies(&t, P1), refunds(&t).last()), (10, Some(&(id(4), 1))));
    assert_eq!((ghosts(&t, P1), price(&t, P1)), (vec![], 1));
}

/// Design change 0289's Verification: "Kill an Agent walking to build: the Supplies come back" (point 1: "If an Agent
/// is killed before construction starts, its order is refunded at the current top price"). Agent 2 walks to [`SITE`],
/// north-east; six of player 2's Guards, on its way, are ordered to attack it and shoot it dead long before it gets
/// there. Its order gives back 1, the price of a Tunnel for the player who has Tunnel 1 only, and its ghost goes.
#[test]
fn an_agent_killed_walking_to_build_gets_its_supplies_back() {
    let mut builder = base().spawn(AGENT, Some(P1), point(10, 16)); // 2
    for (x, y) in [(12, 12), (13, 12), (14, 12), (12, 11), (13, 11), (14, 11)] {
        builder = builder.spawn(GUARD, Some(P2), point(x, y)); // 3 to 8
    }
    let mut t = builder.start();
    t.without_auto_targeting().debug_at(1, supplies_of(P1, 5));
    step(&mut t);
    next(&mut t, P1, build(&[2], SITE, false));
    assert_eq!((supplies(&t, P1), ghosts(&t, P1)), (4, vec![(id(2), SITE)]));
    next(&mut t, P2, order(&[3, 4, 5, 6, 7, 8], OrderKind::Attack(id(2)), false));
    step_until(&mut t, WALK, "the Guards kill Agent 2", |t| {
        t.sim().object(id(2)).is_none()
    });
    let killed = (t.events().iter()).any(|(_, e)| matches!(*e, SimEvent::Died { id: dead, .. } if dead == id(2)));
    assert!(killed, "killed, not removed");
    assert_eq!(starts(&t), [], "before its construction started");
    assert_eq!((supplies(&t, P1), refunds(&t)), (5, vec![(id(2), 1)]));
    assert_eq!((ghosts(&t, P1), price(&t, P1)), (vec![], 1));
}

/// `factions/syndicate/tunnels.md > Losing Tunnels`: "with no Tunnels owned, a new Tunnel costs 0 Supplies"; the next
/// costs 1, for the first's ghost. Player 2's Tunnel counts for player 2 only.
#[test]
fn with_no_tunnels_the_next_costs_0() {
    let mut t = syndicate(&flat(40, 20))
        .spawn(AGENT, Some(P1), point(12, 16)) // 1
        .spawn(TUNNEL, Some(P2), tile(30, 8)) // 2
        .start();
    t.debug_at(1, supplies_of(P1, 0));
    step(&mut t);
    assert_eq!((price(&t, P1), price(&t, P2)), (0, 1));
    assert_eq!(gate(&t, P1, build(&[1], SITE, false)), Ok(()));
    next(&mut t, P1, build(&[1], SITE, false));
    assert_eq!(
        (supplies(&t, P1), price(&t, P1), ghosts(&t, P1)),
        (0, 1, vec![(id(1), SITE)])
    );
}

/// The sites [`the_books_balance_whatever_the_orders`] picks from, as origins of an unturned Tunnel on its 48 × 24 map:
/// twelve whose areas touch at most, all clear of Tunnel 1's; and three over others' areas, refused while those are
/// there.
const CANDIDATES: [(i32, i32); 15] = [
    (14, 2),
    (22, 2),
    (30, 2),
    (38, 2),
    (14, 10),
    (22, 10),
    (30, 10),
    (38, 10),
    (14, 18),
    (22, 18),
    (30, 18),
    (38, 18),
    (18, 6),
    (10, 8),
    (26, 14),
];

/// The expansions Tunnel 1 builds in [`the_books_balance_whatever_the_orders`] (slices M4-F and M4-S2), each a kind and
/// the top-left tile of its footprint, in its area clear of it and the Optics Lab: three Laser Turrets and three
/// Hideouts.
const EXPANSIONS: [(ObjectKind, (i32, i32)); 6] = [
    (TURRET, (6, 6)),
    (TURRET, (9, 7)),
    (TURRET, (8, 10)),
    (ObjectKind::Hideout, (8, 12)),
    (ObjectKind::Hideout, (2, 9)),
    (ObjectKind::Hideout, (5, 12)),
];

/// The Build orders of a Tunnel among `o`'s orders, current or queued.
fn tunnel_builds(o: &Object) -> usize {
    let Some(orders) = o.orders.as_ref() else {
        return 0;
    };
    let current = orders.current.iter().map(|active| active.order.kind);
    (current.chain(orders.queue.iter().map(|order| order.kind)))
        .filter(|kind| matches!(kind, OrderKind::Build { kind: TUNNEL, .. }))
        .count()
}

/// How many Tunnels `player` has for the price, counted from the state on its own: those standing, sites included, and
/// the Build orders of a Tunnel among the player's units' orders, current or queued.
fn owned(sim: &Sim, player: PlayerId) -> u32 {
    let mine = || sim.objects().filter(move |o| o.owner == Some(player));
    let standing = mine().filter(|o| o.kind == TUNNEL).count();
    let ordered: usize = mine().map(tunnel_builds).sum();
    u32::try_from(standing + ordered).expect("a few")
}

/// What player 1's `command`, which the gate takes, pays as it applies: (Space Crystals, Supplies). A Build, of a
/// Tunnel (an Agent builds nothing else, design change 0359), pays its price once the Builds it replaces, its Agent's
/// own, have gone (`Sim::give_order`; the gate refuses one with Shift to an Agent that has one): the count of what
/// player 1 has, its Agent's Builds of a Tunnel left out. An Expand pays its expansion's cost (design change 0254).
fn pays(sim: &Sim, command: &Command) -> (u32, u32) {
    match command {
        Command::Order {
            units,
            kind: OrderKind::Build { kind, .. },
            ..
        } => {
            let price = sim.build_price(P1, *kind).expect("an Agent builds it");
            assert_eq!(price.resource, Resource::Supplies, "a Tunnel's price");
            let agent = sim.object(units[0]).expect("the Agent");
            let replaced = u32::try_from(tunnel_builds(agent)).expect("a few");
            (0, price.amount - replaced)
        }
        Command::Tunnel {
            command: TunnelCommand::Expand { kind, .. },
            ..
        } => (expansion_def(*kind).expect("a Tunnel builds it").cost, 0),
        _ => (0, 0),
    }
}

/// The formula's total for `n` Tunnels (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`): the price
/// of each but the first, 1 + 2 + ... + (n - 1).
fn total(n: u32) -> i64 {
    let n = i64::from(n);
    n * (n - 1) / 2
}

/// A random command of player 1's for [`the_books_balance_whatever_the_orders`], or none when what it drew has nothing to
/// go to: a Build of a Tunnel at one of the [`CANDIDATES`] to one Agent, with Shift or without; a Stop; a Move, with
/// Shift or without; Cancel Construction on a site, an Agent's or an expansion; an Eject from a Tunnel in operation; an
/// Enter into Tunnel 1, with Shift or without; or one of the [`EXPANSIONS`] by Tunnel 1, or its X.
fn random_command(t: &TestSim, rng: &mut Rng) -> Option<Command> {
    let sim = t.sim();
    let mine = |o: &&Object| o.owner == Some(P1);
    let agents: Vec<u32> = (sim.objects().filter(mine))
        .filter(|o| o.kind == AGENT)
        .map(|o| o.id.0)
        .collect();
    let tunnels: Vec<ObjectId> = (sim.objects().filter(mine))
        .filter(|o| o.kind == TUNNEL && o.is_operational())
        .map(|o| o.id)
        .collect();
    let sites = sites(t);
    let draw = rng.below(19);
    let shift = rng.below(2) == 1;
    let mut pick = |len: usize| (len > 0).then(|| rng.below(u32::try_from(len).expect("a few")) as usize);
    let agent = pick(agents.len()).map(|i| agents[i]);
    let orientation = BuildingOrientation::default();
    Some(match draw {
        0..=7 => {
            let (x, y) = CANDIDATES[pick(CANDIDATES.len())?];
            build(&[agent?], at(x, y), shift)
        }
        8 => order(&[agent?], OrderKind::Stop, false),
        9..=10 => {
            let to = at(i32::try_from(pick(48)?).ok()?, i32::try_from(pick(24)?).ok()?).center();
            order(&[agent?], OrderKind::Move(Target::Point(to)), shift)
        }
        11..=12 => cancel(&[sites[pick(sites.len())?]]),
        13..=14 => eject(tunnels[pick(tunnels.len())?]),
        15 => order(&[agent?], OrderKind::Enter(T1), shift),
        16..=17 => {
            let (kind, (x, y)) = EXPANSIONS[pick(EXPANSIONS.len())?];
            let origin = at(x, y);
            let command = TunnelCommand::Expand {
                kind,
                origin,
                orientation,
            };
            Command::Tunnel { tunnel: T1, command }
        }
        _ => Command::Tunnel {
            tunnel: T1,
            command: TunnelCommand::Cancel,
        },
    })
}

/// [`the_books_balance_whatever_the_orders`]' map: 48 × 24, flat but for Water at (43, 15), in the area of the
/// candidate (38, 10), which a Build there finds on arrival, until an Agent explored it; player 1's Tunnel 1 on tiles
/// (4, 8) to (7, 11), four Agents of theirs, 2 to 5, south of it, and their Optics Lab (6) in its area, on tiles (2, 6)
/// to (3, 7), which the Tunnel's Laser Turret requires.
fn books_field() -> TestSim {
    let mut rows: Vec<String> = (0..24).map(|_| ".".repeat(48)).collect();
    rows[15].replace_range(43..44, "~");
    let mut builder = syndicate(&rows.join("\n")).spawn(TUNNEL, Some(P1), tile(4, 8));
    for x in [8, 10, 12, 14] {
        builder = builder.spawn(AGENT, Some(P1), point(x, 20));
    }
    builder.spawn(ObjectKind::OpticsLab, Some(P1), tile(2, 6)).start()
}

/// What the random sequences went through, over every seed.
#[derive(Default, Debug)]
struct Coverage {
    /// The Builds of a Tunnel the gate accepted, and those it refused for want of Supplies, or for anything else.
    paid: u32,
    unpaid: u32,
    refused: u32,
    /// A Tunnel's Build orders that went before their sites started, and its sites cancelled.
    refunded: u32,
    started: u32,
    failed: u32,
    cancelled: u32,
    completed: u32,
    /// The expansions cancelled (slice M4-Z), Hideouts and Laser Turrets.
    expansion_cancelled: u32,
}

/// `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "The Supplies paid for what the player still has
/// always equal the formula's total, so the order of starting and cancelling can't gain or lose Supplies" (design
/// change 0233; the plan's check: "after any sequence of orders and cancels with no destruction, Supplies spent less
/// refunded equals the formula's total"). For each seed of a fixed list, 60 random commands from player 1
/// ([`random_command`]), a random wait of up to 4 seconds after each: some the gate refuses (the price, a busy Agent,
/// an area over another), which changes nothing; Agents complete Tunnels and are ejected again. After every tick the
/// Supplies spent less those refunded equal the formula's total for what player 1 has, over the Tunnel it started with,
/// and the price is the count of what it has. Among the commands, Tunnel 1's expansions, Hideouts and Laser Turrets
/// (slices M4-Z and M4-S2), their sites cancelled and refunded in Space Crystals (design change 0254): both of player
/// 1's banks balance every tick, by
/// `testing::build_order::Books` and `testing::syndicate::SupplyBooks`, each with what the tick's command paid in its
/// resource ([`pays`]).
#[test]
fn the_books_balance_whatever_the_orders() {
    const SEEDS: [u64; 8] = [1, 2, 3, 5, 8, 13, 21, 34];
    const START: u32 = 12;
    let mut seen = Coverage::default();
    for seed in SEEDS {
        let mut t = books_field();
        let banks = DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: START,
            alloys: 0,
            essence: 0,
            flux: 0,
        };
        t.debug_at(1, banks);
        step(&mut t);
        let mut rng = Rng::from_seed(seed);
        for _ in 0..60 {
            // What the command pays as it applies, in the next tick: (Space Crystals, Supplies).
            let mut paying = (0, 0);
            if let Some(command) = random_command(&t, &mut rng) {
                let verdict = gate(&t, P1, command.clone());
                if let Command::Order {
                    kind: OrderKind::Build { kind: TUNNEL, .. },
                    ..
                } = command
                {
                    match verdict {
                        Ok(()) => seen.paid += 1,
                        Err(Reject::InsufficientSupplies) => seen.unpaid += 1,
                        Err(_) => seen.refused += 1,
                    }
                }
                if verdict.is_ok() {
                    paying = pays(t.sim(), &command);
                }
                let tick = t.sim().tick() + 1;
                t.at(tick, P1, command);
            }
            for _ in 0..=rng.below(64) {
                let (crystals, supplies_paid) = std::mem::take(&mut paying);
                let books = Books::open(t.sim(), P1);
                let supply_books = SupplyBooks::open(t.sim(), P1);
                step(&mut t);
                books.close(t.sim(), crystals);
                supply_books.close(t.sim(), supplies_paid);
                let n = owned(t.sim(), P1);
                let spent = i64::from(START) - i64::from(supplies(&t, P1));
                assert_eq!(
                    spent,
                    total(n) - total(1),
                    "seed {seed}, tick {}: {n} Tunnels",
                    t.sim().tick()
                );
                assert_eq!(price(&t, P1), n, "seed {seed}, tick {}", t.sim().tick());
            }
        }
        for (_, e) in t.events() {
            match e {
                SimEvent::BuildRefunded { kind: TUNNEL, .. } => seen.refunded += 1,
                SimEvent::SiteStarted { .. } => seen.started += 1,
                SimEvent::BuildFailed { .. } => seen.failed += 1,
                SimEvent::SiteCancelled { kind: TUNNEL, .. } => seen.cancelled += 1,
                SimEvent::ExpansionCancelled { .. } => seen.expansion_cancelled += 1,
                SimEvent::SiteCompleted { .. } => seen.completed += 1,
                _ => {}
            }
        }
    }
    let Coverage {
        paid,
        unpaid,
        refused,
        refunded,
        started,
        failed,
        cancelled,
        completed,
        expansion_cancelled,
    } = seen;
    let all = [
        paid,
        unpaid,
        refused,
        refunded,
        started,
        failed,
        cancelled,
        completed,
        expansion_cancelled,
    ];
    assert!(
        all.iter().all(|&n| n > 0),
        "every way the books change, and every refusal: {seen:?}"
    );
}
