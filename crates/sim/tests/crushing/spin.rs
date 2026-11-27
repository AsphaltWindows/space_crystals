//! Design change 0418's Verification (`systems/units.md > UnitCollision > Crushing`: "**Turning in place crushes**: a
//! crusher turning in place (Stationary + Turning) **crushes the crushable enemies its turning Silhouette sweeps
//! over**; they don't block the spin. Own and allied crushable units still block it (No friendly crushing): the spin
//! stops at contact. A crusher that is stationary and **not** turning stays a hard obstacle"). A Hand of Heaven stands
//! on (6, 4) facing east, and an infantry stands against the middle of its south side; the tank is sent due south,
//! which it spins to face first (from rest every turn is a spin, `docs/movement.md > 6.12`).

use sc_sim::{MoveStatus, ObjectId, OrderKind, SpawnAt, Vec2};

use super::{
    HAND, HAND_HALF, INFANTRY_HALF, P1, P2, PEACEKEEPER, RECRUIT, alive, any_died, centre, crushes, facing, flat,
    move_to,
};

/// Where the infantry stands: against the middle of the tank's south side, touching it.
fn against_its_side() -> Vec2 {
    centre(6, 4) + Vec2::new(0.0, HAND_HALF.y + INFANTRY_HALF)
}

/// "Put an enemy Light Infantry against a stopped Hand of Heaven's side, then order the tank to face it: the tank
/// turns and the infantry dies when the hull sweeps it." Player 2's Peacekeeper is crushed in the tank's first tick of
/// turning, before it has moved: its side swings into the Peacekeeper as its front turns south. It turns on, the
/// Peacekeeper no obstacle, and goes where it was sent.
#[test]
fn an_enemy_against_its_side_dies_as_it_turns_to_face_it() {
    let mut t = (flat(12, 12))
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(6, 4)))
        .spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(against_its_side()))
        .start();
    t.without_auto_targeting();
    let start = t.pos(1);
    move_to(&mut t, 1, P1, 1, centre(6, 9));
    while alive(&t, 2) {
        assert_eq!(facing(&t, 1), 0.0, "it doesn't turn before the Peacekeeper dies");
        t.step();
        assert!(t.sim().tick() < 10, "the Peacekeeper still stands");
    }
    let crush_tick = t.sim().tick();
    assert_eq!(crushes(&t), [(crush_tick, ObjectId(2), ObjectId(1))]);
    assert!(facing(&t, 1) > 0.0, "it turned as it crushed it");
    assert_eq!(t.pos(1), start, "in place");
    while t.status(1) == MoveStatus::Moving {
        t.step();
        assert!(t.sim().tick() < 200, "it gets there");
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, centre(6, 9)));
    assert!(!any_died(&t));
}

/// "Same with an own Recruit: the tank stops turning at contact; nobody dies." Player 1's Recruit holds its position
/// against the tank's side, so it doesn't make way: the tank's spin is cut at once, every degree of it turning its side
/// into the Recruit, and it turns none of it while the Recruit is there (it creeps out of the contact instead, design
/// change 0035). Nobody is crushed, nobody dies, and the Recruit never moves.
#[test]
fn an_own_recruit_there_stops_the_turn_at_contact_and_nobody_dies() {
    let mut t = (flat(12, 12))
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(6, 4)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(against_its_side()))
        .start();
    t.without_auto_targeting();
    t.order_at(1, P1, &[ObjectId(2)], OrderKind::Hold, false);
    move_to(&mut t, 1, P1, 1, centre(6, 9));
    let recruit = t.pos(2);
    t.run_until(3);
    assert_eq!(facing(&t, 1), 0.0, "the turn stopped at contact");
    t.run_until(200);
    assert!(crushes(&t).is_empty() && !any_died(&t));
    assert_eq!(t.pos(2), recruit, "the Recruit held its ground");
}

/// "A stopped, non-turning tank: enemy infantry can't walk into it." Player 2's Peacekeeper, sent through an idle
/// Hand of Heaven to the far side of it, goes round it, and nobody is crushed: the tank neither moves nor turns.
#[test]
fn a_tank_that_stands_and_does_not_turn_lets_no_enemy_walk_into_it() {
    let mut t = (flat(12, 9))
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(6, 4)))
        .spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(centre(2, 4)))
        .start();
    t.without_auto_targeting();
    let tank = t.pos(1);
    move_to(&mut t, 1, P2, 2, centre(10, 4));
    while t.sim().tick() < 2 || t.status(2) == MoveStatus::Moving {
        t.step();
        assert_eq!((t.pos(1), facing(&t, 1)), (tank, 0.0), "the tank stands");
        assert!(t.sim().tick() < 300, "the Peacekeeper gets round");
    }
    assert_eq!((t.status(2), t.pos(2)), (MoveStatus::Arrived, centre(10, 4)));
    assert!(crushes(&t).is_empty() && !any_died(&t));
}
