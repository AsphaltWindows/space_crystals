//! The reveal (`factions/cults/units/hand_of_heaven.md`: "Reveal: the shared rule applies (core/vision.md > Attacks
//! from outside vision): it is revealed to those it damages when the missile lands"; `core/vision.md > Attacks from
//! outside vision`: "For projectiles and area damage ... the reveal happens **when the damage lands**").

use super::verification::shooting_range;
use super::*;

/// Whether `player` sees object `n` now, and whether that is a reveal.
fn seen(t: &TestSim, player: PlayerId, n: u32) -> (bool, bool) {
    let o = t.sim().object(id(n)).expect("on the map");
    let revealed = (t.sim().vision().player(player)).is_some_and(|vision| vision.is_revealed(id(n)));
    (t.sim().is_visible(player, o), revealed)
}

/// Player 2's Chop Shop sees 4 grid units, and the Hand of Heaven fires at it from 10, with player 1's Recruit for a
/// spotter: player 2 doesn't see the Hand of Heaven as it aims, nor as the missile leaves, nor while it flies; the
/// missile lands in tick `launch + 96`, and from that tick on player 2 sees the Hand of Heaven, revealed (for 1.5 s,
/// `tuning::vision::Vision::reveal_ticks`).
#[test]
fn fired_from_beyond_the_target_s_owner_s_sight_it_is_revealed_when_the_missile_lands() {
    let mut t = shooting_range();
    t.run_until(1);
    assert_eq!(seen(&t, P2, 1), (false, false), "out of player 2's sight");
    assert!(t.sim().is_visible(P1, t.sim().object(id(2)).unwrap()), "the spotter's");
    order(&mut t, 2, P1, &[1], OrderKind::Attack(id(2)));
    until(&mut t, 80, "the launch", |t| {
        assert_eq!(
            seen(t, P2, 1),
            (false, false),
            "tick {}: not while it aims",
            t.sim().tick()
        );
        !launches(t).is_empty()
    });
    let lands = launches(&t)[0].lands_at;
    while t.sim().tick() < lands - 1 {
        t.step();
        assert_eq!(
            seen(&t, P2, 1),
            (false, false),
            "tick {}: not in the flight",
            t.sim().tick()
        );
    }
    t.step();
    assert_eq!(blasts(&t).len(), 1, "it landed in tick {lands}");
    assert_eq!(seen(&t, P2, 1), (true, true), "revealed as it lands");
}
