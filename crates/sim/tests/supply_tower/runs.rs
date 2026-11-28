//! A Supply Chopper's orders by hand (`factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`):
//! Pick Up Supplies, Drop Off Supplies and Attach to Tower, by the right-click and through the gate.

use sc_sim::{ProductionCommand, Target};

use super::*;

/// The station of these tests: on tiles (14, 8) to (15, 9), its spot at (960, 576), holding 3 Supplies.
const S: u32 = FIRST;

/// [`base`] with [`S`], delivering every 960 ticks, a Peacekeeper of player 1's on tile (6, 12) ([`FIRST`] + 1) and a
/// Supply Tower of player 2's on tiles (14, 2) to (16, 4) ([`FIRST`] + 2, spawned, so without a chopper), both in
/// player 1's sight; and a Supply Tower of player 1's placed on tiles (2, 8) to (4, 10), with its free chopper.
fn field() -> (TestSim, u32, u32) {
    let mut t = base(|b| {
        b.spawn_with(station(3, 960), tile(14, 8))
            .spawn(
                ObjectKind::Peacekeeper,
                Some(P1),
                SpawnAt::Point(TilePos::new(6, 12).center()),
            )
            .spawn(TOWER, Some(P2), tile(14, 2))
    });
    let (tower, chopper) = placed_tower(&mut t, TilePos::new(2, 8));
    (t, tower, chopper)
}

const PEACEKEEPER: u32 = FIRST + 1;
const THEIR_TOWER: u32 = FIRST + 2;

/// A chopper trained at `tower`, out over its centre: its id.
fn trained(t: &mut TestSim, tower: u32) -> u32 {
    let train = Command::Production {
        objects: vec![id(tower)],
        command: ProductionCommand::Train(CHOPPER),
    };
    next(t, train);
    let before = spawned(t).len();
    step_until(t, 2 * CHOPPER_TICKS, "trained", |t| spawned(t).len() > before);
    spawned(t)[before].1
}

/// "Right-click SupplyDeliveryStation: issues PickUpSupplies command (fly to SDS, land, pick up all available
/// supplies)": the free chopper, attached and down on its Tower, takes off at the click, its attachment broken (a
/// command), lands on the station's spot in 16 frames, and takes all 3 Supplies as it touches down, the station empty
/// and counting down; its order over, it lifts off (a placeholder). "Right-click own SupplyTower (when carrying
/// supplies): issues DropOffSupplies command": it lands on its Tower, pays the 3 in, and lifts off at once, the Tower
/// not its own any more ("immediately lift off — unless this is the chopper's attached tower"). "(when not carrying
/// supplies): issues AttachToTower command": attached as the order starts, it lands there again and stays.
#[test]
fn a_pick_up_takes_all_a_station_holds_and_a_drop_off_pays_it_in() {
    let (mut t, tower, chopper) = field();
    let click = right_click(&t, &[chopper], S);
    assert_eq!(click, pick_up(&[chopper], S));
    next(&mut t, click);
    assert_eq!(attached(&t, chopper), None, "the command broke its attachment");
    assert_eq!(landing_of(&t, chopper), None, "it takes off");
    let began = step_until(&mut t, 400, "it lands on the station", |t| {
        landing_of(t, chopper).is_some()
    });
    assert_eq!(landing_of(&t, chopper).map(|landing| landing.at), Some(id(S)));
    assert!(t.pos(chopper).distance(spot(&t, S)) <= ON_SPOT);
    t.run(LANDING - 1);
    assert_eq!((held(&t, S), carried(&t, chopper)), (3, 0), "not yet down");
    t.step();
    assert_eq!(
        (held(&t, S), carried(&t, chopper)),
        (0, 3),
        "down in tick {}",
        began + LANDING
    );
    assert_eq!(doing(&t, chopper), None, "its order over");
    assert_eq!(landing_of(&t, chopper), None, "it lifts off");
    assert!(t.mover(chopper).rise.is_some());

    let click = right_click(&t, &[chopper], tower);
    assert_eq!(click, drop_off(&[chopper], tower));
    next(&mut t, click);
    let paid = step_until(&mut t, 400, "paid in", |t| banks(t, P1).1 == 3);
    let delivered = (t.events().iter()).any(|(tick, e)| {
        *tick == paid
            && *e
                == SimEvent::ResourcesDelivered {
                    worker: id(chopper),
                    drop_off: id(tower),
                    amount: 3,
                    resource: sc_sim::defs::Resource::Supplies,
                }
    });
    assert!(delivered);
    assert_eq!(
        (carried(&t, chopper), doing(&t, chopper), landing_of(&t, chopper)),
        (0, None, None)
    );
    assert!(
        t.mover(chopper).rise.is_some(),
        "it lifts off its Tower, no longer its own"
    );

    let click = right_click(&t, &[chopper], tower);
    assert_eq!(click, attach(&[chopper], tower));
    next(&mut t, click);
    assert_eq!(attached(&t, chopper), Some(tower), "attached as its order starts");
    step_until(&mut t, 400, "down on its Tower", |t| down_on(t, chopper, tower));
    t.step();
    assert_eq!(doing(&t, chopper), None, "its order over, in its next turn");
    t.run(100);
    assert!(down_on(&t, chopper, tower), "it stays");
}

/// The gate: a Pick Up Supplies on what is no station (`Reject::NotAStation`); a Drop Off Supplies with nothing on
/// board (`Reject::NothingToDeliver`: "Only available if chopper is carrying supplies"); either tower order on what is
/// no Supply Tower of the player's, the Deployment Center or player 2's Tower (`Reject::NotASupplyTower`), or on one
/// another chopper is attached to ("When a Supply Tower has an attached Supply Chopper, no other Supply Chopper may
/// land on it": `Reject::TowerTaken`), whose right-click is then a Move; any of them for a unit that is no chopper
/// (`Reject::LacksAbility`). A Schedule Deliveries ("Only available if tower has an attached chopper") of a Tower
/// without one, or of what is no Tower, refused (`Reject::NoChopperAttached`), and of a Tower with one on what is no
/// station (`Reject::NotAStation`).
#[test]
fn the_gate_refuses_what_a_chopper_can_t_do() {
    let (mut t, tower, chopper) = field();
    let other = trained(&mut t, tower);
    assert_eq!(
        gate(&t, &pick_up(&[chopper], tower)),
        Err(Reject::NotAStation(id(tower)))
    );
    assert_eq!(gate(&t, &drop_off(&[chopper], tower)), Err(Reject::NothingToDeliver));
    for not_mine in [DC, THEIR_TOWER] {
        let refused = Err(Reject::NotASupplyTower(id(not_mine)));
        assert_eq!(gate(&t, &attach(&[other], not_mine)), refused);
    }
    assert_eq!(gate(&t, &attach(&[other], tower)), Err(Reject::TowerTaken(id(tower))));
    assert_eq!(gate(&t, &attach(&[chopper], tower)), Ok(()), "its own");
    assert_eq!(
        right_click(&t, &[other], tower),
        order(&[other], OrderKind::Move(Target::Object(id(tower))))
    );
    for command in [pick_up(&[PEACEKEEPER], S), attach(&[PEACEKEEPER], tower)] {
        assert_eq!(gate(&t, &command), Err(Reject::LacksAbility), "{command:?}");
    }
    assert_eq!(gate(&t, &schedule(&[tower], S)), Ok(()));
    assert_eq!(gate(&t, &schedule(&[tower], DC)), Err(Reject::NotAStation(id(DC))));
    assert_eq!(gate(&t, &schedule(&[chopper], S)), Err(Reject::NoChopperAttached));
    next(&mut t, pick_up(&[chopper], S));
    assert_eq!(
        gate(&t, &schedule(&[tower], S)),
        Err(Reject::NoChopperAttached),
        "detached"
    );
}
