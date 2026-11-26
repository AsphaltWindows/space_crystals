//! Halls that share tiles, a Hall destroyed, a unit of a lineage lost, and the HUD's figures
//! (`factions/cults/structures/recruitment_hall.md > TileClaiming`, `factions/cults/structures/recruitment_hall.md > On
//! Destruction`, `factions/cults/structures/recruitment_hall.md > Cults Unit Control HUD Display`).

use sc_sim::{DebugCommand, ObjectId};

use super::*;

/// Plan acceptance, overlapping Halls: "the second takes the freed tiles in the same tick, and its capacity rises"
/// (`factions/cults/structures/recruitment_hall.md > TileClaiming`: "if multiple halls contest a newly freed tile, the
/// one that was built first claims it"). Hall 1's area is tiles (5, 5) to (14, 14), Hall 2's (11, 5) to (20, 14):
/// Hall 1, completed first (Hall 2 is a debug spawn's of tick 1, which recruits from tick 2), has the 40 they share.
/// Hall 1 gone in tick 50, Hall 2's Recruit gains 100 tiles in tick 50 already. Each Hall's Recruit in production
/// counts (design change 0342), and Hall 1's goes with it.
#[test]
fn the_second_hall_takes_the_freed_tiles_in_the_same_tick() {
    let (first, second) = (ObjectId(1), ObjectId(2));
    let mut t = flat(30, 20).spawn(HALL, Some(P1), tile(8, 8)).start();
    t.debug_at(
        1,
        DebugCommand::Spawn {
            kind: HALL,
            owner: Some(P1),
            at: tile(14, 8),
            orientation: Default::default(),
        },
    );
    t.debug_at(50, DebugCommand::Remove { id: first });
    t.run_until(49);
    assert_eq!(
        (hall_state(&t, first).claimed, hall_state(&t, second).claimed),
        (100, 60)
    );
    assert_eq!(hall_state(&t, second).capacity.floor(), 12);
    assert_eq!(progress(&t, second), Some(48 * 60));
    assert_eq!(unit_control(&t, P1), (2, 32));
    t.run(1);
    assert_eq!(hall_state(&t, second).claimed, 100);
    assert_eq!(hall_state(&t, second).capacity.floor(), 20);
    assert_eq!(
        progress(&t, second),
        Some(48 * 60 + 100),
        "the freed tiles count in tick 50"
    );
    assert_eq!(unit_control(&t, P1), (1, 20));
}

/// Design change 0341's tie rule on a map: two Halls completed in the same tick, a map's, split the 40 tiles they
/// share by the nearer centre, 2 columns each (`factions/cults/structures/recruitment_hall.md > TileClaiming`: "each
/// contested tile goes to the Hall whose centre is nearer"), whoever owns them; and each recruits at its 80 tiles,
/// 240 ticks a Recruit.
#[test]
fn halls_completed_together_split_the_shared_tiles() {
    let (first, second) = (ObjectId(1), ObjectId(2));
    let mut t = flat(30, 20)
        .spawn(HALL, Some(P1), tile(8, 8))
        .spawn(HALL, Some(P2), tile(14, 8))
        .start();
    t.run_until(240);
    assert_eq!(
        (hall_state(&t, first).claimed, hall_state(&t, second).claimed),
        (80, 80)
    );
    assert_eq!((unit_control(&t, P1), unit_control(&t, P2)), ((2, 16), (2, 16)));
    assert_eq!((ticks(&t, first), ticks(&t, second)), (vec![240], vec![240]));
}

/// Plan acceptance, a destroyed Hall: "its units still in used, its capacity out of available"
/// (`factions/cults/structures/recruitment_hall.md > On Destruction`: "Surviving units produced by this hall remain on
/// the field and continue to count toward the global usage sum. Its LocalUnitControlCapacity is removed from the global
/// sum"). The other Hall recruits by its own usage alone, so the HUD ends over capacity, 23/20. Each Hall's Recruit in
/// production counts (design change 0342); the lost Hall's goes with it.
#[test]
fn a_destroyed_halls_recruits_still_count_and_its_capacity_goes() {
    let (lost, kept) = (ObjectId(1), ObjectId(2));
    let mut t = flat(48, 20)
        .spawn(HALL, Some(P1), tile(8, 8))
        .spawn(HALL, Some(P1), tile(30, 8))
        .start();
    t.debug_at(3 * FRAMES + 10, DebugCommand::Remove { id: lost });
    t.run_until(3 * FRAMES + 9);
    assert_eq!(unit_control(&t, P1), (8, 40), "3 out and 1 in production each");
    t.run(1);
    assert_eq!(
        unit_control(&t, P1),
        (7, 20),
        "Hall 1's three Recruits are orphaned points"
    );
    assert_eq!(hall_state(&t, kept).usage, 4);
    t.run_until(20 * FRAMES + 10);
    assert_eq!(ticks(&t, kept).len(), 20, "the orphans don't count against Hall 2");
    assert_eq!(unit_control(&t, P1), (23, 20));
}

/// Design change 0342 point 4 (`factions/cults/structures/recruitment_hall.md > Production Behavior`: "When the Hall is
/// at capacity nothing is in production; when capacity frees up (a unit dies), the next Recruit starts from 0"): a
/// Hall at capacity makes nothing until a unit of its lineage dies; then its production starts from 0% in that tick,
/// and its usage is 20 again at once. There is no X, before or after (design change 0342: "Recruits can't be
/// cancelled").
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
    assert_eq!(hall_state(&t, hall).usage, 20, "19 and the one in production");
    assert_eq!(
        progress(&t, hall),
        Some(100),
        "started in the tick the Recruit was lost"
    );
    assert_eq!(x(&t), Err(sc_sim::Reject::NothingQueued));
    t.run_until(lost + FRAMES);
    assert_eq!(ticks(&t, hall)[20..], [lost + FRAMES - 1]);
    assert_eq!(unit_control(&t, P1), (20, 20));
}

/// Design change 0344 (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`: "Recruits no Hall
/// produced (placed by a custom map or spawned by a debug command [...]) are orphans from the start: they count in the
/// global display only, against no Hall"): a Cults unit no Hall made counts its whole cost as orphaned Unit Control;
/// the Hall recruits up to its capacity all the same, and the HUD shows the sum
/// (`factions/cults/structures/recruitment_hall.md > Cults Unit Control HUD Display`: "sum of all LocalUnitControlUsage
/// across all active Recruitment Halls and orphaned units"), the Hall's Recruit in production included (design change
/// 0342).
#[test]
fn units_no_hall_made_are_orphaned_points() {
    let hall = ObjectId(1);
    let mut builder = flat(40, 30).spawn(HALL, Some(P1), tile(18, 8));
    for i in 0..6 {
        builder = builder.spawn(RECRUIT, Some(P1), centre(4 + 2 * i, 24));
    }
    let mut t = builder.spawn(ObjectKind::Militant, Some(P1), centre(20, 24)).start();
    assert_eq!(unit_control(&t, P1), (7, 20));
    t.run_until(1);
    assert_eq!(unit_control(&t, P1), (8, 20));
    assert_eq!(hall_state(&t, hall).usage, 1);
    assert!(t.sim().object(ObjectId(2)).expect("a Recruit").lineage.is_empty());
    t.run_until(FRAMES);
    assert_eq!(unit_control(&t, P1), (9, 20));
    assert_eq!(hall_state(&t, hall).usage, 2);
    // Another player's figures are their own: player 2 has no Hall and no unit.
    assert_eq!(unit_control(&t, P2), (0, 0));
}
