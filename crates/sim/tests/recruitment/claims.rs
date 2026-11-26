//! Halls that share tiles, a Hall destroyed, a unit of a lineage lost, and the HUD's figures
//! (`factions/cults/economy.md > RecruitmentHall > TileClaiming`, `factions/cults/economy.md > RecruitmentHall > On
//! Destruction`, `factions/cults/economy.md > RecruitmentHall > Cults Unit Control HUD Display`).

use sc_sim::{DebugCommand, ObjectId};

use super::*;

/// Plan acceptance, overlapping Halls: "the second takes the freed tiles in the same tick, and its capacity rises"
/// (`factions/cults/economy.md > RecruitmentHall > TileClaiming`: "if multiple halls contest a newly freed tile, the
/// one that was built first claims it"). Hall 1's area is tiles (5, 5) to (14, 14), Hall 2's (11, 5) to (20, 14):
/// Hall 1 has the 40 they share. Hall 1 gone in tick 50, Hall 2's Recruit gains 100 tiles in tick 50 already.
#[test]
fn the_second_hall_takes_the_freed_tiles_in_the_same_tick() {
    let (first, second) = (ObjectId(1), ObjectId(2));
    let mut t = flat(30, 20)
        .spawn(HALL, Some(P1), tile(8, 8))
        .spawn(HALL, Some(P1), tile(14, 8))
        .start();
    t.debug_at(50, DebugCommand::Remove { id: first });
    t.run_until(49);
    assert_eq!(
        (hall_state(&t, first).claimed, hall_state(&t, second).claimed),
        (100, 60)
    );
    assert_eq!(hall_state(&t, second).capacity.floor(), 12);
    assert_eq!(progress(&t, second), Some(49 * 60));
    assert_eq!(unit_control(&t, P1), (0, 32));
    t.run(1);
    assert_eq!(hall_state(&t, second).claimed, 100);
    assert_eq!(hall_state(&t, second).capacity.floor(), 20);
    assert_eq!(
        progress(&t, second),
        Some(49 * 60 + 100),
        "the freed tiles count in tick 50"
    );
    assert_eq!(unit_control(&t, P1), (0, 20));
}

/// Plan acceptance, a destroyed Hall: "its units still in used, its capacity out of available"
/// (`factions/cults/economy.md > RecruitmentHall > On Destruction`: "Surviving units produced by this hall remain on
/// the field and continue to count toward the global usage sum. Its LocalUnitControlCapacity is removed from the global
/// sum"). The other Hall recruits by its own usage alone, so the HUD ends over capacity, 23/20.
#[test]
fn a_destroyed_halls_recruits_still_count_and_its_capacity_goes() {
    let (lost, kept) = (ObjectId(1), ObjectId(2));
    let mut t = flat(48, 20)
        .spawn(HALL, Some(P1), tile(8, 8))
        .spawn(HALL, Some(P1), tile(30, 8))
        .start();
    t.debug_at(3 * FRAMES + 10, DebugCommand::Remove { id: lost });
    t.run_until(3 * FRAMES + 9);
    assert_eq!(unit_control(&t, P1), (6, 40));
    t.run(1);
    assert_eq!(
        unit_control(&t, P1),
        (6, 20),
        "Hall 1's three Recruits are orphaned points"
    );
    assert_eq!(hall_state(&t, kept).usage, 3);
    t.run_until(20 * FRAMES + 10);
    assert_eq!(ticks(&t, kept).len(), 20, "the orphans don't count against Hall 2");
    assert_eq!(unit_control(&t, P1), (23, 20));
}

/// **Placeholder (Q0100 point 4)**: a Hall at capacity makes nothing until a unit of its lineage dies; then its
/// production starts from 0% in that tick. X has nothing to cancel meanwhile (`factions/cults/economy.md >
/// RecruitmentHall > ObjectInterfaceState[RecruitmentHall]`: "Only available if ProductionProgress is not None").
#[test]
fn a_lost_recruit_restarts_a_full_hall() {
    let hall = ObjectId(1);
    let mut t = flat(40, 40).spawn(HALL, Some(P1), tile(18, 18)).start();
    t.run_until(20 * FRAMES + 50);
    assert_eq!(progress(&t, hall), None);
    let cancel = production(&[hall], ProductionCommand::CancelProduction);
    let x = |t: &TestSim| {
        t.sim().validate(&sc_sim::PlayerCommand {
            player: P1,
            command: cancel.clone(),
        })
    };
    assert_eq!(x(&t), Err(sc_sim::Reject::NothingQueued));
    let (_, first) = made(&t, hall)[0];
    let lost = 20 * FRAMES + 60;
    t.debug_at(lost, DebugCommand::Remove { id: first });
    t.run_until(lost);
    assert_eq!(hall_state(&t, hall).usage, 19);
    assert_eq!(
        progress(&t, hall),
        Some(100),
        "started in the tick the Recruit was lost"
    );
    assert_eq!(x(&t), Ok(()));
    t.run_until(lost + FRAMES);
    assert_eq!(ticks(&t, hall)[20..], [lost + FRAMES - 1]);
    assert_eq!(unit_control(&t, P1), (20, 20));
}

/// **Placeholder (Q0101 point 2)**: a Cults unit no Hall made (a map's or a debug spawn's) counts its whole cost as
/// orphaned Unit Control, against no Hall; the Hall recruits up to its capacity all the same, and the HUD shows the
/// sum (`factions/cults/economy.md > RecruitmentHall > Cults Unit Control HUD Display`: "sum of all
/// LocalUnitControlUsage across all active Recruitment Halls and orphaned units").
#[test]
fn units_no_hall_made_are_orphaned_points() {
    let hall = ObjectId(1);
    let mut builder = flat(40, 30).spawn(HALL, Some(P1), tile(18, 8));
    for i in 0..6 {
        builder = builder.spawn(RECRUIT, Some(P1), centre(4 + 2 * i, 24));
    }
    let mut t = builder.spawn(ObjectKind::Militant, Some(P1), centre(20, 24)).start();
    t.run_until(1);
    assert_eq!(unit_control(&t, P1), (7, 20));
    assert_eq!(hall_state(&t, hall).usage, 0);
    assert!(t.sim().object(ObjectId(2)).expect("a Recruit").lineage.is_empty());
    t.run_until(FRAMES);
    assert_eq!(unit_control(&t, P1), (8, 20));
    assert_eq!(hall_state(&t, hall).usage, 1);
    // Another player's figures are their own: player 2 has no Hall and no unit.
    assert_eq!(unit_control(&t, P2), (0, 0));
}
