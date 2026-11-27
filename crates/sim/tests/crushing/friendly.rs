//! "**No friendly crushing**: crushers cannot run over their own or allied crushable units; those remain hard
//! obstacles and crushers must path around them" (`systems/units.md > UnitCollision > Crushing`). Every player is
//! their own side so far (`economy::defeat::allied`): there are no allies to try.

use sc_sim::{MoveStatus, ObjectId, OrderKind, SpawnAt};

use super::{HAND, P1, RECRUIT, alive, any_died, centre, crushes, flat, move_to};

/// The plan's acceptance: "Own and allied Light Infantry stop it, and it goes round them." Two of player 1's Recruits
/// hold their position one behind the other on its own Hand of Heaven's way east (holding, they don't make way): the
/// tank meets the first and goes round both, and gets where it was sent. Nobody is crushed, and the Recruits never
/// move.
#[test]
fn own_infantry_on_its_way_stop_it_and_it_goes_round_them() {
    let mut t = (flat(16, 9))
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(2, 4)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(centre(7, 4)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(centre(8, 4)))
        .start();
    t.without_auto_targeting();
    t.order_at(1, P1, &[ObjectId(2), ObjectId(3)], OrderKind::Hold, false);
    let recruits = [t.pos(2), t.pos(3)];
    move_to(&mut t, 1, P1, 1, centre(13, 4));
    let mut stopped = false;
    while t.sim().tick() < 2 || t.status(1) == MoveStatus::Moving {
        let was = t.pos(1);
        t.step();
        // Stopped by them: a tick it made no way east, with the first Recruit right ahead.
        stopped |= t.pos(1).x <= was.x && t.pos(1).x < recruits[0].x;
        assert!(t.sim().tick() < 400, "it gets round");
    }
    assert!(stopped, "the Recruits stopped it");
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, centre(13, 4)));
    assert_eq!([t.pos(2), t.pos(3)], recruits, "they held their ground");
    assert!(alive(&t, 2) && alive(&t, 3) && crushes(&t).is_empty() && !any_died(&t));
}
