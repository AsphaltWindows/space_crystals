//! What an illusion mirrors of its parent (`factions/cults/units/disciple.md > Spells > Mirage`): the channel's stance
//! ("**Shroud mirroring**"), and the energy it shows (design change 0452).

use super::*;
use sc_sim::Target;

/// Design change 0448's Verification, step 4: "Press R on the parent. The illusions play the channelling animation,
/// but only the parent has a cloud. Stop one illusion: it alone stops. Stop the parent: all of them stop" ("when the
/// parent Disciple **starts channelling Shroud**, its illusions **play the same channelling animation**, but make **no
/// cloud**"; "An illusion that is moving or busy when the parent starts channelling stops and plays the animation too
/// (designer's reading)"). The east illusion walks off; the Disciple, back to 10 energy, channels: in that tick all
/// five stand channelling (`sc_sim::shroud::stands_channelling`, what the client draws as the stance), the walker
/// stopped where it was, and the one cloud is the Disciple's. A Stop to the north illusion stops it alone; a Stop to
/// the Disciple ends its channel, and its illusions stop with it, in that tick. A walk of the south one's own then
/// is its own.
#[test]
fn the_illusions_mirror_the_channel_without_a_cloud_and_stop_alone_or_with_it() {
    let mut t = lone();
    cast(&mut t, 1, &[1]);
    step_checked(&mut t);
    regenerate(&mut t, 1, 10.5);
    let walk = t.sim().tick() + 1;
    order(
        &mut t,
        walk,
        P1,
        &[3],
        OrderKind::Move(Target::Point(AT + Vec2::new(8.0 * GU, 0.0))),
    );
    let channel = walk + 4;
    order(&mut t, channel, P1, &[1], OrderKind::Shroud);
    t.run_until(channel - 1);
    assert!(t.pos(3).x > AT.x + GU, "walking: {}", t.pos(3));
    assert!(!stands(&t, 3));
    step_checked(&mut t);
    assert!(
        [1, 2, 3, 4, 5].iter().all(|&id| stands(&t, id)),
        "all five stand channelling"
    );
    assert_eq!(clouds(&t), [vec![1]], "the Disciple's cloud alone");
    let stopped = t.pos(3);
    t.run(10);
    assert_eq!(t.pos(3), stopped, "the walker stopped and stands");
    assert!((2..=5).all(|id| t.sim().object(ObjectId(id)).is_some_and(|o| o.energy.is_none())));

    let one = t.sim().tick() + 1;
    order(&mut t, one, P1, &[2], OrderKind::Stop);
    t.run_until(one);
    assert!(!stands(&t, 2), "the north one stops");
    assert!([1, 3, 4, 5].iter().all(|&id| stands(&t, id)), "alone");
    assert_eq!(clouds(&t), [vec![1]]);

    let all = t.sim().tick() + 5;
    order(&mut t, all, P1, &[1], OrderKind::Stop);
    t.run_until(all);
    let ended = (all, SimEvent::ShroudEnded { disciple: ObjectId(1) });
    assert!(t.events().contains(&ended), "the channel ends");
    assert!((1..=5).all(|id| !stands(&t, id)), "all of them stop with it");
    let shroud_ends: Vec<&(Tick, SimEvent)> = (t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::ShroudEnded { .. }))
        .collect();
    assert_eq!(shroud_ends, [&ended], "an illusion has no channel to end");
    let south = t.sim().tick() + 1;
    let away = AT + Vec2::new(0.0, 5.0 * GU);
    order(&mut t, south, P1, &[4], OrderKind::Move(Target::Point(away)));
    t.run(40);
    assert!(t.pos(4).y > AT.y + 2.0 * GU, "its own walk: {}", t.pos(4));
}

/// Design change 0452's Verification: "1. Cast Mirage. All five Disciples show the same energy. 2. Channel Shroud on
/// the parent. All five bars drain together. 3. Stop the channel. All five regenerate together" ("an illusion's
/// energy bar and InfoPanel always show its **parent's current energy**, live"). What each of the five shows
/// (`Sim::energy_shown`, which the bar and the InfoPanel read), every tick: the Disciple's own pool, the same on all
/// five, after the cast; falling every tick of the channel, the drain of 5 a second over the regeneration of 0.5; and
/// rising every tick once a Stop has ended it.
#[test]
fn the_five_show_the_parent_s_energy_draining_and_regenerating_together() {
    let mut t = lone();
    cast(&mut t, 1, &[1]);
    step_checked(&mut t);
    let five = |t: &TestSim| -> f32 {
        let own = (energy(t, 1), 200.0);
        for id in 1..=5 {
            assert_eq!(shown(t, id), Some(own), "#{id} in tick {}", t.sim().tick());
        }
        own.0
    };
    assert_eq!(five(&t), A_TICK, "the cast's");
    let mut last = five(&t);
    for _ in 0..20 {
        step_checked(&mut t);
        let now = five(&t);
        assert_eq!(now - last, A_TICK, "regenerating");
        last = now;
    }
    regenerate(&mut t, 1, 12.0);
    let channel = t.sim().tick() + 1;
    order(&mut t, channel, P1, &[1], OrderKind::Shroud);
    t.run_until(channel - 1);
    let mut last = five(&t);
    for _ in 0..20 {
        step_checked(&mut t);
        let now = five(&t);
        assert!(now < last, "draining together: {last} to {now}");
        assert!(
            (last - now - (5.0 - 0.5) / 16.0).abs() < 1e-4,
            "a tick's drain: {}",
            last - now
        );
        last = now;
    }
    assert_eq!(clouds(&t), [vec![1]]);
    let stop = t.sim().tick() + 1;
    order(&mut t, stop, P1, &[1], OrderKind::Stop);
    t.run_until(stop);
    let mut last = five(&t);
    for _ in 0..20 {
        step_checked(&mut t);
        let now = five(&t);
        assert_eq!(now - last, A_TICK, "regenerating together");
        last = now;
    }
}
