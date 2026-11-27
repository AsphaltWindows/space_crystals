use super::*;
use crate::ids::ObjectId;
use crate::orders::Target;

/// The Artillery Rig's pair (`factions/gdo/units/artillery_rig.md > Modes`): each type deploys into the other, in 48
/// frames, and only the deployed one can't move. A type with no row deploys into nothing and takes no frames.
#[test]
fn the_rigs_types_deploy_into_each_other() {
    let (mobile, deployed) = (ObjectKind::ArtilleryRig, ObjectKind::ArtilleryRigDeployed);
    assert!(deploys_into(mobile, deployed) && deploys_into(deployed, mobile));
    assert!(!deploys_into(mobile, mobile) && !deploys_into(ObjectKind::Peacekeeper, deployed));
    assert_eq!((begin(mobile), begin(deployed)), (Progress::new(48), Progress::new(48)));
    assert_eq!(begin(ObjectKind::Peacekeeper), Progress::new(0));
    assert!(immobile(deployed) && !immobile(mobile) && !immobile(ObjectKind::Peacekeeper));
    assert!(!immobile(ObjectKind::Factory), "a structure is no unit");
}

/// The orders that take a unit somewhere, which a type that can't move can't carry out: the moves and a unit's walks;
/// not an Attack, an Attack Ground, a Hold, a Stop nor a deploy.
#[test]
fn only_the_orders_that_take_a_unit_somewhere_move_it() {
    let at = glam::Vec2::new(10.0, 10.0);
    let moving = [
        OrderKind::Move(Target::Point(at)),
        OrderKind::Move(Target::Object(ObjectId(3))),
        OrderKind::AttackMove(at),
        OrderKind::Patrol(at),
        OrderKind::Reverse(at),
        OrderKind::Enter(ObjectId(3)),
    ];
    let standing = [
        OrderKind::Attack(ObjectId(3)),
        OrderKind::AttackGround(at),
        OrderKind::Hold,
        OrderKind::Stop,
        OrderKind::VehicleDeploy {
            into: ObjectKind::ArtilleryRig,
        },
    ];
    assert!(moving.into_iter().all(moves), "{moving:?}");
    assert!(!standing.into_iter().any(moves), "{standing:?}");
}
