//! Unit tests of crushing ([`crushes`] as `space::blocks`'s rule 3, [`drives`], [`victims`]), without a game, on a
//! 64 × 40 Silhouette that turns with its facing (the Hand of Heaven's) among 24 × 24 Light Infantry and a 36 × 36
//! Guard. The game's tests are `crates/sim/tests/crushing/`.

use super::*;
use crate::defs::UnitBase;
use crate::ids::{ObjectId, PlayerId};
use crate::movement::space::{Oriented, blocks};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Half the Hand of Heaven's Silhouette: 64 su along its facing.
const TANK: Vec2 = Vec2::new(32.0, 20.0);
const HALF_PI: f32 = core::f32::consts::FRAC_PI_2;

/// A body of `owner` at `pos`, half extents `half`, lying along `facing` if it turns with it.
fn body(id: u32, owner: Option<PlayerId>, pos: Vec2, half: Vec2, facing: Option<f32>) -> Body {
    let oriented = facing.map(|facing| Oriented::facing(half, facing));
    Body {
        id: ObjectId(id),
        owner,
        pos,
        half: oriented.map_or(half, |o| o.bound_half()),
        oriented,
        velocity: Vec2::ZERO,
        heading: Vec2::ZERO,
        stationary: true,
        progress: None,
        arrived_with: None,
        yields: false,
        gathers: false,
        leads: true,
        moved: 0,
        crushes: false,
        crushable: false,
    }
}

/// The tank, id 1, of player 1 at `pos` facing `facing`, its move this tick crushing or not.
fn tank(pos: Vec2, facing: f32, crushes: bool) -> Body {
    Body {
        crushes,
        ..body(1, Some(P1), pos, TANK, Some(facing))
    }
}

/// A Light Infantry of `owner` at `pos`.
fn infantry(id: u32, owner: Option<PlayerId>, pos: Vec2) -> Body {
    Body {
        crushable: true,
        ..body(id, owner, pos, Vec2::splat(12.0), None)
    }
}

/// The crushes [`victims`] finds for the tank, `bodies[0]`, which moved from `from` facing `start` to where it lies.
fn crushed(bodies: &[Body], from: Vec2, start: f32) -> Vec<usize> {
    let mut layer = Layer::default();
    layer.reset(32, 32);
    for (index, b) in bodies.iter().enumerate() {
        layer.insert(index as u32, b.pos, b.half, b.kinds());
    }
    let mut found = Vec::new();
    victims(bodies, &layer, 0, from, start, &mut found);
    assert!(found.iter().all(|c| c.by == 0));
    found.iter().map(|c| c.victim).collect()
}

/// `space::blocks`'s rule 3 (`systems/units.md > UnitCollision > Crushing`): a move that crushes goes through the
/// crushable units of another player and through nothing else: own infantry ("No friendly crushing"), a neutral one,
/// a Guard (Heavy Infantry, "cannot be crushed") all block it; a crusher whose move doesn't crush is "a normal hard
/// obstacle"; and infantry moving into a crusher meet it ("Crushable units avoid crushers").
#[test]
fn a_crushing_move_goes_through_crushable_enemies_alone() {
    let at = Vec2::new(500.0, 500.0);
    let (crushing, standing) = (tank(at, 0.0, true), tank(at, 0.0, false));
    let enemy = infantry(2, Some(P2), at + Vec2::new(50.0, 0.0));
    assert!(!blocks(&crushing, &enemy));
    assert!(blocks(&standing, &enemy));
    assert!(blocks(&enemy, &crushing), "the infantry's own move");
    assert!(blocks(&crushing, &infantry(3, Some(P1), enemy.pos)), "its own");
    assert!(blocks(&crushing, &infantry(4, None, enemy.pos)), "a neutral one");
    let guard = body(5, Some(P2), enemy.pos, Vec2::splat(18.0), None);
    assert!(blocks(&crushing, &guard), "Heavy Infantry");
}

/// Design change 0418: "a crusher turning in place (Stationary + Turning) crushes the crushable enemies its turning
/// Silhouette sweeps over". A quarter turn in place from east to south sweeps a Light Infantry standing off the tank's
/// front right corner, 34 su east and south of its centre (its near corner 31.1 su out along the diagonal, inside the
/// 32 su of the tank's half length there), which the tank overlaps neither facing east nor facing south. An own
/// infantry there is no victim, and a turn of 20° the other way doesn't reach it.
#[test]
fn a_turn_in_place_crushes_the_enemy_its_silhouette_sweeps() {
    let at = Vec2::new(500.0, 500.0);
    let swept = at + Vec2::new(34.0, 34.0);
    let ending = tank(at, HALF_PI, false);
    assert!(
        !ending
            .shape_at(at)
            .overlaps(&infantry(2, Some(P2), swept).shape_at(swept))
    );
    assert!(
        !tank(at, 0.0, false)
            .shape_at(at)
            .overlaps(&infantry(2, Some(P2), swept).shape_at(swept))
    );
    assert_eq!(crushed(&[ending, infantry(2, Some(P2), swept)], at, 0.0), [1]);
    assert!(crushed(&[ending, infantry(2, Some(P1), swept)], at, 0.0).is_empty());
    let away = tank(at, -20f32.to_radians(), false);
    assert!(crushed(&[away, infantry(2, Some(P2), swept)], at, 0.0).is_empty());
}

/// The move crushes what its shape ends in, and what it passed over on the way (`Shape::sweep`): a move of 100 su east
/// (far more than a tick's, to pass the infantry whole) crushes the infantry it went over, 50 su east of where it
/// began, which it overlaps neither where it began nor where it ended; and the one it ends on.
#[test]
fn a_move_crushes_what_it_ends_on_and_what_it_passed_over() {
    let from = Vec2::new(500.0, 500.0);
    let to = from + Vec2::new(100.0, 0.0);
    let bodies = [
        tank(to, 0.0, false),
        infantry(2, Some(P2), from + Vec2::new(50.0, 0.0)),
        infantry(3, Some(P2), to + Vec2::new(30.0, 0.0)),
        infantry(4, Some(P2), to + Vec2::new(0.0, 40.0)),
    ];
    assert_eq!(crushed(&bodies, from, 0.0), [1, 2]);
}

/// "A crusher that is stationary and **not** turning stays a hard obstacle": one that neither moved nor turned crushes
/// nothing, not even an enemy infantry that overlaps it already (design change 0106's units that already overlap).
#[test]
fn a_crusher_that_neither_moves_nor_turns_crushes_nothing() {
    let at = Vec2::new(500.0, 500.0);
    let bodies = [tank(at, 0.0, false), infantry(2, Some(P2), at + Vec2::new(30.0, 0.0))];
    assert!(bodies[0].shape_at(at).overlaps(&bodies[1].shape_at(bodies[1].pos)));
    assert!(crushed(&bodies, at, 0.0).is_empty());
}

/// [`drives`]: a crusher's move crushes unless it is Stopping (module docs) or its attack holds it; a Light Infantry's
/// never. At rest every move crushes. With momentum: one that steers along its path does, unless it brakes against
/// its leg faster than one tick's Deceleration (1 su a tick) takes away, a Reverse leg's included; one that doesn't
/// steer coasts, and doesn't.
#[test]
fn a_crusher_s_move_crushes_unless_it_is_stopping() {
    let tracked = UnitBase::TrackedVehicle.def();
    let movement = MovementAttrs::SpeedTurnRadius {
        track_width: 30.0,
        acceleration: Some(0.5),
        deceleration: Some(1.0),
        max_speed: 5.0,
        reverse_max_speed: 2.5,
    };
    let east = Vec2::new(3.0, 0.0);
    let mover = |momentum: Vec2, reverses: bool| Mover {
        momentum,
        reverses,
        ..Mover::default()
    };
    let drives = |base: &UnitBaseDef, mover: &Mover, facing: f32, held: bool, steers: bool| {
        super::drives(base, &movement, mover, facing, held, None, || steers)
    };
    let at_rest = mover(Vec2::ZERO, false);
    assert!(drives(tracked, &at_rest, 0.0, false, false));
    assert!(!drives(tracked, &at_rest, 0.0, true, true), "held by its attack");
    assert!(!drives(UnitBase::LightInfantry.def(), &at_rest, 0.0, false, true));
    assert!(drives(tracked, &mover(east, false), 0.0, false, true), "on its way");
    assert!(!drives(tracked, &mover(east, false), 0.0, false, false), "coasting");
    let pi = core::f32::consts::PI;
    assert!(
        !drives(tracked, &mover(east, false), pi, false, true),
        "braking before it sets off west"
    );
    assert!(
        drives(tracked, &mover(Vec2::new(1.0, 0.0), false), pi, false, true),
        "at rest within the tick"
    );
    assert!(
        !drives(tracked, &mover(east, true), 0.0, false, true),
        "braking before it backs up"
    );
    assert!(drives(tracked, &mover(-east, true), 0.0, false, true), "backing up");
}
