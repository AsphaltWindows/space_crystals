//! Soft separation (`super`): `systems/units.md > Air Collision`: "Air units use soft separation with other air units —
//! a gentle repulsion force that prevents stacking but does not hard-block"; "SeparationRadius — per unit type,
//! circular, the distance at which air-to-air repulsion activates. Must be larger than the unit's Silhouette", the
//! Sparrow's 48 su; and (M5-H2) "**Gliders push apart slowly** (user, 2026-11-27; Q0148): gliders do separate, but with
//! a **much weaker** repulsion than other air units (placeholder: a quarter of the HoverCraft strength), so they drift
//! apart gradually, mostly while circling, and their attack runs and loops are barely bent" (design change 0405,
//! answering Q0148 part 3). The two tests are 0405's Verification.

use super::*;

/// The closest pair of the units `ids`: its distance, centre to centre.
fn closest_pair(t: &TestSim, ids: &[u32]) -> f32 {
    let mut closest = f32::INFINITY;
    for (n, &a) in ids.iter().enumerate() {
        for &b in &ids[n + 1..] {
            closest = closest.min(t.pos(a).distance(t.pos(b)));
        }
    }
    closest
}

/// Design change 0405's Verification: "Six Sparrows sent to one point drift apart over a few seconds while circling,
/// and end about 48 su apart." Six Sparrows in a row 50 su apart, each sent to one point on its own (no group
/// destinations: the same point for all), arrive in a crowd, 4.4 su apart at closest after 2.5 s. Pushed by a quarter
/// of a HoverCraft's push (`UnitBaseDef::separation_strength`), they part gradually as they circle the point: the
/// closest pair is last under 90 % of the SeparationRadius at 13.4 s, under 99 % at 20 s and more than 0.01 su short
/// of it at 28.8 s, and they all end on the point's idle circle, idle. (At a HoverCraft's full push, measured once,
/// those were 3.4, 9.8 and 17.2 s.)
#[test]
fn six_sparrows_sent_to_one_point_drift_apart_while_circling_and_end_about_48_su_apart() {
    let goal = Vec2::new(1280.0, 1280.0);
    let mut builder = flat(40, 40);
    for i in 0..6 {
        builder = builder.spawn(SPARROW, Some(P1), point(1030.0 + 50.0 * i as f32, 1080.0));
    }
    let mut t = quiet(builder.start());
    let ids = [1, 2, 3, 4, 5, 6];
    for id in ids {
        order(&mut t, 1, P1, &[id], move_to(goal.x, goal.y));
    }
    let mut crowd = f32::INFINITY;
    // The last tick the closest pair was under 90 % of the radius, under 99 %, and more than 0.01 su short of it.
    let mut under = [0; 3];
    for _ in 0..16 * 60 {
        t.step();
        let tick = t.sim().tick();
        let closest = closest_pair(&t, &ids);
        if tick <= 16 * 4 {
            crowd = crowd.min(closest);
        }
        let least = [SEPARATION * 0.9, SEPARATION * 0.99, SEPARATION - 0.01];
        for (last, least) in under.iter_mut().zip(least) {
            if closest < least {
                *last = tick;
            }
        }
    }
    assert!(crowd < 10.0, "they arrive in a crowd: {crowd}");
    let [ninety, ninety_nine, about] = under;
    assert!(
        (16 * 10..16 * 16).contains(&ninety),
        "they drift apart, under 90 % until tick {ninety}"
    );
    assert!(ninety_nine <= 16 * 24, "under 99 % until tick {ninety_nine}");
    assert!(about <= 16 * 32, "more than 0.01 su short until tick {about}");
    for id in ids {
        let off = t.pos(id).distance(goal);
        assert!(near(off, IDLE_RADIUS, 0.05), "{id}: {off} su off the point");
        assert_eq!((current(&t, id), t.mover(id).orbit), (None, Some(goal)), "{id}");
    }
}

/// Where player 2's Hand of Heaven stands, a tile's corner, as in `super::strafing`.
const TARGET: Vec2 = Vec2::new(1280.0, 1280.0);

/// A hit (`super::hits`): its tick, its attacker, its target and the HP it took.
type Hit = (Tick, u32, u32, f32);

/// A Sparrow's tick: where it is, its own flight's heading (`Mover::momentum`), and the separation's push, its move
/// beyond its flight (`Mover::velocity`).
#[derive(Copy, Clone, Debug)]
struct Step {
    pos: Vec2,
    heading: Vec2,
    push: Vec2,
}

/// Player 1's Sparrows started at `starts` (ids 1, 2, ...), each ordered on its own to Attack player 2's Hand of Heaven
/// at [`TARGET`] (the next id), which a Militant of player 1's keeps in sight from 150 su south, for `ticks`: each
/// one's ticks, its Attack going on throughout, and the hits.
fn strafe(starts: &[Vec2], ticks: u32) -> (Vec<Vec<Step>>, Vec<Hit>) {
    let mut builder = flat(40, 40);
    for &s in starts {
        builder = builder.spawn(SPARROW, Some(P1), point(s.x, s.y));
    }
    let target = starts.len() as u32 + 1;
    builder = builder
        .spawn(ObjectKind::HandOfHeaven, Some(P2), point(TARGET.x, TARGET.y))
        .spawn(ObjectKind::Militant, Some(P1), point(TARGET.x, TARGET.y + 150.0));
    let mut t = quiet(builder.start());
    let ids: Vec<u32> = (1..target).collect();
    for &id in &ids {
        order(&mut t, 1, P1, &[id], OrderKind::Attack(ObjectId(target)));
    }
    let mut tracks = vec![Vec::new(); ids.len()];
    for _ in 0..ticks {
        let before: Vec<Vec2> = ids.iter().map(|&id| t.pos(id)).collect();
        t.step();
        for ((&id, track), before) in ids.iter().zip(&mut tracks).zip(before) {
            assert_eq!(current(&t, id), Some(OrderKind::Attack(ObjectId(target))));
            let mover = t.mover(id);
            track.push(Step {
                pos: t.pos(id),
                heading: mover.momentum.normalize(),
                push: t.pos(id) - before - mover.velocity,
            });
        }
    }
    (tracks, hits(&t))
}

/// The passes of `attacker` in `hits`, its hits no more than 2 s apart, each over the stretch of `track` from the end
/// of the pass before (from the order for the first) to 2 s after its last hit: that last hit's tick, its hits, how
/// close it came to the target, and how far the separation pushed it across its heading, the pushes added up.
fn passes(hits: &[Hit], attacker: u32, track: &[Step]) -> Vec<(Tick, u32, f32, f32)> {
    let mut grouped: Vec<(Tick, u32)> = Vec::new();
    for &(tick, a, _, _) in hits {
        match grouped.last_mut() {
            _ if a != attacker => {}
            Some(pass) if tick - pass.0 <= 32 => *pass = (tick, pass.1 + 1),
            _ => grouped.push((tick, 1)),
        }
    }
    let mut from = 0;
    let mut passes = Vec::new();
    for (last, count) in grouped {
        let to = (last as usize + 32).min(track.len());
        let stretch = &track[from..to];
        let closest = (stretch.iter()).fold(f32::INFINITY, |c, s| c.min(s.pos.distance(TARGET)));
        let pushed: f32 = stretch.iter().map(|s| s.heading.perp_dot(s.push)).sum();
        passes.push((last, count, closest, pushed.abs()));
        from = to;
    }
    passes
}

/// Design change 0405's Verification: "Two Sparrows making passes on the same target stay on their lines", read as:
/// every pass still runs over the target as a lone Sparrow's does, and the separation pushes it little across its
/// heading. Two Sparrows side by side 60 su apart, 880 su west of a Hand of Heaven that a Militant keeps in sight (as
/// in `super::strafing`), each ordered on its own to Attack it, converge on it at every pass, the worst case: they meet
/// over it, 1 su apart at closest. Over 60 s each makes the 7 passes a lone Sparrow makes, each landing 7 hits and
/// passing within 6 su of the target's centre (alone: 6 or 7 hits, within 4.4 su). On the first approach each keeps
/// within 12 su of the lone Sparrow's line until it is over the target; the pushes across its heading add up to 28 su
/// over the first pass, from the order to 2 s after its last hit, and to 4 su at most over each later one, its loop
/// included. (At a HoverCraft's full push, measured once: 21 su off the line, pushes of 62 su and up to 13 su.)
#[test]
fn two_sparrows_making_passes_on_one_target_stay_on_their_lines() {
    let starts = [Vec2::new(400.0, 1250.0), Vec2::new(400.0, 1310.0)];
    let ticks = 16 * 60;
    let (paired, paired_hits) = strafe(&starts, ticks);
    let nearest = (paired[0].iter().zip(&paired[1])).fold(f32::INFINITY, |n, (a, b)| n.min(a.pos.distance(b.pos)));
    assert!(nearest < 2.0, "they meet over the target, {nearest} su apart");
    for (k, track) in paired.iter().enumerate() {
        let (alone, alone_hits) = strafe(&starts[k..=k], ticks);
        let lone = passes(&alone_hits, 1, &alone[0]);
        let pair = passes(&paired_hits, k as u32 + 1, track);
        assert_eq!((lone.len(), pair.len()), (7, 7), "{k}: {lone:?}, {pair:?}");
        for &(last, count, closest, _) in &lone {
            assert!(
                count >= 6 && closest < 4.4,
                "{k} alone, the pass ending {last}: {lone:?}"
            );
        }
        for (n, &(last, count, closest, pushed)) in pair.iter().enumerate() {
            assert!(count >= 6 && closest < 6.0, "{k}, the pass ending {last}: {pair:?}");
            let most = if n == 0 { 28.0 } else { 4.0 };
            assert!(
                pushed <= most,
                "{k}, the pass ending {last}: pushed {pushed} su across: {pair:?}"
            );
        }
        // The first approach, until the lone Sparrow is over the target in its first pass: off its line, across its
        // heading.
        let alone = &alone[0];
        let over = (0..lone[0].0 as usize)
            .min_by(|&a, &b| alone[a].pos.distance(TARGET).total_cmp(&alone[b].pos.distance(TARGET)))
            .expect("a first pass");
        let off = (alone[..=over].iter().zip(track))
            .fold(0.0_f32, |off, (l, p)| off.max(l.heading.perp_dot(p.pos - l.pos).abs()));
        assert!(off <= 12.0, "{k}: {off} su off the lone Sparrow's line");
    }
}
