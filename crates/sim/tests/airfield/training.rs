//! The Airfield's Hornet (`factions/gdo/structures/airfield.md > Produces (so far)`): "Hornet: 150 Space Crystals,
//! 320 frames", and its 2 Supplies (design changes 0435 and 0441: "Paid with the Space Crystals at the moment of
//! queueing (the order needs enough of both), and refunded with them on cancel"). A new Hornet appears over the
//! Airfield (D34, slice M6-GH1's placeholder: slice M6-GH2 lands it on a free pad), assigned to it.

use sc_sim::math;

use super::*;

/// "**Q** Hornet" with 1 Supply: the gate refuses it (`Reject::InsufficientSupplies`, the client's greyed button and
/// its "Not enough Supplies"), and the bank keeps everything. With 2 the Airfield takes them with the 150 Space Crystals
/// as it queues the Hornet; a cancel (X) gives both back.
#[test]
fn the_hornet_costs_150_crystals_and_2_supplies() {
    let mut t = field_with(1, |builder| builder);
    assert_eq!(gate(t.sim(), P1, &train(A)), Err(Reject::InsufficientSupplies));
    next(&mut t, P1, train(A));
    assert_eq!(queued(t.sim(), A), []);
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK, 1), "nothing paid");

    let mut t = field_with(SUPPLIES, |builder| builder);
    assert_eq!(gate(t.sim(), P1, &train(A)), Ok(()));
    next(&mut t, P1, train(A));
    assert_eq!(queued(t.sim(), A), [HORNET]);
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK - COST, 0));
    assert_eq!(
        gate(t.sim(), P1, &train(B)),
        Err(Reject::InsufficientSupplies),
        "the crystals suffice, the Supplies are spent"
    );
    next(&mut t, P1, cancel(A));
    assert_eq!(queued(t.sim(), A), []);
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK, SUPPLIES), "both back");
}

/// Steps until a unit comes out of `producer`, at most two of its trainings on: the tick and the unit.
fn until_produced(t: &mut TestSim, producer: u32) -> (Tick, u32) {
    let before = produced(t).len();
    let until = t.sim().tick() + 2 * FRAMES;
    loop {
        if let Some(&(tick, by, unit)) = produced(t).get(before) {
            assert_eq!(by, id(producer));
            return (tick, unit.0);
        }
        assert!(t.sim().tick() < until, "nothing out of #{producer} by tick {until}");
        t.step();
    }
}

/// The Hornet comes out 320 frames after its Q, over the centre of its Airfield (D34), assigned to the Airfield that
/// built it (`> Assignment`: "An aircraft is assigned to the Airfield that built it") with its 5 missiles, and hovers
/// there without a rally point (`ExitRule::Pad`). The next, with a rally point set on the ground, appears facing it
/// and flies there: its default order on the point, a Move (design change 0136).
#[test]
fn a_new_hornet_appears_over_its_airfield_assigned_to_it() {
    let mut t = field(|builder| builder);
    next(&mut t, P1, train(A));
    let queued_at = t.sim().tick();
    let (tick, hornet) = until_produced(&mut t, A);
    assert_eq!(tick, queued_at + FRAMES - 1, "320 frames");
    let o = object(t.sim(), hornet);
    assert_eq!(
        (o.kind, o.owner, o.pos),
        (HORNET, Some(P1), A_AT),
        "over the Airfield's centre"
    );
    assert_eq!(
        aircraft(t.sim(), hornet),
        AircraftState {
            ammo: AMMO,
            base: Some(id(A))
        }
    );
    t.run(40);
    assert_eq!(t.pos(hornet), A_AT, "it hovers there");
    assert_eq!(t.mover(hornet).velocity, Vec2::ZERO);

    let rally = A_AT + Vec2::new(0.0, 640.0);
    let set_rally = Command::Production {
        objects: vec![id(A)],
        command: ProductionCommand::SetRally(Target::Point(rally)),
    };
    next(&mut t, P1, set_rally);
    next(&mut t, P1, train(A));
    let (_, second) = until_produced(&mut t, A);
    let o = object(t.sim(), second);
    assert_eq!(o.pos, A_AT);
    let south = math::vector_angle(rally - A_AT);
    assert!(
        math::wrap_angle(o.facing - south).abs() < 1e-4,
        "facing its rally point: {}",
        o.facing
    );
    assert_eq!(base(t.sim(), second), Some(A));
    assert_eq!(doing(t.sim(), second), Some(OrderKind::Move(Target::Point(rally))));
    step_until(&mut t, 400, "at the rally point", |sim| {
        let o = sim.object(id(second)).expect("on the map");
        o.pos.distance(rally) < 2.0 && o.mover.as_ref().is_some_and(|m| m.velocity == Vec2::ZERO)
    });
    assert_eq!(base(t.sim(), second), Some(A), "still assigned to its Airfield");
}
