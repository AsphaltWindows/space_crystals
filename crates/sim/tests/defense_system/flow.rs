//! The Defense System's one slot (design change 0392, `factions/gdo/structures/defense_system.md > Placeholder
//! stats`: "**Works like the Extraction Facility** [...] one slot, Q, no build menu"), on M2-D1's constructor
//! (`sc_sim::construction`): "**Idle**: **Q: Build Missile Turret** (100 Space Crystals, starts construction)";
//! "**Constructing**: [...] **X: Cancel Construction** (full refund)"; "**Ready**: **Q: Place Missile Turret** enters
//! AwaitingPlacement (ghost within the GDO build area, as for the Deployment Center's buildings); **X: Cancel Ready
//! Turret** (75% refund, rounded down)".

use sc_sim::{PlacementProblem, WholeProblem};

use super::*;

/// The `ConstructionCancelled` events of the latest tick: constructor, building, refund.
fn cancellations(t: &TestSim) -> Vec<(u32, ObjectKind, u32)> {
    (t.sim().events().iter())
        .filter_map(|e| match *e {
            SimEvent::ConstructionCancelled {
                constructor,
                kind,
                refund,
            } => Some((constructor.0, kind, refund)),
            _ => None,
        })
        .collect()
}

/// The plan's acceptance, "build, cancel (full refund), ready, cancel ready (75%, rounded down), and placing inside
/// the build area only", and 0392's Verification, "the same for a Missile Turret at the Defense System": the Defense
/// System starts a turret for 100, from its command's tick; it constructs nothing else, and nothing more while busy
/// (`Reject::ConstructorBusy`), and no one else constructs a turret. Cancelled, it gives the 100 back; started again it
/// is ready at the end of its 240th tick; cancelled ready, it gives 75 back. Ready a third time, it can't be placed
/// outside the GDO build area (`WholeProblem::OutsideBuildArea`), and stays ready; placed inside it, it is a Missile
/// Turret of player 1's, complete, and the Defense System is idle again.
#[test]
fn the_defense_system_builds_one_turret_at_a_time_as_the_extraction_facility_does() {
    let mut t = started(base());
    let ds = |command| construction(DS, command);
    assert_eq!(state(&t, DS), Constructor::Idle);
    let started_at = next(&mut t, ds(Construct(TURRET)));
    assert_eq!(bank(&t), 1000 - TURRET_COST);
    assert_eq!(state(&t, DS).current(), Some(TURRET));
    assert_eq!(state(&t, DS).frames_done(), Some(1), "the command's own tick the first");
    assert_eq!(
        gate(&t, P1, ds(Construct(TURRET))),
        Err(Reject::ConstructorBusy(id(DS)))
    );
    for other in [ObjectKind::Generator, ObjectKind::RadarStation, DEFENSE] {
        assert_eq!(gate(&t, P1, ds(Construct(other))), Err(Reject::CannotConstruct(other)));
    }
    let by_dc = construction(DC, Construct(TURRET));
    assert_eq!(gate(&t, P1, by_dc), Err(Reject::CannotConstruct(TURRET)));
    t.run_until(started_at + 20);
    next(&mut t, ds(CancelConstruction));
    assert_eq!(cancellations(&t), [(DS, TURRET, TURRET_COST)], "a full refund");
    assert_eq!((state(&t, DS), bank(&t)), (Constructor::Idle, 1000));

    let again = next(&mut t, ds(Construct(TURRET)));
    until_ready(&mut t, DS);
    assert_eq!(completions(&t), [(again + TURRET_TICKS - 1, DS, TURRET)]);
    assert_eq!(state(&t, DS), Constructor::Ready { kind: TURRET });
    assert_eq!(
        gate(&t, P1, ds(Construct(TURRET))),
        Err(Reject::ConstructorBusy(id(DS)))
    );
    assert_eq!(gate(&t, P1, ds(CancelConstruction)), Err(Reject::NotConstructing));
    next(&mut t, ds(CancelReady));
    assert_eq!(cancellations(&t), [(DS, TURRET, 75)], "75%, rounded down");
    assert_eq!((state(&t, DS), bank(&t)), (Constructor::Idle, 975));

    next(&mut t, ds(Construct(TURRET)));
    until_ready(&mut t, DS);
    let outside = TilePos::new(40, 24);
    let problems = t.sim().placement_check(P1, TURRET, outside, Default::default());
    let first = PlacementProblem::Whole(WholeProblem::OutsideBuildArea);
    assert_eq!(problems.first_problem(), Some(first), "{problems:?}");
    assert_eq!(gate(&t, P1, ds(place(outside))), Err(Reject::Unplaceable(first)));
    next(&mut t, ds(place(outside)));
    assert_eq!(state(&t, DS), Constructor::Ready { kind: TURRET }, "still ready");
    let inside = TilePos::new(13, 9);
    let ids = t.sim().objects().map(|o| o.id.0).max().unwrap();
    next(&mut t, ds(place(inside)));
    let turret = t.sim().object(id(ids + 1)).expect("placed");
    assert_eq!((turret.kind, turret.owner), (TURRET, Some(P1)));
    assert_eq!(turret.placement.map(|p| p.origin), Some(inside));
    assert!(turret.is_operational(), "complete as it is placed");
    assert_eq!(state(&t, DS), Constructor::Idle);
    assert_eq!(bank(&t), 875, "nothing more paid");
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (70, 70), "its Power −10");
}

/// The plan's acceptance under the penalty: the Defense System is a consumer (Power −20), so at 30 generated for 51
/// consumed it constructs at the ratio, as the Extraction Facility (design change 0089): a turret ordered in tick 2 is
/// ready at the end of tick 409, `⌈240 × 65536 / 38551⌉` = 408 ticks, while the Deployment Center builds a Generator at
/// full speed, in 240.
#[test]
fn at_low_power_its_construction_slows_by_the_ratio() {
    let mut t = started(super::turret::short());
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (30, 51));
    let c = t.sim().tick() + 1;
    t.at(c, P1, construction(super::turret::SHORT_DS, Construct(TURRET)))
        .at(c, P1, construction(3, Construct(ObjectKind::Generator)))
        .run_until(c + 500);
    let ticks = (u64::from(TURRET_TICKS) * 65_536).div_ceil(38_551) as Tick;
    assert_eq!(ticks, 408);
    assert_eq!(
        completions(&t),
        [
            (c + 239, 3, ObjectKind::Generator),
            (c + ticks - 1, super::turret::SHORT_DS, TURRET)
        ]
    );
}
