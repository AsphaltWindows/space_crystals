//! A Hand of Heaven driving through enemy infantry (`systems/units.md > UnitCollision > Crushing`: "**Pathing
//! exception to hard collision**: crushers may path and drive through crushable enemies — those enemies are not
//! obstacles to them"; "**Trigger**: [...] while the crusher has any speed (Locomotion = Moving, Reversing or
//! **Stopping** [...])"), and past a Guard, which isn't crushable (`systems/units.md > HeavyInfantry`: "cannot be
//! crushed").

use sc_sim::{MoveStatus, ObjectId, OrderKind, SpawnAt};

use super::{GUARD, HAND, P1, P2, PEACEKEEPER, alive, any_died, centre, crushes, facing, flat, move_to};

/// The plan's acceptance: "A moving test tracked unit drives through 5 enemy Peacekeepers. All die, and its path stays
/// straight." Player 2's Peacekeepers stand a tile apart on the tank's way east, from (5, 4) to (9, 4); the tank,
/// sent from (2, 4) to (20, 4), crushes each as it reaches it, in order, and never leaves its line nor turns.
#[test]
fn driving_through_five_enemy_peacekeepers_crushes_them_all_on_a_straight_path() {
    let mut builder = flat(24, 8).spawn(HAND, Some(P1), SpawnAt::Point(centre(2, 4)));
    for x in 5..10 {
        builder = builder.spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(centre(x, 4)));
    }
    let mut t = builder.start();
    t.without_auto_targeting();
    let line = t.pos(1).y;
    move_to(&mut t, 1, P1, 1, centre(20, 4));
    while t.sim().tick() < 2 || t.status(1) == MoveStatus::Moving {
        t.step();
        assert_eq!(
            (t.pos(1).y, facing(&t, 1)),
            (line, 0.0),
            "tick {}: straight on",
            t.sim().tick()
        );
        assert!(t.sim().tick() < 300, "it gets there");
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, centre(20, 4)));
    let crushed: Vec<(ObjectId, ObjectId)> = crushes(&t).into_iter().map(|(_, id, by)| (id, by)).collect();
    let expected: Vec<(ObjectId, ObjectId)> = (2..7).map(|id| (ObjectId(id), ObjectId(1))).collect();
    assert_eq!(crushed, expected, "each in its turn");
    assert!((2..7).all(|id| !alive(&t, id)));
    assert!(!any_died(&t));
}

/// "Reversing" crushes as Moving does: a Hand of Heaven facing east, ordered to back up to (2, 4), backs straight
/// through player 2's Peacekeeper on (6, 4), crushing it, and gets there facing east still.
#[test]
fn backing_up_through_an_enemy_peacekeeper_crushes_it() {
    let mut t = (flat(14, 8))
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(10, 4)))
        .spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(centre(6, 4)))
        .start();
    t.without_auto_targeting();
    let line = t.pos(1).y;
    t.order_at(1, P1, &[ObjectId(1)], OrderKind::Reverse(centre(2, 4)), false);
    while t.sim().tick() < 2 || t.status(1) == MoveStatus::Moving {
        t.step();
        assert_eq!(
            (t.pos(1).y, facing(&t, 1)),
            (line, 0.0),
            "tick {}: straight back",
            t.sim().tick()
        );
        assert!(t.sim().tick() < 400, "it gets there");
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, centre(2, 4)));
    let crushed: Vec<(ObjectId, ObjectId)> = crushes(&t).into_iter().map(|(_, id, by)| (id, by)).collect();
    assert_eq!(crushed, [(ObjectId(2), ObjectId(1))]);
    assert!(!any_died(&t));
}

/// "Heavy Infantry (Guards) are never crushed": player 2's Guard on the tank's way east is an obstacle to it, as any
/// unit that isn't crushable. The tank goes round it and gets where it was sent; the Guard is never crushed.
#[test]
fn a_guard_is_never_crushed() {
    let mut t = (flat(16, 9))
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(2, 4)))
        .spawn(GUARD, Some(P2), SpawnAt::Point(centre(7, 4)))
        .start();
    t.without_auto_targeting();
    move_to(&mut t, 1, P1, 1, centre(13, 4));
    while t.sim().tick() < 2 || t.status(1) == MoveStatus::Moving {
        t.step();
        assert!(t.sim().tick() < 400, "it gets round");
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, centre(13, 4)));
    assert!(alive(&t, 2) && crushes(&t).is_empty() && !any_died(&t));
}
