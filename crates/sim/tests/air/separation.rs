//! Soft separation (`super`): `systems/units.md > Air Collision`: "Air units use soft separation with other air units —
//! a gentle repulsion force that prevents stacking but does not hard-block"; "SeparationRadius — per unit type,
//! circular, the distance at which air-to-air repulsion activates. Must be larger than the unit's Silhouette"; the
//! Sparrow's, 32 su (design change 0328).

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

/// Six Sparrows in a row 50 su apart, each sent to one point on its own (no group destinations: the same point for
/// all), circle it without stacking. They arrive in a crowd after 2 s, nearer than 20 su; the pushes part them, and
/// from 10 s on the closest pair is within 1 % of the SeparationRadius, from 20 s on within 0.001 su of it: a soft
/// push (`movement::air`) leaves a pair at the radius, and each tick's flight may bring it a hair nearer again before
/// the next. By then they all circle the point on its idle circle.
#[test]
fn six_sparrows_sent_to_one_point_circle_it_without_stacking() {
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
    for _ in 0..16 * 10 {
        t.step();
        crowd = crowd.min(closest_pair(&t, &ids));
    }
    assert!(crowd < 20.0, "they arrive in a crowd: {crowd}");
    for second in 10..40 {
        for _ in 0..16 {
            t.step();
            let closest = closest_pair(&t, &ids);
            let least = if second < 20 {
                SEPARATION * 0.99
            } else {
                SEPARATION - 0.001
            };
            assert!(closest >= least, "{second} s: {closest}");
        }
    }
    for id in ids {
        assert!(
            near(t.pos(id).distance(goal), IDLE_RADIUS, 0.5),
            "{}",
            t.pos(id).distance(goal)
        );
    }
}
