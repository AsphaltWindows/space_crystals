//! The occupants' weapons and where their shots leave from (slice M5-N2, `factions/cults/structures/garrison.md >
//! Garrisoning`). Their fire through the whole sim is `tests/garrison/`.

use glam::Vec2;

use super::*;
use crate::combat::weapon_of;
use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::scenario::SpawnAt;
use crate::testing::ScenarioBuilder;
use crate::units::{TilePos, gu};

const P1: PlayerId = PlayerId(1);
/// The kinds a Garrison takes in (`> Garrisoning > Who can enter`).
const INFANTRY: [ObjectKind; 4] = [
    ObjectKind::Recruit,
    ObjectKind::Militant,
    ObjectKind::Gunner,
    ObjectKind::Disciple,
];

fn garrison_fire() -> &'static OccupantFire {
    let structure = ObjectKind::Garrison.def().structure.as_ref();
    structure
        .and_then(|s| s.occupants_fire.as_ref())
        .expect("its occupants fire")
}

/// A Garrison of player 1's on tile (4, 4), object 1, and one unit of each of [`INFANTRY`], objects 2 to 5.
fn objects() -> crate::testing::TestSim {
    let mut builder = ScenarioBuilder::new(&"................\n".repeat(12)).spawn(
        ObjectKind::Garrison,
        Some(P1),
        SpawnAt::Tile(TilePos::new(4, 4)),
    );
    for (i, kind) in INFANTRY.into_iter().enumerate() {
        builder = builder.spawn(
            kind,
            Some(P1),
            SpawnAt::Point(Vec2::new(gu(2.0 * i as f32 + 1.5), gu(10.5))),
        );
    }
    builder.start()
}

/// `> Garrisoning > Weapons` and `> Range bonus`: "a Recruit or Disciple inside attacks with the Militant's
/// AttackAttributes (see Militant); Militants and Gunners use their own AttackAttributes", and "**Militants and
/// Gunners** inside get **+1 grid unit of Range** on their attack. Recruits and Disciples (firing with the Militant's
/// attack) do not get the bonus — they fire at the Militant's normal Range": Range 4, 5, 6 and 4; the Gunner's MinRange
/// of 2 stays, measured from the footprint.
#[test]
fn each_kind_fires_its_own_or_the_militants_attack_with_its_bonus() {
    let t = objects();
    let fire = garrison_fire();
    let militant = *weapon_of(ObjectKind::Militant).expect("the Militant's attack");
    let gunner = *weapon_of(ObjectKind::Gunner).expect("the Gunner's attack");
    let ranged = |attrs: AttackAttrs, range: f32| AttackAttrs { range, ..attrs };
    let expected = [
        ranged(militant, 4.0),
        ranged(militant, 5.0),
        ranged(gunner, 6.0),
        ranged(militant, 4.0),
    ];
    for (i, (kind, expected)) in INFANTRY.into_iter().zip(expected).enumerate() {
        let o = t.sim().object(ObjectId(i as u32 + 2)).expect("spawned");
        assert_eq!(o.kind, kind);
        assert_eq!(weapon(fire, o), Some(expected), "{kind:?}");
    }
    assert_eq!(
        weapon(fire, t.sim().object(ObjectId(4)).unwrap()).map(|a| a.min_range),
        Some(2.0)
    );
}

/// Plan decision T72: "shots start from the footprint's edge nearest the target". The Garrison covers x and y 256 to
/// 384: a shot east of it leaves from its east edge, level with the target; one off a corner from the corner; a
/// structure's own shot (the Laser Turret's) from its centre.
#[test]
fn an_occupants_shots_leave_from_the_footprints_edge_nearest_the_target() {
    let t = objects();
    let garrison = t.sim().object(ObjectId(1)).expect("spawned");
    assert_eq!(garrison.pos, Vec2::splat(320.0));
    let attrs = *weapon_of(ObjectKind::Militant).expect("the Militant's attack");
    let map = t.sim().map();
    let shooter = Shooter::occupant(map, garrison, ObjectId(3), &attrs, None);
    assert_eq!(
        (shooter.id, shooter.pos),
        (ObjectId(3), garrison.pos),
        "the unit's shot, from the building"
    );
    assert_eq!(shooter.muzzle(Vec2::new(gu(9.0), 330.0)), Vec2::new(384.0, 330.0));
    assert_eq!(shooter.muzzle(Vec2::new(300.0, 20.0)), Vec2::new(300.0, 256.0));
    assert_eq!(shooter.muzzle(Vec2::new(600.0, 600.0)), Vec2::splat(384.0));
    let own = Shooter::new(map, garrison, &attrs, None, 0.0);
    assert_eq!(own.muzzle(Vec2::new(600.0, 600.0)), garrison.pos);
}
