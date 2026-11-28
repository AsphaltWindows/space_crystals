//! Assignment (`factions/gdo/structures/airfield.md > Assignment`, `> Production limit`): each aircraft is assigned to
//! an Airfield, which holds at most 4, its production queue counting; a right-click on another Airfield with room
//! reassigns; a destroyed Airfield's aircraft are unassigned, and none is reassigned by itself.

use super::*;

/// Three points east of [`A`], apart, for three Hornets.
fn east_of_a() -> [Vec2; 3] {
    [0.0, 96.0, 192.0].map(|dy| A_AT + Vec2::new(448.0, dy))
}

/// "An Airfield **can't queue another aircraft** if its assignments plus the aircraft already in its production
/// queue would exceed 4": with 3 Hornets assigned and 1 queued, Q is refused (`Reject::AirfieldFull`, the client's
/// "Airfield full") and the bank keeps everything; the other Airfield still trains. Once one of the 3 is gone, Q works.
#[test]
fn an_airfield_holds_four_aircraft_its_queue_counting() {
    let mut t = field(|builder| builder);
    let hornets = based_hornets(&mut t, 7, &east_of_a(), A);
    assert_eq!(gate(t.sim(), P1, &train(A)), Ok(()), "3 of 4");
    next(&mut t, P1, train(A));
    assert_eq!(queued(t.sim(), A), [HORNET]);
    let bank = (crystals(t.sim()), supplies(t.sim()));
    assert_eq!(bank, (BANK - COST, BANK_SUPPLIES - SUPPLIES));
    assert_eq!(gate(t.sim(), P1, &train(A)), Err(Reject::AirfieldFull));
    next(&mut t, P1, train(A));
    assert_eq!(queued(t.sim(), A), [HORNET], "refused");
    assert_eq!((crystals(t.sim()), supplies(t.sim())), bank);
    assert_eq!(gate(t.sim(), P1, &train(B)), Ok(()), "B has room");

    debug(&mut t, DebugCommand::Remove { id: id(hornets[1]) });
    assert_eq!(gate(t.sim(), P1, &train(A)), Ok(()), "a place came free");
    next(&mut t, P1, train(A));
    assert_eq!(queued(t.sim(), A), [HORNET, HORNET]);
    assert_eq!(gate(t.sim(), P1, &train(A)), Err(Reject::AirfieldFull));
}

/// "**Reassigning** an aircraft to another Airfield is possible only if that Airfield has room [...] Input: select the
/// aircraft and **right-click the other Airfield**": its right-click there is a Rebase, which assigns it; on its own
/// Airfield, a Return to Base (`> Returning and rearming`: "Right-clicking its own Airfield does the same"). A
/// Rebase onto a full one, its queue counting, is refused (`Reject::AirfieldFull`), and of a group only as many as it
/// has room for move over, the first ones; the rest keep theirs.
#[test]
fn a_right_click_on_another_airfield_with_room_reassigns() {
    let mut t = field(|builder| builder);
    let [hornet] = based_hornets(&mut t, 7, &[B_AT + Vec2::new(448.0, 0.0)], B)[..] else {
        panic!("one Hornet");
    };
    let click = right_click(t.sim(), &[hornet], Target::Object(id(A)));
    assert_eq!(click, order(&[hornet], OrderKind::Rebase(id(A))));
    next(&mut t, P1, click);
    assert_eq!(base(t.sim(), hornet), Some(A), "reassigned");
    assert_eq!(
        right_click(t.sim(), &[hornet], Target::Object(id(A))),
        order(&[hornet], OrderKind::ReturnToBase),
        "its own Airfield"
    );
    assert_eq!(
        right_click(t.sim(), &[hornet], Target::Object(id(B))),
        order(&[hornet], OrderKind::Rebase(id(B)))
    );

    // A fills up: two more assigned, and one queued.
    based_hornets(&mut t, 8, &east_of_a()[..2], A);
    next(&mut t, P1, train(A));
    assert_eq!(queued(t.sim(), A), [HORNET]);
    let on_b = [0.0, 96.0].map(|dy| B_AT + Vec2::new(448.0, 192.0 + dy));
    let back = based_hornets(&mut t, 10, &on_b, B);
    let rebase = |units: &[u32], to: u32| order(units, OrderKind::Rebase(id(to)));
    assert_eq!(
        gate(t.sim(), P1, &rebase(&back, A)),
        Err(Reject::AirfieldFull),
        "3 assigned, 1 queued"
    );
    next(&mut t, P1, rebase(&back, A));
    assert_eq!(
        (base(t.sim(), back[0]), base(t.sim(), back[1])),
        (Some(B), Some(B)),
        "refused"
    );

    // With one place on A, of the two Hornets on B the first one moves over.
    next(&mut t, P1, cancel(A));
    assert_eq!(queued(t.sim(), A), []);
    assert_eq!(gate(t.sim(), P1, &rebase(&back, A)), Ok(()));
    next(&mut t, P1, rebase(&back, A));
    assert_eq!((base(t.sim(), back[0]), base(t.sim(), back[1])), (Some(A), Some(B)));
    assert_eq!(doing(t.sim(), back[0]), Some(OrderKind::Rebase(id(A))), "it flies over");
    assert_ne!(
        doing(t.sim(), back[1]),
        Some(OrderKind::Rebase(id(A))),
        "the other gets no order"
    );
    assert_eq!(
        gate(t.sim(), P1, &rebase(&back[1..], A)),
        Err(Reject::AirfieldFull),
        "full again"
    );
    assert_eq!(gate(t.sim(), P1, &rebase(&back, A)), Ok(()), "one of them is A's");
}

/// A Rebase needs one of its owner's own Airfields (`Reject::NotAnAirfield`), and aircraft (`Reject::LacksAbility`).
#[test]
fn only_aircraft_rebase_and_only_onto_an_own_airfield() {
    let mut t = field(|builder| {
        builder
            .spawn(ObjectKind::Airfield, Some(P2), tile(30, 3)) // 7
            .spawn(ObjectKind::Peacekeeper, Some(P1), at(1400.0, 600.0)) // 8
    });
    let [hornet] = based_hornets(&mut t, 9, &[A_AT + Vec2::new(448.0, 0.0)], A)[..] else {
        panic!("one Hornet");
    };
    let rebase = |units: &[u32], to: u32| order(units, OrderKind::Rebase(id(to)));
    assert_eq!(
        gate(t.sim(), P1, &rebase(&[hornet], 7)),
        Err(Reject::NotAnAirfield(id(7)))
    );
    assert_eq!(
        gate(t.sim(), P1, &rebase(&[hornet], 1)),
        Err(Reject::NotAnAirfield(id(1)))
    );
    assert_eq!(gate(t.sim(), P1, &rebase(&[8], B)), Err(Reject::LacksAbility));
    assert_eq!(
        gate(t.sim(), P1, &order(&[8], OrderKind::ReturnToBase)),
        Err(Reject::LacksAbility)
    );
    assert_eq!(
        gate(t.sim(), P1, &rebase(&[hornet, 8], B)),
        Ok(()),
        "the aircraft of the group"
    );
}

/// "**Airfield destroyed**: its aircraft become **unassigned** [...] **There is no automatic reassignment**", and
/// "An unassigned aircraft has no base [...] Return to Base is unavailable until it is reassigned": with A gone, its
/// Hornets have no base, though B has room, and F is refused (`Reject::NoBase`) until a right-click on B reassigns one.
/// A Hornet on its way home when its Airfield goes stops, drifting to rest short of where it stood.
#[test]
fn a_destroyed_airfield_leaves_its_aircraft_unassigned() {
    let mut t = field(|builder| builder);
    let hornets = based_hornets(&mut t, 7, &east_of_a(), A);
    let home = order(&hornets[..1], OrderKind::ReturnToBase);
    next(&mut t, P1, home.clone());
    assert_eq!(doing(t.sim(), hornets[0]), Some(OrderKind::ReturnToBase));
    t.run(10);
    assert!(t.mover(hornets[0]).velocity.length() > 1.0, "on its way home");

    debug(&mut t, DebugCommand::Remove { id: id(A) });
    for &n in &hornets {
        let state = aircraft(t.sim(), n);
        assert_eq!((state.ammo, state.base, state.landing), (AMMO, None, None), "#{n}");
    }
    assert_eq!(
        gate(t.sim(), P1, &order(&hornets, OrderKind::ReturnToBase)),
        Err(Reject::NoBase)
    );
    step_until(&mut t, 200, "at rest", |sim| {
        sim.object(id(hornets[0]))
            .and_then(|o| o.mover.as_ref())
            .is_some_and(|m| m.velocity == Vec2::ZERO)
    });
    assert_eq!(doing(t.sim(), hornets[0]), None, "the return ended");
    let rest = t.pos(hornets[0]);
    assert!(rest.distance(A_AT) > 64.0, "it stopped short of where A stood: {rest}");
    t.run(64);
    assert_eq!(t.pos(hornets[0]), rest);
    for &n in &hornets {
        assert_eq!(base(t.sim(), n), None, "no automatic reassignment: #{n}");
    }
    let click = right_click(t.sim(), &hornets[1..2], Target::Object(id(B)));
    assert_eq!(click, order(&hornets[1..2], OrderKind::Rebase(id(B))));
    next(&mut t, P1, click);
    assert_eq!(base(t.sim(), hornets[1]), Some(B));
    assert_eq!(
        gate(t.sim(), P1, &order(&hornets[1..2], OrderKind::ReturnToBase)),
        Ok(())
    );
}
