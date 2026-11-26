//! Repair's rules (`crate::repair`): the terms of the design's types, who repairs what, and the steps of a price.

use super::*;
use crate::defs::test_defs::TestDef;
use crate::defs::{MovementAttrs, UnitBase};

/// The terms of `kind`'s repair: the frames one repairer takes from 0 to full, and the price.
fn of(kind: ObjectKind) -> Option<(f32, Option<f32>)> {
    terms(kind).map(|terms| (terms.frames, terms.price))
}

/// The Chop Shop with a test recipe for the Seeker, which no building makes until slice N6: 75 Space Crystals and 320
/// frames. `crate::sim::repairs`' tests repair a Seeker by it too.
pub(crate) fn seeker_recipe() -> TestDef {
    static RECIPES: [Recipe; 1] = [Recipe {
        kind: ObjectKind::Seeker,
        cost: 75,
        frames: 320,
        requires: &[],
    }];
    TestDef::structure(ObjectKind::ChopShop, |structure| {
        structure.constructs = &RECIPES;
    })
}

/// "**Placeholder rate and cost** (designer, 2026-11-26): repairing from 0 to full HP takes **half the target's build
/// time** with one repairer, and costs **half its price**" (`factions/cults/units/recruit.md > Repair`), by what built
/// it: a site, an expansion's build, a recipe.
#[test]
fn a_repair_takes_half_the_build_time_and_costs_half_the_price() {
    // Design change 0321's example: the Armory (480 frames, 200 Space Crystals) from 0 to full in 240 frames for 100.
    assert_eq!(of(ObjectKind::Armory), Some((240.0, Some(100.0))));
    assert_eq!(of(ObjectKind::Storage), Some((160.0, Some(50.0))));
    assert_eq!(of(ObjectKind::RecruitmentHall), Some((720.0, Some(150.0))));
    // Design change 0328: "a Tunnel's rate uses its construction time, 480 frames, at every Tier"; its price is in
    // Supplies, so no repairer that pays repairs it.
    assert_eq!(of(ObjectKind::Tunnel), Some((240.0, None)));
    // The Laser Turret by its Tunnel's build of it (design change 0359): 320 frames, 100 Space Crystals.
    assert_eq!(of(ObjectKind::LaserTurret), Some((160.0, Some(50.0))));
    // A vehicle by its recipe, "its **Space Crystals only**, not the Recruits it consumed": none makes the Seeker yet.
    assert_eq!(of(ObjectKind::Seeker), None);
    let _recipe = seeker_recipe();
    assert_eq!(of(ObjectKind::Seeker), Some((160.0, Some(37.5))));
}

/// Who repairs what (`WorkerDef::repairs`): the Recruit its owner's buildings and vehicles, for Space Crystals; the
/// Agent its Tunnels for free and its Laser Turrets for Space Crystals; no one a Tunnel's expansions, a unit of an
/// infantry base or a glider.
#[test]
fn each_repairer_repairs_what_its_def_says() {
    let _recipe = seeker_recipe();
    let (recruit, agent, drone) = (ObjectKind::Recruit, ObjectKind::Agent, ObjectKind::Drone);
    for building in [
        ObjectKind::Storage,
        ObjectKind::Armory,
        ObjectKind::RecruitmentHall,
        ObjectKind::Gunworks,
        ObjectKind::ChopShop,
    ] {
        assert_eq!(payment(recruit, building), Some(RepairPayment::Paid), "{building:?}");
        assert_eq!(payment(agent, building), None, "{building:?}");
        assert_eq!(payment(drone, building), None, "{building:?}");
    }
    assert_eq!(payment(recruit, ObjectKind::Seeker), Some(RepairPayment::Paid));
    assert_eq!(payment(agent, ObjectKind::Seeker), None);
    // Infantry are no vehicles.
    for unit in [
        ObjectKind::Recruit,
        ObjectKind::Militant,
        ObjectKind::Gunner,
        ObjectKind::Agent,
    ] {
        assert_eq!(payment(recruit, unit), None, "{unit:?}");
    }
    assert_eq!(payment(agent, ObjectKind::Tunnel), Some(RepairPayment::Free));
    assert_eq!(payment(agent, ObjectKind::LaserTurret), Some(RepairPayment::Paid));
    // The Recruit pays in Space Crystals, and the Tunnel's price is in Supplies.
    assert_eq!(payment(recruit, ObjectKind::Tunnel), None);
    // "Not repaired; they regenerate" (`factions/syndicate/structures/README.md > Tunnel Expansions`).
    for expansion in [ObjectKind::Hideout, ObjectKind::ChemicalPlant, ObjectKind::OpticsLab] {
        assert!(terms(expansion).is_some(), "{expansion:?} has a build");
        assert_eq!(payment(agent, expansion), None, "{expansion:?}");
        assert_eq!(payment(recruit, expansion), None, "{expansion:?}");
    }
    assert!(repairs(agent, ObjectKind::Tunnel) && !repairs(drone, ObjectKind::Tunnel));
}

/// "**Gliders can't be repaired** (they can't stop)": told by the movement, not the name nor the base.
#[test]
fn a_glider_is_no_vehicle_by_its_movement() {
    assert!(is_vehicle(ObjectKind::Seeker) && is_vehicle(ObjectKind::Drone));
    assert!(!is_vehicle(ObjectKind::Sparrow), "a glider");
    assert!(!is_vehicle(ObjectKind::Recruit) && !is_vehicle(ObjectKind::Storage));
    let glider = ObjectKind::Sparrow.def().unit.as_ref().unwrap().movement;
    {
        let _gliding_seeker = TestDef::unit(ObjectKind::Seeker, |unit| unit.movement = glider);
        assert!(!is_vehicle(ObjectKind::Seeker), "a Seeker that glides");
    }
    let _driving_sparrow = TestDef::unit(ObjectKind::Sparrow, |unit| {
        assert_eq!(unit.base, UnitBase::Glider);
        unit.movement = MovementAttrs::TurnRate {
            turn_rate: 0.1,
            acceleration: None,
            deceleration: None,
            max_speed: 4.0,
        };
    });
    assert!(
        is_vehicle(ObjectKind::Sparrow),
        "a Sparrow that drives, of the Glider base"
    );
}

/// The steps of a price (`step_of`, `top_of_step`): one crystal per MaxHP / price HP, the top of each in it.
#[test]
fn a_price_is_paid_in_steps_of_whole_crystals() {
    // The Armory's 100 over 300 HP: a crystal for every 3 HP.
    let (max, price) = (300.0, 100.0);
    assert_eq!(step_of(0.0, max, price), 0);
    assert_eq!(step_of(150.0, max, price), 50);
    assert_eq!(step_of(150.5, max, price), 51);
    assert_eq!(step_of(153.0, max, price), 51);
    assert_eq!(step_of(300.0, max, price), 100);
    assert_eq!(top_of_step(51, max, price, max), 153.0);
    assert_eq!(top_of_step(51, max, price, 152.0), 152.0, "no further than asked");
    assert_eq!(top_of_step(0, max, price, max), 0.0);
    // The Seeker's 37.5 over 120 HP: 3.2 HP a step, its whole 38 from 0 to full.
    let (max, price) = (120.0, 37.5);
    assert_eq!(step_of(max, max, price), 38);
    for step in 0..38 {
        let top = top_of_step(step, max, price, max);
        assert!(step_of(top, max, price) <= step, "{step}: {top}");
        assert!(step_of(top.next_up(), max, price) > step, "{step}: {top} is the top");
    }
    assert_eq!(top_of_step(38, max, price, max), max);
    // A free repair: no steps.
    assert_eq!(top_of_step(0, max, 0.0, 77.0), 77.0);
    assert_eq!(step_of(10.0, 0.0, price), 0);
}
