//! Slice M4-S1's acceptance for the turret's fire, through the whole sim: 0260's "a turret kills a Guard in 4 shots",
//! its rate (one shot per 24 ticks, 16 damage a second against armour 0) and its Universal domain against a test air
//! unit (`defs::test_defs`); and its own pick (design change 0298 point 8,
//! `factions/syndicate/structures/laser_turret.md > Targeting`): threatening targets first, the one it engages kept,
//! ties by id, nothing of its own side, neutral or while auto-targeting is off; and its barrel.

use glam::Vec2;

use crate::command::DebugCommand;
use crate::defs::test_defs::TestDef;
use crate::defs::{Armor, ObjectDef, ObjectKind, UnitBase};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math::PI;
use crate::orders::OrderKind;
use crate::scenario::SpawnAt;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::{Tick, TilePos, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Player 1's Laser Turret, on [`TILE`].
const TURRET: ObjectId = ObjectId(1);
/// The turret's tile: its footprint spans x 128 to 192 and y 256 to 320, around (160, 288).
const TILE: TilePos = TilePos::new(2, 4);
const CENTRE: Vec2 = Vec2::new(160.0, 288.0);

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// Where a unit of `kind` stands `gap` grid units east of the turret's footprint, level with it.
fn east(kind: ObjectKind, gap: f32) -> Vec2 {
    let half = kind.def().half_extents().expect("a unit");
    Vec2::new(192.0 + gu(gap) + half.x, CENTRE.y)
}

/// Where a unit of `kind` stands `gap` grid units south of the turret's footprint, below it.
fn south(kind: ObjectKind, gap: f32) -> Vec2 {
    let half = kind.def().half_extents().expect("a unit");
    Vec2::new(CENTRE.x, 320.0 + gu(gap) + half.y)
}

/// Player 1's Laser Turret (1) on [`TILE`], and `units`, ids 2 on, each of its owner's at its position, all spawned in
/// tick 1; those of player 2 hold their positions from tick 2, so that none closes in or shoots back from beyond its
/// own Range.
fn field(units: &[(ObjectKind, Option<PlayerId>, Vec2)]) -> TestSim {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.debug_at(
        1,
        DebugCommand::Spawn {
            kind: ObjectKind::LaserTurret,
            owner: Some(P1),
            at: SpawnAt::Tile(TILE),
            orientation: Default::default(),
        },
    );
    for &(kind, owner, pos) in units {
        let at = SpawnAt::Point(pos);
        let orientation = Default::default();
        t.debug_at(
            1,
            DebugCommand::Spawn {
                kind,
                owner,
                at,
                orientation,
            },
        );
    }
    let held: Vec<ObjectId> = (units.iter().zip(2..))
        .filter(|((_, owner, _), _)| *owner == Some(P2))
        .map(|(_, id)| ObjectId(id))
        .collect();
    t.order_at(2, P2, &held, OrderKind::Hold, false);
    t
}

/// The turret's shots so far: (tick, target).
fn shots(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Attacked { attacker, target, .. } if attacker == TURRET => Some((*tick, target)),
            _ => None,
        })
        .collect()
}

/// The HP `id` lost in each hit so far: (tick, damage).
fn hits_on(t: &TestSim, id: ObjectId) -> Vec<(Tick, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit { target, damage, .. } if target == id => Some((*tick, damage)),
            _ => None,
        })
        .collect()
}

/// The tick `id` died in, if it did.
fn died(t: &TestSim, id: ObjectId) -> Option<Tick> {
    (t.events().iter())
        .find_map(|(tick, e)| matches!(e, SimEvent::Died { id: dead, .. } if *dead == id).then_some(*tick))
}

/// Design change 0260's verification: "a turret kills a Guard in 4 shots", 24 − 1 = 23 a shot against its 80 HP and
/// PointArmor 1, one shot every 24 ticks (Aim 4, Firing 1, Cooldown 3, Reload 16): each a red laser from the turret's
/// centre to the Guard's.
#[test]
fn a_turret_kills_a_guard_in_four_shots() {
    let guard = ObjectId(2);
    let at = east(ObjectKind::Guard, 4.0);
    let mut t = field(&[(ObjectKind::Guard, Some(P2), at)]);
    t.run(150);
    let hits = hits_on(&t, guard);
    let first = hits.first().expect("hit").0;
    let expected: Vec<(Tick, f32)> = (0..4).map(|i| (first + 24 * i, 23.0)).collect();
    assert_eq!(hits, expected);
    assert_eq!(died(&t, guard), Some(first + 72), "the fourth shot kills");
    let lasers = (t.events().iter()).filter(|(_, e)| {
        *e == SimEvent::Attacked {
            attacker: TURRET,
            target: guard,
            from: CENTRE,
            to: at,
        }
    });
    assert_eq!(lasers.count(), 4);
    assert!(t.sim().object(TURRET).is_some_and(|o| o.hp == o.max_hp()), "untouched");
}

/// The firing rate: "one shot per 24 frames, 16 damage per second" against armour 0 (a Guard given none, and HP
/// enough to last): 10 shots of 24 in 240 ticks, 15 seconds at 16 ticks a second.
#[test]
fn it_deals_16_damage_a_second_against_armour_0() {
    let _bare = TestDef::set(ObjectDef {
        armor: Armor { point: 0.0, full: 0.0 },
        max_hp: Some(10_000.0),
        ..ObjectKind::Guard.def().clone()
    });
    let guard = ObjectId(2);
    let mut t = field(&[(ObjectKind::Guard, Some(P2), east(ObjectKind::Guard, 5.0))]);
    t.run(300);
    let hits = hits_on(&t, guard);
    let first = hits.first().expect("hit").0;
    let window: Vec<f32> = (hits.iter())
        .filter(|(tick, _)| (first..first + 240).contains(tick))
        .map(|&(_, damage)| damage)
        .collect();
    assert_eq!(window, [24.0; 10]);
    assert_eq!(window.iter().sum::<f32>() / 15.0, 16.0);
}

/// `factions/syndicate/structures/laser_turret.md > AttackAttributes.TargetDomain - Universal`: it hits a test air unit
/// (a Peacekeeper given an air base: no type of the game flies yet) as it hits the ground.
#[test]
fn its_universal_laser_hits_an_air_unit() {
    let _flyer = TestDef::unit(ObjectKind::Peacekeeper, |unit| unit.base = UnitBase::HoverCraft);
    let flyer = ObjectId(2);
    let mut t = field(&[(ObjectKind::Peacekeeper, Some(P2), east(ObjectKind::Peacekeeper, 3.0))]);
    t.run(60);
    assert_eq!(shots(&t).first().map(|&(_, target)| target), Some(flyer));
    assert!(hits_on(&t, flyer).iter().any(|&(_, damage)| damage > 0.0));
}

/// Design change 0298 point 8: "BaseAutoTargeting within Range, then the closest", BaseAutoTargeting's priority being
/// "threatening targets first". A Drone, which can't attack, stands nearer than a Guard: the turret shoots the Guard,
/// which can, until it dies, and then the Drone.
#[test]
fn it_shoots_what_threatens_it_first() {
    let (drone, guard) = (ObjectId(2), ObjectId(3));
    let mut t = field(&[
        (ObjectKind::Drone, Some(P2), east(ObjectKind::Drone, 1.0)),
        (ObjectKind::Guard, Some(P2), south(ObjectKind::Guard, 4.0)),
    ]);
    t.run(150);
    let targets: Vec<ObjectId> = shots(&t).into_iter().map(|(_, target)| target).collect();
    assert_eq!(targets[..5], [guard, guard, guard, guard, drone]);
}

/// Design change 0298 point 8: "It keeps its target while that target stays valid and in range", whatever comes nearer
/// meanwhile; of two alike at the same gap, the lower id first ("ties broken deterministically"). Guards 2 and 3 stand
/// 4 grid units east and south; Guard 4, spawned later half a grid unit north of the turret (where it shoots back),
/// waits its turn, and then goes before Guard 3.
#[test]
fn it_keeps_its_target_and_breaks_ties_by_id() {
    let (east_guard, south_guard, late) = (ObjectId(2), ObjectId(3), ObjectId(4));
    let mut t = field(&[
        (ObjectKind::Guard, Some(P2), east(ObjectKind::Guard, 4.0)),
        (ObjectKind::Guard, Some(P2), south(ObjectKind::Guard, 4.0)),
    ]);
    t.run(20);
    assert_eq!(shots(&t).first().map(|&(_, target)| target), Some(east_guard));
    let near = Vec2::new(CENTRE.x, 256.0 - gu(0.5) - 18.0);
    t.spawn_unit_at(21, ObjectKind::Guard, P2, near)
        .order_at(22, P2, &[late], OrderKind::Hold, false);
    t.run(300);
    let targets: Vec<ObjectId> = shots(&t).into_iter().map(|(_, target)| target).collect();
    assert_eq!(targets[..4], [east_guard; 4], "kept until it died");
    assert_eq!(targets[4..8], [late; 4], "then the closest");
    assert_eq!(targets[8], south_guard);
}

/// A structure picks only enemies (`systems/combat.md > ValidTarget` 1): neither its own side's units nor a neutral
/// one, which auto-targeting ignores (`systems/controls/commands.md > Attack`); and nothing at all while auto-targeting
/// is off (`DebugCommand::SetAutoTargeting`), every target of its being one it picks itself.
#[test]
fn it_shoots_neither_friends_nor_neutrals_nor_without_auto_targeting() {
    let mut t = field(&[
        (ObjectKind::Guard, Some(P1), east(ObjectKind::Guard, 1.0)),
        (ObjectKind::Guard, None, south(ObjectKind::Guard, 1.0)),
    ]);
    t.run(100);
    assert_eq!(shots(&t), []);
    let mut t = field(&[(ObjectKind::Guard, Some(P2), east(ObjectKind::Guard, 4.0))]);
    t.without_auto_targeting().run(100);
    assert_eq!(shots(&t), []);
}

/// Plan decision T49: "A structure has no turn rate (it fires in any direction)". Its barrel, the facing the client
/// draws, points at what it fires at: south at a Guard south of it.
#[test]
fn its_barrel_faces_its_target() {
    let mut t = field(&[(ObjectKind::Guard, Some(P2), south(ObjectKind::Guard, 2.0))]);
    t.run(10);
    assert_eq!(shots(&t).len(), 1);
    let facing = t.sim().object(TURRET).expect("standing").facing;
    assert!((facing - PI / 2.0).abs() < 1e-6, "{facing}");
}
