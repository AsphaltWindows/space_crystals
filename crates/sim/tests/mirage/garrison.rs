//! Illusions in a Garrison (`factions/cults/structures/garrison.md > Garrisoning`: "Recruits, Militants, Gunners, and
//! Disciples, and Disciples' **illusions**"; design change 0450: "**Disciples and illusions don't fire** from inside").

use super::*;

/// Player 1's Garrison (1), as the Garrison tests have it: on a 30 × 22 map, unturned, over tiles (12, 10) to (13, 11),
/// x 768..896 and y 640..768, its door, B, facing west, its middle at [`DOOR`].
const GARRISON: ObjectId = ObjectId(1);
const DOOR: Vec2 = Vec2::new(768.0, 704.0);

/// The Garrison and player 1's Disciple (2), 3 grid units west of its door, auto-targeting on or off; the Disciple casts
/// in tick 1, and its four illusions (3 to 6) are ordered in in tick 2: the game once they are all inside, the Disciple
/// outside. The next object is 7.
fn garrisoned(auto_targeting: bool) -> TestSim {
    let mut t = flat(30, 22)
        .spawn(ObjectKind::Garrison, Some(P1), tile(12, 10))
        .spawn(DISCIPLE, Some(P1), point(DOOR.x - 3.0 * GU, DOOR.y))
        .start();
    if !auto_targeting {
        t.without_auto_targeting();
    }
    cast(&mut t, 1, &[2]);
    let enter = Command::Order {
        units: ids(&[3, 4, 5, 6]),
        kind: OrderKind::Enter(GARRISON),
        queue: false,
    };
    t.at(2, P1, enter);
    step_until(&mut t, 300, "the four in", |t| t.sim().held_in(GARRISON).len() == 4);
    assert_eq!(held_illusions(&t, 2), [3, 4, 5, 6], "the illusions inside");
    assert!(t.sim().object(ObjectId(2)).is_some(), "the Disciple outside");
    t
}

/// Design change 0450's Verification, step 3: "Put a Disciple's illusions inside instead. The Garrison doesn't fire"
/// (so 0448's step 7, as 0450 has it: "illusions may enter a Garrison, but **don't fire** from it, as Disciples
/// don't"). The Disciple's four illusions go in, a full Garrison; player 2's Recruit (7) 1 grid unit east of the
/// footprint, in the occupants' reach, and player 1's Recruit (8) beyond it, which keeps it in sight, both holding
/// (unarmed, they fire nothing): in 100 ticks with auto-targeting on, nothing fires. The first two steps, with
/// Disciples, are `tests/garrison`'s (`fire`).
#[test]
fn illusions_go_into_a_garrison_and_fire_nothing_from_it() {
    let mut t = garrisoned(true);
    let near = Vec2::new(896.0 + GU, DOOR.y);
    let now = t.sim().tick() + 1;
    t.spawn_unit_at(now, RECRUIT, P2, near);
    t.spawn_unit_at(now, RECRUIT, P1, near + Vec2::new(2.0 * GU, 0.0));
    order(&mut t, now + 1, P2, &[7], OrderKind::Hold);
    order(&mut t, now + 1, P1, &[8], OrderKind::Hold);
    t.run_until(now + 1);
    let kinds = [7, 8].map(|id| t.sim().object(ObjectId(id)).map(|o| (o.kind, o.owner)));
    assert_eq!(kinds, [Some((RECRUIT, Some(P2))), Some((RECRUIT, Some(P1)))]);
    for _ in 0..100 {
        step_checked(&mut t);
    }
    let shots: Vec<&(Tick, SimEvent)> = (t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. } | SimEvent::Hit { .. }))
        .collect();
    assert_eq!(shots, Vec::<&(Tick, SimEvent)>::new(), "the Garrison doesn't fire");
    assert_eq!(t.sim().held_in(GARRISON).len(), 4, "they stay inside");
}

/// "An illusion inside a Garrison when it ends simply vanishes" (`factions/cults/units/disciple.md > Spells >
/// Mirage`): with the illusions inside, player 2's Militant (7) kills the Disciple outside, and in that tick its
/// illusions leave the game from inside, with no death and no way out (`SimEvent::LeftBuilding`): the Garrison is
/// empty.
#[test]
fn illusions_inside_vanish_with_their_parent() {
    let mut t = garrisoned(false);
    let now = t.sim().tick() + 1;
    let from = t.pos(2) + Vec2::new(-3.0 * GU, 2.0 * GU);
    t.spawn_unit_at(now, MILITANT, P2, from);
    order(&mut t, now + 1, P2, &[7], OrderKind::Attack(ObjectId(2)));
    step_until(&mut t, 300, "the Disciple dies", |t| {
        t.sim().object(ObjectId(2)).is_none()
    });
    let death = t.sim().tick();
    assert_eq!(died(&t), [(death, 2)], "the Disciple alone dies");
    let gone: Vec<u32> = (removed(&t).iter())
        .filter(|(tick, _)| *tick == death)
        .map(|&(_, id)| id)
        .collect();
    assert_eq!(gone, [2, 3, 4, 5, 6], "its illusions with it");
    assert!(t.sim().held_in(GARRISON).is_empty(), "vanished from inside");
    let left = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::LeftBuilding { .. }));
    assert!(!left, "none came out");
    assert!(held_illusions(&t, 2).is_empty());
}
