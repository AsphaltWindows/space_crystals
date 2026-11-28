//! The trail (`factions/syndicate/units/shadebringer.md > Sludge (toggle)`: "While on, the Shadebringer leaves a
//! **trail of sludge** on the ground in its wake. Each patch of sludge lasts **X seconds** and then disappears"; "the
//! sludge is spread **behind** it, so laying the trail doesn't slow it"; `> Placeholder stats`: "each patch lasts **X
//! = 10 s**", "trail width 48 su"; plan decision T93: "A Shadebringer with a toggle on drops a patch behind its motion
//! (its rear edge, or its front when it reverses; D44) each time it has moved 24 su since the last. The trail is a
//! chain of discs").

use std::collections::BTreeMap;

use super::*;

/// A patch's centre as a key, in 64ths of a su.
fn key(at: Vec2) -> (i64, i64) {
    ((at.x * 64.0) as i64, (at.y * 64.0) as i64)
}

/// The patches on the ground now that unit `id` laid, by their centres ([`key`]), with their ends.
fn ends(t: &TestSim, id: ObjectId) -> BTreeMap<(i64, i64), Tick> {
    laid_by(t, id).iter().map(|p| (key(p.centre), p.until)).collect()
}

/// A Shadebringer driving east from (320, 800) to (1700, 800) with its Sludge on from tick 2 lays its trail behind it,
/// on its way: each new patch at its rear edge (its centre less half its 56 su length, along its facing) where the
/// tick's move left it, a disc 48 su across, the next once that edge is 24 su from it (plan decision T93: the trail's
/// discs 24 to 32 su apart at its top speed of 8 su a tick), the last laid again until then; it is never in the sludge
/// it lays, and never slowed. Every patch is on the ground for 10 s from the tick it was last laid, and gone in the tick
/// after; its tank empty in tick 161, 10 s on, the trail ends there, the last patch gone 10 s later.
#[test]
fn the_trail_is_laid_behind_a_driving_shadebringer_48_su_wide_each_patch_lasting_10_s() {
    let mut t = started(field().spawn(SHADEBRINGER, Some(P1), point(320.0, 800.0)));
    let sb = id(1);
    t.at(2, P1, press(&[sb], sludge(true)));
    t.at(2, P1, order(&[sb], move_to(1700.0, 800.0)));
    run_until(&mut t, 1);
    let mut before = ends(&t, sb);
    let mut centres: Vec<Vec2> = Vec::new();
    while t.sim().tick() < 1 + FULL_USE + LIFE + 1 {
        t.step();
        let tick = t.sim().tick();
        let o = object(&t, sb);
        assert_eq!(slow_until(&t, sb), None, "tick {tick}: never slowed");
        let now = ends(&t, sb);
        for (centre, until) in &before {
            match now.get(centre) {
                None => assert_eq!(tick, *until, "{centre:?}: gone in its tick"),
                Some(&again) => assert!(again == *until || again == tick + LIFE, "{centre:?}: {again}"),
            }
        }
        let new: Vec<Patch> = (laid_by(&t, sb).into_iter())
            .filter(|p| !before.contains_key(&key(p.centre)))
            .collect();
        if tick <= 1 + FULL_USE {
            let rear = o.pos - Vec2::new(HALF_LENGTH, 0.0);
            assert_eq!(o.facing, 0.0, "east");
            match new[..] {
                [patch] => {
                    assert_eq!(
                        (patch.centre, patch.radius, patch.until),
                        (rear, RADIUS, tick + LIFE),
                        "tick {tick}"
                    );
                    if let Some(last) = centres.last() {
                        let gap = patch.centre.x - last.x;
                        assert!((24.0..24.0 + TOP).contains(&gap), "tick {tick}: {gap} su on");
                    }
                    centres.push(patch.centre);
                }
                [] => {
                    let newest = *centres.last().expect("laid from tick 2");
                    assert!(rear.x - newest.x < 24.0, "tick {tick}: laid again");
                    assert_eq!(now.get(&key(newest)), Some(&(tick + LIFE)), "tick {tick}: laid again");
                }
                _ => panic!("tick {tick}: {new:?}"),
            }
            assert!(!in_sludge(&t, o.pos), "tick {tick}: its centre ahead of it");
        } else {
            assert_eq!(new, [], "tick {tick}: off");
        }
        before = now;
    }
    assert!(t.sim().patches().is_empty(), "all gone");
    assert!(centres.len() >= 40, "{} patches", centres.len());
    assert_eq!(centres[0], Vec2::new(321.0 - HALF_LENGTH, 800.0), "its first move's");
}

/// Backing up (`systems/controls/commands.md > Reverse`), the Shadebringer lays its trail at its front, behind its
/// motion (plan decision T93: "its front when it reverses"): ordered to reverse west, its Sludge switched on once it
/// backs up, it lays each new patch at its front edge, half its length ahead of its centre along its facing, which
/// stays east; it is never in the sludge it lays.
#[test]
fn backing_up_the_shadebringer_lays_its_trail_at_its_front() {
    let mut t = started(field().spawn(SHADEBRINGER, Some(P1), point(1200.0, 800.0)));
    let sb = id(1);
    t.at(2, P1, order(&[sb], OrderKind::Reverse(Vec2::new(700.0, 800.0))));
    t.at(10, P1, press(&[sb], sludge(true)));
    run_until(&mut t, 9);
    assert!(object(&t, sb).pos.x < 1200.0, "backing up");
    let mut fronts = 0;
    while t.sim().tick() < 120 {
        let before = ends(&t, sb);
        t.step();
        let o = object(&t, sb);
        let tick = t.sim().tick();
        assert_eq!(o.facing, 0.0, "tick {tick}: east, backing up west");
        assert!(o.pos.x < o.prev_pos.x, "tick {tick}: backing up");
        assert_eq!(slow_until(&t, sb), None, "tick {tick}");
        assert!(!in_sludge(&t, o.pos), "tick {tick}: its centre behind it");
        for patch in laid_by(&t, sb).iter().filter(|p| !before.contains_key(&key(p.centre))) {
            assert_eq!(
                patch.centre,
                o.pos + Vec2::new(HALF_LENGTH, 0.0),
                "tick {tick}: at its front"
            );
            fronts += 1;
        }
    }
    assert!(fronts >= 8, "{fronts} patches");
}
