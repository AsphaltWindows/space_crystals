use super::*;
use crate::defs::{AreaAbility, MovementAttrs, ObjectKind, ProductionDef, Tech};

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
/// (a field the signs don't take: it isn't structure).
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
}

fn production(rules: &mut Ruleset, kind: ObjectKind) -> &mut ProductionDef {
    let structure = rules.defs.get_mut(kind).structure.as_mut().expect("a structure");
    structure.production.as_mut().expect("a producer")
}
