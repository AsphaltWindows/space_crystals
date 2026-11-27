//! (M5-O3) `systems/combat.md > TailDisjointed`: what a shot strikes at the spot it locked ([`at_the_spot`]); and,
//! through the whole sim, a target walking off the spot in the Gunner's Firing of 4 frames (design change 0333), and,
//! with the Gunner given other numbers (`defs::test_defs`) where its own can't show a rule, an area attack's blast at
//! the spot (the GDO's Artillery Rig's, which isn't built). The Gunner is played in `crates/sim/tests/gunner.rs`.

use super::*;
use crate::combat::overlap;
use crate::combat::phases::effects::at_the_spot;
use crate::defs::test_defs::TestDef;
use crate::defs::{BlastDomain, UnitBase};
use crate::events::SimEvent;
use crate::orders::{OrderKind, Target};
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::{SU_PER_GU, Tick, gu};

const P2: PlayerId = PlayerId(2);
const GUNNER: ObjectId = ObjectId(1);
/// `factions/cults/units/gunner.md > AttackAttributes`: "FiringDuration: 4 frames" (design change 0333).
const FIRING: Tick = 4;

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// The tick of the end of the Gunner's next Aiming, and the spot it locked (`AttackPhase::FiringAt`).
fn until_locked(t: &mut TestSim) -> (Tick, Vec2) {
    loop {
        assert!(t.sim().tick() < 300, "it never locked a spot\n{}", t.ascii(None));
        t.step();
        let attack = t.sim().object(GUNNER).unwrap().attack.as_ref().unwrap();
        if let Some(AttackPhase::FiringAt { at, .. }) = attack.phase
            && attack.elapsed == 0
        {
            return (t.sim().tick(), at);
        }
    }
}

/// The HP damage `id` took in each hit of the run so far.
fn hits_on(t: &TestSim, id: ObjectId) -> Vec<f32> {
    let hit = |(_, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit { target, damage, .. } if target == id => Some(damage),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

/// Design change 0304, "whatever unit stands at the spot then is hit instead (any owner)", and the placeholder for two
/// at once. Player 2's Peacekeeper (#2) and a neutral Guard (#3) stand side by side, their edges touching along x = 212;
/// a Militant of player 2 on a Hover Craft base (#4), an air unit, further on. A shot at a point of their shared edge
/// strikes the one it was aimed at, or the first in id order if it was aimed at neither; one at a point of a single
/// shape strikes that one, whatever it was aimed at; one at a point of none misses. A Ground attack never strikes the air
/// unit (`systems/combat.md > ValidTarget` 3), nor the shooter itself.
#[test]
fn a_shot_strikes_what_stands_at_the_spot_the_object_aimed_at_first() {
    let _flyer = TestDef::unit(ObjectKind::Militant, |unit| unit.base = UnitBase::HoverCraft);
    let map = Map::new(8, 8);
    let objects: BTreeMap<ObjectId, Object> = [
        me(Some(MINE)),
        object(ObjectId(2), ObjectKind::Peacekeeper, Some(P2), Vec2::new(200.0, 100.0)),
        object(ObjectId(3), ObjectKind::Guard, None, Vec2::new(230.0, 100.0)),
        object(ObjectId(4), ObjectKind::Militant, Some(P2), Vec2::new(300.0, 100.0)),
    ]
    .into_iter()
    .map(|o| (o.id, o))
    .collect();
    let targets = Targets::snapshot(&map, &objects);
    let attrs = AttackAttrs {
        attack_type: AttackType::TailDisjointed,
        ..attrs(4, 4, 6, 8)
    };
    let shooter = Shooter::new(&map, &objects[&ME], &attrs, None, 0.0);
    let struck = |x: f32, aimed: Option<u32>| {
        let at = Vec2::new(x, 100.0);
        at_the_spot(&shooter, at, aimed.map(ObjectId), &targets).map(|t| t.id.0)
    };
    assert_eq!(struck(212.0, None), Some(2), "on both: the first in id order");
    assert_eq!(struck(212.0, Some(3)), Some(3), "on both: the one aimed at");
    assert_eq!(struck(212.0, Some(2)), Some(2));
    assert_eq!(
        struck(240.0, Some(2)),
        Some(3),
        "the one aimed at has left: the one there"
    );
    assert_eq!(struck(260.0, Some(3)), None, "nothing there: a miss");
    assert_eq!(
        struck(300.0, None),
        None,
        "an air unit: a Ground attack can't target it"
    );
    assert_eq!(struck(0.0, None), None, "nor the shooter itself");
}

/// `systems/combat.md > TailDisjointed`: "Targets can dodge by moving out of the locked location". The Gunner, whose
/// Firing lasts 4 frames (design change 0333: "at 4 a walking infantry unit can step off it"), attacks a Peacekeeper of
/// player 2 standing 3 grid units off; in the frame after the spot is locked, the Peacekeeper sets off north at 4 su a
/// frame. Walking all through Firing, it is 16 su off the spot when the shot lands, clear of it (its Silhouette reaches
/// 12 su from its centre): a miss. Stopped after two frames, 8 su off, it still covers the spot, and is hit.
#[test]
fn a_target_that_walks_off_the_spot_during_firing_dodges() {
    let peacekeeper = ObjectId(2);
    let run = |walk: Tick| {
        let from = Vec2::new(300.0, 320.0);
        let mut t = ScenarioBuilder::new(&open()).start();
        t.without_auto_targeting()
            .spawn_unit_at(1, ObjectKind::Gunner, MINE, from)
            .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, from + Vec2::new(24.0 + gu(3.0), 0.0))
            .order_at(2, MINE, &[GUNNER], OrderKind::Attack(peacekeeper), false);
        let (locked, at) = until_locked(&mut t);
        let north = Target::Point(Vec2::new(at.x, 40.0));
        t.order_at(locked + 1, P2, &[peacekeeper], OrderKind::Move(north), false)
            .order_at(locked + 1 + walk, P2, &[peacekeeper], OrderKind::Stop, false)
            .run_until(locked + FIRING);
        let off = at.y - t.sim().object(peacekeeper).unwrap().pos.y;
        let missed = (t.events().iter()).any(|(tick, e)| {
            *tick == locked + FIRING
                && *e
                    == SimEvent::Missed {
                        attacker: GUNNER,
                        target: Some(peacekeeper),
                        from,
                        to: at,
                    }
        });
        (off, missed, hits_on(&t, peacekeeper))
    };
    assert_eq!(run(FIRING), (16.0, true, vec![]), "walking all the while: dodged");
    assert_eq!(run(2), (8.0, false, vec![15.0]), "stopped on the spot: hit");
}

/// `factions/gdo/units/artillery_rig.md > AttackAttributes`, the GDO's Artillery Rig's TailDisjointed area attack:
/// "the blast lands there at the end of Firing (no tracking projectile)". The Gunner, given an area attack of Damage 100
/// over 40 su, attacks the ground at the spot where a Peacekeeper of player 2 stands, and one of its own side's
/// Militants stands across the circle's edge. At the end of Firing a `SimEvent::Blast` there hits both with their
/// shares (`systems/combat.md > AoEDamage`: friendly fire), and no shot event comes with it.
#[test]
fn an_area_attack_blasts_the_spot_at_the_end_of_firing() {
    let _artillery = TestDef::unit(ObjectKind::Gunner, |unit| {
        let attack = unit.attack.as_mut().unwrap();
        attack.target_type = TargetType::AoE {
            radius: 40.0 / SU_PER_GU,
            blast: BlastDomain::Attack,
        };
        attack.damage = 100.0;
    });
    let (peacekeeper, militant) = (ObjectId(2), ObjectId(3));
    let from = Vec2::new(300.0, 320.0);
    let at = from + Vec2::new(12.0 + gu(3.0), 0.0);
    let edge = at + Vec2::new(0.0, 40.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Gunner, MINE, from)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, at)
        .spawn_unit_at(1, ObjectKind::Militant, MINE, edge)
        .order_at(2, MINE, &[GUNNER], OrderKind::AttackGround(at), false);
    let (locked, spot) = until_locked(&mut t);
    assert_eq!(spot, at);
    t.run_until(locked + FIRING);
    let fired: Vec<&SimEvent> = (t.events().iter())
        .filter(|(tick, _)| *tick == locked + FIRING)
        .map(|(_, e)| e)
        .filter(|e| !matches!(e, SimEvent::Hit { .. }))
        .collect();
    assert_eq!(
        fired,
        [&SimEvent::Blast {
            at,
            radius: 40.0,
            attacker: Some(GUNNER)
        }]
    );
    let circle = PI * 40.0 * 40.0;
    let share = 100.0 * (24.0 * 24.0 / circle);
    assert_eq!(
        hits_on(&t, peacekeeper),
        [share - 1.0],
        "all of it inside, less its FullArmor 1"
    );
    let half = Vec2::splat(12.0);
    let inside = overlap(at, 40.0, edge - half, edge + half);
    assert!(inside > 0.0 && inside < 24.0 * 24.0, "across the edge");
    assert_eq!(
        hits_on(&t, militant),
        [100.0 * (inside / circle)],
        "its share; no armour"
    );
}
