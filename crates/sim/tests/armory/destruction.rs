//! An Armory destroyed mid-training (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on
//! Destruction`: "**Everyone inside is evacuated alive**: units in the internal pool, units reserved by queued orders,
//! the unit currently being trained"; "a Recruit mid-training comes out as a Recruit"; "**Orders are cancelled with a
//! full refund**"). Where they come out, and a unit whose training finished while it waited for the doorway, are
//! design change 0349's, slice M5-F3's.

use super::*;

/// The plan's acceptance (slice M5-F2): "Destroying the Armory mid-training brings a Recruit out; every entry is
/// refunded." Two Militants and a Release queued, the Armory goes 10 ticks into the first training: the four Recruits
/// come out around the footprint as Recruits, unchanged (the Constitution one with its augmentation), the reserved
/// ones first, in queue order (Placeholder (Q0106 point 2): "The unit in training, then the reserved ones in queue
/// order, then the pool"), and the 40 Space Crystals come back, each entry with its cancellation event. Nothing is
/// trained, no one dies.
#[test]
fn an_armory_destroyed_mid_training_lets_its_recruits_out_and_refunds_every_entry() {
    // Player 1 keeps a Storage (2), and so isn't defeated with the Armory.
    let mut t = base().spawn(ObjectKind::Storage, Some(P1), tile(2, 2)).start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, 4, 4, Vec2::new(1150.0, 704.0));
    augment(&mut t, 2, recruits[3], RecruitKind::Constitution);
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    fill(&mut t, A1, 4);
    let order = inside(&t, A1);
    let constitution = recruits[3];
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start, P1, release(&[A1], RecruitKind::Constitution));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 40);
    let pool: Vec<ObjectId> = order.iter().copied().filter(|&unit| unit != constitution).collect();
    assert_eq!(reserved(&t, A1), [[pool[0]], [constitution], [pool[1]]]);
    let gone = start + 10;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    let evacuated: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Evacuated { unit, building: A1 } if *tick == gone => Some(unit),
            _ => None,
        })
        .collect();
    assert_eq!(evacuated, [pool[0], constitution, pool[1], pool[2]]);
    for &unit in &evacuated {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!((o.kind, o.hp), (RECRUIT, Some(40.0)), "{unit:?} as a Recruit");
    }
    let augmented = t.sim().object(constitution).unwrap().augmentation;
    assert_eq!(augmented, Augmentation::of(RecruitKind::Constitution));
    assert_eq!(bank(&t, P1), BANK);
    let cancellations: Vec<&SimEvent> = (t.events().iter())
        .filter(|(tick, e)| {
            *tick == gone
                && matches!(
                    e,
                    SimEvent::ProductionCancelled { .. } | SimEvent::ReleaseCancelled { .. }
                )
        })
        .map(|(_, e)| e)
        .collect();
    assert_eq!(
        cancellations,
        [
            &SimEvent::ProductionCancelled {
                producer: A1,
                kind: MILITANT,
                refund: 20
            },
            &SimEvent::ReleaseCancelled {
                producer: A1,
                unit: constitution
            },
            &SimEvent::ProductionCancelled {
                producer: A1,
                kind: MILITANT,
                refund: 20
            },
        ]
    );
    run_for(&mut t, 64);
    assert!(produced(&t).is_empty() && exits(&t).is_empty());
    assert!(!t.events().iter().any(|(_, e)| matches!(e, SimEvent::Died { .. })));
    assert!(t.sim().held().next().is_none());
}
