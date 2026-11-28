use std::collections::BTreeMap;

use super::*;
use crate::combat::damage::{apply, resolve};
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, UnitBase};
use crate::events::SimEvent;
use crate::ids::PlayerId;
use crate::map::Map;
use crate::object::{Object, Placement};
const DIRECTIONAL_FRONT: f32 = crate::tuning::combat::BUILTIN.directional_front;
const DIRECTIONAL_REAR: f32 = crate::tuning::combat::BUILTIN.directional_rear;
use crate::units::TilePos;
const MINIMUM_DAMAGE: f32 = crate::tuning::combat::BUILTIN.minimum_damage;

/// The test grenade's blast (plan §3.3 O's acceptance: damage 100, radius 40 su) at `at`, of `domain`.
fn grenade(at: Vec2, domain: TargetDomain) -> Blast {
    Blast {
        attacker: ObjectId(99),
        at,
        radius: 40.0,
        damage: 100.0,
        domain,
    }
}

/// An object of `kind` with nothing but its kind's numbers: a unit at `pos`, or a structure whose footprint's top-left
/// tile is `pos`'s.
fn object(id: u32, kind: ObjectKind, owner: Option<PlayerId>, pos: Vec2) -> Object {
    let def = kind.def();
    let placement = def
        .structure
        .is_some()
        .then(|| Placement::new(TilePos::containing(pos), def.footprint));
    let pos = placement.map_or(pos, |p| p.center());
    Object {
        id: ObjectId(id),
        kind,
        owner,
        pos,
        prev_pos: pos,
        facing: 0.0,
        prev_facing: 0.0,
        hp: def.max_hp,
        shields: def.max_shields,
        placement,
        resource: None,
        orders: None,
        mover: None,
        attack: None,
        riot_shield: None,
        extractor: None,
        production: None,
        constructor: None,
        worker: None,
        transform: None,
        conversion: None,
        tunnel: crate::tunnel::fresh(kind),
        construction: None,
        decay: None,
        popup: None,
        lineage: Vec::new(),
        augmentation: Default::default(),
        energy: None,
        kept_research: None,
        buffs: Vec::new(),
        stock: None,
        tether: None,
        drone: None,
        cooldown: None,
        aircraft: None,
        illusion_of: None,
        tanks: None,
        chopper: None,
        schedule: None,
    }
}

/// The hits of `blast` on `objects`, on an open map.
fn strike_on(objects: &BTreeMap<ObjectId, Object>, blast: &Blast) -> Vec<Hit> {
    let map = Map::new(16, 16);
    let mut hits = Vec::new();
    strike(blast, &Targets::snapshot(&map, objects), &mut hits);
    hits
}

fn objects(list: impl IntoIterator<Item = Object>) -> BTreeMap<ObjectId, Object> {
    list.into_iter().map(|o| (o.id, o)).collect()
}

/// The area of the circle of `r` around `c` inside the box, by the midpoint rule over thin vertical strips, in `f64`.
fn integrated(c: Vec2, r: f32, lo: Vec2, hi: Vec2) -> f64 {
    let (r, cx, cy) = (f64::from(r), f64::from(c.x), f64::from(c.y));
    let (x0, x1, y0, y1) = (f64::from(lo.x), f64::from(hi.x), f64::from(lo.y), f64::from(hi.y));
    let strips = 200_000;
    let width = (x1 - x0) / f64::from(strips);
    (0..strips)
        .map(|i| {
            let x = x0 + (f64::from(i) + 0.5) * width - cx;
            let half = (r * r - x * x).max(0.0).sqrt();
            let (top, bottom) = ((cy + half).min(y1), (cy - half).max(y0));
            (top - bottom).max(0.0) * width
        })
        .sum()
}

/// Plan decision T46, and the Director's note on determinism: the overlap is computed with basic arithmetic and
/// `crate::math` (libm) only, so it is the same bits on every machine. These pin a few: each kind of case, the
/// general one twice. A change here changes every area attack's damage, and the goldens with it.
#[test]
fn the_overlap_is_the_same_bits_everywhere() {
    let cases = [
        // Wholly inside: the rectangle's own area.
        (Vec2::new(0.0, 0.0), Vec2::new(-18.0, -18.0), Vec2::new(18.0, 18.0)),
        // A chord across the rectangle's left side.
        (Vec2::new(10.0, 5.0), Vec2::new(20.0, -30.0), Vec2::new(70.0, 30.0)),
        // A corner in the circle.
        (Vec2::new(0.0, 0.0), Vec2::new(25.0, 25.0), Vec2::new(90.0, 90.0)),
        // The circle across three of its sides.
        (Vec2::new(3.5, -7.25), Vec2::new(-30.0, -20.0), Vec2::new(15.0, 50.0)),
        // The circle wholly inside.
        (Vec2::new(64.0, 64.0), Vec2::new(0.0, 0.0), Vec2::new(128.0, 128.0)),
        // Off-centre and thin: the blast's edge on a 2-tile Hideout's corner.
        (
            Vec2::new(1000.25, 517.5),
            Vec2::new(1024.0, 512.0),
            Vec2::new(1152.0, 640.0),
        ),
    ];
    let bits: Vec<u32> = cases
        .iter()
        .map(|&(c, lo, hi)| overlap(c, 40.0, lo, hi).to_bits())
        .collect();
    assert_eq!(
        bits,
        [
            0x44a2_0000,
            0x44be_34e0,
            0x41a6_6a40,
            0x4508_b994,
            0x459d_1463,
            0x43e3_4508
        ],
        "{:?}",
        bits.iter().map(|&b| f32::from_bits(b)).collect::<Vec<_>>()
    );
}

/// The cases with an exact answer: none, the rectangle, the circle, its halves and quarters.
#[test]
fn the_overlap_of_the_simple_cases() {
    let c = Vec2::new(100.0, 200.0);
    let circle = PI * 40.0 * 40.0;
    let at = |x0: f32, y0: f32, x1: f32, y1: f32| overlap(c, 40.0, c + Vec2::new(x0, y0), c + Vec2::new(x1, y1));
    assert_eq!(at(-18.0, -18.0, 18.0, 18.0), 1296.0, "wholly inside");
    assert_eq!(at(-100.0, -100.0, 100.0, 100.0), circle, "the circle wholly inside");
    assert_eq!(at(40.0, -10.0, 60.0, 10.0), 0.0, "touching is not overlapping");
    assert_eq!(at(30.0, 30.0, 60.0, 60.0), 0.0, "a corner just out of reach");
    assert_eq!(at(10.0, 10.0, 10.0, 60.0), 0.0, "an empty rectangle");
    assert_eq!(overlap(c, 0.0, c - Vec2::ONE, c + Vec2::ONE), 0.0, "no circle");
    let close = |got: f32, want: f32| (got - want).abs() <= 1e-3 * want;
    assert!(close(at(0.0, -100.0, 100.0, 100.0), circle / 2.0), "the right half");
    assert!(close(at(-100.0, -100.0, 100.0, 0.0), circle / 2.0), "the upper half");
    assert!(close(at(0.0, 0.0, 100.0, 100.0), circle / 4.0), "a quarter");
    assert!(
        close(at(-100.0, -100.0, 0.0, 0.0), circle / 4.0),
        "the opposite quarter"
    );
}

/// The overlap against a numeric integration, over rectangles in every position against the circle.
#[test]
fn the_overlap_is_the_area_inside_both() {
    let c = Vec2::new(321.5, 123.25);
    let mut cases = 0;
    for x0 in [-70.0, -41.0, -30.0, -12.5, 0.0, 7.0, 25.0, 39.0] {
        for y0 in [-55.0, -38.0, -18.0, -3.0, 0.0, 11.0, 33.0] {
            for (w, h) in [(18.0, 18.0), (36.0, 36.0), (128.0, 128.0), (5.0, 90.0), (64.0, 7.5)] {
                let (lo, hi) = (c + Vec2::new(x0, y0), c + Vec2::new(x0 + w, y0 + h));
                let got = overlap(c, 40.0, lo, hi);
                let want = integrated(c, 40.0, lo, hi);
                assert!(
                    (f64::from(got) - want).abs() <= 0.05,
                    "box {lo}..{hi}: {got} against {want}"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 280);
}

/// `systems/combat.md > AoEDamage`: "Ground AoE hits Ground and surfaced Underground units, Air AoE hits Air units,
/// Universal AoE hits all units regardless of domain", and "Underground objects are hit by area damage too (...
/// Ground and Universal AoE)".
#[test]
fn which_domains_a_blast_hits() {
    use Domain::{Air, Ground, Underground};
    let table = [
        (TargetDomain::Ground, [true, false, true]),
        (TargetDomain::Air, [false, true, false]),
        (TargetDomain::Universal, [true, true, true]),
    ];
    for (attack, hits) in table {
        assert_eq!(
            [Ground, Air, Underground].map(|d| hits_domain(attack, d)),
            hits,
            "{attack:?}"
        );
    }
}

/// Plan §3.3 O's acceptance, the numbers: a 36x36 infantry fully inside takes 100 × 1296 / 5026.5 ≈ 25.8 before
/// armour, and 24.8 after its FullArmor 1, met whole.
#[test]
fn infantry_wholly_inside_takes_its_share_less_its_full_armor() {
    let guard = object(1, ObjectKind::Guard, Some(PlayerId(1)), Vec2::new(300.0, 300.0));
    assert_eq!((guard.def().armor.point, guard.def().armor.full), (1.0, 1.0));
    let hits = strike_on(
        &objects([guard]),
        &grenade(Vec2::new(305.0, 290.0), TargetDomain::Ground),
    );
    let [hit] = hits[..] else {
        panic!("one hit: {hits:?}");
    };
    assert_eq!((hit.attacker, hit.target), (ObjectId(99), ObjectId(1)));
    assert_eq!(hit.damage, 100.0 * (1296.0 / (PI * 40.0 * 40.0)));
    assert_eq!((hit.damage * 10.0).round(), 258.0);
    assert_eq!(
        (hit.armor, hit.minimum),
        (1.0, MINIMUM_DAMAGE),
        "the whole of it is inside"
    );
    let taken = resolve(&hit, None);
    assert_eq!(taken.hp, hit.damage - 1.0);
    assert_eq!((taken.hp * 10.0).round(), 248.0);
}

/// Plan §3.3 O's acceptance: a blast wholly over a hidden 2x2 Hideout (PointArmor 6, FullArmor 24, design change 0264)
/// takes 100 − 24 × 5026.5 / 16384 ≈ 92.6 ("about 93"), seen or not: player 1's Hideout is hidden from player 2,
/// whose grenade it is, and from the Peacekeeper standing on it, which takes its share too. The Hideout dies of
/// enough of them (design changes 0218 and 0262: "Syndicate expansions ... take their share of any blast over them,
/// even unseen").
#[test]
fn a_hidden_hideout_under_a_blast_takes_its_share() {
    let hideout = object(2, ObjectKind::Hideout, Some(PlayerId(1)), Vec2::new(256.0, 256.0));
    let centre = hideout.pos;
    assert_eq!(hideout.def().tile_size(), Some((2, 2)));
    assert!(crate::vision::hidden_from(&hideout, PlayerId(2)));
    let peacekeeper = object(
        3,
        ObjectKind::Peacekeeper,
        Some(PlayerId(1)),
        centre + Vec2::new(30.0, 0.0),
    );
    let mut objects = objects([hideout, peacekeeper]);
    let blast = grenade(centre, TargetDomain::Ground);
    let hits = strike_on(&objects, &blast);
    assert_eq!(
        hits.iter().map(|h| h.target).collect::<Vec<_>>(),
        [ObjectId(2), ObjectId(3)]
    );
    let on_hideout = hits[0];
    assert_eq!(on_hideout.damage, 100.0, "the whole circle is over it");
    assert_eq!(on_hideout.armor, 24.0 * (PI * 1600.0 / 16384.0));
    let taken = resolve(&on_hideout, None).hp;
    assert_eq!((taken * 10.0).round(), 926.0, "{taken}");
    let mut events = Vec::new();
    assert_eq!(apply(&mut objects, &hits, &mut events), []);
    assert_eq!(objects[&ObjectId(2)].hp, Some(400.0 - taken));
    objects.get_mut(&ObjectId(2)).unwrap().hp = Some(50.0);
    let hits = strike_on(&objects, &blast);
    assert_eq!(apply(&mut objects, &hits, &mut events), [ObjectId(2)]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::Died { id: ObjectId(2), .. }))
    );
}

/// `systems/combat.md > DamageModifierOrder` 0: "For AoE, the full value is the unit's damage share": the Shields
/// take the share, not the attack's whole Damage, and the overflow meets the effective armour.
#[test]
fn shields_take_the_share_at_full_value() {
    let guard = object(1, ObjectKind::Guard, Some(PlayerId(1)), Vec2::new(300.0, 300.0));
    let [hit] = strike_on(
        &objects([guard]),
        &grenade(Vec2::new(300.0, 300.0), TargetDomain::Ground),
    )[..] else {
        panic!("one hit");
    };
    let plenty = resolve(&hit, Some(1000.0));
    assert_eq!((plenty.shields, plenty.hp), (hit.damage, 0.0), "the share, all of it");
    let some = resolve(&hit, Some(10.0));
    assert_eq!(some.shields, 10.0);
    assert_eq!(some.hp, hit.damage - 10.0 - hit.armor);
}

/// `systems/combat.md > AoEDamage > DirectionalArmor`: "the angle from the unit's center to the AoE center" picks the
/// zone: a test DrillUnit (directional armour) facing east takes the front multiplier from a blast ahead of it, the
/// rear one from a blast behind it, the same overlap either way.
#[test]
fn the_directional_zone_is_taken_from_the_blast_centre() {
    let _drill = TestDef::unit(ObjectKind::Guard, |unit| unit.base = UnitBase::DrillUnit);
    let drill = object(1, ObjectKind::Guard, Some(PlayerId(1)), Vec2::new(300.0, 300.0));
    let from = |dx: f32| {
        strike_on(
            &objects([drill.clone()]),
            &grenade(drill.pos + Vec2::new(dx, 0.0), TargetDomain::Ground),
        )
    };
    let (ahead, behind) = (from(30.0), from(-30.0));
    assert_eq!(ahead[0].directional, DIRECTIONAL_FRONT);
    assert_eq!(behind[0].directional, DIRECTIONAL_REAR);
    assert_eq!(ahead[0].damage, behind[0].damage);
}

/// `systems/combat.md > MinimumDamage > AoE`: "0.5 × overlap fraction". A weak blast (Damage 5) whose edge is over a
/// corner of a Hideout, whose effective FullArmor there outweighs the share, still does the minimum of the part inside.
#[test]
fn the_minimum_is_half_the_overlap_fraction() {
    let hideout = object(2, ObjectKind::Hideout, Some(PlayerId(1)), Vec2::new(1024.0, 512.0));
    let (lo, _) = crate::shape::Shape::of(&hideout).bounds();
    let weak = Blast {
        damage: 5.0,
        ..grenade(lo - Vec2::new(10.0, 10.0), TargetDomain::Ground)
    };
    let [hit] = strike_on(&objects([hideout]), &weak)[..] else {
        panic!("one hit");
    };
    let fraction = overlap(lo - Vec2::new(10.0, 10.0), 40.0, lo, lo + Vec2::splat(128.0)) / 16384.0;
    assert!(fraction > 0.0 && fraction < 0.1, "{fraction}");
    assert_eq!(hit.minimum, MINIMUM_DAMAGE * fraction);
    assert!(hit.damage - hit.armor < hit.minimum, "the armour outweighs the share");
    assert_eq!(resolve(&hit, None).hp, hit.minimum);
}

/// Plan §3.3 S2's acceptance: "A test AoE over a lowered pop-up deals the share less 6 × the overlap fraction. Raised,
/// less 1 × it" (`factions/syndicate/structures/laser_turret.md`: "raised PointArmor 1, FullArmor 1; underground
/// (pop-up) PointArmor 6, FullArmor 6"; `Object::armor`). The grenade's circle (radius 40) around the middle of the
/// turret's one tile covers it but for its corners: the same share of the blast either way, less the FullArmor of
/// its form times the overlap fraction. A lowered turret, hidden, takes it all the same (design change 0267: hidden things "can only be hurt by
/// area damage over them"); rising and retracting it has its raised armour (design change 0296 point 1).
#[test]
fn a_blast_over_a_popup_turret_meets_the_armour_of_its_form() {
    use crate::popup::PopUp;
    use crate::progress::Progress;
    let blast_on = |state: Option<PopUp>| {
        let mut turret = object(2, ObjectKind::LaserTurret, Some(PlayerId(1)), Vec2::new(256.0, 256.0));
        turret.popup = state;
        assert_eq!(
            crate::vision::hidden_from(&turret, PlayerId(2)),
            state == Some(PopUp::LOWERED)
        );
        let blast = grenade(turret.pos, TargetDomain::Ground);
        let hits = strike_on(&objects([turret]), &blast);
        assert_eq!(hits.len(), 1);
        hits[0]
    };
    let lowered = blast_on(Some(PopUp::LOWERED));
    // The overlap fraction: the share of the turret's tile under the circle, all of it but the corners.
    let fraction = lowered.armor / 6.0;
    assert!((0.95..1.0).contains(&fraction), "{fraction}");
    // The share of the blast: the same area, of the circle's.
    let share = 100.0 * fraction * 4096.0 / (PI * 1600.0);
    assert!((lowered.damage - share).abs() < 1e-3, "{}", lowered.damage);
    let taken = resolve(&lowered, None).hp;
    assert_eq!(taken, lowered.damage - 6.0 * fraction);
    let raised = [
        None,
        Some(PopUp::Rising(Progress::new(8))),
        Some(PopUp::Raised {
            idle: Progress::new(32),
        }),
        Some(PopUp::Retracting(Progress::new(8))),
    ];
    for state in raised {
        let hit = blast_on(state);
        assert_eq!((hit.damage, hit.armor), (lowered.damage, 1.0 * fraction), "{state:?}");
        assert_eq!(resolve(&hit, None).hp, lowered.damage - 1.0 * fraction);
    }
}

/// Plan §3.3 O's acceptance, domains: a Ground blast hits a hidden expansion and a surfaced test DrillUnit, and not an
/// air unit; a Universal one hits all three. Owners don't matter (design change 0293, answering Q0083 point 1: friendly
/// fire): the attacker's own objects are hit too.
#[test]
fn a_blast_hits_by_domain_whoever_owns_what() {
    let _drill = TestDef::unit(ObjectKind::Guard, |unit| unit.base = UnitBase::DrillUnit);
    let _air = TestDef::unit(ObjectKind::Peacekeeper, |unit| unit.base = UnitBase::HoverCraft);
    let hideout = object(1, ObjectKind::Hideout, Some(PlayerId(1)), Vec2::new(256.0, 256.0));
    let at = hideout.pos;
    let list = [
        hideout,
        object(2, ObjectKind::Guard, Some(PlayerId(2)), at + Vec2::new(20.0, 0.0)),
        object(3, ObjectKind::Peacekeeper, Some(PlayerId(2)), at - Vec2::new(20.0, 0.0)),
    ];
    let objects = objects(list);
    let hit = |domain| -> Vec<ObjectId> {
        strike_on(&objects, &grenade(at, domain))
            .iter()
            .map(|h| h.target)
            .collect()
    };
    assert_eq!(hit(TargetDomain::Ground), [ObjectId(1), ObjectId(2)]);
    assert_eq!(hit(TargetDomain::Air), [ObjectId(3)]);
    assert_eq!(hit(TargetDomain::Universal), [ObjectId(1), ObjectId(2), ObjectId(3)]);
}

/// (M5-S8) [`blast_domain`], by the attack's def: the Rocketman's missile's burst takes the domain of what it was fired
/// at (`factions/gdo/units/rocketman.md > AttackAttributes (normal attack)`, design change 0445), Air for an aircraft
/// and Ground for anything else, a ground or an underground object or the ground (an Attack Ground); every other area
/// attack's is its TargetDomain, a Universal one's every domain (`systems/combat.md > AoEDamage`), and a single-target
/// attack's too.
#[test]
fn a_blast_takes_its_domain_by_its_def() {
    let attack = |kind: ObjectKind| kind.def().unit.as_ref().unwrap().attack.unwrap();
    let rocketman = attack(ObjectKind::Rocketman);
    assert_eq!(blast_domain(&rocketman, Some(Domain::Air)), TargetDomain::Air);
    assert_eq!(blast_domain(&rocketman, Some(Domain::Ground)), TargetDomain::Ground);
    assert_eq!(
        blast_domain(&rocketman, Some(Domain::Underground)),
        TargetDomain::Ground
    );
    assert_eq!(blast_domain(&rocketman, None), TargetDomain::Ground, "the ground");
    let universal = AttackAttrs {
        target_type: TargetType::AoE {
            radius: 0.5,
            blast: BlastDomain::Attack,
        },
        ..rocketman
    };
    let guard = attack(ObjectKind::Guard);
    for target in [Some(Domain::Air), Some(Domain::Ground), Some(Domain::Underground), None] {
        assert_eq!(blast_domain(&universal, target), TargetDomain::Universal, "{target:?}");
        for kind in [ObjectKind::Grenadier, ObjectKind::HandOfHeaven] {
            assert_eq!(blast_domain(&attack(kind), target), TargetDomain::Ground, "{kind:?}");
        }
        assert_eq!(blast_domain(&guard, target), guard.target_domain, "single-target");
    }
}
