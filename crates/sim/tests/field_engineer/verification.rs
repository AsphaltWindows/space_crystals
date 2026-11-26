//! Design change 0328's verification end to end: "A Field Engineer from a Barracks (with a Career Campus) repairs a
//! damaged Barracks, spending crystals", by the repairers' one mechanism (`sc_sim::repair`, slice M5-N8).

use sc_sim::repair;
use sc_sim::testing::build_order::{BuildOrder, Order};
use sc_sim::{Object, Target};

use super::*;

/// Player 1's Career Campus, if one stands.
fn campus(sim: &Sim) -> Option<&Object> {
    sim.objects().find(|o| o.kind == CAREER_CAMPUS && o.owner == Some(P1))
}

/// One tick of `t`, with `line`'s commands for it.
fn play(t: &mut TestSim, line: &mut BuildOrder) {
    let tick = t.sim().tick() + 1;
    for command in line.commands(t.sim()) {
        t.at(tick, command.player, command.command);
    }
    t.step();
}

/// Unit `unit`'s current order, if any.
fn current(sim: &Sim, unit: ObjectId) -> Option<OrderKind> {
    let orders = sim.object(unit)?.orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Whether `gained` is `rate`, give or take the rounding of an HP in the hundreds.
fn about(gained: f32, rate: f32) -> bool {
    (gained - rate).abs() < 1e-3
}

/// Runs `sim` until the Barracks gains `rate` HP in a tick: every one of its repairers at work on it. Panics after
/// [`LONGEST`] ticks.
fn until_gaining(sim: &mut Sim, rate: f32) {
    for _ in 0..LONGEST {
        let before = hp(sim, BARRACKS);
        step(sim, Vec::new());
        if about(hp(sim, BARRACKS) - before, rate) {
            return;
        }
    }
    panic!("the repairers never all got to work: {}", hp(sim, BARRACKS));
}

/// Runs `sim` until the Barracks is at `max` HP, checking that it gains `rate` HP in every tick, its last up to `max`;
/// returns the ticks it took. Panics after [`LONGEST`] ticks.
fn restored_to(sim: &mut Sim, max: f32, rate: f32) -> u32 {
    for ticks in 1..=LONGEST {
        let before = hp(sim, BARRACKS);
        step(sim, Vec::new());
        let now = hp(sim, BARRACKS);
        assert!(now <= max, "never above its MaxHP: {now}");
        if now == max {
            assert!(now - before <= rate + 1e-3, "its last tick: {before} to {now}");
            return ticks;
        }
        assert!(about(now - before, rate), "tick {ticks}: {before} to {now}");
    }
    panic!("the Barracks never got back to {max}: {}", hp(sim, BARRACKS));
}

/// `units` of `sim` ordered to repair its Barracks, nearly destroyed, and run until all of them are at work on it; then
/// its HP set to `hp`.
fn at_work(sim: &Sim, units: &[ObjectId], hp: f32) -> Sim {
    let mut sim = damaged(sim, BARRACKS, 1.0);
    step(&mut sim, vec![repair(units, BARRACKS)]);
    until_gaining(&mut sim, BARRACKS_RATE * units.len() as f32);
    damaged(&sim, BARRACKS, hp)
}

/// Design change 0328's verification. The Barracks' E is refused while player 1 owns no Career Campus (the client's
/// "Requires a Career Campus"). The Deployment Center constructs one for 150 Space Crystals and places it, complete
/// (`factions/gdo/structures/career_campus.md`: "Power −20": 50 consumed of the 50 the Deployment Center and the
/// Generator make). Then the Barracks takes E: two Field Engineers for 75 each, which take their Unit Control as their
/// production starts, one after the other (`factions/gdo/overview.md > GDOResources > Unit Control`, design change
/// 0143: "Reserved when production starts"), and come out, the first 160 frames after its Train (`ARCHITECTURE.md >
/// Economy > Recipe timing`: released in the recipe's last frame). The Barracks damaged to 150 of its 300 HP, a
/// right-click on it gives a Field Engineer a Repair (`Sim::default_order`, the client's rule), and by N8's terms
/// (`repair::terms`: half of its 160 frames and of its 200 Space Crystals) one Field Engineer repairs it to full in 40
/// frames, 3.75 HP a frame, paying a crystal for every 3 HP, 50 in all, and stops there; the two together in 20
/// frames, for the same 50.
#[test]
fn a_field_engineer_from_a_barracks_with_a_career_campus_repairs_it_for_crystals() {
    let mut t = gdo(|scenario| scenario);
    let refused = Err(Reject::Requires(CAREER_CAMPUS));
    assert_eq!(gate(t.sim(), train()), refused, "E without a Career Campus");
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (50, 30));

    let mut line = BuildOrder::new(P1).track([Order::new(ObjectKind::DeploymentCenter, CAREER_CAMPUS)]);
    for _ in 0..LONGEST {
        if campus(t.sim()).is_some() {
            break;
        }
        play(&mut t, &mut line);
    }
    let o = campus(t.sim()).expect("the Deployment Center placed a Career Campus");
    assert!(o.is_operational(), "complete as it stands");
    assert_eq!(
        line.placed().iter().map(|p| p.kind).collect::<Vec<_>>(),
        [CAREER_CAMPUS]
    );
    assert_eq!(crystals(t.sim()), BANK - CAREER_CAMPUS_COST);
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (50, 50), "it draws 20 Power");

    assert_eq!(gate(t.sim(), train()), Ok(()));
    let control = t.sim().unit_control(P1);
    let trained = t.sim().tick() + 1;
    next(&mut t, train());
    assert_eq!(t.sim().unit_control(P1), control + 1, "the first, in production");
    next(&mut t, train());
    assert_eq!(t.sim().unit_control(P1), control + 1, "the second, queued only");
    let paid = CAREER_CAMPUS_COST + 2 * FIELD_ENGINEER_COST;
    assert_eq!(crystals(t.sim()), BANK - paid);
    let (out, _) = until_produced(&mut t, 1)[0];
    assert_eq!(out, trained + FIELD_ENGINEER_TICKS - 1, "160 frames");
    t.step();
    assert_eq!(
        t.sim().unit_control(P1),
        control + 2,
        "the first out, the second in production"
    );
    let engineers: Vec<ObjectId> = until_produced(&mut t, 2).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(t.sim().unit_control(P1), control + 2);
    for &unit in &engineers {
        let o = t.sim().object(unit).expect("out of the Barracks");
        assert_eq!((o.kind, o.owner), (FIELD_ENGINEER, Some(P1)));
    }

    let sim = damaged(t.sim(), BARRACKS, 150.0);
    let barracks = Target::Object(id(BARRACKS));
    assert_eq!(
        sim.default_order(engineers[0], barracks),
        Some(OrderKind::Repair(id(BARRACKS))),
        "a right-click on the damaged Barracks"
    );
    let terms = repair::terms(ObjectKind::Barracks).expect("the Deployment Center builds it");
    assert_eq!((terms.frames, terms.price), (80.0, Some(100.0)));
    assert_eq!(300.0 / terms.frames, BARRACKS_RATE);

    let mut one = at_work(&sim, &engineers[..1], 150.0);
    let bank = crystals(&one);
    assert_eq!(restored_to(&mut one, 300.0, BARRACKS_RATE), 40);
    assert_eq!(bank - crystals(&one), 50, "a crystal for every 3 HP");
    step(&mut one, Vec::new());
    assert_eq!(current(&one, engineers[0]), None, "it stops at full HP");

    let mut two = at_work(&sim, &engineers, 150.0);
    let bank = crystals(&two);
    assert_eq!(restored_to(&mut two, 300.0, 2.0 * BARRACKS_RATE), 20, "twice as fast");
    assert_eq!(bank - crystals(&two), 50, "for the same price");
}
