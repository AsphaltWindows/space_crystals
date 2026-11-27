//! `factions/gdo/units/drone_operator.md > Drone Stock`: "Starts with **1** drone when trained; holds a **maximum of
//! 1**"; "**Build Drone** [...] once its drone has been used, it purchases/builds a replacement"; "**Not while a drone
//! is out** (user, 2026-11-26; Q0155): Build Drone can start only once the operator's drone has **ended**" (design
//! change 0421); "**Build Drone** (hotkey **F**): 50 Space Crystals, 160 frames (10 s)" (`crate::stock`, plan decision
//! T89).

use super::*;

/// F's whole cycle for operator 1: refused with its stock full (`Reject::StockFull`); after R, refused while the drone
/// is out (`Reject::DroneOut`); once the drone has ended, accepted, paid at the press (50 Space Crystals, the
/// operator's orders kept), and refused again while the purchase runs (`Reject::PurchaseUnderWay`), R finding nothing
/// to launch (`Reject::StockEmpty`). The drone is in stock 160 frames after the press (`SimEvent::StockBought`), the
/// press's own frame the first, and the stock is full again: it never passes 1.
#[test]
fn build_drone_waits_for_the_drone_to_end() {
    let mut t = field(|builder| builder);
    assert_eq!(stock(t.sim(), 1), Stock { count: 1, buying: None });
    assert_eq!(gate(t.sim(), P1, buy(&[1])), Err(Reject::StockFull));
    next(&mut t, P1, launch(&[1]));
    assert_eq!(launches(&t), [(2, id(1), id(2))]);
    assert_eq!(stock(t.sim(), 1), Stock { count: 0, buying: None });
    assert_eq!(gate(t.sim(), P1, buy(&[1])), Err(Reject::DroneOut));

    next(&mut t, P1, Command::Debug(DebugCommand::Remove { id: id(2) }));
    assert_eq!(object(t.sim(), 1).drone, None, "the drone has ended");
    assert_eq!(gate(t.sim(), P1, buy(&[1])), Ok(()));
    next(&mut t, P1, move_to(&[1], from_home(0.0, 400.0)));
    next(&mut t, P1, buy(&[1]));
    let pressed = t.sim().tick();
    assert_eq!(crystals(t.sim()), 1000 - DRONE_PRICE);
    let operator = object(t.sim(), 1);
    assert!(operator.stock.is_some_and(|s| s.count == 0 && s.buying.is_some()));
    let walking = (operator.orders.as_ref())
        .and_then(|o| o.current.as_ref())
        .map(|o| o.order.kind);
    assert!(
        matches!(walking, Some(OrderKind::Move(_))),
        "its orders kept: {walking:?}"
    );
    assert_eq!(gate(t.sim(), P1, buy(&[1])), Err(Reject::PurchaseUnderWay));
    assert_eq!(gate(t.sim(), P1, launch(&[1])), Err(Reject::StockEmpty));

    t.run_until(pressed + DRONE_FRAMES - 2);
    assert_eq!(stock(t.sim(), 1).count, 0, "a frame to go");
    t.step();
    let bought: Vec<Tick> = (t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::StockBought { unit } if *unit == id(1)))
        .map(|(tick, _)| *tick)
        .collect();
    assert_eq!(bought, [pressed + DRONE_FRAMES - 1]);
    assert_eq!(stock(t.sim(), 1), Stock { count: 1, buying: None });
    assert_eq!(crystals(t.sim()), 1000 - DRONE_PRICE, "paid once");
    assert_eq!(gate(t.sim(), P1, buy(&[1])), Err(Reject::StockFull), "never past 1");
    assert_eq!(gate(t.sim(), P1, launch(&[1])), Ok(()));
}

/// F's price through the bank's checked spend (plan decision T7): refused without the 50 Space Crystals
/// (`Reject::InsufficientCrystals`); two operators given F together with 70 pay in id order as long as the bank
/// covers one, so only the first buys.
#[test]
fn build_drone_is_paid_while_the_bank_allows() {
    let mut t = field(|builder| builder.spawn(OPERATOR, Some(P1), SpawnAt::Point(from_home(64.0, 0.0))));
    next(&mut t, P1, launch(&[1, 2]));
    assert_eq!(launches(&t), [(2, id(1), id(3)), (2, id(2), id(4))], "one drone each");
    next(&mut t, P1, Command::Debug(DebugCommand::Remove { id: id(3) }));
    next(&mut t, P1, Command::Debug(DebugCommand::Remove { id: id(4) }));
    bank(&mut t, 5, 40, 0);
    t.run_until(5);
    assert_eq!(gate(t.sim(), P1, buy(&[1, 2])), Err(Reject::InsufficientCrystals));
    bank(&mut t, 6, 70, 0);
    t.run_until(6);
    next(&mut t, P1, buy(&[1, 2]));
    assert_eq!(crystals(t.sim()), 70 - DRONE_PRICE);
    assert!(stock(t.sim(), 1).buying.is_some());
    assert_eq!(
        stock(t.sim(), 2),
        Stock { count: 0, buying: None },
        "the bank couldn't pay"
    );
}
