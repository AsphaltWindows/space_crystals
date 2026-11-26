use super::*;
use crate::nav::NavClass;

#[test]
fn every_base_has_its_def() {
    for base in UnitBase::ALL {
        assert_eq!(base.def().base, base);
    }
}

/// `systems/units.md > UnitBase`: the bases that cross Rugged tiles, the crushable one, the air ones.
#[test]
fn base_flags_follow_the_design() {
    let rugged: Vec<UnitBase> = UnitBase::ALL.into_iter().filter(|b| b.def().rugged_terrain).collect();
    assert_eq!(
        rugged,
        [
            UnitBase::LightInfantry,
            UnitBase::HeavyInfantry,
            // "RuggedTerrain - true, with double drag" (design change 0039).
            UnitBase::HoverVehicle,
            UnitBase::Mech
        ]
    );
    let crushable: Vec<UnitBase> = UnitBase::ALL.into_iter().filter(|b| b.def().crushable).collect();
    assert_eq!(crushable, [UnitBase::LightInfantry]);
    let air: Vec<UnitBase> = UnitBase::ALL
        .into_iter()
        .filter(|b| b.def().domain == Domain::Air)
        .collect();
    assert_eq!(air, [UnitBase::HoverCraft, UnitBase::Glider]);
    assert_eq!(UnitBase::DrillUnit.def().domain, Domain::Underground);
    // `systems/units.md > UnitBase > CanHaveTurret` (design change 0154): infantry can't carry a turret; every
    // vehicle, mech and aircraft base can.
    for base in UnitBase::ALL {
        let infantry = matches!(base, UnitBase::LightInfantry | UnitBase::HeavyInfantry);
        assert_eq!(base.def().can_have_turret, !infantry, "{base:?}");
    }
}

/// (M5-H2) `systems/units.md > Air Collision`: the air bases keep apart softly (`movement::air`), a HoverCraft by the
/// whole push, as in slice M5-H1, a Glider by a quarter of it (design change 0405: "**Gliders push apart slowly** [...]
/// (placeholder: a quarter of the HoverCraft strength)"); the others collide hard and have no strength.
#[test]
fn gliders_push_apart_at_a_quarter_of_the_hovercraft_s_strength() {
    for base in UnitBase::ALL {
        let strength = match base {
            UnitBase::HoverCraft => Some(1.0),
            UnitBase::Glider => Some(0.25),
            _ => None,
        };
        assert_eq!(base.def().separation_strength, strength, "{base:?}");
        assert_eq!(strength.is_none(), base.def().domain.hard_collision(), "{base:?}");
    }
}

#[test]
fn nav_classes_follow_domain_and_rugged_terrain() {
    assert_eq!(UnitBase::LightInfantry.def().nav_class(), Some(NavClass::GroundRugged));
    assert_eq!(UnitBase::Mech.def().nav_class(), Some(NavClass::GroundRugged));
    assert_eq!(UnitBase::HoverVehicle.def().nav_class(), Some(NavClass::GroundRugged));
    assert_eq!(UnitBase::WheeledVehicle.def().nav_class(), Some(NavClass::Ground));
    assert_eq!(UnitBase::DrillUnit.def().nav_class(), Some(NavClass::Underground));
    assert_eq!(UnitBase::Glider.def().nav_class(), None);
}

#[test]
fn drag_top_speed_balances_thrust_and_drag() {
    let drag = MovementAttrs::Drag {
        forward_acceleration: 0.3,
        omni_directional_acceleration: 0.6,
        drag_ratio: 0.1,
        turn_rate: 0.5,
    };
    assert!((drag.max_speed() - 9.0).abs() < 1e-5);
    assert_eq!(drag.turn_rate(), Some(0.5));
    assert_eq!(drag.model(), MovementModel::Drag);
}
