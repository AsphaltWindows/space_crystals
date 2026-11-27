//! The soft separation of air units (`movement::air`) in a sim: test air units (a Peacekeeper given a HoverCraft base
//! and a SeparationRadius of 32 su, the Sparrow's in slice M5-H1: no HoverCraft type of the game has one yet; (M5-H2) a
//! Guard given the Glider base, which stands where it is pushed, unlike a Glider in flight) and Sparrows. The scenario
//! tests in `crates/sim/tests/air/` cover a crowd of Sparrows and two Sparrows strafing one target.

use glam::Vec2;

use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, UnitBase};
use crate::ids::{ObjectId, PlayerId};
use crate::testing::{ScenarioBuilder, TestSim};
use crate::tuning::air::SEPARATION_PUSH_SU;

const P1: PlayerId = PlayerId(1);

/// A Peacekeeper that flies, keeping 32 su apart from other air units.
fn hovercraft() -> TestDef {
    TestDef::unit(ObjectKind::Peacekeeper, |unit| {
        unit.base = UnitBase::HoverCraft;
        unit.separation_radius = Some(32.0);
    })
}

/// (M5-H2) A Guard built on the Glider base, keeping 32 su apart from other air units: it takes a Glider's share of
/// each push (`UnitBaseDef::separation_strength`), and stands where it is pushed, its movement still a Guard's.
fn glider_based() -> TestDef {
    TestDef::unit(ObjectKind::Guard, |unit| {
        unit.base = UnitBase::Glider;
        unit.separation_radius = Some(32.0);
    })
}

/// 16 × 16 open tiles with units of `kind` of player 1 at `points`.
fn field(kind: ObjectKind, points: &[Vec2]) -> TestSim {
    field_of(&points.iter().map(|&p| (kind, p)).collect::<Vec<_>>())
}

/// 16 × 16 open tiles with player 1's `units`, each of its kind at its point.
fn field_of(units: &[(ObjectKind, Vec2)]) -> TestSim {
    let mut builder = ScenarioBuilder::new(&"................\n".repeat(16));
    for &(kind, p) in units {
        builder = builder.spawn(kind, Some(P1), crate::scenario::SpawnAt::Point(p));
    }
    let mut t = builder.start();
    t.without_auto_targeting();
    t
}

/// `systems/units.md > Air Collision`: "a gentle repulsion force that prevents stacking but does not hard-block".
/// Two idle air units 10 su apart part along the line between them, each by `SEPARATION_PUSH_SU` a tick (T81: "by at
/// most 2 su a tick each"), the last step by half of what the pair lacks, until they are their SeparationRadius
/// apart; then they stand. Pushed, an idle unit has neither a goal nor a velocity (the checks of every tick pass).
#[test]
fn two_idle_air_units_part_by_at_most_2_su_a_tick_each_to_their_separation_radius() {
    let _air = hovercraft();
    let mut t = field(
        ObjectKind::Peacekeeper,
        &[Vec2::new(500.0, 500.0), Vec2::new(510.0, 500.0)],
    );
    let mut gaps = Vec::new();
    for _ in 0..10 {
        t.step();
        gaps.push(t.pos(2).x - t.pos(1).x);
        assert_eq!(t.pos(1).y, 500.0);
        assert_eq!(t.mover(1).velocity, Vec2::ZERO);
    }
    // In the game from the start, they part from tick 1 on.
    let want = [14.0, 18.0, 22.0, 26.0, 30.0, 32.0, 32.0, 32.0, 32.0, 32.0];
    assert_eq!(gaps, want, "{gaps:?}");
    assert_eq!(t.pos(1).x + t.pos(2).x, 1010.0, "each by as much");
    assert_eq!(SEPARATION_PUSH_SU, 2.0);
}

/// (M5-H2) Design change 0405: "**Gliders push apart slowly** [...] with a **much weaker** repulsion than other air
/// units", (M5-N7) at "**a third** of the HoverCraft strength [...]; was a quarter" (design change 0412). Two idle units
/// of the Glider base 10 su apart part by a third of what two HoverCraft would: two thirds of a space unit a tick each
/// while the pair lacks 4 su or more, then a third of half of what it lacks, so that the last few su close by a third
/// each tick and the pair comes ever nearer its SeparationRadius without a last step onto it.
#[test]
fn two_idle_units_of_the_glider_base_part_by_a_third_of_the_push() {
    let _glider = glider_based();
    let mut t = field(ObjectKind::Guard, &[Vec2::new(500.0, 500.0), Vec2::new(510.0, 500.0)]);
    let mut gaps = Vec::new();
    for _ in 0..21 {
        t.step();
        gaps.push(t.pos(2).x - t.pos(1).x);
        assert_eq!(t.mover(1).velocity, Vec2::ZERO);
    }
    // 4/3 su a tick from 10 to 28.667, then two thirds of what the pair lacks of 32 left each tick.
    let mut want: Vec<f32> = (1..=14).map(|n| 10.0 + 4.0 * n as f32 / 3.0).collect();
    want.extend([29.778, 30.519, 31.012, 31.342, 31.561, 31.707, 31.805]);
    for (gap, want) in gaps.iter().zip(&want) {
        assert!((gap - want).abs() < 0.001, "{gap} su, not {want}: {gaps:?}");
    }
    assert!((t.pos(1).x + t.pos(2).x - 1010.0).abs() < 0.001, "each by as much");
    t.run(40);
    let gap = t.pos(2).x - t.pos(1).x;
    assert!((31.99..=32.0).contains(&gap), "{gap}");
}

/// (M5-H2) Each unit of a pair takes its own base's share of the push: a HoverCraft and a unit of the Glider base 10 su
/// apart part by 2 su a tick and (M5-N7, design change 0412) two thirds of a space unit, until the pair lacks less than
/// 4 su.
#[test]
fn a_hovercraft_and_a_glider_take_each_its_own_share_of_the_push() {
    let (_air, _glider) = (hovercraft(), glider_based());
    let mut t = field_of(&[
        (ObjectKind::Peacekeeper, Vec2::new(500.0, 500.0)),
        (ObjectKind::Guard, Vec2::new(510.0, 500.0)),
    ]);
    for n in 1..=7 {
        t.step();
        let n = n as f32;
        assert_eq!(t.pos(1).x, 500.0 - 2.0 * n);
        assert!((t.pos(2).x - (510.0 + 2.0 * n / 3.0)).abs() < 0.001, "{}", t.pos(2).x);
    }
}

/// Units on the very same point part along the x axis, the lower id to the west (`movement::air`'s module docs).
#[test]
fn air_units_on_one_point_part_west_and_east() {
    let _air = hovercraft();
    let mut t = field(
        ObjectKind::Peacekeeper,
        &[Vec2::new(500.0, 500.0), Vec2::new(500.0, 500.0)],
    );
    t.run(1);
    assert_eq!((t.pos(1), t.pos(2)), (Vec2::new(498.0, 500.0), Vec2::new(502.0, 500.0)));
}

/// A HoverCraft stays on the map, pushed or not (design change 0045: "clamp HoverCraft to the map"), its box inside
/// it; a Glider may be pushed off it, as it may fly there (`movement::glider`). The one at the edge moves no further,
/// and the other one goes on alone, by half of what the pair lacks.
#[test]
fn a_hovercraft_is_pushed_no_further_than_the_map_s_edge() {
    let _air = hovercraft();
    let half = ObjectKind::Peacekeeper.def().half_extents().unwrap();
    let mut t = field(
        ObjectKind::Peacekeeper,
        &[Vec2::new(half.x, 500.0), Vec2::new(half.x + 4.0, 500.0)],
    );
    t.run(20);
    assert_eq!(t.pos(1).x, half.x);
    let gap = t.pos(2).x - t.pos(1).x;
    assert!((31.9..32.0).contains(&gap), "{gap}");
}

/// The pairs come from the movement's spatial grid, cells of a tile: a pair across a cell's border, or a few cells
/// apart within the larger radius, is found as well as one within a cell; units farther apart than both radii aren't
/// pushed.
#[test]
fn only_air_units_within_their_separation_radius_are_pushed() {
    let _air = hovercraft();
    // Across the border of two tiles (x = 512), and 40 su apart, beyond 32.
    let points = [
        Vec2::new(500.0, 300.0),
        Vec2::new(520.0, 300.0),
        Vec2::new(500.0, 700.0),
        Vec2::new(540.0, 700.0),
    ];
    let mut t = field(ObjectKind::Peacekeeper, &points);
    t.run(1);
    assert_eq!((t.pos(1).x, t.pos(2).x), (498.0, 522.0));
    assert_eq!((t.pos(3), t.pos(4)), (points[2], points[3]));
    assert_eq!(t.sim().object(ObjectId(4)).map(|o| o.pos), Some(points[3]));
}

/// Ground units and air units don't push one another: a test air unit over a Guard stays where it is, and so does
/// the Guard (`systems/units.md > Air Collision`: air units "do not collide with ground units").
#[test]
fn a_ground_unit_under_an_air_unit_is_not_pushed() {
    let _air = hovercraft();
    let at = Vec2::new(500.0, 500.0);
    let mut t = field(ObjectKind::Peacekeeper, &[at]);
    t.spawn_unit_at(1, ObjectKind::Guard, P1, at + Vec2::new(4.0, 0.0));
    t.run(10);
    assert_eq!((t.pos(1), t.pos(2)), (at, at + Vec2::new(4.0, 0.0)));
}
