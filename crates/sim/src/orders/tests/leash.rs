//! Design change 0180's Verification (`systems/controls/command_behaviors.md > AttackMovingToLocation`: "the leash
//! limits chasing only"), with Ranges no type has yet (`defs::test_defs`). The detour and the PathReference after a
//! fight are tested with today's units in `tests/attack_move_leash.rs`.

use glam::Vec2;

use crate::command::DebugCommand;
use crate::defs::ObjectKind;
use crate::defs::test_defs::TestDef;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::orders::state::Behaviour;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::tuning::orders::ATTACK_MOVE_LEASH_GU;
use crate::units::{SU_PER_GU, Tick, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const UNIT: ObjectId = ObjectId(1);
const ENEMY: ObjectId = ObjectId(2);

/// 40 × 24 open tiles.
fn open() -> String {
    "........................................\n".repeat(24)
}

/// The target of `UNIT`'s AttackMove, if it is engaged.
fn engaged(t: &TestSim) -> Option<ObjectId> {
    let orders = t.sim().object(UNIT).unwrap().orders.as_ref().unwrap();
    match orders.current.as_ref().map(|active| &active.state) {
        Some(Behaviour::AttackMove(leg)) => leg.engaged.map(|pursuit| pursuit.target),
        _ => None,
    }
}

fn attacks(t: &TestSim, attacker: ObjectId) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == attacker))
        .map(|&(tick, _)| tick)
        .collect()
}

fn is_idle(t: &TestSim, id: ObjectId) -> bool {
    t.sim().object(id).unwrap().orders.as_ref().unwrap().is_idle()
}

/// The route of [`a_unit_fights_back_at_an_enemy_beyond_its_leash_within_its_range`] and
/// [`outranging_artillery_far_off_the_route_is_left_alone`]: west to east along y = 200.
const FROM: Vec2 = Vec2::new(200.0, 200.0);
const TO: Vec2 = Vec2::new(2400.0, 200.0);

/// "Attack-move past an enemy that shoots from 1 tile beyond the leash but within the unit's range: the unit fights
/// back." A Guard given a Range and a SightRange of 8 grid units attack-moves east along y = 200; a Peacekeeper of
/// player 2 given the same holds its position 7 grid units south of the route, 1 beyond the leash of 6. They come
/// into each other's sight and range together: the Guard fights back from its route, without a step towards the
/// Peacekeeper, kills it and walks on.
///
/// Then the case the old rule got wrong ("beyond the leash a unit used to disengage even from an enemy in weapon
/// range"): a second such Peacekeeper, and the Guard, once it is fighting it, is put 7 grid units off its route
/// (a debug teleport), with the Peacekeeper still in range. It fights on.
#[test]
fn a_unit_fights_back_at_an_enemy_beyond_its_leash_within_its_range() {
    let _guard = TestDef::unit(ObjectKind::Guard, |unit| unit.attack.as_mut().unwrap().range = 8.0);
    let _pk = TestDef::unit(ObjectKind::Peacekeeper, |unit| {
        unit.attack.as_mut().unwrap().range = 8.0
    });
    let _sight = SightRanges::new(8.0);
    let off = gu(ATTACK_MOVE_LEASH_GU + 1.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, ObjectKind::Guard, P1, FROM)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, Vec2::new(900.0, FROM.y + off))
        .order_at(2, P2, &[ENEMY], OrderKind::Hold, false)
        .order_at(2, P1, &[UNIT], OrderKind::AttackMove(TO), false)
        .run_until(2);
    let mut fought = None;
    while t.sim().object(ENEMY).is_some() {
        assert!(t.sim().tick() < 400, "the Peacekeeper lives\n{}", t.ascii(None));
        t.step();
        if fought.is_none() && engaged(&t) == Some(ENEMY) {
            fought = Some(t.pos(1));
        }
        assert_eq!(t.pos(1).y, FROM.y, "tick {}: never off its route", t.sim().tick());
    }
    let first = attacks(&t, ENEMY).first().copied().expect("the Peacekeeper shot");
    let fought = fought.expect("the Guard fought back");
    assert!(attacks(&t, UNIT).len() >= 3, "{:?}", attacks(&t, UNIT));
    assert!(
        attacks(&t, UNIT)[0] <= first + 24,
        "at once: {first} {:?}",
        attacks(&t, UNIT)
    );
    assert!(fought.x < 900.0, "before it got abreast of it: {fought}");
    while !is_idle(&t, UNIT) {
        assert!(t.sim().tick() < 1200, "it never arrives\n{}", t.ascii(None));
        t.step();
    }
    assert_eq!(t.pos(1), TO, "and walked on");

    // Beyond the leash, fighting an enemy in range.
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, ObjectKind::Guard, P1, FROM)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, Vec2::new(900.0, FROM.y + off))
        .order_at(2, P2, &[ENEMY], OrderKind::Hold, false)
        .order_at(2, P1, &[UNIT], OrderKind::AttackMove(TO), false)
        .run_until(2);
    while engaged(&t) != Some(ENEMY) {
        assert!(t.sim().tick() < 400, "it never fought\n{}", t.ascii(None));
        t.step();
    }
    let beyond = Vec2::new(t.pos(1).x, FROM.y + off);
    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::SetPosition { id: UNIT, pos: beyond })
        .run_until(now + 1);
    assert!(beyond.y - FROM.y > gu(ATTACK_MOVE_LEASH_GU));
    let before = attacks(&t, UNIT).len();
    while t.sim().object(ENEMY).is_some() {
        assert!(t.sim().tick() < now + 200, "it stopped fighting\n{}", t.ascii(None));
        t.step();
        if t.sim().object(ENEMY).is_some() {
            assert_eq!(engaged(&t), Some(ENEMY), "tick {}: it let go", t.sim().tick());
        }
        assert_eq!(t.pos(1), beyond, "it stood where it was");
    }
    assert!(attacks(&t, UNIT).len() > before, "and fired from there");
}

/// "Enemy artillery outranging from far off the route: the unit keeps walking." A Peacekeeper of player 2 given a
/// Range of 12 grid units (and a SightRange of 13: an artillery's, though with a Damage of 2) holds its position 10
/// grid units south of a Guard's route and shoots it as it passes. The Guard sees it (it is given a SightRange of
/// 11), but out of its own Range of 3 it could only get to it from 7 grid units off its route or more, beyond the
/// leash: it never goes for it, and walks on along its route to its destination.
#[test]
fn outranging_artillery_far_off_the_route_is_left_alone() {
    let _artillery = TestDef::unit(ObjectKind::Peacekeeper, |unit| {
        let attack = unit.attack.as_mut().unwrap();
        (attack.range, attack.damage) = (12.0, 2.0);
    });
    let _sight = SightRanges::with(&[(ObjectKind::Guard, 11.0), (ObjectKind::Peacekeeper, 13.0)]);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, ObjectKind::Guard, P1, FROM)
        .spawn_unit_at(
            1,
            ObjectKind::Peacekeeper,
            P2,
            Vec2::new(1200.0, FROM.y + 10.0 * SU_PER_GU),
        )
        .order_at(2, P2, &[ENEMY], OrderKind::Hold, false)
        .order_at(2, P1, &[UNIT], OrderKind::AttackMove(TO), false)
        .run_until(2);
    let mut seen = false;
    while !is_idle(&t, UNIT) {
        assert!(t.sim().tick() < 800, "it never arrives\n{}", t.ascii(None));
        t.step();
        assert_eq!(engaged(&t), None, "tick {}: it went for the artillery", t.sim().tick());
        assert_eq!(t.pos(1).y, FROM.y, "tick {}: never off its route", t.sim().tick());
        let enemy = t.sim().object(ENEMY).unwrap();
        seen |= t.sim().is_visible(P1, enemy);
    }
    assert!(seen, "the Guard saw it");
    assert!(!attacks(&t, ENEMY).is_empty(), "it shot the Guard");
    assert_eq!(attacks(&t, UNIT), [] as [Tick; 0]);
    assert_eq!(t.pos(1), TO);
}

/// Gives kinds other SightRanges while it lives (`TestDef`).
struct SightRanges {
    _defs: Vec<TestDef>,
}

impl SightRanges {
    /// The Guard's and the Peacekeeper's, both `range` grid units.
    fn new(range: f32) -> Self {
        Self::with(&[(ObjectKind::Guard, range), (ObjectKind::Peacekeeper, range)])
    }

    fn with(ranges: &[(ObjectKind, f32)]) -> Self {
        let defs = ranges.iter().map(|&(kind, range)| {
            let mut def = kind.def().clone();
            def.sight_range = range;
            TestDef::set(def)
        });
        Self { _defs: defs.collect() }
    }
}
