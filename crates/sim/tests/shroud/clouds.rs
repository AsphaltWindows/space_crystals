//! Which circles are one cloud, and when the buffers go (`super`): "Clouds are connected when their circles **touch or
//! overlap** (user, 2026-11-26; Q0113); a connected cloud is simply one cloud"; "if the channel ends or the connected
//! cloud splits, the buffer is **released immediately** over the cloud as it was. **When clouds join** (e.g. a new
//! cloud forms between two others; user, 2026-11-26; Q0113), each joining cloud **releases its buffer immediately**
//! over its own area as it was, and the joined cloud then **starts buffering together** with a fresh buffer"
//! (`factions/cults/units/disciple.md > Spells > Shroud`).

use super::*;

/// The plan's acceptance, "Two Disciples 5 GU apart share one buffer; a third 8 GU away doesn't": Disciples 1 and 2,
/// 320 su apart, are one cloud, and 3, 512 su from 2, another. Player 2's Militant (6) shoots player 1's (4) under 1's
/// circle: the release goes over 1's and 2's circles, hitting the two Disciples and the Militant, and nothing under
/// 3's, its Disciple and player 1's Militant there (5).
#[test]
fn two_disciples_5_gu_apart_share_one_buffer_and_a_third_8_gu_away_doesnt() {
    let mut t = quiet(
        flat(24, 14)
            .spawn(DISCIPLE, Some(P1), point(300.0, 400.0))
            .spawn(DISCIPLE, Some(P1), point(620.0, 400.0))
            .spawn(DISCIPLE, Some(P1), point(1132.0, 400.0))
            .spawn(MILITANT, Some(P1), point(300.0, 480.0))
            .spawn(MILITANT, Some(P1), point(1132.0, 480.0))
            .spawn(MILITANT, Some(P2), point(300.0, 700.0))
            .start(),
    );
    order(&mut t, 1, P1, &[1, 2, 3], OrderKind::Shroud);
    order(&mut t, 2, P2, &[6], OrderKind::Attack(ObjectId(4)));
    t.run(1);
    assert_eq!(clouds(&t), [vec![1, 2], vec![3]]);
    step_until(&mut t, 60, "the shot absorbed", |t| buffer(t, 1) > 0.0);
    assert_eq!((buffer(&t, 1), buffer(&t, 3)), (4.0, 0.0));
    let next = t.sim().tick() + 1;
    order(&mut t, next, P2, &[6], OrderKind::Stop);
    step_until(&mut t, 9, "the release", |t| !releases(t).is_empty());
    let (_, circles, damage) = &releases(&t)[0];
    assert_eq!(
        (circles, *damage),
        (&vec![Vec2::new(300.0, 400.0), Vec2::new(620.0, 400.0)], 4.0)
    );
    for id in [1, 2, 4] {
        assert_eq!(hits_on(&t, id).len(), 1, "#{id} under the joint cloud");
    }
    assert_eq!(
        (hits_on(&t, 3), hits_on(&t, 5)),
        (vec![], vec![]),
        "nothing under the other"
    );
}

/// The plan's acceptance, "Two Disciples a short gap apart: a third channelling in between makes both clouds release at
/// once, then they buffer together": Disciples 1 and 2, 448 su apart (a gap of 64 su between their circles), each with
/// a Militant of player 1's under it (4, 5) that one of player 2's shoots from outside (6, 7). Once both clouds hold a
/// shot, Disciple 3 channels between them: in that tick each cloud releases its own buffer over its own circle, and the
/// next release is the joined cloud's, over all three circles.
#[test]
fn a_disciple_channelling_between_two_clouds_makes_both_release_at_once_then_they_buffer_together() {
    let mut t = quiet(
        flat(20, 14)
            .spawn(DISCIPLE, Some(P1), point(300.0, 400.0))
            .spawn(DISCIPLE, Some(P1), point(748.0, 400.0))
            .spawn(DISCIPLE, Some(P1), point(524.0, 400.0))
            .spawn(MILITANT, Some(P1), point(300.0, 480.0))
            .spawn(MILITANT, Some(P1), point(748.0, 480.0))
            .spawn(MILITANT, Some(P2), point(300.0, 700.0))
            .spawn(MILITANT, Some(P2), point(748.0, 700.0))
            .start(),
    );
    order(&mut t, 1, P1, &[1, 2], OrderKind::Shroud);
    order(&mut t, 2, P2, &[6], OrderKind::Attack(ObjectId(4)));
    order(&mut t, 2, P2, &[7], OrderKind::Attack(ObjectId(5)));
    step_until(&mut t, 60, "both shots absorbed", |t| {
        buffer(t, 1) > 0.0 && buffer(t, 2) > 0.0
    });
    assert_eq!(clouds(&t), [vec![1], vec![2]]);
    let (one, two) = (buffer(&t, 1), buffer(&t, 2));
    let join = t.sim().tick() + 1;
    order(&mut t, join, P1, &[3], OrderKind::Shroud);
    t.step();
    assert_eq!(clouds(&t), [vec![1, 2, 3]]);
    let at_join: Vec<(Tick, Vec<Vec2>, f32)> = (releases(&t).into_iter()).filter(|r| r.0 == join).collect();
    let each = [
        (join, vec![Vec2::new(300.0, 400.0)], one),
        (join, vec![Vec2::new(748.0, 400.0)], two),
    ];
    assert_eq!(at_join, each, "both at once, each over its own circle");
    step_until(&mut t, 40, "the joined cloud's release", |t| releases(t).len() > 2);
    let (_, circles, _) = &releases(&t)[2];
    let all = [
        Vec2::new(300.0, 400.0),
        Vec2::new(748.0, 400.0),
        Vec2::new(524.0, 400.0),
    ];
    assert_eq!(circles, &all, "together");
}

/// The plan's acceptance, "Stop on one of a joined pair releases the shared buffer at once over both circles", as
/// (M6-CM) the linger has it (design change 0448: "when the channel ends [...], the cloud **stays for 1.5 s (24 frames,
/// placeholder)** and keeps absorbing as usual", and "if a cloud **vanishes** (at the end of its linger) or the
/// connected cloud splits, the buffer is **released immediately** over the cloud as it was"): Disciples 1 and 2, 5 GU
/// apart, are one cloud, and player 2's Militant (4) shoots player 1's (3) under 1's circle once, in tick `shot` (a
/// first game, without the Stop, finds it). A Stop to 2, 20 ticks before the shot, ends its channel; its circle lingers
/// where it was, the cloud as it was, and the shot is absorbed as without the Stop. 2's circle vanishes 4 ticks after
/// the shot, and the shared buffer goes in that tick, over both circles, not 8 ticks after the shot. The cloud of 1
/// alone goes on, empty.
#[test]
fn stop_on_one_of_a_joined_pair_releases_the_shared_buffer_over_both_circles_as_its_circle_vanishes() {
    let game = |stop: Option<Tick>| {
        let mut t = quiet(
            flat(20, 14)
                .spawn(DISCIPLE, Some(P1), point(300.0, 400.0))
                .spawn(DISCIPLE, Some(P1), point(620.0, 400.0))
                .spawn(MILITANT, Some(P1), point(300.0, 480.0))
                .spawn(MILITANT, Some(P2), point(300.0, 700.0))
                .start(),
        );
        order(&mut t, 1, P1, &[1, 2], OrderKind::Shroud);
        order(&mut t, 40, P2, &[4], OrderKind::Attack(ObjectId(3)));
        if let Some(stop) = stop {
            order(&mut t, stop, P1, &[2], OrderKind::Stop);
        }
        step_until(&mut t, 100, "the shot absorbed", |t| buffer(t, 1) > 0.0);
        let shot = t.sim().tick();
        order(&mut t, shot + 1, P2, &[4], OrderKind::Stop);
        (t, shot)
    };
    let (_, shot) = game(None);
    let stop = shot - 20;
    assert!(stop > 1, "the Stop after the channel started: shot in tick {shot}");
    let (mut t, again) = game(Some(stop));
    assert_eq!(again, shot, "absorbed as without the Stop");
    let ended = (stop, SimEvent::ShroudEnded { disciple: ObjectId(2) });
    assert!(t.events().contains(&ended), "the Stop ended its channel");
    let lingering = t.sim().clouds().list()[0].members[1];
    assert_eq!((lingering.id, lingering.until), (ObjectId(2), Some(stop + LINGER)));
    assert_eq!(
        (clouds(&t), buffer(&t, 1)),
        (vec![vec![1, 2]], 4.0),
        "the cloud as it was"
    );
    t.run_until(stop + LINGER);
    let both = vec![Vec2::new(300.0, 400.0), Vec2::new(620.0, 400.0)];
    assert_eq!(
        releases(&t),
        [(stop + LINGER, both, 4.0)],
        "at once as it vanishes, over the cloud as it was"
    );
    assert_eq!((clouds(&t), buffer(&t, 1)), (vec![vec![1]], 0.0));
    t.run(20);
    assert_eq!(releases(&t).len(), 1);
}
