//! Training of the units inside, where a test needs the sim's inside: a hurt Recruit's damage carried into the unit it
//! becomes (design change 0332's Verification; no command hurts a unit at will, so the test sets its HP), a vehicle's
//! full HP and lost augmentations, and a unit made of two Recruits (plan decision T70: a Seeker's two, before the Chop
//! Shop exists), all of an Armory whose Militant a test def changes.

use glam::Vec2;

use crate::augmentation::{Augmentation, RecruitKind};
use crate::command::{Command, PlayerCommand, TickInputs};
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, UnitBase};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::player::Faction;
use crate::production::ProductionCommand;
use crate::recruitment::Lineages;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;
use crate::testing::invariants::Invariants;
use crate::units::{Tick, TilePos};

const P1: PlayerId = PlayerId(1);
const ARMORY: ObjectId = ObjectId(1);
/// No object's id: the Hall the Recruits' lineage names, gone, so that their Unit Control is orphaned points that
/// [`Lineages`] counts whatever produces meanwhile.
const GONE_HALL: ObjectId = ObjectId(99);

/// Player 1's Armory (1) on tiles (12, 10) to (14, 11) of a flat 30 × 22 map, unturned (its entrance, B, faces east,
/// its exit, C, west), with 1000 Space Crystals, and its Recruits east of it, ids from 2 on, the nearest to the
/// entrance first, each with `hp` of its HP and `lineage`, and walking in: in by tick 60, in that order.
fn armory(recruits: &[(f32, &[ObjectId])]) -> (Sim, Invariants) {
    let mut builder = ScenarioBuilder::new(&"..............................\n".repeat(22))
        .faction(P1, Faction::Cults)
        .spawn(ObjectKind::Armory, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)));
    for i in 0..recruits.len() {
        let at = Vec2::new(1050.0 + 40.0 * i as f32, 704.0);
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(at));
    }
    let mut sim = Sim::new(&builder.build());
    sim.auto_targeting = false;
    sim.players.iter_mut().for_each(|p| p.credit_crystals(1000));
    let units: Vec<ObjectId> = (0..recruits.len()).map(|i| ObjectId(2 + i as u32)).collect();
    for (&id, &(hp, lineage)) in units.iter().zip(recruits) {
        let o = sim.objects.get_mut(&id).unwrap();
        o.hp = Some(hp);
        o.lineage = lineage.to_vec();
    }
    let mut invariants = Invariants::new(&sim);
    let enter = Command::Order {
        units,
        kind: OrderKind::Enter(ARMORY),
        queue: false,
    };
    step(
        &mut sim,
        &mut invariants,
        vec![PlayerCommand {
            player: P1,
            command: enter,
        }],
    );
    for _ in 0..60 {
        step(&mut sim, &mut invariants, Vec::new());
    }
    assert_eq!(sim.held_in(ARMORY).len(), recruits.len(), "all inside");
    (sim, invariants)
}

/// One tick with `commands`, checking the invariants after it.
fn step(sim: &mut Sim, invariants: &mut Invariants, commands: Vec<PlayerCommand>) {
    let inputs = TickInputs { commands };
    sim.step(&inputs);
    if let Err(violation) = invariants.check(sim, &inputs) {
        panic!("{violation:?}");
    }
}

/// Queues a Militant of a Plain Recruit at the Armory, and runs until it is out: the tick it comes out, its id, and
/// the units that left the game into it on the way.
fn train(sim: &mut Sim, invariants: &mut Invariants) -> (Tick, ObjectId, Vec<ObjectId>) {
    let command = Command::Production {
        objects: vec![ARMORY],
        command: ProductionCommand::TrainFrom {
            kind: ObjectKind::Militant,
            from: RecruitKind::Plain,
        },
    };
    step(sim, invariants, vec![PlayerCommand { player: P1, command }]);
    let mut removed = Vec::new();
    for _ in 0..80 {
        step(sim, invariants, Vec::new());
        assert!(
            !sim.events().iter().any(|e| matches!(e, SimEvent::Died { .. })),
            "no unit dies: {:?}",
            sim.events()
        );
        removed.extend(sim.events().iter().filter_map(|e| match *e {
            SimEvent::Removed(id) => Some(id),
            _ => None,
        }));
        let produced = sim.events().iter().find_map(|e| match *e {
            SimEvent::UnitProduced { producer, unit } if producer == ARMORY => Some(unit),
            _ => None,
        });
        if let Some(unit) = produced {
            return (sim.tick(), unit, removed);
        }
    }
    panic!("no Militant came out");
}

/// Design change 0332's Verification (`factions/cults/structures/armory.md > Training`: "infantry trained from a
/// Recruit **keep the Recruit's damage** (the missing HP carries over)"): a hurt Recruit, at 25 of its 40 HP, goes in
/// first, a healthy one after it ("Send in a hurt Recruit, then a healthy one, and queue a Militant: the Militant comes
/// out with the hurt Recruit's missing HP"); the Militant queued comes out of the hurt one, which "entered the building
/// first", missing its 15 HP: at 45 of the 60 a test def gives the Militant (`transform::follow_max`), the same object. The
/// healthy one stays inside, free.
#[test]
fn a_militant_trained_of_a_hurt_recruit_keeps_its_damage() {
    let _militant = TestDef::set(crate::defs::ObjectDef {
        max_hp: Some(60.0),
        ..ObjectKind::Militant.def().clone()
    });
    let (mut sim, mut invariants) = armory(&[(25.0, &[]), (40.0, &[])]);
    let (hurt, healthy) = (ObjectId(2), ObjectId(3));
    let (_, unit, removed) = train(&mut sim, &mut invariants);
    assert_eq!((unit, removed), (hurt, Vec::new()), "the first in, alone");
    let militant = sim.object(hurt).expect("out");
    assert_eq!((militant.kind, militant.hp), (ObjectKind::Militant, Some(45.0)));
    assert_eq!(sim.free_units(ARMORY, RecruitKind::Plain), [healthy]);
    assert_eq!(sim.held_unit(healthy).unwrap().object.hp, Some(40.0));
}

/// `factions/cults/structures/armory.md > Training`: "vehicles and aircraft come out at full HP", and
/// `factions/cults/augmentation.md > Persistence`: "if augmented Recruits are consumed to produce a vehicle, their
/// Augmentations are lost", while infantry keep theirs (`factions/cults/structures/armory.md > Training`: "The trained
/// unit inherits the Recruit's AugmentationState unchanged"). A test def makes the Militant a wheeled vehicle: the hurt
/// Recruit with Enhanced Constitution comes out of it at full HP, with no augmentation; as infantry, with its damage
/// and its augmentation.
#[test]
fn a_vehicle_comes_out_at_full_hp_without_the_augmentations() {
    let (mut sim, mut invariants) = armory(&[(25.0, &[])]);
    let recruit = ObjectId(2);
    let constitution = Augmentation::of(RecruitKind::Constitution);
    sim.held.get_mut(&recruit).unwrap().object.augmentation = constitution;
    let infantry = sim.clone();
    let _vehicle = TestDef::unit(ObjectKind::Militant, |unit| unit.base = UnitBase::WheeledVehicle);
    let command = |from| Command::Production {
        objects: vec![ARMORY],
        command: ProductionCommand::TrainFrom {
            kind: ObjectKind::Militant,
            from,
        },
    };
    let queued = |sim: &Sim| (sim.object(ARMORY).unwrap().production.as_ref()).map_or(0, |p| p.queue.len());
    step(
        &mut sim,
        &mut invariants,
        vec![PlayerCommand {
            player: P1,
            command: command(RecruitKind::Constitution),
        }],
    );
    assert_eq!(queued(&sim), 1, "of a Constitution Recruit");
    for _ in 0..60 {
        step(&mut sim, &mut invariants, Vec::new());
    }
    let out = sim.object(recruit).expect("out");
    assert_eq!(
        (out.kind, out.hp, out.augmentation),
        (ObjectKind::Militant, Some(40.0), Augmentation::NONE)
    );
    drop(_vehicle);
    let mut sim = infantry;
    let mut invariants = Invariants::new(&sim);
    step(
        &mut sim,
        &mut invariants,
        vec![PlayerCommand {
            player: P1,
            command: command(RecruitKind::Constitution),
        }],
    );
    for _ in 0..60 {
        step(&mut sim, &mut invariants, Vec::new());
    }
    let out = sim.object(recruit).expect("out");
    assert_eq!(
        (out.kind, out.hp, out.augmentation),
        (ObjectKind::Militant, Some(25.0), constitution)
    );
}

/// Plan decision T70 (`factions/cults/structures/README.md > Cults Vehicles`: "a trained unit's Unit Control cost
/// equals the number of Recruits consumed"): a unit of Unit Control cost 2, a test def's Militant, reserves the two
/// Plain Recruits that entered first, and comes out of the first, the second leaving the game into it with no death
/// (`SimEvent::Removed`, no `Died`), its lineage joining the product's: one entry for each Recruit consumed (plan
/// decision T56 as amended), so the Unit Control counted is the same before and after. A third Recruit stays inside.
/// With one Plain Recruit free, the order is refused for want of a second (`Reject::NoRecruit`).
#[test]
fn a_unit_of_two_recruits_consumes_both_with_both_lineages() {
    let _two = TestDef::unit(ObjectKind::Militant, |unit| unit.unit_control_cost = Some(2));
    let (mut sim, mut invariants) = armory(&[(40.0, &[GONE_HALL]), (40.0, &[GONE_HALL]), (40.0, &[])]);
    let (first, second, third) = (ObjectId(2), ObjectId(3), ObjectId(4));
    let counted = |sim: &Sim| {
        let lineages = Lineages::of(sim.objects().chain(sim.held.values().map(|held| &held.object)));
        lineages.orphaned(P1)
    };
    assert_eq!(counted(&sim), 3, "two gone Hall's points and an orphan's");
    let (_, unit, removed) = train(&mut sim, &mut invariants);
    assert_eq!((unit, removed), (first, vec![second]));
    let militant = sim.object(first).expect("out");
    assert_eq!(militant.kind, ObjectKind::Militant);
    assert_eq!(militant.lineage, [GONE_HALL, GONE_HALL]);
    assert!(
        sim.held_unit(second).is_none() && sim.object(second).is_none(),
        "consumed"
    );
    assert_eq!(counted(&sim), 3, "the same Unit Control");
    assert_eq!(sim.free_units(ARMORY, RecruitKind::Plain), [third]);
    let again = PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![ARMORY],
            command: ProductionCommand::TrainFrom {
                kind: ObjectKind::Militant,
                from: RecruitKind::Plain,
            },
        },
    };
    assert_eq!(
        sim.validate(&again),
        Err(crate::sim::Reject::NoRecruit(RecruitKind::Plain))
    );
}

/// The same unit of two Recruits, made of a Hall's Recruit and one no Hall produced (an orphan from the start, design
/// change 0344): the product carries the one entry, and the point without one stays orphaned, so the player's Unit
/// Control counts the same (`Lineages::of`: every point of a unit's cost without an entry is orphaned).
#[test]
fn a_unit_of_an_orphan_and_a_halls_recruit_keeps_the_orphans_point() {
    let _two = TestDef::unit(ObjectKind::Militant, |unit| unit.unit_control_cost = Some(2));
    let (mut sim, mut invariants) = armory(&[(40.0, &[]), (40.0, &[GONE_HALL])]);
    let counted =
        |sim: &Sim| Lineages::of(sim.objects().chain(sim.held.values().map(|held| &held.object))).orphaned(P1);
    assert_eq!(counted(&sim), 2);
    let (_, unit, _) = train(&mut sim, &mut invariants);
    assert_eq!(sim.object(unit).unwrap().lineage, [GONE_HALL]);
    assert_eq!(counted(&sim), 2);
}
