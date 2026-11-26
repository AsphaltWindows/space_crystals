//! The buffs' rules on objects taken out of a game: who a cast reaches, a recast, and a tick of their effects. The
//! casts through the gate, in a whole game, are `tests/shrine/`'s.

use super::*;
use crate::augmentation::{Augmentation, Level, Levels};
use crate::defs::ObjectKind;
use crate::player::Faction;
use crate::scenario::SpawnAt;
use crate::testing::ScenarioBuilder;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HEALTH: AreaAbility = AreaAbility::HealthRegeneration;
const ENERGY: AreaAbility = AreaAbility::EnergyRegeneration;
/// The circle's radius, space units: 4 grid units.
const RADIUS: f32 = 256.0;
/// Where the tests cast.
const AT: Vec2 = Vec2::new(640.0, 640.0);

/// The objects of a game of the Cults' on a 24 × 24 map, each `(kind, owner, offset from AT)` in that order, from id
/// 1: as a cast in the commands phase finds them.
fn objects(spawns: &[(ObjectKind, PlayerId, Vec2)]) -> BTreeMap<ObjectId, Object> {
    let map = vec![".".repeat(24); 24].join("\n");
    let mut scenario = ScenarioBuilder::new(&map)
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults);
    for &(kind, owner, offset) in spawns {
        scenario = scenario.spawn(kind, Some(owner), SpawnAt::Point(AT + offset));
    }
    let t = scenario.start();
    (t.sim().objects()).map(|o| (o.id, o.clone())).collect()
}

fn buffs(objects: &BTreeMap<ObjectId, Object>, id: u32) -> Vec<Buff> {
    objects[&ObjectId(id)].buffs.clone()
}

/// "a unit is affected when its **centre** is inside" (Q0118): a centre on the circle's edge is in it, a hair beyond
/// it isn't, whatever the unit's size.
#[test]
fn a_unit_is_in_the_circle_by_its_centre() {
    assert!(in_circle(HEALTH, AT, AT));
    assert!(in_circle(HEALTH, AT, AT + Vec2::new(RADIUS, 0.0)), "on the edge");
    assert!(in_circle(ENERGY, AT, AT + Vec2::new(0.0, -RADIUS)));
    assert!(!in_circle(HEALTH, AT, AT + Vec2::new(RADIUS + 0.01, 0.0)));
    assert!(
        !in_circle(HEALTH, AT, AT + Vec2::new(181.1, 181.1)),
        "a diagonal of 256.1"
    );
    assert!(
        in_circle(HEALTH, AT, AT + Vec2::new(181.0, 181.0)),
        "a diagonal of 255.97"
    );
}

/// "Only the caster's own units are affected", of the types the ability affects ("Own Cults infantry", "Own
/// Disciples"), and only those in the circle; "**Recasting** on units that already have the effect **restarts its
/// duration**; effects of the same ability **don't stack**".
#[test]
fn a_cast_reaches_its_owner_s_units_of_its_types_in_the_circle_and_a_recast_restarts_it() {
    let mut objects = objects(&[
        (ObjectKind::Recruit, P1, Vec2::ZERO),                   // 1
        (ObjectKind::Militant, P1, Vec2::new(100.0, 0.0)),       // 2
        (ObjectKind::Disciple, P1, Vec2::new(0.0, 200.0)),       // 3
        (ObjectKind::Recruit, P2, Vec2::new(-50.0, 0.0)),        // 4: an enemy
        (ObjectKind::Seeker, P1, Vec2::new(0.0, -100.0)),        // 5: no infantry
        (ObjectKind::Gunner, P1, Vec2::new(RADIUS + 40.0, 0.0)), // 6: outside
    ]);
    assert_eq!(give(&mut objects, P1, HEALTH, AT, 10), 3);
    let health = |until| vec![Buff { ability: HEALTH, until }];
    assert_eq!(
        [1, 2, 3].map(|id| buffs(&objects, id)),
        [health(250), health(250), health(250)]
    );
    assert!([4, 5, 6].iter().all(|&id| buffs(&objects, id).is_empty()));
    // Energy regeneration reaches the Disciple alone, and its buffs keep the abilities' order.
    assert_eq!(give(&mut objects, P1, ENERGY, AT, 20), 1);
    let both = vec![
        Buff {
            ability: HEALTH,
            until: 250,
        },
        Buff {
            ability: ENERGY,
            until: 340,
        },
    ];
    assert_eq!(buffs(&objects, 3), both);
    assert_eq!(buffs(&objects, 1), health(250));
    // A recast restarts it, one buff still.
    assert_eq!(give(&mut objects, P1, HEALTH, AT, 100), 3);
    assert_eq!(buffs(&objects, 1), health(340));
    assert_eq!(
        buffs(&objects, 3)[0],
        Buff {
            ability: HEALTH,
            until: 340
        }
    );
    // Player 2's cast reaches theirs alone.
    assert_eq!(give(&mut objects, P2, HEALTH, AT, 100), 1);
    assert_eq!(buffs(&objects, 4), health(340));
}

/// Health regeneration's 0.125 HP a tick, every tick before its end and not after, up to the MaxHP with the unit's
/// Augmentations (an Enhanced Constitution Recruit's at Level 1, above its 40), never above it, and never lowering HP
/// above it; Energy regeneration's 4096 a tick ([`energy_bonus`]), for the energy's own step to add.
#[test]
fn a_buff_works_every_tick_before_its_end() {
    let mut objects = objects(&[
        (ObjectKind::Recruit, P1, Vec2::ZERO),
        (ObjectKind::Disciple, P1, Vec2::new(0.0, 60.0)),
    ]);
    give(&mut objects, P1, HEALTH, AT, 10);
    give(&mut objects, P1, ENERGY, AT, 10);
    let recruit = objects.get_mut(&ObjectId(1)).unwrap();
    recruit.hp = Some(10.0);
    for tick in 10..250 {
        step_object(recruit, Research::NONE, tick);
        assert_eq!(recruit.buffs.is_empty(), tick == 249, "tick {tick}");
    }
    assert_eq!(recruit.hp, Some(40.0), "30 HP in 240 ticks, to the MaxHP");
    // Up to the MaxHP with the Augmentations, at the research's levels.
    let constitution = Augmentation {
        constitution: true,
        metabolism: false,
    };
    recruit.augmentation = constitution;
    recruit.buffs = vec![Buff {
        ability: HEALTH,
        until: 300,
    }];
    let levels = Levels {
        constitution: Level::L1,
        metabolism: Level::NoResearch,
    };
    let max = recruit.max_hp_with(Research::at(levels)).unwrap();
    recruit.hp = Some(max - 0.1);
    step_object(recruit, Research::at(levels), 260);
    assert_eq!(recruit.hp, Some(max), "capped");
    assert!(max > 40.0, "{max}");
    recruit.hp = Some(max + 1.0);
    step_object(recruit, Research::at(levels), 261);
    assert_eq!(recruit.hp, Some(max + 1.0), "never lowered");
    let disciple = &objects[&ObjectId(2)];
    assert_eq!(energy_bonus(disciple), 4096);
    assert_eq!(energy_bonus(&objects[&ObjectId(1)]), 0);
}
