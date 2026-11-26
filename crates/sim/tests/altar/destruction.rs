//! An Altar destroyed while it augments (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on
//! Destruction`: "**Everyone inside is evacuated alive**: units in the internal pool, units reserved by queued orders,
//! the unit currently being trained"; "a unit mid-augmentation does not receive the Augmentation"; "**Orders are
//! cancelled with a full refund**"; design change 0349, slice M5-F3's evacuation). Where they come out is
//! `tests/holding/`'s.

use sc_sim::QueueEntry;

use super::*;

/// The middle of the tiles of [`base`]'s Altar, x 768..960 and y 640..768.
const FOOTPRINT: (Vec2, Vec2) = (Vec2::new(768.0, 640.0), Vec2::new(960.0, 768.0));

/// Player 1's Storage (2), which keeps them in the game when the Altar goes (`core/game_rules.md > Defeat
/// Condition`).
fn with_a_storage(scenario: ScenarioBuilder) -> ScenarioBuilder {
    scenario.spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
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
            *at == tick && matches!(e, SimEvent::AugmentCancelled { .. } | SimEvent::ReleaseCancelled { .. })
        })
        .map(|(_, e)| e.clone())
        .collect()
}

/// Unit `unit` is on the map inside the Altar's former footprint, idle.
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

/// The brief's acceptance: "Destruction: evacuates everyone, unaugmented if mid-augmentation, and refunds every order."
/// Both for a Recruit, under way, Enhanced Constitution for a Militant and a Release of a Gunner behind it, and a free
/// Disciple: the Altar goes 50 ticks into the first order (of 96, design change 0391), and the four come out on its
/// footprint, all in the tick it goes, longest inside first, as they went in, the Recruit without the Augmentations it
/// was receiving; the 75 Space Crystals come back, each entry with its cancellation event, in the queue's order.
/// Nothing is augmented, no one dies, no one is left inside.
#[test]
fn an_altar_destroyed_mid_augmentation_lets_everyone_out_unaugmented_and_refunds_every_order() {
    let (mut t, units) = spawned_on(with_a_storage(base()), &[RECRUIT, MILITANT, GUNNER, DISCIPLE]);
    let order = go_in(&mut t, &units);
    let [recruit, militant, gunner, _] = units[..] else {
        unreachable!()
    };
    let start = t.sim().tick() + 1;
    let (both, enhanced) = (AugmentOrder::Both, AugmentOrder::EnhancedConstitution);
    t.at(start, P1, augment(&[A1], RECRUIT, RecruitKind::Plain, both));
    t.at(start, P1, augment(&[A1], MILITANT, RecruitKind::Plain, enhanced));
    t.at(start, P1, release(&[A1], GUNNER, RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 75);
    assert_eq!(reserved(&t, A1), [[recruit], [militant], [gunner]]);
    let gone = start + 50;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    assert_eq!(
        evacuated_in(&t, gone, A1),
        order,
        "all four at once, longest inside first"
    );
    for &unit in &order {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!(o.augmentation, Augmentation::default(), "{unit:?} unaugmented");
        assert_eq!(hp(&t, unit).0, hp(&t, unit).1, "{unit:?} unharmed");
        idle_on_the_footprint(&t, unit);
    }
    assert_eq!(hp(&t, recruit), (40.0, 40.0));
    assert_eq!(bank(&t, P1), BANK);
    assert_eq!(
        cancellations_in(&t, gone),
        [
            SimEvent::AugmentCancelled {
                producer: A1,
                unit: recruit,
                order: both,
                refund: 50
            },
            SimEvent::AugmentCancelled {
                producer: A1,
                unit: militant,
                order: enhanced,
                refund: 25
            },
            SimEvent::ReleaseCancelled {
                producer: A1,
                unit: gunner
            },
        ]
    );
    run_for(&mut t, 400);
    assert!(produced(&t).is_empty() && exits(&t).is_empty());
    assert!(!t.events().iter().any(|(_, e)| matches!(e, SimEvent::Died { .. })));
    assert!(t.sim().held().next().is_none());
    assert!(
        order
            .iter()
            .all(|&unit| augmentation(&t, unit) == Augmentation::default())
    );
}

/// [`base`]'s map with Mountain against the Altar's exit side, C: tiles (11, 10) and (11, 11). A unit let out stays in
/// the doorway, fully inside the footprint, and every unit ready after it waits inside (`core/entities.md > Structure
/// Type > Exit Doorway` 5: "A building or cliff right against the exit side leaves the first unit stuck in the doorway
/// and the building effectively stalled").
fn walled_in() -> ScenarioBuilder {
    let mut rows: Vec<Vec<u8>> = (0..22).map(|_| vec![b'.'; 30]).collect();
    rows[10][11] = b'M';
    rows[11][11] = b'M';
    let rows: Vec<String> = rows.into_iter().map(|row| String::from_utf8(row).unwrap()).collect();
    cults(&rows.join("\n")).spawn(ALTAR, Some(P1), tile(12, 10))
}

/// **Placeholder (not in the design: an augmentation finished and waiting for the doorway as its building goes)**, read
/// as a training is (design change 0349 point 3: "**A unit that had finished training** and was waiting for the
/// doorway comes out **as the trained unit**, with no refund"): an Altar whose exit is walled off, so that the Recruit
/// a first Release let out stands in its doorway, holds a Militant whose Enhanced Constitution is done, waiting for the
/// doorway, still unaugmented (an order's Augmentations are the unit's as it comes out), and a second Militant whose
/// Improved Metabolism is queued behind. Destroyed, the Altar lets them out at once on its footprint: the first
/// Militant with Enhanced Constitution, at its 60 HP, unrefunded and uncancelled; the second as it went in, its 25
/// Space Crystals back. The Recruit in the doorway stays where it is.
#[test]
fn a_unit_whose_augmentation_is_done_comes_out_augmented_and_unrefunded() {
    let (mut t, units) = spawned_on(with_a_storage(walled_in()), &[RECRUIT, MILITANT, MILITANT]);
    let order = go_in(&mut t, &units);
    let [recruit] = inside_of(&t, A1, RECRUIT, RecruitKind::Plain)[..] else {
        unreachable!()
    };
    let militants = inside_of(&t, A1, MILITANT, RecruitKind::Plain);
    let (enhanced, metabolism) = (AugmentOrder::EnhancedConstitution, AugmentOrder::ImprovedMetabolism);
    let start = t.sim().tick() + 1;
    t.at(start, P1, release(&[A1], RECRUIT, RecruitKind::Plain));
    t.at(start, P1, augment(&[A1], MILITANT, RecruitKind::Plain, enhanced));
    t.at(start, P1, augment(&[A1], MILITANT, RecruitKind::Plain, metabolism));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 50);
    assert_eq!(reserved(&t, A1), [[recruit], [militants[0]], [militants[1]]]);
    let waiting = |t: &TestSim| {
        let status = t.sim().production_status(A1);
        status.is_some_and(|status| status.waiting && status.entry == QueueEntry::Augment { order: enhanced })
    };
    while !waiting(&t) {
        assert!(t.sim().tick() < start + 200, "the first order under way");
        step_checked(&mut t);
    }
    run_for(&mut t, 8);
    let out: Vec<ObjectId> = exits(&t).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(out, [recruit], "in the doorway");
    let doorway = t.sim().object(recruit).expect("in the doorway").pos;
    assert!(waiting(&t) && produced(&t).is_empty(), "the Militant still waits");
    assert!(
        t.sim()
            .held_unit(militants[0])
            .is_some_and(|held| held.leaving.is_some())
    );
    assert_eq!(augmentation(&t, militants[0]), Augmentation::default(), "not yet");
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    let inside_then: Vec<ObjectId> = order.iter().copied().filter(|&unit| unit != recruit).collect();
    assert_eq!(evacuated_in(&t, gone, A1), inside_then, "longest inside first");
    assert_eq!(
        augmentation(&t, militants[0]),
        Augmentation::of(RecruitKind::Constitution)
    );
    assert_eq!(hp(&t, militants[0]), (60.0, 60.0));
    assert_eq!(augmentation(&t, militants[1]), Augmentation::default());
    assert_eq!(hp(&t, militants[1]), (40.0, 40.0));
    for unit in inside_then {
        idle_on_the_footprint(&t, unit);
    }
    assert_eq!(bank(&t, P1), BANK - 25, "the done order unrefunded");
    assert_eq!(
        cancellations_in(&t, gone),
        [SimEvent::AugmentCancelled {
            producer: A1,
            unit: militants[1],
            order: metabolism,
            refund: 25
        }]
    );
    assert_eq!(t.sim().object(recruit).expect("still there").pos, doorway);
    assert!(produced(&t).is_empty(), "evacuated, not produced");
}
