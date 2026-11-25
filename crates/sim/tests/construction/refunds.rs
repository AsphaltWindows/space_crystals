//! Refunds (`factions/gdo/structures.md > DeploymentCenter > Cancellation`): a cancel while constructing and
//! once ready, and a constructor destroyed, by a removal or in combat, refunding as if cancelled (design change
//! 0084).

use super::*;

/// The `ConstructionCancelled` events of the latest tick: constructor, building, refund.
fn cancellations(t: &TestSim) -> Vec<(u32, ObjectKind, u32)> {
    t.sim()
        .events()
        .iter()
        .filter_map(|e| match e {
            SimEvent::ConstructionCancelled {
                constructor,
                kind,
                refund,
            } => Some((constructor.0, *kind, *refund)),
            _ => None,
        })
        .collect()
}

/// The plan's acceptance, refunds (`> Cancellation`): cancelling while constructing gives the whole cost back,
/// cancelling a ready building 75% of it, rounded down: 75 → 56, 100 → 75, 200 → 150. The constructor is idle again,
/// and a `ConstructionCancelled` event in the cancel's tick says what went back.
#[test]
fn cancelling_refunds_in_full_while_constructing_and_75_percent_once_ready() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000)).run_until(1);
    let mut refunds = Vec::new();
    for (n, kind) in [
        (1, Generator),
        (1, Barracks),
        (1, ExtractionFacility),
        (2, ExtractionPlate),
    ] {
        for ready in [false, true] {
            let before = bank(&t, P1);
            next(&mut t, construction(&[n], Construct(kind)));
            let paid = before - bank(&t, P1);
            if ready {
                until_ready(&mut t, n);
            } else {
                t.run(20);
            }
            let before = bank(&t, P1);
            let cancel = if ready { CancelReady } else { CancelConstruction };
            next(&mut t, construction(&[n], cancel));
            assert!(state(&t, n).is_idle(), "{kind:?}");
            let refund = bank(&t, P1) - before;
            assert_eq!(cancellations(&t), [(n, kind, refund)]);
            refunds.push((kind, paid, refund));
        }
    }
    assert_eq!(
        refunds,
        [
            (Generator, 100, 100),
            (Generator, 100, 75),
            (Barracks, 200, 200),
            (Barracks, 200, 150),
            (ExtractionFacility, 200, 200),
            (ExtractionFacility, 200, 150),
            (ExtractionPlate, 75, 75),
            (ExtractionPlate, 75, 56),
        ]
    );
    assert_eq!(bank(&t, P1), 1000 - 25 - 50 - 50 - 19);
    assert!(
        completions(&t).len() == 4 && t.sim().objects().count() == 4,
        "nothing placed"
    );
}

/// Design change 0084's verifications 2 and 3 (`> Cancellation`: "Destroyed while constructing or holding a ready
/// building: refunded as if cancelled"): a Deployment Center destroyed while it builds a Generator gives 100 back,
/// one holding a ready Generator 75; an Extraction Facility 75 and 56 for its plate; an idle one nothing. Each refund
/// has its `ConstructionCancelled` event, before the constructor's `Removed`. A debug removal stands for a destruction
/// (`Sim::remove_objects`, the one way out of the game).
#[test]
fn a_destroyed_constructor_refunds_as_if_cancelled() {
    let mut t = ScenarioBuilder::new(&flat(32, 12))
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 1
        .spawn(DeploymentCenter, Some(P1), tile(6, 1)) // 2
        .spawn(ExtractionFacility, Some(P1), tile(11, 1)) // 3
        .spawn(ExtractionFacility, Some(P1), tile(15, 1)) // 4
        .spawn(DeploymentCenter, Some(P1), tile(19, 1)) // 5
        .start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1, 2], Construct(Generator)))
        .at(1, P1, construction(&[3, 4], Construct(ExtractionPlate)));
    let removal = |t: &mut TestSim, tick: Tick, n: u32, kind: Option<ObjectKind>| -> u32 {
        t.run_until(tick - 1);
        let before = bank(t, P1);
        t.debug_at(tick, DebugCommand::Remove { id: id(n) }).step();
        assert!(t.sim().object(id(n)).is_none());
        let refund = bank(t, P1) - before;
        let cancelled = kind.map(|kind| SimEvent::ConstructionCancelled {
            constructor: id(n),
            kind,
            refund,
        });
        let order: Vec<&SimEvent> = t
            .sim()
            .events()
            .iter()
            .filter(|e| matches!(e, SimEvent::ConstructionCancelled { .. } | SimEvent::Removed(_)))
            .collect();
        let expected: Vec<SimEvent> = cancelled.into_iter().chain([SimEvent::Removed(id(n))]).collect();
        assert_eq!(order, expected.iter().collect::<Vec<_>>(), "#{n}");
        refund
    };
    let refunds = [
        removal(&mut t, 100, 1, Some(Generator)),
        removal(&mut t, 101, 3, Some(ExtractionPlate)),
        removal(&mut t, 300, 2, Some(Generator)),
        removal(&mut t, 450, 4, Some(ExtractionPlate)),
        removal(&mut t, 451, 5, None),
    ];
    assert_eq!(refunds, [100, 75, Refund::Ready.of(100), Refund::Ready.of(75), 0]);
    assert_eq!(refunds[2..4], [75, 56]);
    assert_eq!(bank(&t, P1), 1000 - 350 + 306);
    assert_eq!(dangling_references(t.sim()), []);
}

/// Design change 0084 in combat: player 2's Extraction Facility, holding a ready plate, is destroyed by player 1's
/// Peacekeepers; player 2 gets 56 back in the tick it dies, with its `ConstructionCancelled` event.
#[test]
fn a_constructor_destroyed_in_combat_refunds_its_owner() {
    let mut builder = ScenarioBuilder::new(&flat(20, 12))
        .spawn(ExtractionFacility, Some(P2), tile(8, 4)) // 1: (8, 4) to (10, 6)
        .spawn(DeploymentCenter, Some(P2), tile(14, 7)); // 2
    for (x, y) in [(6, 3), (6, 5), (6, 7), (8, 8), (10, 8), (12, 5)] {
        builder = builder.spawn(Peacekeeper, Some(P1), SpawnAt::Point(at(x, y).center()));
    }
    let mut t = builder.start();
    t.without_auto_targeting()
        .debug_at(1, fund(P2, 75))
        .at(1, P2, construction(&[1], Construct(ExtractionPlate)));
    until_ready(&mut t, 1);
    assert_eq!(bank(&t, P2), 0);
    let tick = t.sim().tick() + 1;
    let attackers: Vec<ObjectId> = (3..=8).map(id).collect();
    t.order_at(tick, P1, &attackers, OrderKind::Attack(id(1)), false);
    let mut died = None;
    for _ in 0..3000 {
        t.step();
        if t.sim().object(id(1)).is_none() {
            died = Some(t.sim().tick());
            break;
        }
        assert_eq!(bank(&t, P2), 0, "nothing back before it dies");
    }
    let died = died.expect("the Extraction Facility is destroyed");
    assert!(
        t.sim()
            .events()
            .iter()
            .any(|e| matches!(e, SimEvent::Died { id: dead, .. } if *dead == id(1)))
    );
    assert_eq!(bank(&t, P2), 56, "refunded in tick {died}");
    assert_eq!(cancellations(&t), [(1, ExtractionPlate, 56)]);
}
