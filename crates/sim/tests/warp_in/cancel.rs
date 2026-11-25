//! Cancel Warp In (`factions/colonists/economy.md > Cancel Warp In`): 75% of the cost refunded and the Beacon
//! back with the fraction of its HP (design change 0111), and the cancel going to every transforming object
//! it names (plan decision T22).

use super::*;

/// The plan's acceptance, "Cancel" (`> Cancel Warp In`: "Cancelling refunds 75% of the building's resource cost
/// (rounded down), and the structure reverts to being a normal, fully-functional Beacon (no HP/Shields loss)"): 75
/// → 56, 250 → 187 (the Robo Portal's cost since design change 0167), 150 → 112 ([`Refund::Ready`]). The Beacon
/// returns on its old tiles, at its full 120 HP, the same object, and can warp in again.
#[test]
fn a_cancel_refunds_75_percent_and_the_beacon_returns() {
    for (b, refund) in BUILDINGS.into_iter().zip([56, 187, 112]) {
        let (kind, cost) = (b.kind, b.cost);
        assert_eq!(Refund::Ready.of(cost), refund);
        let mut t = base().start();
        let before = object(&t, 1).clone();
        t.debug_at(1, fund(P1, 1000)).at(1, P1, construction(&[1], warp(kind)));
        t.run_until(99);
        next(&mut t, construction(&[1], CancelTransform));
        assert_eq!(cancellations(&t), [(100, 1, kind, refund)]);
        assert_eq!(bank(&t, P1), 1000 - cost + refund, "{kind:?}");
        assert!(t.sim().events().contains(&SimEvent::KindChanged {
            id: id(1),
            from: kind,
            to: Beacon
        }));
        let o = object(&t, 1);
        assert_eq!(
            (o.kind, o.placement, o.pos, o.hp, o.owner, &o.transform),
            (Beacon, before.placement, before.pos, Some(120.0), Some(P1), &None),
            "{kind:?}"
        );
        assert_eq!(o.max_hp(), Some(120.0));
        let again = PlayerCommand {
            player: P1,
            command: construction(&[1], warp(kind)),
        };
        assert_eq!(t.sim().validate(&again), Ok(()), "a Beacon again");
    }
}

/// Design change 0111 ("current_after = max(1, original_max × current_before ÷ max_before)"): a damaged building
/// being warped in returns to a Beacon with the fraction of its HP it had: player 2's Peacekeeper 6 shoots the
/// warping Robo Portal, which is cancelled later.
#[test]
fn a_cancel_keeps_the_fraction_of_hp() {
    let mut t = shot_at();
    t.run_until(299);
    let o = object(&t, 1);
    let fraction = o.hp.unwrap() / o.max_hp().unwrap();
    assert!(fraction < 1.0, "damaged through its Shields: {fraction}");
    next(&mut t, construction(&[1], CancelTransform));
    let o = object(&t, 1);
    assert_eq!(o.kind, Beacon);
    assert!((o.hp.unwrap() - 120.0 * fraction).abs() < 1e-3, "{:?}", o.hp);
}

/// Design change 0111's Verification for a Warp In: "Cancel a Warp In at 50% HP → Beacon 60/120"; "Cancel a nearly
/// dead transforming object: it survives at ≥ 1 HP". A game saved with the Robo Portal on Beacon 1 a quarter warped
/// in (MaxHP 240) is given 120 HP, or half an HP, and cancelled.
#[test]
fn a_cancel_at_half_hp_leaves_a_beacon_at_60_and_a_nearly_dead_one_alive() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], warp(RoboPortal)));
    t.run_until(160);
    assert_eq!(object(&t, 1).max_hp(), Some(240.0));
    let cancel = TickInputs {
        commands: vec![PlayerCommand {
            player: P1,
            command: construction(&[1], CancelTransform),
        }],
    };
    for (hp, after) in [(120.0, 60.0), (0.5, 1.0)] {
        let mut sim = edited(t.sim(), 1, "hp", serde_json::json!(hp));
        sim.step(&cancel);
        let o = sim.object(id(1)).unwrap();
        assert_eq!(
            (o.kind, o.hp, o.max_hp()),
            (Beacon, Some(after), Some(120.0)),
            "{hp} of 240"
        );
    }
}

/// `ConstructionCommand::CancelTransform` goes to every transforming object it names (plan decision T22), in id
/// order, and to nothing else: a Beacon or a Generator among them is left alone; with nothing transforming it is
/// refused ([`Reject::NotTransforming`]), and anyone else's object refuses it ([`Reject::NotOwned`]).
#[test]
fn a_cancel_goes_to_every_transforming_object_named() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .debug_at(1, fund(P2, 1000))
        .at(1, P1, construction(&[1], warp(EgressNode)))
        .at(1, P2, construction(&[4], warp(EgressNode)));
    t.step();
    assert_rejected(
        &mut t,
        P1,
        construction(&[2, 3], CancelTransform),
        Reject::NotTransforming,
    );
    assert_rejected(
        &mut t,
        P1,
        construction(&[1, 4], CancelTransform),
        Reject::NotOwned(id(4)),
    );
    assert_rejected(&mut t, P1, construction(&[], CancelTransform), Reject::NotTransforming);
    next(&mut t, construction(&[1], warp(RoboPortal)));
    assert_eq!(
        object(&t, 1).kind,
        EgressNode,
        "a building being warped in warps nothing in"
    );
    next(&mut t, construction(&[2], warp(RoboPortal)));
    assert_eq!(object(&t, 2).kind, RoboPortal);
    next(&mut t, construction(&[3, 2, 1, 2, 99], CancelTransform));
    let tick = t.sim().tick();
    assert_eq!(
        cancellations(&t),
        [(tick, 1, EgressNode, 56), (tick, 2, RoboPortal, 187)]
    );
    assert_eq!((object(&t, 1).kind, object(&t, 2).kind), (Beacon, Beacon));
    assert_eq!(bank(&t, P1), 1000 - 75 - 250 + 56 + 187);
    assert_eq!(object(&t, 4).kind, EgressNode, "player 2's goes on");
}
