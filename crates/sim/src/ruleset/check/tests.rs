use super::*;
use crate::defs::{AreaAbility, MovementAttrs, ObjectKind, ProductionDef, StockDef, Tech};

/// The built-in rules with `change` made: what [`Ruleset::check`] says of them, by entry, field and why.
fn checked(change: &dyn Fn(&mut Ruleset)) -> Option<(&'static str, &'static str, &'static str)> {
    let mut rules = Ruleset::builtin();
    change(&mut rules);
    (rules.check()).map(|invalid| (invalid.field.entry, invalid.field.field, invalid.why))
}

fn max_speed(rules: &mut Ruleset, kind: ObjectKind) -> &mut f32 {
    match &mut rules.defs.get_mut(kind).unit.as_mut().expect("a unit").movement {
        MovementAttrs::TurnRate { max_speed, .. } | MovementAttrs::SpeedTurnRadius { max_speed, .. } => max_speed,
        other => panic!("{kind:?} moves by {other:?}"),
    }
}

/// The built-in rules pass, and so does a number changed within its sign: a Peacekeeper's Damage of 100, its speed
/// halved, a Recruit's armour of 0 raised to 2, and the one negative tunable more negative.
#[test]
fn numbers_may_change_within_their_signs() {
    assert_eq!(Ruleset::builtin().check(), None);
    assert_eq!(
        checked(&|r| {
            let unit = r.defs.get_mut(ObjectKind::Peacekeeper).unit.as_mut().unwrap();
            unit.attack.as_mut().unwrap().damage = 100.0;
            *max_speed(r, ObjectKind::Peacekeeper) /= 2.0;
            r.defs.get_mut(ObjectKind::Recruit).armor.point = 2.0;
            r.tuning.movement.lane_oncoming_cos = -0.9;
        }),
        None
    );
}

/// A number the built-in rules have positive stays positive: a zero speed, a negative cost (an Inspire's energy), a
/// zero production time, a zero tunable step. One they have at 0 can't go negative (a Recruit's armour), and one they
/// have negative stays negative. Each is named where it is.
#[test]
fn numbers_keep_their_signs() {
    assert_eq!(Ruleset::builtin().defs[ObjectKind::Recruit].armor.point, 0.0);
    assert_eq!(
        checked(&|r| *max_speed(r, ObjectKind::Peacekeeper) = 0.0),
        Some(("Peacekeeper", "max_speed", "must be positive"))
    );
    assert_eq!(
        checked(&|r| r.factions.cults.abilities.get_mut(AreaAbility::Inspire).cost = -5.0),
        Some(("Inspire", "cost", "must be positive"))
    );
    assert_eq!(
        checked(&|r| r.factions.cults.techs.get_mut(Tech::ImprovedMetabolismL2).frames = 0),
        Some(("ImprovedMetabolismL2", "frames", "must be positive"))
    );
    assert_eq!(
        checked(&|r| r.tuning.spells.lattice_step = 0.0),
        Some(("tuning", "lattice_step", "must be positive"))
    );
    assert_eq!(
        checked(&|r| r.defs.get_mut(ObjectKind::Recruit).armor.point = -1.0),
        Some(("Recruit", "point", "must not be negative"))
    );
    assert_eq!(
        checked(&|r| r.tuning.movement.lane_oncoming_cos = 0.5),
        Some(("tuning", "lane_oncoming_cos", "must be negative"))
    );
}

/// The bounds of `defs/tests.rs` beyond the signs: a recipe's time and a technology's, a Recruitment Area that
/// covers its Hall, an air unit's SeparationRadius beyond its Silhouette, a default rally point out of its building
/// (a field the signs don't take: it isn't structure), (M6-GD) a stock's purchase time and its start, (M6-SD) an
/// aircraft's time at an air door, (M6-GA) a vehicle deploy's time.
#[test]
fn numbers_stay_within_the_defs_bounds() {
    let longest = |r: &mut Ruleset| {
        let barracks = production(r, ObjectKind::Barracks);
        let mut recipes = barracks.recipes.to_vec();
        recipes[0].frames = LONGEST_RECIPE + 1;
        barracks.recipes = recipes.leak();
    };
    assert_eq!(
        checked(&longest),
        Some(("Barracks", "frames", "must be at most 65535 ticks"))
    );
    assert_eq!(
        checked(&|r| r.factions.cults.techs.get_mut(Tech::ImprovedMetabolismL2).frames = LONGEST_RECIPE + 1),
        Some(("ImprovedMetabolismL2", "frames", "must be at most 65535 ticks"))
    );
    let area = |side| {
        move |r: &mut Ruleset| {
            let hall = production(r, ObjectKind::RecruitmentHall);
            hall.recruitment.as_mut().unwrap().area = side;
        }
    };
    assert_eq!(
        checked(&area(3)),
        Some(("RecruitmentHall", "area", "must cover the building"))
    );
    assert_eq!(
        checked(&area(WIDEST_RECRUITMENT_AREA + 1)),
        Some(("RecruitmentHall", "area", "must be at most 255 tiles"))
    );
    assert_eq!(checked(&area(4)), None, "the Hall's own 4 × 4");
    let sparrow = |r: &mut Ruleset| {
        let unit = r.defs.get_mut(ObjectKind::Sparrow).unit.as_mut().unwrap();
        unit.separation_radius = Some(10.0);
    };
    assert_eq!(
        checked(&sparrow),
        Some(("Sparrow", "separation_radius", "must be longer than the Silhouette"))
    );
    assert_eq!(
        checked(&|r| production(r, ObjectKind::ChopShop).default_rally = Some(0)),
        Some(("ChopShop", "default_rally", "must be positive"))
    );
    // (M6-GD) The Drone Operator's stock: a purchase as long as a recipe at most, and no more drones at the start
    // than it holds; as many is its own.
    assert_eq!(
        checked(&|r| stock(r).frames = LONGEST_RECIPE + 1),
        Some(("DroneOperator", "frames", "must be at most 65535 ticks"))
    );
    assert_eq!(
        checked(&|r| stock(r).start = 2),
        Some(("DroneOperator", "start", "must be at most the stock's max"))
    );
    assert_eq!(
        checked(&|r| {
            stock(r).max = 3;
            stock(r).start = 3;
        }),
        None
    );
    // (M6-SD) The Phantom's rise and descent at an air door: as long as a recipe at most.
    let door = |ticks| {
        move |r: &mut Ruleset| {
            r.defs
                .get_mut(ObjectKind::Phantom)
                .unit
                .as_mut()
                .unwrap()
                .air_door_ticks = ticks;
        }
    };
    assert_eq!(
        checked(&door(LONGEST_RECIPE + 1)),
        Some(("Phantom", "air_door_ticks", "must be at most 65535 ticks"))
    );
    // (M6-GA) The Artillery Rig's deploy: as long as a recipe at most, and never instant.
    let deploy = |frames| {
        move |r: &mut Ruleset| {
            let unit = r.defs.get_mut(ObjectKind::ArtilleryRig).unit.as_mut().unwrap();
            unit.vehicle_deploy.as_mut().unwrap().frames = frames;
        }
    };
    assert_eq!(
        checked(&deploy(LONGEST_RECIPE + 1)),
        Some(("ArtilleryRig", "frames", "must be at most 65535 ticks"))
    );
    assert_eq!(checked(&deploy(LONGEST_RECIPE)), None);
    assert_eq!(
        checked(&deploy(0)),
        Some(("ArtilleryRig", "frames", "must be positive")),
        "the sign's rule"
    );
    assert_eq!(checked(&door(LONGEST_RECIPE)), None);
    assert_eq!(
        checked(&door(0)),
        Some(("Phantom", "air_door_ticks", "must be positive")),
        "the sign's rule"
    );
}

/// (M6-SN) A Null Chamber turn's seconds, of a ruleset whose Null Chamber's turn is `turn`: as long as a recipe at most,
/// in ticks (4095.9 s round to 65534, 4096 to 65536), and positive by the sign's rule.
#[test]
fn a_null_chamber_turn_stays_within_a_recipe_s_bound() {
    let turn = |turn: f32| {
        move |r: &mut Ruleset| {
            let structure = r.defs.get_mut(ObjectKind::NullChamber).structure.as_mut().unwrap();
            structure.chamber.as_mut().expect("a chamber").turn = turn;
        }
    };
    assert_eq!(
        checked(&turn(4096.0)),
        Some(("NullChamber", "turn", "must be at most 65535 ticks (4095 s)"))
    );
    assert_eq!(checked(&turn(4095.9)), None);
    assert_eq!(checked(&turn(0.0)), Some(("NullChamber", "turn", "must be positive")));
}

/// (M6-GH2) An Airfield's landing and its slots, and an aircraft's rearm of a round: the landing and the rearm as long
/// as a recipe at most, in fixed point, and positive by the sign's rule; the capacity at most the 4 slots of the
/// placeholder layout, one per assignment; the slots inside the 3 × 3 footprint, 96 su from its centre or nearer.
#[test]
fn an_airfield_s_landing_and_slots_stay_within_their_bounds() {
    fn airfield(rules: &mut Ruleset) -> &mut crate::defs::AirfieldDef {
        let structure = rules.defs.get_mut(ObjectKind::Airfield).structure.as_mut().unwrap();
        structure.airfield.as_mut().expect("an Airfield")
    }
    let landing = |ticks| move |r: &mut Ruleset| airfield(r).landing_ticks = ticks;
    assert_eq!(
        checked(&landing(LONGEST_RECIPE + 1)),
        Some(("Airfield", "landing_ticks", "must be at most 65535 ticks"))
    );
    assert_eq!(checked(&landing(LONGEST_RECIPE)), None);
    assert_eq!(
        checked(&landing(0)),
        Some(("Airfield", "landing_ticks", "must be positive"))
    );
    assert_eq!(
        checked(&|r| airfield(r).capacity = 5),
        Some((
            "Airfield",
            "capacity",
            "must be at most 4, the Airfield's landing slots"
        ))
    );
    assert_eq!(checked(&|r| airfield(r).capacity = 3), None);
    assert_eq!(
        checked(&|r| airfield(r).slot_offset_su = 96.0),
        Some((
            "Airfield",
            "slot_offset_su",
            "must keep the landing slots inside the footprint"
        ))
    );
    assert_eq!(checked(&|r| airfield(r).slot_offset_su = 95.0), None);
    let rearm = |ticks| {
        move |r: &mut Ruleset| {
            let unit = r.defs.get_mut(ObjectKind::Hornet).unit.as_mut().unwrap();
            unit.ammo.as_mut().expect("ammo").rearm_ticks_per_ammo = ticks;
        }
    };
    assert_eq!(
        checked(&rearm(LONGEST_RECIPE + 1)),
        Some(("Hornet", "rearm_ticks_per_ammo", "must be at most 65535 ticks"))
    );
    assert_eq!(checked(&rearm(LONGEST_RECIPE)), None);
}

/// A Supply Chopper's landing and a Supply Tower's spot: the landing as long as a recipe at most, in fixed
/// point, and positive by the sign's rule; the spot inside the 3 × 3 footprint, less than 96 su from its centre each
/// way, and not to the negative side of it by the sign's rule (the built-in offset is 0).
#[test]
fn a_chopper_s_landing_and_a_tower_s_spot_stay_within_their_bounds() {
    fn chopper(rules: &mut Ruleset) -> &mut crate::defs::SupplyChopperDef {
        let unit = rules.defs.get_mut(ObjectKind::SupplyChopper).unit.as_mut().unwrap();
        unit.supply_chopper.as_mut().expect("a Supply Chopper")
    }
    fn tower(rules: &mut Ruleset) -> &mut crate::defs::SupplyTowerDef {
        let structure = rules.defs.get_mut(ObjectKind::SupplyTower).structure.as_mut().unwrap();
        structure.supply_tower.as_mut().expect("a Supply Tower")
    }
    let landing = |ticks| move |r: &mut Ruleset| chopper(r).landing_ticks = ticks;
    assert_eq!(
        checked(&landing(LONGEST_RECIPE + 1)),
        Some(("SupplyChopper", "landing_ticks", "must be at most 65535 ticks"))
    );
    assert_eq!(checked(&landing(LONGEST_RECIPE)), None);
    assert_eq!(
        checked(&landing(0)),
        Some(("SupplyChopper", "landing_ticks", "must be positive"))
    );
    let spot = |offset| move |r: &mut Ruleset| tower(r).spot_offset_su = offset;
    let outside = Some((
        "SupplyTower",
        "spot_offset_su",
        "must keep the landing spot inside the footprint",
    ));
    assert_eq!(checked(&spot([96.0, 0.0])), outside);
    assert_eq!(checked(&spot([0.0, 96.0])), outside);
    assert_eq!(checked(&spot([95.0, 95.0])), None);
    assert_eq!(
        checked(&spot([-1.0, 0.0])),
        Some(("SupplyTower", "spot_offset_su", "must not be negative"))
    );
}

/// (M6-SB) The Shadebringer's Sludge's tank: at most 65535 units of charge, its charge counted in fixed point, and its
/// slow less than the whole speed, both positive by the sign's rule.
#[test]
fn a_tank_stays_within_its_fixed_point_and_a_slow_below_a_standstill() {
    use crate::defs::TankDef;
    let tank = |change: fn(&mut TankDef)| {
        move |r: &mut Ruleset| {
            let unit = r.defs.get_mut(ObjectKind::Shadebringer).unit.as_mut().expect("a unit");
            let mut tanks = unit.tanks.to_vec();
            change(&mut tanks[0]);
            unit.tanks = Vec::leak(tanks);
        }
    };
    assert_eq!(checked(&tank(|t| t.capacity = 65535.0)), None);
    assert_eq!(
        checked(&tank(|t| t.capacity = 65536.0)),
        Some(("Shadebringer", "capacity", "must be at most 65535"))
    );
    assert_eq!(
        checked(&tank(|t| t.capacity = 0.0)),
        Some(("Shadebringer", "capacity", "must be positive"))
    );
    assert_eq!(checked(&tank(|t| t.slow.as_mut().unwrap().by = 0.99)), None);
    assert_eq!(
        checked(&tank(|t| t.slow.as_mut().unwrap().by = 1.0)),
        Some(("Shadebringer", "by", "must be less than 1"))
    );
    assert_eq!(
        checked(&tank(|t| t.slow.as_mut().unwrap().by = 0.0)),
        Some(("Shadebringer", "by", "must be positive"))
    );
}

/// (M6-GD) The Drone Operator's stock in `rules`.
fn stock(rules: &mut Ruleset) -> &mut StockDef {
    let unit = rules
        .defs
        .get_mut(ObjectKind::DroneOperator)
        .unit
        .as_mut()
        .expect("a unit");
    unit.stock.as_mut().expect("a stock")
}

fn production(rules: &mut Ruleset, kind: ObjectKind) -> &mut ProductionDef {
    let structure = rules.defs.get_mut(kind).structure.as_mut().expect("a structure");
    structure.production.as_mut().expect("a producer")
}

/// The Prism's thrusts (`crate::thrust`): its charges, and each thrust's distance and frames, positive by the sign's
/// rule, so that a burst lasts a tick at least.
#[test]
fn a_thrust_keeps_its_numbers_positive() {
    use crate::defs::ThrustDef;
    let thrust = |change: fn(&mut ThrustDef)| {
        move |r: &mut Ruleset| {
            let unit = r.defs.get_mut(ObjectKind::Prism).unit.as_mut().expect("a unit");
            change(unit.thrust.as_mut().expect("its thrusts"));
        }
    };
    assert_eq!(
        checked(&thrust(|t| (t.charges, t.distance, t.frames) = (1, 0.5, 1))),
        None
    );
    assert_eq!(
        checked(&thrust(|t| t.charges = 0)),
        Some(("Prism", "charges", "must be positive"))
    );
    assert_eq!(
        checked(&thrust(|t| t.distance = 0.0)),
        Some(("Prism", "distance", "must be positive"))
    );
    assert_eq!(
        checked(&thrust(|t| t.frames = 0)),
        Some(("Prism", "frames", "must be positive"))
    );
}
