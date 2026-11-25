//! The prices and the refunds (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`; design changes 0229,
//! 0231, 0232 and 0284): "**Paid when the upgrade is ordered**"; "**Upgrade to Tier 2**: 2 + 2 x (number of T2+
//! Tunnels owned)", "**Upgrade to Tier 3**: 3 + 3 x (number of T3 Tunnels owned)"; "Higher-tier Tunnels count toward
//! lower-tier cost calculations"; "**Upgrades in progress count**"; "**Refunds are the current top price**"; and the
//! books, over random orders.

use sc_sim::{Rng, Sim};

use super::*;

/// The origins of [`four`]'s Tunnels, 1 to 4.
const CORNERS: [(i32, i32); 4] = [(4, 4), (20, 4), (4, 18), (20, 18)];

/// Four Tier 1 Tunnels of player 1's on a flat 48 × 32 map, at [`CORNERS`]: their Tier 3 areas, 12 × 12, are clear of
/// each other's and of the map's edge, so that each may upgrade to the top tier whatever the others do.
fn four() -> ScenarioBuilder {
    let mut builder = syndicate(&flat(48, 32));
    for (x, y) in CORNERS {
        builder = builder.spawn(TUNNEL, Some(P1), tile(x, y));
    }
    builder
}

/// The plan's Acceptance: "The first Tier 2 costs 2, the second 4; Tier 3 costs 3, then 6", paid when ordered. A
/// Tunnel's price while it upgrades is what its X would refund, not counting itself: 2 for the first.
#[test]
fn tier_2_costs_2_then_4_and_tier_3_costs_3_then_6() {
    let mut t = started(four(), 100);
    let (a, b) = (id(1), id(2));
    assert_eq!((price(&t, a), price(&t, b)), (Some(2), Some(2)));
    next(&mut t, P1, upgrade(a));
    assert_eq!((starts(&t), supplies(&t, P1)), (vec![(a, 2, 2)], 98));
    assert_eq!(price(&t, b), Some(4), "A, upgrading to Tier 2, counts");
    assert_eq!(price(&t, a), Some(2), "not counting itself");
    t.run(FRAMES);
    assert_eq!(tier(&t, a), 2);
    assert_eq!((price(&t, a), price(&t, b)), (Some(3), Some(4)));
    next(&mut t, P1, upgrade(b));
    assert_eq!((starts(&t)[1], supplies(&t, P1)), ((b, 2, 4), 94));
    t.run(FRAMES);
    assert_eq!(tier(&t, b), 2);

    next(&mut t, P1, upgrade(a));
    assert_eq!(price(&t, b), Some(6), "A, upgrading to Tier 3, counts");
    next(&mut t, P1, upgrade(b));
    assert_eq!(&starts(&t)[2..], [(a, 3, 3), (b, 3, 6)]);
    assert_eq!(supplies(&t, P1), 85);
    t.run(FRAMES);
    assert_eq!((tier(&t, a), tier(&t, b)), (3, 3));
    assert_eq!((price(&t, a), price(&t, b)), (None, None), "no Tier 4");
    assert_eq!(
        price(&t, id(3)),
        Some(6),
        "two Tier 3 Tunnels count in the Tier 2 price"
    );
}

/// "Higher-tier Tunnels count toward lower-tier cost calculations": a Tier 3 Tunnel counts in the Tier 2 price, and
/// in the Tier 3 price; a Tier 2 Tunnel in the Tier 2 price only.
#[test]
fn a_tier_3_tunnel_counts_in_the_tier_2_price() {
    let mut t = started(four(), 100);
    next(&mut t, P1, set_tier(id(1), 3));
    assert_eq!(price(&t, id(2)), Some(4));
    next(&mut t, P1, set_tier(id(3), 2));
    assert_eq!(price(&t, id(2)), Some(6), "a Tier 3 and a Tier 2 Tunnel");
    assert_eq!(price(&t, id(3)), Some(6), "to Tier 3: the Tier 3 Tunnel only");
    assert_eq!(price(&t, id(1)), None, "at the top tier");
    next(&mut t, P1, upgrade(id(2)));
    assert_eq!(starts(&t), [(id(2), 2, 6)]);
    assert_eq!(supplies(&t, P1), 94);
}

/// "Two upgrades started together cost 2 and 4, not 2 each": in one tick, in the order the commands came.
#[test]
fn two_upgrades_started_together_cost_2_and_4() {
    let mut t = started(four(), 100);
    next_all(&mut t, vec![(P1, upgrade(id(2))), (P1, upgrade(id(1)))]);
    assert_eq!(starts(&t), [(id(2), 2, 2), (id(1), 2, 4)]);
    assert_eq!(supplies(&t, P1), 94);
    let mut short = started(four(), 5);
    next_all(&mut short, vec![(P1, upgrade(id(1))), (P1, upgrade(id(2)))]);
    assert_eq!(starts(&short), [(id(1), 2, 2)], "the second can't be paid");
    assert_eq!(supplies(&short, P1), 3);
    assert_eq!(gate(&short, P1, upgrade(id(2))), Err(Reject::InsufficientSupplies));
}

/// Design change 0232's verification (and 0284's): "A and B upgrade together. Cancel B: refund 4. Then cancel A:
/// refund 2", and in the other order: each X refunds the price its upgrade would cost if ordered now, not counting
/// itself. Every Supply comes back, and the Tunnels are where they were, free to upgrade again.
#[test]
fn cancelling_refunds_the_current_top_price_in_either_order() {
    let (a, b) = (id(1), id(2));
    for (first, then) in [(b, a), (a, b)] {
        let mut t = started(four(), 100);
        next_all(&mut t, vec![(P1, upgrade(a)), (P1, upgrade(b))]);
        assert_eq!(supplies(&t, P1), 94);
        t.run(100);
        assert_eq!(price(&t, first), Some(4));
        next(&mut t, P1, cancel(first));
        assert_eq!(supplies(&t, P1), 98, "{first:?} first");
        assert_eq!(price(&t, then), Some(2));
        next(&mut t, P1, cancel(then));
        assert_eq!(supplies(&t, P1), 100);
        assert_eq!(cancels(&t), [(first, 2, 4), (then, 2, 2)]);
        for tunnel in [a, b] {
            assert_eq!((tier(&t, tunnel), upgrading(&t, tunnel)), (1, None));
            assert_eq!(gate(&t, P1, cancel(tunnel)), Err(Reject::NoOperation(tunnel)));
            assert_eq!(gate(&t, P1, upgrade(tunnel)), Ok(()));
        }
    }
}

/// The refund at Tier 3 counts the Tunnels at Tier 3 or upgrading to it, standing or not, whoever started first: B
/// upgrades to Tier 3 for 3; then A, at Tier 3 already (`DebugCommand::SetTier`), counts in its refund: 6.
#[test]
fn a_tier_3_refund_counts_the_tier_3_tunnels_at_the_cancel() {
    let mut t = started(four(), 100);
    let (a, b) = (id(1), id(2));
    next(&mut t, P1, set_tier(b, 2));
    next(&mut t, P1, upgrade(b));
    assert_eq!(starts(&t), [(b, 3, 3)]);
    next(&mut t, P1, set_tier(a, 3));
    assert_eq!(price(&t, b), Some(6));
    next(&mut t, P1, cancel(b));
    assert_eq!(cancels(&t), [(b, 3, 6)]);
    assert_eq!((tier(&t, b), supplies(&t, P1)), (2, 103));
}

/// The formula's total for `n2` Tunnels at Tier 2 or above and `n3` at Tier 3, each at the tier it holds or upgrades
/// to: the price of each in turn, 2 + 2k for the k-th to reach Tier 2 and 3 + 3k for the k-th to reach Tier 3, from
/// k = 0.
fn total(n2: u32, n3: u32) -> u32 {
    n2 * (n2 + 1) + 3 * n3 * (n3 + 1) / 2
}

/// How many of player 1's Tunnels hold or upgrade to Tier 2 or above, and to Tier 3, counted from the state on its
/// own (`TunnelState::reserved_tier`).
fn counts(sim: &Sim) -> (u32, u32) {
    let reserved: Vec<u8> = (sim.objects())
        .filter(|o| o.owner == Some(P1))
        .filter_map(|o| o.tunnel.as_ref().map(|state| state.reserved_tier()))
        .collect();
    let at_least = |tier| u32::try_from(reserved.iter().filter(|&&r| r >= tier).count()).expect("a few");
    (at_least(2), at_least(3))
}

/// What the random sequences went through, over every seed.
#[derive(Default, Debug)]
struct Coverage {
    started: u32,
    cancelled: u32,
    completed: u32,
    /// The Upgrades refused for want of Supplies, for an operation under way, and at the top tier; the cancels with
    /// nothing to cancel.
    unpaid: u32,
    busy: u32,
    top: u32,
    idle: u32,
}

/// `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs` (design change 0232): "The Supplies paid for what the
/// player still has always equal the formula's total, so the order of starting and cancelling can't gain or lose
/// Supplies". For each seed of a fixed list, 50 commands from player 1, each an Upgrade or an X on one of [`four`]'s
/// Tunnels drawn at random, a random wait of up to 6 seconds after each: some the gate refuses (the price, an
/// operation under way, the top tier, nothing to cancel), which changes nothing; upgrades complete. After every tick
/// the Supplies spent less those refunded equal the formula's total for the tiers player 1's Tunnels hold or upgrade
/// to ([`total`]).
#[test]
fn the_books_balance_whatever_the_order_of_starts_and_cancels() {
    const SEEDS: [u64; 6] = [1, 2, 3, 5, 8, 13];
    const START: u32 = 40;
    let mut seen = Coverage::default();
    for seed in SEEDS {
        let mut t = started(four(), START);
        let mut rng = Rng::from_seed(seed);
        for _ in 0..50 {
            let tunnel = id(1 + rng.below(4));
            let command = if rng.below(3) == 0 {
                cancel(tunnel)
            } else {
                upgrade(tunnel)
            };
            match gate(&t, P1, command.clone()) {
                Ok(()) => {}
                Err(Reject::InsufficientSupplies) => seen.unpaid += 1,
                Err(Reject::OperationUnderWay(_)) => seen.busy += 1,
                Err(Reject::TopTier(_)) => seen.top += 1,
                Err(Reject::NoOperation(_)) => seen.idle += 1,
                Err(other) => panic!("seed {seed}: {other:?}"),
            }
            let tick = t.sim().tick() + 1;
            t.at(tick, P1, command);
            for _ in 0..=rng.below(96) {
                t.step();
                let (n2, n3) = counts(t.sim());
                let spent = START - supplies(&t, P1);
                assert_eq!(
                    spent,
                    total(n2, n3),
                    "seed {seed}, tick {}: {n2} at Tier 2+, {n3} at Tier 3",
                    t.sim().tick()
                );
            }
        }
        seen.started += u32::try_from(starts(&t).len()).expect("a few");
        seen.cancelled += u32::try_from(cancels(&t).len()).expect("a few");
        seen.completed += u32::try_from(completions(&t).len()).expect("a few");
    }
    let Coverage {
        started,
        cancelled,
        completed,
        unpaid,
        busy,
        top,
        idle,
    } = seen;
    let all = [started, cancelled, completed, unpaid, busy, top, idle];
    assert!(
        all.iter().all(|&n| n > 0),
        "every way the books change, and every refusal: {seen:?}"
    );
}
