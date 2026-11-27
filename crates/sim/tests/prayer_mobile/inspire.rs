//! Inspire (`factions/cults/units/prayer_mobile.md > Placeholder stats`: "**Inspire** (hotkey **R**, instant, no
//! target): 75 energy; every own infantry unit within **4 grid units** of the vehicle gets **+1.5 su/frame MaxSpeed**
//! for **10 s**; Recruits also attack **25% faster** (their attack cycle × 0.75) for the same time"; `> Abilities`:
//! "every own infantry type (Recruit, Militant, Gunner, Disciple) gets the movement speed; Recruits also get the attack
//! speed"; "**Stacks with augmentation** (e.g. Improved Metabolism): different sources add up. **Recasting** on units
//! that are already inspired **restarts the duration**; Inspire doesn't stack with itself"). The cycle's frames saved
//! come off its FiringDuration (`sc_sim::buffs::inspired`).

use sc_sim::defs::AttackAttrs;
use sc_sim::{Augmentation, DebugCommand, OrderKind, RecruitKind, SpawnAt, Target, TilePos};

use super::*;

/// Unit `id`'s weapon now, its Augmentations' and its buffs' included (`combat::weapon`).
fn weapon(t: &TestSim, id: ObjectId) -> AttackAttrs {
    let o = t.sim().object(id).unwrap_or_else(|| panic!("no unit {id:?}"));
    sc_sim::combat::weapon(o, t.sim().research()).expect("armed")
}

/// A weapon's attack cycle: its AimDuration, FiringDuration, CooldownDuration and ReloadDuration, frames.
fn cycle(attrs: &AttackAttrs) -> u32 {
    attrs.aim_ticks + attrs.firing_ticks + attrs.cooldown_ticks + attrs.reload_ticks
}

/// The gaps between successive ticks of `ticks` that both fall in `from..=to`.
fn gaps(ticks: &[Tick], from: Tick, to: Tick) -> Vec<Tick> {
    let inside: Vec<Tick> = ticks
        .iter()
        .copied()
        .filter(|tick| (from..=to).contains(tick))
        .collect();
    inside.windows(2).map(|w| w[1] - w[0]).collect()
}

/// Design change 0365's Verification: "Inspire next to Recruits: they move and punch faster for 10 s". Player 1's
/// Prayer-mobile (1) has two Recruits beside it (2, 3) and a third (4) punching player 2's Storage (5) west of it,
/// every 12 ticks (`factions/cults/units/recruit.md`: Aim 2, Firing 8, Cooldown 1, Reload 1). With its 75, at the end
/// of tick 800, it casts Inspire in tick 801, and the two beside it walk east from then on: all three get the buff,
/// until tick 961, the vehicle none. Each walker covers 5.5 su a tick, 4 + 1.5, to the end of tick 960, and 4 from
/// tick 961; the third punches every 9 ticks, 12 × 0.75, while the buff lasts, and every 12 again once it is over.
#[test]
fn inspire_next_to_recruits_they_move_and_punch_faster_for_10_s() {
    let mut t = versus(Faction::Cults, 40, 20)
        .spawn(PRAYER_MOBILE, Some(P1), SpawnAt::Point(Vec2::new(1000.0, 640.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(1000.0, 540.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(1000.0, 740.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(880.0, 640.0)))
        .spawn(ObjectKind::Storage, Some(P2), SpawnAt::Tile(TilePos::new(9, 9)))
        .start();
    t.without_auto_targeting();
    let (pm, walkers, puncher, storage) = (id(1), [id(2), id(3)], id(4), id(5));
    t.order_at(1, P1, &[puncher], OrderKind::Attack(storage), false);
    charge(&mut t, pm, 75);
    assert_eq!(t.sim().tick(), 800);
    let c = t.sim().tick() + 1;
    let inspire = cast(&t, pm, INSPIRE);
    t.at(c, P1, inspire);
    for walker in walkers {
        let to = Vec2::new(2400.0, t.sim().object(walker).expect("beside it").pos.y);
        t.order_at(c, P1, &[walker], OrderKind::Move(Target::Point(to)), false);
    }
    let mut steps: Vec<(Tick, [f32; 2])> = Vec::new();
    let mut last = walkers.map(|walker| t.sim().object(walker).expect("on the map").pos);
    while t.sim().tick() < c + INSPIRE_TICKS + 60 {
        t.step();
        let now = walkers.map(|walker| t.sim().object(walker).expect("on the map").pos);
        steps.push((t.sim().tick(), [now[0].distance(last[0]), now[1].distance(last[1])]));
        last = now;
        if t.sim().tick() == c {
            for unit in [walkers[0], walkers[1], puncher] {
                assert_eq!(buffs(&t, unit), buff(INSPIRE, c, INSPIRE_TICKS), "{unit:?}");
            }
            assert_eq!(buffs(&t, pm), [], "not the vehicle");
        }
    }
    let end = c + INSPIRE_TICKS;
    for (tick, step) in steps {
        let expected = match tick {
            tick if tick < c + 2 => continue, // setting off
            tick if tick < end => 5.5,
            _ => 4.0,
        };
        assert_eq!(step, [expected; 2], "tick {tick}");
    }
    for unit in [walkers[0], walkers[1], puncher] {
        assert_eq!(buffs(&t, unit), [], "{unit:?}: over at {end}");
    }
    let punches = hits(&t, puncher, storage);
    let before = gaps(&punches, 0, c - 1);
    assert!(before.len() >= 30 && before.iter().all(|&gap| gap == 12), "{before:?}");
    let during = gaps(&punches, c + 12, end - 1);
    assert!(during.len() >= 14 && during.iter().all(|&gap| gap == 9), "{during:?}");
    let after = gaps(&punches, end + 12, t.sim().tick());
    assert!(after.len() >= 3 && after.iter().all(|&gap| gap == 12), "{after:?}");
}

/// The rest of Inspire's rules, read off the accessors (`Object::max_speed`, `combat::weapon`). Player 1's
/// Prayer-mobile (1) has round it a Militant (2), an Improved Metabolism Recruit (3), a Plain Recruit (4), player 2's
/// Recruit (5), its own Seeker (6), all within 4 grid units, and its own Recruit 260 su away (7), just outside them.
/// Its first cast buffs the Militant and its two Recruits within, until 10 s on:
/// - the Militant moves at 5.5 and keeps its attack (Aim 2, Firing 2, Cooldown 1, Reload 3: "Recruits also get the
///   attack speed");
/// - the Plain Recruit moves at 5.5 and fires in 5 frames, its cycle 9;
/// - the Improved Metabolism one moves at 6.5, its own 5 and the buff's 1.5 ("different sources add up"), and its
///   cycle of 10 becomes 8, 7.5 rounded up (`factions/cults/augmentation.md`: "+2 Metabolism" at no research, each
///   "+0.5 space units/frame MaxSpeed" and "−1 frame of FiringDuration");
/// - player 2's Recruit, the Seeker, the vehicle itself and the Recruit outside the circle get nothing.
///
/// A recast 80 ticks later, with the 77.5 left, restarts each buff for 10 s from it, one buff each, the speeds as
/// they were: they outlast the first cast's 10 s, and end with the second's.
#[test]
fn a_militant_gets_the_speed_only_and_a_recast_restarts_the_10_s() {
    let mut t = versus(Faction::Cults, 30, 20)
        .spawn(PRAYER_MOBILE, Some(P1), SpawnAt::Point(Vec2::new(800.0, 640.0)))
        .spawn(MILITANT, Some(P1), SpawnAt::Point(Vec2::new(800.0, 540.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(900.0, 640.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(700.0, 640.0)))
        .spawn(RECRUIT, Some(P2), SpawnAt::Point(Vec2::new(800.0, 740.0)))
        .spawn(ObjectKind::Seeker, Some(P1), SpawnAt::Point(Vec2::new(800.0, 840.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(1060.0, 640.0)))
        .start();
    t.without_auto_targeting();
    let (pm, militant, metabolism, plain, theirs, seeker, outside) = (id(1), id(2), id(3), id(4), id(5), id(6), id(7));
    let augmentation = Augmentation::of(RecruitKind::Metabolism);
    t.debug_at(
        2,
        DebugCommand::SetAugmentation {
            id: metabolism,
            augmentation,
        },
    );
    charge(&mut t, pm, 150);
    let bare = |t: &TestSim, unit| (top_speed(t, unit), weapon(t, unit));
    let (militant_before, metabolism_before, plain_before) =
        (bare(&t, militant), bare(&t, metabolism), bare(&t, plain));
    assert_eq!(militant_before.0, 4.0);
    assert_eq!((metabolism_before.0, cycle(&metabolism_before.1)), (5.0, 10));
    assert_eq!(
        (plain_before.0, cycle(&plain_before.1), plain_before.1.firing_ticks),
        (4.0, 12, 8)
    );

    let first = cast_now(&mut t, pm, INSPIRE);
    for unit in [militant, metabolism, plain] {
        assert_eq!(buffs(&t, unit), buff(INSPIRE, first, INSPIRE_TICKS), "{unit:?}");
    }
    for unit in [pm, theirs, seeker, outside] {
        assert_eq!(buffs(&t, unit), [], "{unit:?}");
    }
    let inspired = |t: &TestSim| {
        assert_eq!(top_speed(t, militant), 5.5);
        assert_eq!(weapon(t, militant), militant_before.1, "a Militant's attack as it was");
        assert_eq!(top_speed(t, metabolism), 6.5, "5 and 1.5");
        assert_eq!(cycle(&weapon(t, metabolism)), 8, "10 × 0.75, rounded up");
        let plain_now = weapon(t, plain);
        assert_eq!(
            (top_speed(t, plain), cycle(&plain_now), plain_now.firing_ticks),
            (5.5, 9, 5)
        );
        assert_eq!(
            AttackAttrs {
                firing_ticks: plain_before.1.firing_ticks,
                ..plain_now
            },
            plain_before.1,
            "only its FiringDuration"
        );
        assert_eq!(
            (top_speed(t, pm), top_speed(t, seeker), top_speed(t, theirs)),
            (6.0, 8.0, 4.0)
        );
    };
    inspired(&t);

    t.run_until(first + 79);
    assert_eq!(
        energy(&t, pm),
        75 * ONE + 80 * A_TICK,
        "150 less 75, and 80 ticks of 1/32"
    );
    let second = cast_now(&mut t, pm, INSPIRE);
    assert_eq!(second, first + 80);
    for unit in [militant, metabolism, plain] {
        assert_eq!(
            buffs(&t, unit),
            buff(INSPIRE, second, INSPIRE_TICKS),
            "{unit:?}: restarted"
        );
    }
    inspired(&t);
    t.run_until(first + INSPIRE_TICKS);
    inspired(&t);
    t.run_until(second + INSPIRE_TICKS - 2);
    inspired(&t);
    t.run_until(second + INSPIRE_TICKS - 1); // its last tick
    for unit in [militant, metabolism, plain] {
        assert_eq!(buffs(&t, unit), [], "{unit:?}: over 10 s after the recast");
    }
    assert_eq!(
        (bare(&t, militant), bare(&t, metabolism), bare(&t, plain)),
        (militant_before, metabolism_before, plain_before)
    );
}
