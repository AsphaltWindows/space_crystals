//! An Armory destroyed while it trains (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on
//! Destruction`: "**Everyone inside is evacuated alive**: units in the internal pool, units reserved by queued orders,
//! the unit currently being trained"; "a Recruit mid-training comes out as a Recruit"; "**A unit that had finished
//! training** and was waiting for the doorway comes out **as the trained unit**, with no refund"; "**Orders are
//! cancelled with a full refund**"; design change 0349, slice M5-F3). Where they come out is `tests/holding/`'s.

use super::*;

/// The middle of the tiles of [`base`]'s Armory, x 768..960 and y 640..768.
const FOOTPRINT: (Vec2, Vec2) = (Vec2::new(768.0, 640.0), Vec2::new(960.0, 768.0));

/// The full HP of Recruit `unit` of these tests: 40, (M5-N1a) 60 for `constitution`, the one with Enhanced Constitution
/// (+2 Constitution increments at no research, `factions/cults/augmentation.md > Augmentation Effects`).
fn full_hp(unit: ObjectId, constitution: ObjectId) -> f32 {
    if unit == constitution { 60.0 } else { 40.0 }
}

/// The units evacuated out of `building` in tick `tick`, in order.
fn evacuated_in(t: &TestSim, tick: Tick, building: ObjectId) -> Vec<ObjectId> {
    (t.events().iter())
        .filter_map(|(at, e)| match *e {
            SimEvent::Evacuated { unit, building: from } if *at == tick && from == building => Some(unit),
            _ => None,
        })
        .collect()
}

/// The cancellations of tick `tick`, in order.
fn cancellations_in(t: &TestSim, tick: Tick) -> Vec<SimEvent> {
    (t.events().iter())
        .filter(|(at, e)| {
            *at == tick
                && matches!(
                    e,
                    SimEvent::ProductionCancelled { .. } | SimEvent::ReleaseCancelled { .. }
                )
        })
        .map(|(_, e)| e.clone())
        .collect()
}

/// Unit `unit` is on the map inside the Armory's former footprint, idle.
fn idle_on_the_footprint(t: &TestSim, unit: ObjectId) {
    let o = t.sim().object(unit).unwrap_or_else(|| panic!("{unit:?} evacuated"));
    let half = o.def().half_extents().unwrap();
    let (lo, hi) = FOOTPRINT;
    assert!(
        (o.pos - half).cmpge(lo).all() && (o.pos + half).cmple(hi).all(),
        "{unit:?} at {} inside the footprint",
        o.pos
    );
    assert!(o.orders.as_ref().unwrap().is_idle(), "{unit:?} idle");
}

/// The plan's acceptance (slice M5-F2): "Destroying the Armory mid-training brings a Recruit out; every entry is
/// refunded." Two Militants and a Release queued, the Armory goes 10 ticks into the first training: the four Recruits
/// come out on its footprint, all in the tick it goes, longest inside first, as Recruits, unchanged (the Constitution
/// one with its augmentation), and the 40 Space Crystals come back, each entry with its cancellation event. Nothing
/// is trained, no one dies.
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
    assert_eq!(
        evacuated_in(&t, gone, A1),
        order,
        "all four at once, longest inside first"
    );
    for &unit in &order {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!(
            (o.kind, o.hp),
            (RECRUIT, Some(full_hp(unit, constitution))),
            "{unit:?} as a Recruit"
        );
        idle_on_the_footprint(&t, unit);
    }
    let augmented = t.sim().object(constitution).unwrap().augmentation;
    assert_eq!(augmented, Augmentation::of(RecruitKind::Constitution));
    assert_eq!(bank(&t, P1), BANK);
    assert_eq!(
        cancellations_in(&t, gone),
        [
            SimEvent::ProductionCancelled {
                producer: A1,
                kind: MILITANT,
                refund: 20
            },
            SimEvent::ReleaseCancelled {
                producer: A1,
                unit: constitution
            },
            SimEvent::ProductionCancelled {
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

/// [`base`]'s map with Mountain against the Armory's exit side, C: tiles (11, 10) and (11, 11). A unit let out stays in
/// the doorway, fully inside the footprint, and every unit ready after it waits inside (`core/entities.md > Structure
/// Type > Exit Doorway` 5: "A building or cliff right against the exit side leaves the first unit stuck in the doorway
/// and the building effectively stalled").
fn walled_in() -> ScenarioBuilder {
    let mut rows: Vec<Vec<u8>> = (0..22).map(|_| vec![b'.'; 30]).collect();
    rows[10][11] = b'M';
    rows[11][11] = b'M';
    let rows: Vec<String> = rows.into_iter().map(|row| String::from_utf8(row).unwrap()).collect();
    cults(&rows.join("\n")).spawn(ARMORY, Some(P1), tile(12, 10))
}

/// The brief's mixed evacuation (slice M5-F3; design change 0349 point 3): an Armory whose exit is walled off, so
/// that the Recruit a first Release let out stands in its doorway, holds a trained Militant waiting for the doorway,
/// a second Militant queued behind it, a Release queued for its Constitution Recruit, and two free Recruits.
/// Destroyed, it lets them all out at once on its footprint: the waiting Militant as a Militant, with no refund and no
/// cancellation (its training was complete), with the Recruit's full HP; the second Militant's entry and the Release
/// refunded in full, with their cancellations, their Recruits out as Recruits (the Constitution one augmented still);
/// the free ones as they were. All idle; the Recruit in the doorway stays where it is.
#[test]
fn a_trained_militant_waiting_for_the_doorway_comes_out_a_militant_the_rest_refunded() {
    // Player 1 keeps a Storage (2), and so isn't defeated with the Armory.
    let mut t = walled_in().spawn(ObjectKind::Storage, Some(P1), tile(2, 2)).start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, 6, 3, Vec2::new(1150.0, 704.0));
    augment(&mut t, 2, recruits[5], RecruitKind::Constitution);
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    fill(&mut t, A1, 6);
    let order = inside(&t, A1);
    let constitution = recruits[5];
    let pool: Vec<ObjectId> = order.iter().copied().filter(|&unit| unit != constitution).collect();
    let start = t.sim().tick() + 1;
    t.at(start, P1, release(&[A1], RecruitKind::Plain));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start, P1, release(&[A1], RecruitKind::Constitution));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 40);
    assert_eq!(reserved(&t, A1), [[pool[0]], [pool[1]], [pool[2]], [constitution]]);
    // The first Release's Recruit comes out into the doorway and stays there; the first Militant's training ends, and
    // it waits inside for the doorway.
    let (in_doorway, militant, queued) = (pool[0], pool[1], pool[2]);
    let waiting = |t: &TestSim| {
        let status = t.sim().production_status(A1);
        status.is_some_and(|status| status.waiting && status.entry == sc_sim::QueueEntry::Unit(MILITANT))
    };
    while !waiting(&t) {
        assert!(t.sim().tick() < start + 100, "the first Militant training");
        step_checked(&mut t);
    }
    run_for(&mut t, 8);
    assert_eq!(
        exits(&t).iter().map(|&(_, unit)| unit).collect::<Vec<_>>(),
        [in_doorway]
    );
    let doorway = t.sim().object(in_doorway).expect("in the doorway").pos;
    assert!(waiting(&t) && produced(&t).is_empty(), "the Militant still waits");
    assert!(t.sim().held_unit(militant).is_some_and(|held| held.leaving.is_some()));
    let free: Vec<ObjectId> = pool[3..].to_vec();
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    let out: Vec<ObjectId> = order.iter().copied().filter(|&unit| unit != in_doorway).collect();
    assert_eq!(
        evacuated_in(&t, gone, A1),
        out,
        "all five inside, at once, longest inside first"
    );
    let kind_changes: Vec<&SimEvent> = (t.events().iter())
        .filter(|(at, e)| *at == gone && matches!(e, SimEvent::KindChanged { .. }))
        .map(|(_, e)| e)
        .collect();
    assert_eq!(
        kind_changes,
        [&SimEvent::KindChanged {
            id: militant,
            from: RECRUIT,
            to: MILITANT
        }]
    );
    let o = t.sim().object(militant).expect("evacuated");
    assert_eq!(
        (o.kind, o.hp),
        (MILITANT, o.max_hp()),
        "a Militant, the Recruit's full HP"
    );
    for &unit in [queued, constitution].iter().chain(&free) {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!(
            (o.kind, o.hp),
            (RECRUIT, Some(full_hp(unit, constitution))),
            "{unit:?} as a Recruit"
        );
    }
    for &unit in &out {
        idle_on_the_footprint(&t, unit);
    }
    let augmented = t.sim().object(constitution).unwrap().augmentation;
    assert_eq!(augmented, Augmentation::of(RecruitKind::Constitution));
    // 20 for the trained Militant, which stays paid; the second Militant's 20 back, the Release free.
    assert_eq!(bank(&t, P1), BANK - 20);
    assert_eq!(
        cancellations_in(&t, gone),
        [
            SimEvent::ProductionCancelled {
                producer: A1,
                kind: MILITANT,
                refund: 20
            },
            SimEvent::ReleaseCancelled {
                producer: A1,
                unit: constitution
            },
        ]
    );
    let o = t.sim().object(in_doorway).unwrap();
    assert_eq!(
        (o.kind, o.pos, o.mover.as_ref().unwrap().exit),
        (RECRUIT, doorway, None)
    );
    run_for(&mut t, 64);
    assert!(produced(&t).is_empty(), "evacuated, not produced");
    assert!(!t.events().iter().any(|(_, e)| matches!(e, SimEvent::Died { .. })));
    assert!(t.sim().held().next().is_none());
    for &unit in &out {
        assert!(t.sim().object(unit).unwrap().orders.as_ref().unwrap().is_idle());
    }
}
