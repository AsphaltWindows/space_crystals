//! What a hit does (`super`; `systems/combat.md`): the modifier order, the directional zones, the Shields and the tick's
//! list applied.

use super::*;
const DIRECTIONAL_FRONT: f32 = crate::tuning::combat::BUILTIN.directional_front;
const DIRECTIONAL_REAR: f32 = crate::tuning::combat::BUILTIN.directional_rear;
const DIRECTIONAL_SIDE: f32 = crate::tuning::combat::BUILTIN.directional_side;
const MINIMUM_DAMAGE: f32 = crate::tuning::combat::BUILTIN.minimum_damage;

#[test]
fn single_target_damage_follows_the_modifier_order() {
    // Peacekeeper (10) on a Guard (PointArmor 1): 9.
    assert_eq!(hit_damage(10.0, 1.0, 1.0, 1.0), 9.0);
    // Guard (6) on a Peacekeeper (PointArmor 1) on Rugged ground (50% off before armor): 6 × 0.5 − 1 = 2.
    assert_eq!(hit_damage(6.0, 0.5, 1.0, 1.0), 2.0);
    // Armor before the directional multiplier: (10 − 2) × 0.75 = 6, not 10 × 0.75 − 2 = 5.5.
    assert_eq!(hit_damage(10.0, 1.0, 2.0, DIRECTIONAL_FRONT), 6.0);
    assert_eq!(hit_damage(10.0, 1.0, 2.0, DIRECTIONAL_REAR), 10.0);
    // The minimum applies last, whatever the armor.
    assert_eq!(hit_damage(3.0, 1.0, 5.0, DIRECTIONAL_REAR), MINIMUM_DAMAGE);
    assert_eq!(hit_damage(1.0, 0.5, 0.0, DIRECTIONAL_FRONT), MINIMUM_DAMAGE);
    // Fractions stay fractions.
    assert_eq!(hit_damage(5.0, 0.5, 1.0, 1.0), 1.5);
}

#[test]
fn directional_zones() {
    let facing = 0.0; // east
    let deg = |d: f32| math::facing_vector(math::degrees_to_radians(d));
    assert_eq!(directional_multiplier(facing, deg(0.0)), DIRECTIONAL_FRONT);
    assert_eq!(directional_multiplier(facing, deg(44.0)), DIRECTIONAL_FRONT);
    assert_eq!(directional_multiplier(facing, deg(-44.0)), DIRECTIONAL_FRONT);
    assert_eq!(directional_multiplier(facing, deg(46.0)), DIRECTIONAL_SIDE);
    assert_eq!(directional_multiplier(facing, deg(-90.0)), DIRECTIONAL_SIDE);
    assert_eq!(directional_multiplier(facing, deg(134.0)), DIRECTIONAL_SIDE);
    assert_eq!(directional_multiplier(facing, deg(136.0)), DIRECTIONAL_REAR);
    assert_eq!(directional_multiplier(facing, deg(180.0)), DIRECTIONAL_REAR);
    assert_eq!(directional_multiplier(facing, deg(-170.0)), DIRECTIONAL_REAR);
    // Facing south: a source to the north is behind.
    assert_eq!(directional_multiplier(PI / 2.0, Vec2::new(0.0, -5.0)), DIRECTIONAL_REAR);
    assert_eq!(directional_multiplier(PI / 2.0, Vec2::new(0.0, 5.0)), DIRECTIONAL_FRONT);
    assert_eq!(directional_multiplier(facing, Vec2::ZERO), DIRECTIONAL_FRONT);
}

/// A hit of `damage` against PointArmor `point_armor`, from no particular direction.
fn hit(damage: f32, point_armor: f32) -> Hit {
    Hit::single(ObjectId(1), ObjectId(2), damage, 1.0, point_armor, 1.0)
}

fn taken(shields: f32, hp: f32) -> Taken {
    Taken { shields, hp }
}

/// The plan's acceptance, "The design's cases" (`systems/combat.md > Shields > Shield Damage Resolution`, `>
/// MinimumDamage > Shield overflow`), exactly.
#[test]
fn shields_take_a_hit_first_and_the_overflow_meets_the_armour() {
    // 15 on 10 Shields, armour 0: the Shields reach 0, the HP drops by 5.
    assert_eq!(resolve(&hit(15.0, 0.0), Some(10.0)), taken(10.0, 5.0));
    // 25 on a Robo Portal's 20 Shields, PointArmor 1: 4.
    assert_eq!(resolve(&hit(25.0, 1.0), Some(20.0)), taken(20.0, 4.0));
    // 10.25 on 10 Shields, PointArmor 1: the minimum is capped at the overflow, 0.25.
    assert_eq!(resolve(&hit(10.25, 1.0), Some(10.0)), taken(10.0, 0.25));
    // 8 on 10 Shields: no HP lost, whatever the armour ("armor is never consulted").
    for armour in [0.0, 1.0, 100.0] {
        assert_eq!(resolve(&hit(8.0, armour), Some(10.0)), taken(8.0, 0.0));
    }
    // Exactly the Shields: absorbed ("If damage exceeds current Shields" it overflows).
    assert_eq!(resolve(&hit(10.0, 0.0), Some(10.0)), taken(10.0, 0.0));
    // No Shields left: the usual damage, the minimum whole.
    assert_eq!(resolve(&hit(10.0, 1.0), Some(0.0)), taken(0.0, 9.0));
    assert_eq!(resolve(&hit(3.0, 5.0), Some(0.0)), taken(0.0, MINIMUM_DAMAGE));
    // An object without Shields: `hit_damage`, as ever.
    assert_eq!(resolve(&hit(10.0, 1.0), None), taken(0.0, 9.0));
    assert_eq!(resolve(&hit(3.0, 5.0), None), taken(0.0, MINIMUM_DAMAGE));
}

/// `> DamageModifierOrder` (design change 0147): "no modifier of any step touches Shields"; the pre-armor
/// multipliers and the DirectionalArmor touch the overflow only.
#[test]
fn the_multipliers_touch_the_overflow_only() {
    let rear = Hit {
        directional: DIRECTIONAL_REAR,
        ..hit(15.0, 0.0)
    };
    assert_eq!(resolve(&rear, Some(10.0)), taken(10.0, 6.25));
    let halved = Hit {
        pre_armor: 0.5,
        ..hit(15.0, 1.0)
    };
    assert_eq!(resolve(&halved, Some(10.0)), taken(10.0, 1.5));
    // A front hit whose overflow the armour stops: the minimum, capped at the overflow, after the multiplier.
    let front = Hit {
        directional: DIRECTIONAL_FRONT,
        ..hit(10.25, 1.0)
    };
    assert_eq!(resolve(&front, Some(10.0)), taken(10.0, 0.25));
}

/// A target of `kind` with `shields`, for [`apply`].
fn target(kind: crate::defs::ObjectKind, shields: f32) -> Object {
    Object {
        id: ObjectId(2),
        kind,
        owner: None,
        pos: Vec2::ZERO,
        prev_pos: Vec2::ZERO,
        facing: 0.0,
        prev_facing: 0.0,
        hp: kind.def().max_hp,
        shields: Some(shields),
        placement: None,
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
    }
}

/// Plan decision T18: the hits of a tick are resolved as they are applied, in their order, so the second of two
/// hits meets the Shields the first left: two hits of 10 on a Drone's 10 Shields cost it its Shields, then 10 of
/// its 20 HP, where resolving both at the shots would have taken nothing but Shields. A third, of 10 more,
/// destroys it, and each hit says what it took.
#[test]
fn the_second_hit_of_a_tick_meets_the_shields_the_first_left() {
    let mut objects = BTreeMap::from([(ObjectId(2), target(crate::defs::ObjectKind::Drone, 10.0))]);
    let mut events = Vec::new();
    let hits = [hit(10.0, 0.0), hit(10.0, 0.0)];
    assert_eq!(apply(&mut objects, &hits, &mut events), []);
    let drone = &objects[&ObjectId(2)];
    assert_eq!((drone.shields, drone.hp), (Some(0.0), Some(10.0)));
    let took = |events: &[SimEvent]| -> Vec<(f32, f32)> {
        events
            .iter()
            .filter_map(|e| match e {
                SimEvent::Hit {
                    damage, shield_damage, ..
                } => Some((*shield_damage, *damage)),
                _ => None,
            })
            .collect()
    };
    assert_eq!(took(&events), [(10.0, 0.0), (0.0, 10.0)]);
    events.clear();
    assert_eq!(apply(&mut objects, &[hit(10.0, 0.0)], &mut events), [ObjectId(2)]);
    assert_eq!(took(&events), [(0.0, 10.0)]);
    assert!(matches!(events[1], SimEvent::Died { id: ObjectId(2), .. }));
}

/// (M5-N4b) The plan's acceptance, "A 3x3 building wholly under a cloud takes about 32% of the absorbed damage, before
/// armour": a release of 600 over one 3-GU circle that holds a Scrap Hangar's whole footprint (2,304 of the circle's
/// 7,232 lattice points here: `crate::shroud::lattice`) puts 31.9% of it on the hangar, with its whole FullArmor, 9, and
/// the minimum of 0.5, as an area hit (`systems/combat.md > AoEDamage`), and no direction (design change 0334). A game
/// can't put a 3x3 building wholly under one circle, whose Disciple would stand inside it: the circle is the cloud's
/// data alone here. A unit beyond the circle takes nothing.
#[test]
fn a_3x3_building_wholly_under_one_circle_takes_about_32_percent_of_a_release() {
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::scenario::SpawnAt;
    use crate::shroud::Member;
    use crate::sim::Sim;
    use crate::testing::ScenarioBuilder;
    let builder = ScenarioBuilder::new(&"..............................\n".repeat(16))
        .spawn(
            ObjectKind::ScrapHangar,
            Some(PlayerId(1)),
            SpawnAt::Tile(TilePos::new(4, 4)),
        )
        .spawn(
            ObjectKind::Militant,
            Some(PlayerId(2)),
            SpawnAt::Point(Vec2::new(1000.0, 400.0)),
        );
    let sim = Sim::new(&builder.build());
    let mut objects: BTreeMap<ObjectId, Object> = sim.objects().map(|o| (o.id, o.clone())).collect();
    let hangar = objects[&ObjectId(1)].pos;
    let targets = Targets::snapshot(sim.map(), &objects);
    let (mut clouds, mut events) = (Clouds::default(), Vec::new());
    let member = Member {
        id: ObjectId(9),
        since: 0,
        at: hangar,
        radius: 192.0,
    };
    clouds.update(vec![member], &mut events);
    clouds.absorb(0, 600.0, 3);
    let mut hits = Vec::new();
    release(&mut clouds, &targets, 10, &mut hits, &mut events);
    assert!(hits.is_empty(), "not due before tick 11");
    release(&mut clouds, &targets, 11, &mut hits, &mut events);
    let points = Union::of(&[member]).count() as f32;
    assert!((points - 7_238.2).abs() < 72.0, "within 1% of its exact area: {points}");
    let share = 600.0 * (2_304.0 / points);
    assert_eq!(hits.len(), 1, "the hangar alone: {hits:?}");
    let hit = hits[0];
    assert_eq!((hit.attacker, hit.target), (ObjectId(9), ObjectId(1)));
    assert_eq!(
        (hit.damage, hit.armor, hit.directional, hit.minimum),
        (share, 9.0, 1.0, MINIMUM_DAMAGE)
    );
    assert!(
        (hit.damage / 600.0 - 0.32).abs() < 0.003,
        "about 32%: {}",
        hit.damage / 600.0
    );
    let released = SimEvent::ShroudReleased {
        circles: vec![(hangar, 192.0)],
        damage: 600.0,
    };
    assert!(events.contains(&released), "{events:?}");
    let before = objects[&ObjectId(1)].hp.unwrap();
    apply(&mut objects, &hits, &mut events);
    assert_eq!(objects[&ObjectId(1)].hp, Some(before - (share - 9.0)));
    assert!(
        clouds.list()[0].buffer == 0.0 && !clouds.due(12),
        "empty, and waiting for its next hit"
    );
}
