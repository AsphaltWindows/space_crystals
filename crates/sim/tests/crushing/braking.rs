//! Design change 0453 (answering Q0178): "A crusher crushes while it has **any speed**: Moving, Reversing or
//! **Stopping**. A tank braking for its goal, or coasting after Stop, rolls over enemy infantry in its path instead of
//! halting against them. It becomes a hard obstacle only once it is Stationary and not turning"
//! (`systems/units.md > UnitCollision > Crushing`: "**Trigger**: [...] while the crusher has any speed"). Its
//! Verification: a Hand of Heaven sent onto a spot covered by enemy Peacekeepers crushes those it overlaps as it brakes
//! onto it; one given Stop just before it reaches them coasts into them and crushes them; and once at rest it blocks
//! them like any obstacle. A tank's braking to set off the other way is Stopping too.

use sc_sim::mover::Locomotion;
use sc_sim::testing::TestSim;
use sc_sim::{MoveStatus, ObjectId, OrderKind, SpawnAt, Tick, Vec2};

use super::{HAND, HAND_HALF, INFANTRY_HALF, P1, P2, PEACEKEEPER, alive, any_died, centre, crushes, flat, move_to};

/// The Hand of Heaven's Locomotion, and its speed (su a tick).
fn drive(t: &TestSim) -> (Locomotion, f32) {
    let mover = t.mover(1);
    (mover.locomotion, mover.momentum.length())
}

/// Steps until the Hand of Heaven (unit 1) is at rest with no goal, at most `ticks` more ticks, noting the Locomotion it
/// was in after each tick's move that crushed: each crush's tick, victim and that Locomotion.
fn until_at_rest(t: &mut TestSim, ticks: Tick) -> Vec<(Tick, ObjectId, Locomotion)> {
    let until = t.sim().tick() + ticks;
    let mut noted = Vec::new();
    loop {
        let before = crushes(t).len();
        t.step();
        let (locomotion, _) = drive(t);
        noted.extend(crushes(t)[before..].iter().map(|&(tick, id, _)| (tick, id, locomotion)));
        if t.status(1) != MoveStatus::Moving && drive(t) == (Locomotion::Stationary, 0.0) {
            return noted;
        }
        assert!(t.sim().tick() < until, "at rest by tick {until}");
    }
}

/// "Order a Hand of Heaven to stop on a spot covered by enemy Peacekeepers": player 2's nine Peacekeepers stand on a
/// square 24 su apart round (12, 4), and a tenth 56 su east of it, beyond the tank's front where it stops. Sent there
/// from (2, 4), it brakes onto the spot along its leg (a Moving drive, slowing to its approach speed, which crushed
/// before 0453 too), crushing the nine as it reaches each, and stands exactly on the spot: the tenth, which it never
/// overlaps, stands. Once at rest the survivor, sent to (12, 2) right through it, goes round it: the tank blocks it,
/// and crushes it never.
#[test]
fn a_tank_sent_onto_a_spot_covered_by_enemy_infantry_crushes_those_it_overlaps_and_then_blocks() {
    let spot = centre(12, 4);
    let mut builder = flat(20, 9).spawn(HAND, Some(P1), SpawnAt::Point(centre(2, 4)));
    for dy in [-24.0, 0.0, 24.0] {
        for dx in [-24.0, 0.0, 24.0] {
            builder = builder.spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(spot + Vec2::new(dx, dy)));
        }
    }
    let survivor = 11;
    let mut t = builder
        .spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(spot + Vec2::new(56.0, 0.0)))
        .start();
    t.without_auto_targeting();
    move_to(&mut t, 1, P1, 1, spot);
    let noted = until_at_rest(&mut t, 300);
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, spot));
    let mut victims: Vec<u32> = noted.iter().map(|&(_, id, _)| id.0).collect();
    victims.sort_unstable();
    assert_eq!(victims, (2..11).collect::<Vec<_>>(), "the nine it overlaps: {noted:?}");
    assert!(alive(&t, survivor));
    assert!(
        t.pos(survivor).x - spot.x >= HAND_HALF.x + INFANTRY_HALF,
        "never overlapped"
    );

    let at = t.pos(1);
    let tick = t.sim().tick() + 1;
    move_to(&mut t, tick, P2, survivor, centre(12, 2) + Vec2::new(-56.0, 0.0));
    while t.sim().tick() <= tick || t.status(survivor) == MoveStatus::Moving {
        t.step();
        assert!(t.sim().tick() < tick + 300, "it gets round");
    }
    assert_eq!(t.status(survivor), MoveStatus::Arrived);
    assert_eq!(crushes(&t).len(), 9, "at rest, it crushes nothing");
    assert_eq!(t.pos(1), at, "nor moves");
    assert!(!any_died(&t));
}

/// "Or press Stop just before it reaches them": player 2's Peacekeeper stands on (9, 4), on the way of a Hand of Heaven
/// sent east from (2, 4) at full speed (5 su a tick), and a second on (12, 4), beyond. Stop comes as the tank's front
/// is a tick's move from the first: it coasts to rest (Stopping, braking by its Deceleration, 1 su a tick, so some
/// 10 su), its front into the Peacekeeper, which it crushes as it brakes: before 0453 the Peacekeeper stopped it at
/// contact. It stops short of the second, which stands.
#[test]
fn a_tank_given_stop_just_before_enemy_infantry_coasts_into_it_and_crushes_it() {
    let mut t = (flat(16, 9))
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(2, 4)))
        .spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(centre(9, 4)))
        .spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(centre(12, 4)))
        .start();
    t.without_auto_targeting();
    move_to(&mut t, 1, P1, 1, centre(14, 4));
    // Its front's gap to the first Peacekeeper's back.
    let gap = |t: &TestSim| t.pos(2).x - INFANTRY_HALF - (t.pos(1).x + HAND_HALF.x);
    while gap(&t) > 5.0 {
        t.step();
        assert!(t.sim().tick() < 200, "it nears the Peacekeeper");
    }
    assert_eq!(drive(&t), (Locomotion::Moving, 5.0), "at full speed");
    assert!(gap(&t) > 0.0);
    let stop = t.sim().tick() + 1;
    t.order_at(stop, P1, &[ObjectId(1)], OrderKind::Stop, false);
    let noted = until_at_rest(&mut t, 40);
    assert_eq!(
        noted
            .iter()
            .map(|&(_, id, locomotion)| (id, locomotion))
            .collect::<Vec<_>>(),
        [(ObjectId(2), Locomotion::Stopping)],
        "crushed as it coasts"
    );
    assert!(noted[0].0 >= stop);
    assert!(alive(&t, 3), "short of the second");
    assert!(t.pos(3).x - t.pos(1).x > HAND_HALF.x + INFANTRY_HALF);
    assert!(!any_died(&t));
}

/// Braking against its leg is Stopping too (`movement::tracked`: "Moving the other way from its leg, it brakes to rest
/// first"): a Hand of Heaven driving east at full speed, ordered to back up to (2, 4) as its front is a tick's move
/// from player 2's Peacekeeper on (9, 4), brakes on east into it before it sets off backwards, and crushes it as it
/// brakes. It then backs up where it was sent. (Sent forward the other way, it would slow for its turn, a Moving drive.)
#[test]
fn a_tank_braking_to_set_off_the_other_way_crushes_what_it_brakes_into() {
    let mut t = (flat(16, 9))
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(2, 4)))
        .spawn(PEACEKEEPER, Some(P2), SpawnAt::Point(centre(9, 4)))
        .start();
    t.without_auto_targeting();
    move_to(&mut t, 1, P1, 1, centre(14, 4));
    let gap = |t: &TestSim| t.pos(2).x - INFANTRY_HALF - (t.pos(1).x + HAND_HALF.x);
    while gap(&t) > 5.0 {
        t.step();
        assert!(t.sim().tick() < 200, "it nears the Peacekeeper");
    }
    let back = t.sim().tick() + 1;
    t.order_at(back, P1, &[ObjectId(1)], OrderKind::Reverse(centre(2, 4)), false);
    while crushes(&t).is_empty() {
        t.step();
        assert!(t.sim().tick() < back + 10, "it crushes it as it brakes");
    }
    assert_eq!(drive(&t).0, Locomotion::Stopping, "crushed as it brakes");
    let crushed: Vec<ObjectId> = crushes(&t).iter().map(|&(_, id, _)| id).collect();
    assert_eq!(crushed, [ObjectId(2)]);
    while t.status(1) == MoveStatus::Moving {
        t.step();
        assert!(t.sim().tick() < back + 400, "it gets back");
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, centre(2, 4)));
    assert!(!any_died(&t));
}
