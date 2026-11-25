use std::collections::BTreeSet;

use glam::Vec2;

use super::*;
use crate::combat::AttackState;
use crate::defs::RIOT_SHIELD_TOGGLE_TICKS;
use crate::ids::PlayerId;
use crate::mover::Mover;
use crate::orders::Orders;

const T: u32 = RIOT_SHIELD_TOGGLE_TICKS;
const PK: ObjectKind = ObjectKind::Peacekeeper;
const SHIELDED: ObjectKind = ObjectKind::PeacekeeperShielded;

/// A unit of `kind` whose shield is `up` ticks up, as id 1.
fn unit(kind: ObjectKind, up: u32) -> Object {
    let pos = Vec2::splat(100.0);
    Object {
        id: ObjectId(1),
        kind,
        owner: Some(PlayerId(1)),
        pos,
        prev_pos: pos,
        facing: 0.0,
        prev_facing: 0.0,
        hp: kind.def().max_hp,
        shields: None,
        placement: None,
        resource: None,
        orders: Some(Orders::default()),
        mover: Some(Mover::default()),
        attack: Some(AttackState::default()),
        riot_shield: Some(RiotShield { up }),
        extractor: None,
        production: None,
        constructor: None,
        worker: None,
        transform: None,
        conversion: None,
    }
}

/// Design change 0173: the two types share one shield, which names them; the Peacekeeper starts with it down, the
/// Peacekeeper (Shielded) with it up. `> Toggle time`: 6 frames, a placeholder (design change 0174) that "should stay
/// under half a second".
#[test]
fn both_types_carry_the_one_shield() {
    let def = |kind: ObjectKind| *kind.def().unit.as_ref().unwrap().riot_shield.as_ref().unwrap();
    assert_eq!(def(PK), def(SHIELDED));
    let shield = def(PK);
    assert_eq!((shield.lowered, shield.raised), (PK, SHIELDED));
    assert_eq!((shield.damage_factor, shield.toggle_ticks), (0.5, 6));
    const { assert!(T > 1 && T < crate::units::TICKS_PER_SECOND / 2) };
    assert_eq!(RiotShield::fresh(PK, &shield), RiotShield { up: 0 });
    assert_eq!(RiotShield::fresh(SHIELDED, &shield), RiotShield { up: T });
}

/// The state is how far up the shield is; the type says which way it goes (design change 0171: "Shield state =
/// progress 0..1 with direction"). A save edited by hand past the toggle time reads as all the way up, or as just
/// starting down.
#[test]
fn the_mode_is_the_type_and_how_far_up_the_shield_is() {
    let modes = |kind| [0, 1, T - 1, T, T + 5].map(|up| mode(&unit(kind, up)).unwrap());
    assert_eq!(
        modes(SHIELDED),
        [
            ShieldMode::Raising { left: T },
            ShieldMode::Raising { left: T - 1 },
            ShieldMode::Raising { left: 1 },
            ShieldMode::Raised,
            ShieldMode::Raised,
        ]
    );
    assert_eq!(
        modes(PK),
        [
            ShieldMode::Lowered,
            ShieldMode::Lowering { left: 1 },
            ShieldMode::Lowering { left: T - 1 },
            ShieldMode::Lowering { left: T },
            ShieldMode::Lowering { left: T },
        ]
    );
    let mut guard = unit(ObjectKind::Guard, 0);
    guard.riot_shield = None;
    assert_eq!(mode(&guard), None);
    assert!(allows_attack(&guard));
    assert_eq!((damage_factor(&guard), speed_factor(&guard)), (1.0, 1.0));
    let progress = [
        ShieldMode::Lowered,
        ShieldMode::Raising { left: 4 },
        ShieldMode::Lowering { left: 3 },
        ShieldMode::Raised,
    ]
    .map(|mode| mode.progress(6));
    assert_eq!(progress, [0.0, 2.0 / 6.0, 0.5, 1.0]);
    assert_eq!(ShieldMode::Raised.progress(0), 1.0);
}

/// `> Riot Shield` and design change 0171 (`> During the toggle`): half speed and no attacking from the start of a
/// raise to the end of a lowering, the damage reduction only all the way up. The Shielded type moves at its own
/// MaxSpeed, the Peacekeeper's halved; a Peacekeeper on its way down at the same speed, by its speed factor. Which way
/// the shield goes is the unit's type (design change 0173: "The type switches **when the toggle starts**"): up, or on
/// its way up, on the Peacekeeper (Shielded) alone ([`mode`]).
#[test]
fn the_effects_follow_the_mode() {
    let cases = [(PK, 0), (SHIELDED, 1), (SHIELDED, T), (PK, 1), (PK, T)];
    let effects = cases.map(|(kind, up)| {
        let o = unit(kind, up);
        let max_speed = kind.def().unit.as_ref().unwrap().movement.max_speed();
        (
            mode(&o).unwrap(),
            allows_attack(&o),
            max_speed * speed_factor(&o),
            damage_factor(&o),
        )
    });
    assert_eq!(
        effects,
        [
            (ShieldMode::Lowered, true, 4.0, 1.0),
            (ShieldMode::Raising { left: T - 1 }, false, 2.0, 1.0),
            (ShieldMode::Raised, false, 2.0, 0.5),
            (ShieldMode::Lowering { left: 1 }, false, 2.0, 1.0),
            (ShieldMode::Lowering { left: T }, false, 2.0, 1.0),
        ]
    );
    let going_up = cases.map(|(kind, up)| {
        let mode = mode(&unit(kind, up)).unwrap();
        matches!(mode, ShieldMode::Raising { .. } | ShieldMode::Raised)
    });
    assert_eq!(going_up, cases.map(|(kind, _)| kind == SHIELDED));
}

/// The Modes step's part: a shield goes the way its unit's type takes it, a tick at a time, and stops at either end;
/// the unit's speed follows. Objects from `first_new` on wait a tick.
#[test]
fn a_shield_goes_the_way_its_type_takes_it() {
    let mut objects = BTreeMap::from([(ObjectId(1), unit(SHIELDED, 0)), (ObjectId(2), unit(PK, 2))]);
    objects.get_mut(&ObjectId(2)).unwrap().id = ObjectId(2);
    for o in objects.values_mut() {
        settle(o);
    }
    let course = |objects: &BTreeMap<ObjectId, Object>| {
        objects
            .values()
            .map(|o| (o.riot_shield.unwrap().up, o.mover.as_ref().unwrap().speed_factor))
            .collect::<Vec<_>>()
    };
    assert_eq!(course(&objects), [(0, 1.0), (2, 0.5)]);
    step(&mut objects, ObjectId(2));
    assert_eq!(course(&objects), [(1, 1.0), (2, 0.5)], "unit 2 is new");
    let mut seen = Vec::new();
    for _ in 0..T + 1 {
        step(&mut objects, ObjectId(3));
        seen.push(course(&objects));
    }
    assert_eq!(seen[0], [(2, 1.0), (1, 0.5)]);
    assert_eq!(seen[1], [(3, 1.0), (0, 1.0)], "down, and fast again");
    assert_eq!(seen[T as usize - 2], [(T, 1.0), (0, 1.0)]);
    assert_eq!(seen[T as usize], [(T, 1.0), (0, 1.0)], "they stay there");
}

/// A toggle switches the unit to the type it asks for, and a toggle the way the shield already goes asks for none.
#[test]
fn a_toggle_asks_for_the_other_type_or_none() {
    assert_eq!(toggled_kind(&unit(PK, 0), true), Some(SHIELDED));
    assert_eq!(toggled_kind(&unit(PK, 3), true), Some(SHIELDED), "lowering: back up");
    assert_eq!(toggled_kind(&unit(PK, 3), false), None);
    assert_eq!(toggled_kind(&unit(SHIELDED, 3), false), Some(PK));
    assert_eq!(toggled_kind(&unit(SHIELDED, T), true), None);
    let mut guard = unit(ObjectKind::Guard, 0);
    guard.riot_shield = None;
    assert_eq!(toggled_kind(&guard, true), None);
}

/// A shield all the way down hashes as one tag, as it did before the two types; one up any way hashes its height.
#[test]
fn a_shield_down_hashes_as_a_tag() {
    let hash = |shield: RiotShield| {
        let mut h = StateHasher::new();
        shield.hash(&mut h);
        h.finish()
    };
    let tag = |tag: u8| {
        let mut h = StateHasher::new();
        h.tag(tag);
        h.finish()
    };
    assert_eq!(hash(RiotShield { up: 0 }), tag(0));
    let hashes: BTreeSet<u64> = (0..=T).map(|up| hash(RiotShield { up })).collect();
    assert_eq!(hashes.len(), T as usize + 1);
}
