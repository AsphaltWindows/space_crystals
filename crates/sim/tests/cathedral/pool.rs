//! The pool's shared paths, re-run on the Cathedral (`factions/cults/structures/cathedral.md > Placeholder stats`:
//! "Panel like the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject
//! All): **Q: Train Disciple**, **A: Release** in the kind menu; **A: Eject All**"; "InternalRecruitCapacity 10,
//! counting reserved Recruits; so at most 10 orders queued"): Release (design change 0337), Eject All and the reserved
//! units, the capacity (0347 point 5), the evacuation on destruction (0349, `factions/cults/economy.md > Cults
//! Building Mechanics > Evacuation on Destruction`), and the right-click that sends Recruits in.

use super::*;

/// [`base`]'s Cathedral's footprint, x 768..960 and y 640..768.
const FOOTPRINT: (Vec2, Vec2) = (Vec2::new(768.0, 640.0), Vec2::new(960.0, 768.0));

/// A Release (design change 0337: "a **no-op order**: **free**, very fast (**4 frames**, placeholder), queued like any
/// order; when it completes the reserved unit walks out of the exit **unchanged**"): the Constitution Recruit of three
/// comes out of side C a Recruit with its augmentation, 4 ticks after it is queued, the command's counting; a second
/// Release finds none of its kind left. The Plain ones stay inside.
#[test]
fn a_release_lets_a_recruit_out_unchanged_after_4_ticks() {
    let mut t = base().start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, 3, 3, Vec2::new(1150.0, 704.0));
    let constitution = recruits[2];
    augment(&mut t, 2, constitution, RecruitKind::Constitution);
    t.at(2, P1, enter(&recruits, C1));
    run_checked(&mut t, 2);
    fill(&mut t, C1, 3);
    assert_eq!(t.sim().pool_counts(C1), [2, 1, 0, 0], "W shows 1");
    let start = t.sim().tick() + 1;
    t.at(start, P1, release(&[C1], RecruitKind::Constitution));
    run_checked(&mut t, start);
    assert_eq!(
        (bank(&t, P1), reserved(&t, C1)),
        (BANK, vec![vec![constitution]]),
        "free"
    );
    assert_eq!(
        validate(&t, release(&[C1], RecruitKind::Constitution)),
        Err(Reject::NoRecruit(RecruitKind::Constitution)),
        "the second press is greyed"
    );
    run_checked(&mut t, start + 2);
    assert!(exits(&t).is_empty());
    step_checked(&mut t);
    assert_eq!(exits(&t), [(start + 3, constitution)], "4 ticks");
    let o = t.sim().object(constitution).expect("out");
    assert_eq!(
        (o.kind, o.augmentation, o.energy),
        (RECRUIT, Augmentation::of(RecruitKind::Constitution), None)
    );
    assert!(o.pos.x < EXIT.x + 12.5, "out of side C: {}", o.pos);
    assert_eq!(produced(&t), [], "nothing trained");
    assert_eq!(inside(&t, C1).len(), 2);
}

/// `factions/cults/structures/armory.md > Eject All`, the pool buildings' (design change 0337): "Recruits already
/// reserved by a queued training order are not ejected and remain committed to that order". Two of four Recruits
/// reserved by two Disciples, Eject All lets the other two out, and the reserved ones stay to be trained in turn; with
/// only reserved Recruits inside, Eject All has nothing to eject.
#[test]
fn eject_all_leaves_the_reserved_recruits_inside() {
    let (mut t, order) = filled(4);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    t.at(start + 1, P1, eject_all(&[C1]));
    run_checked(&mut t, start + 1);
    assert_eq!(reserved(&t, C1), [[order[0]], [order[1]]]);
    run_for(&mut t, 24);
    let ejected: Vec<ObjectId> = exits(&t).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(ejected, order[2..]);
    assert_eq!(inside(&t, C1), order[..2]);
    assert_eq!(validate(&t, eject_all(&[C1])), Err(Reject::NothingInside));
    run_for(&mut t, 2 * FRAMES);
    let trained: Vec<ObjectId> = produced(&t).iter().map(|&(_, _, unit)| unit).collect();
    assert_eq!(trained, order[..2]);
    assert!(inside(&t, C1).is_empty());
}

/// "InternalRecruitCapacity 10, counting reserved Recruits; so at most 10 orders queued (bounded by capacity: reserved
/// units count toward it; Q0104)": ten Disciples queued reserve the ten Recruits, an eleventh finds none, and an
/// eleventh Recruit's Enter fails for want of room (`SimEvent::EnterFailed` with `full`).
#[test]
fn ten_queued_fill_the_cathedral_and_an_eleventh_recruit_is_refused() {
    let (mut t, order) = filled(10);
    t.debug_at(t.sim().tick() + 1, crystals(P1, 10 * PRICE));
    let start = t.sim().tick() + 2;
    for _ in 0..10 {
        t.at(start, P1, train(&[C1], RecruitKind::Plain));
    }
    let eleventh = id(next_id(&t));
    t.spawn_unit_at(start, RECRUIT, P1, Vec2::new(1060.0, 704.0));
    t.at(start + 1, P1, enter(&[eleventh], C1));
    run_checked(&mut t, start + 1);
    assert_eq!(
        reserved(&t, C1),
        order.iter().map(|&unit| vec![unit]).collect::<Vec<_>>()
    );
    assert_eq!((bank(&t, P1), t.sim().room(C1)), (0, 0), "the reserved ones count");
    assert_eq!(
        validate(&t, train(&[C1], RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
    let failed = |t: &TestSim| {
        (t.events().iter()).any(|(_, e)| {
            *e == SimEvent::EnterFailed {
                unit: eleventh,
                target: C1,
                full: true,
            }
        })
    };
    while !failed(&t) {
        assert!(t.sim().tick() < start + 60, "the eleventh's Enter fails");
        step_checked(&mut t);
    }
    assert!(
        t.sim()
            .object(eleventh)
            .is_some_and(|o| o.orders.as_ref().unwrap().is_idle())
    );
    assert!(produced(&t).is_empty());
}

/// The evacuation on destruction (design change 0349): "**Everyone inside is evacuated alive** [...] a Recruit
/// mid-training comes out as a Recruit"; "**A unit that had finished training** and was waiting for the doorway comes
/// out **as the trained unit**, with no refund"; "**Orders are cancelled with a full refund**". The Cathedral's exit is
/// walled off by Mountain, so a first Release's Recruit stays in its doorway: a Disciple trained then waits inside for
/// it, a second Disciple is queued behind it, and a free Recruit waits. Destroyed, it lets the three out at once on its
/// footprint: the waiting Disciple a Disciple, with its start's energy (and the tick's regeneration), unrefunded; the
/// second's Recruit a Recruit, its 75 Space Crystals back; the free one as it was.
#[test]
fn a_cathedral_destroyed_lets_everyone_out_a_trained_disciple_a_disciple() {
    let mut rows: Vec<Vec<u8>> = (0..22).map(|_| vec![b'.'; 30]).collect();
    rows[10][11] = b'M';
    rows[11][11] = b'M';
    let rows: Vec<String> = rows.into_iter().map(|row| String::from_utf8(row).unwrap()).collect();
    let mut t = cults(&rows.join("\n"))
        .spawn(CATHEDRAL, Some(P1), tile(12, 10))
        .spawn(TEMPLE, Some(P1), tile(2, 2))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, 4, 4, Vec2::new(1150.0, 704.0));
    t.at(2, P1, enter(&recruits, C1));
    run_checked(&mut t, 2);
    fill(&mut t, C1, 4);
    let order = inside(&t, C1);
    let (in_doorway, trained, training, free) = (order[0], order[1], order[2], order[3]);
    let start = t.sim().tick() + 1;
    t.at(start, P1, release(&[C1], RecruitKind::Plain));
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 2 * PRICE);
    let waiting = |t: &TestSim| {
        let status = t.sim().production_status(C1);
        status.is_some_and(|status| status.waiting && status.entry == sc_sim::QueueEntry::Unit(DISCIPLE))
    };
    while !waiting(&t) {
        assert!(t.sim().tick() < start + FRAMES + 40, "the first Disciple training");
        step_checked(&mut t);
    }
    run_for(&mut t, 8);
    assert_eq!(
        exits(&t).iter().map(|&(_, unit)| unit).collect::<Vec<_>>(),
        [in_doorway]
    );
    assert!(t.sim().held_unit(trained).is_some_and(|held| held.leaving.is_some()));
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, C1);
    run_checked(&mut t, gone);
    let evacuated: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(at, e)| match *e {
            SimEvent::Evacuated { unit, building } if *at == gone && building == C1 => Some(unit),
            _ => None,
        })
        .collect();
    assert_eq!(evacuated, [trained, training, free], "at once, longest inside first");
    let o = t.sim().object(trained).expect("evacuated");
    assert_eq!(
        (o.kind, o.hp),
        (DISCIPLE, o.max_hp()),
        "a Disciple, the Recruit's full HP"
    );
    assert_eq!(
        energy(&t, trained),
        Some(50.031_25),
        "its start and a tick's 0.5 / 16: trained before the energy step (`sc_sim::energy`)"
    );
    for unit in [training, free] {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!((o.kind, o.energy), (RECRUIT, None), "{unit:?} as a Recruit");
    }
    for &unit in &evacuated {
        let o = t.sim().object(unit).unwrap();
        let half = o.def().half_extents().unwrap();
        let (lo, hi) = FOOTPRINT;
        assert!(
            (o.pos - half).cmpge(lo).all() && (o.pos + half).cmple(hi).all(),
            "{unit:?} at {} inside the footprint",
            o.pos
        );
        assert!(o.orders.as_ref().unwrap().is_idle(), "{unit:?} idle");
    }
    assert_eq!(
        bank(&t, P1),
        BANK - PRICE,
        "the trained one stays paid, the other refunded"
    );
    let cancelled = SimEvent::ProductionCancelled {
        producer: C1,
        kind: DISCIPLE,
        refund: PRICE,
    };
    let cancellations: Vec<&SimEvent> = (t.events().iter())
        .filter(|(at, e)| *at == gone && matches!(e, SimEvent::ProductionCancelled { .. }))
        .map(|(_, e)| e)
        .collect();
    assert_eq!(cancellations, [&cancelled]);
    run_for(&mut t, 64);
    assert!(produced(&t).is_empty(), "evacuated, not produced");
    assert!(!t.events().iter().any(|(_, e)| matches!(e, SimEvent::Died { .. })));
    assert!(t.sim().held().next().is_none());
}

/// Right-click Enter Cathedral for Recruits (`factions/cults/units/recruit.md > Right-Click Resolution`: "Damaged own
/// building or vehicle (completed) → Repair, **unless the building can be entered** by the selection: then **Enter**
/// wins"; `systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits only"): a Recruit's default
/// order on its Cathedral is Enter, and still Enter once an enemy has damaged it; a Disciple, which the Cathedral does
/// not take, moves to it.
#[test]
fn a_right_click_on_the_cathedral_is_enter_for_a_recruit_damaged_or_not() {
    let mut t = base().start();
    t.without_auto_targeting();
    let (recruit, disciple, enemy) = (id(3), id(4), id(5));
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1150.0, 704.0));
    t.spawn_unit_at(1, DISCIPLE, P1, Vec2::new(1150.0, 600.0));
    t.spawn_unit_at(1, RECRUIT, P2, Vec2::new(1000.0, 820.0));
    run_checked(&mut t, 1);
    let on_cathedral = |t: &TestSim, unit| t.sim().default_order(unit, Target::Object(C1));
    assert_eq!(on_cathedral(&t, recruit), Some(OrderKind::Enter(C1)));
    assert_eq!(on_cathedral(&t, disciple), Some(OrderKind::Move(Target::Object(C1))));
    t.at(
        2,
        P2,
        Command::Order {
            units: vec![enemy],
            kind: OrderKind::Attack(C1),
            queue: false,
        },
    );
    let damaged = |t: &TestSim| t.sim().object(C1).is_some_and(|o| o.hp < o.max_hp());
    while !damaged(&t) {
        assert!(t.sim().tick() < 120, "the enemy Recruit hits the Cathedral");
        step_checked(&mut t);
    }
    assert_eq!(
        on_cathedral(&t, recruit),
        Some(OrderKind::Enter(C1)),
        "Enter wins over Repair"
    );
    assert_eq!(on_cathedral(&t, disciple), Some(OrderKind::Move(Target::Object(C1))));
}
