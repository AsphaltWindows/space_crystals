//! The repairs of vehicles in a game, with test defs (`defs::test_defs`) for what the design's types can't show yet:
//! a recipe for the Seeker, which no building makes until slice N6 (`crate::repair::tests::seeker_recipe`), and one
//! for the Sparrow, a glider. Two Recruits repair a Seeker and follow it as it drives off (`orders::repair`: a vehicle
//! is followed, its repairers parking within reach of it); the gate refuses a glider, by its movement.

use glam::Vec2;

use super::*;
use crate::command::{Command, PlayerCommand, TickInputs};
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, Recipe};
use crate::ids::PlayerId;
use crate::mover::MoveStatus;
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::repair::tests::seeker_recipe;
use crate::scenario::SpawnAt;
use crate::shape::Shape;
use crate::sim::gate::Reject;
use crate::testing::ScenarioBuilder;
use crate::testing::invariants::Invariants;
use crate::tuning::orders::PROXIMITY_SU;

const P1: PlayerId = PlayerId(1);
/// Player 1's vehicle in [`game`], and their two Recruits west of it.
const VEHICLE: ObjectId = ObjectId(1);
const RECRUITS: [ObjectId; 2] = [ObjectId(2), ObjectId(3)];
/// Player 1's bank in [`game`], Space Crystals.
const BANK: u32 = 1000;
/// Where [`VEHICLE`] starts, and where it drives off to.
const START: Vec2 = Vec2::new(1000.0, 960.0);
const AWAY: Vec2 = Vec2::new(1800.0, 960.0);

/// A flat 40 × 30 map of the Cults' with player 1's `vehicle` at [`START`] and two Recruits 120 su west of it, a bank
/// of [`BANK`] Space Crystals, and no auto-targeting.
fn game(vehicle: ObjectKind) -> (Sim, Invariants) {
    let builder = ScenarioBuilder::new(&vec![".".repeat(40); 30].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(vehicle, Some(P1), SpawnAt::Point(START))
        .spawn(
            ObjectKind::Recruit,
            Some(P1),
            SpawnAt::Point(START + Vec2::new(-120.0, -30.0)),
        )
        .spawn(
            ObjectKind::Recruit,
            Some(P1),
            SpawnAt::Point(START + Vec2::new(-120.0, 30.0)),
        );
    let mut sim = Sim::new(&builder.build());
    sim.auto_targeting = false;
    sim.players.iter_mut().find(|p| p.id == P1).expect("player 1").crystals = BANK;
    let invariants = Invariants::new(&sim);
    (sim, invariants)
}

/// One tick with player 1's `commands`, checking the invariants after it.
fn step(sim: &mut Sim, invariants: &mut Invariants, commands: Vec<Command>) {
    let commands = commands
        .into_iter()
        .map(|command| PlayerCommand { player: P1, command })
        .collect();
    let inputs = TickInputs { commands };
    sim.step(&inputs);
    if let Err(violation) = invariants.check(sim, &inputs) {
        panic!("{violation:?}");
    }
}

fn order(units: &[ObjectId], kind: OrderKind) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind,
        queue: false,
    }
}

fn hp(sim: &Sim, id: ObjectId) -> f32 {
    sim.objects[&id].hp.expect("it has HP")
}

/// Whether unit `id`'s current order is a Repair of the vehicle, as the step reads it.
fn repairs_it(sim: &Sim, id: ObjectId) -> bool {
    repairing(&sim.objects[&id]) == Some(VEHICLE)
}

/// How far unit `id` stands from the vehicle, edge to edge.
fn gap(sim: &Sim, id: ObjectId) -> f32 {
    Shape::of(&sim.objects[&id]).gap(&Shape::of(&sim.objects[&VEHICLE]))
}

/// `factions/cults/units/recruit.md > Repair`: "Recruits can **repair** own **buildings and vehicles**"; design change
/// 0321's verification: "Two Recruits repair a vehicle and follow it when it drives off". A Seeker at a quarter of its
/// 120 HP: the two walk to it and repair it, 0.75 HP a tick each (its 120 HP in half its recipe's 320 frames); it
/// drives 800 su east, and they follow it, their Repairs on all the way; once it stops they catch up, within reach of
/// it, and repair it to full, and stop. Its price is half the recipe's 75 Space Crystals, a crystal every 3.2 HP: 28
/// from 30 HP to 120, however the work was split.
#[test]
fn two_recruits_repair_a_vehicle_and_follow_it_when_it_drives_off() {
    let _recipe = seeker_recipe();
    let (mut sim, mut invariants) = game(ObjectKind::Seeker);
    let max = sim.objects[&VEHICLE].max_hp().expect("HP");
    assert_eq!(max, 120.0);
    sim.objects.get_mut(&VEHICLE).expect("the Seeker").hp = Some(30.0);
    step(
        &mut sim,
        &mut invariants,
        vec![order(&RECRUITS, OrderKind::Repair(VEHICLE))],
    );
    let mut before = hp(&sim, VEHICLE);
    let mut at_work = false;
    for _ in 0..200 {
        step(&mut sim, &mut invariants, Vec::new());
        let gained = hp(&sim, VEHICLE) - before;
        before = hp(&sim, VEHICLE);
        if (gained - 1.5).abs() < 1e-4 {
            at_work = true;
            break;
        }
    }
    assert!(at_work, "both at work: {before}");

    step(
        &mut sim,
        &mut invariants,
        vec![order(&[VEHICLE], OrderKind::Move(Target::Point(AWAY)))],
    );
    let mut ticks = 0;
    while sim.objects[&VEHICLE]
        .mover
        .as_ref()
        .is_some_and(|m| m.status == MoveStatus::Moving)
        || ticks < 4
    {
        step(&mut sim, &mut invariants, Vec::new());
        ticks += 1;
        assert!(ticks < 400, "it gets there");
        for recruit in RECRUITS {
            assert!(
                repairs_it(&sim, recruit),
                "{recruit:?} repairs it all the way, tick {ticks}"
            );
        }
    }
    let driven = sim.objects[&VEHICLE].pos;
    assert!(driven.distance(AWAY) < 64.0, "it drove off: {driven}");
    let left = hp(&sim, VEHICLE);
    assert!(left < max - 30.0, "much left to repair once it stops: {left}");
    while hp(&sim, VEHICLE) < max {
        step(&mut sim, &mut invariants, Vec::new());
        ticks += 1;
        assert!(ticks < 800, "repaired to full: {}", hp(&sim, VEHICLE));
    }
    assert!(sim.objects[&VEHICLE].pos.distance(driven) < 16.0, "it stood");
    for recruit in RECRUITS {
        assert!(!repairs_it(&sim, recruit), "{recruit:?} stopped at full HP");
        assert!(
            gap(&sim, recruit) <= PROXIMITY_SU + 1.0,
            "{recruit:?} by it: {}",
            gap(&sim, recruit)
        );
    }
    let paid = BANK - sim.players.iter().find(|p| p.id == P1).expect("player 1").crystals;
    assert_eq!(paid, 28);
}

/// "**Gliders can't be repaired** (they can't stop; user, 2026-11-26)": told by the movement (`repair::is_vehicle`),
/// not by the name nor by what builds it. A damaged Sparrow, a glider, with a recipe (test data: 75 Space Crystals and
/// 320 frames, as the Seeker's) is refused by the gate; so is the Seeker once it glides (a test def with the Sparrow's
/// movement), and a driving Seeker is taken.
#[test]
fn the_gate_refuses_a_glider_by_its_movement() {
    static SPARROW: [Recipe; 1] = [Recipe {
        kind: ObjectKind::Sparrow,
        cost: 75,
        frames: 320,
        requires: &[],
    }];
    let _seeker = seeker_recipe();
    let _sparrow = TestDef::structure(ObjectKind::Airstrip, |structure| structure.constructs = &SPARROW);
    let repair = |sim: &Sim| {
        sim.validate(&PlayerCommand {
            player: P1,
            command: order(&RECRUITS, OrderKind::Repair(VEHICLE)),
        })
    };
    let (mut sparrow, _) = game(ObjectKind::Sparrow);
    sparrow.objects.get_mut(&VEHICLE).expect("the Sparrow").hp = Some(10.0);
    assert!(crate::repair::terms(ObjectKind::Sparrow).is_some(), "it has its price");
    assert_eq!(repair(&sparrow), Err(Reject::CantRepair(VEHICLE)));
    let (mut seeker, _) = game(ObjectKind::Seeker);
    seeker.objects.get_mut(&VEHICLE).expect("the Seeker").hp = Some(10.0);
    assert_eq!(repair(&seeker), Ok(()));
    let glide = ObjectKind::Sparrow.def().unit.as_ref().expect("a unit").movement;
    let _gliding = TestDef::unit(ObjectKind::Seeker, |unit| unit.movement = glide);
    assert_eq!(repair(&seeker), Err(Reject::CantRepair(VEHICLE)));
}
