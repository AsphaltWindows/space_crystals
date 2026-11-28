//! Rearming (`factions/gdo/structures/airfield.md > Returning and rearming`): "Rearming an aircraft takes
//! RearmTimePerAmmo × missing ammo"; "After rearming, the aircraft **stays landed** at the Airfield"; design change
//! 0428 point 4, "**An order during rearming**: the aircraft takes off at once **with the ammo rearmed so far**";
//! design change 0423, "Each frame, these timers advance by the speed ratio (generated ÷ consumed, capped at 100%), as
//! production does", the rearm among them.

use sc_sim::PowerSummary;

use super::*;

/// One Hornet of player 1 landed on [`A`]'s first slot, home (7), in `t`.
fn one_landed(t: &mut TestSim) -> u32 {
    let hornets = landed_hornets(t, 7, &[Vec2::new(864.0, 760.0)], A);
    assert_eq!(slot_of(t.pos(7)), Some(0));
    hornets[0]
}

/// The spot the Hornets fire at, south of [`A`].
const SPOT: Vec2 = Vec2::new(816.0, 900.0);

/// The plan's acceptance, "A Hornet with 2 of 5 missiles rearms in 96 ticks and stays landed. A Move lifts it off in
/// 16 ticks, and then it flies": its 3 missing rounds, one every 32 frames from the tick it is down, then it stays down
/// and idle. A Move lifts it off at once: it rises, holding still, and moves in the take-off's 16th tick, its rise's
/// last (`Mover::rise`).
#[test]
fn two_of_five_rearm_in_96_ticks_then_it_stays_landed_until_a_move_lifts_it_off() {
    let mut t = field(|builder| builder);
    let hornet = one_landed(&mut t);
    next(&mut t, P1, order(&[hornet], OrderKind::AttackGround(SPOT)));
    until_launched(&mut t, hornet, 3, 300);
    next(&mut t, P1, order(&[hornet], OrderKind::ReturnToBase));
    let (_, down_at) = until_down(&mut t, hornet, 300);
    assert_eq!(ammo(t.sim(), hornet), 2);
    let mut rounds = Vec::new();
    while ammo(t.sim(), hornet) < AMMO {
        assert!(t.sim().tick() < down_at + 200, "rearmed");
        let before = ammo(t.sim(), hornet);
        t.step();
        if ammo(t.sim(), hornet) > before {
            rounds.push(t.sim().tick() - down_at + 1);
        }
    }
    assert_eq!(
        rounds,
        [REARM, 2 * REARM, 3 * REARM],
        "a round every 32 ticks, 96 in all"
    );
    assert_eq!(
        doing(t.sim(), hornet),
        Some(OrderKind::ReturnToBase),
        "its flight home ends once rearmed"
    );
    t.step();
    assert!(home(t.sim(), hornet));
    let held = t.pos(hornet);
    t.run(100);
    assert!(home(t.sim(), hornet), "it stays landed");
    assert_eq!(t.pos(hornet), held);

    let to = Vec2::new(816.0, 760.0);
    next(&mut t, P1, order(&[hornet], OrderKind::Move(Target::Point(to))));
    let rose = t.sim().tick();
    assert_eq!(landing_of(t.sim(), hornet), None, "lifted off");
    assert!(t.mover(hornet).rise.is_some(), "rising");
    while t.sim().tick() < rose + LANDING - 1 {
        assert_eq!(t.pos(hornet), held, "it rises, holding still");
        t.step();
    }
    assert_eq!(t.sim().tick(), rose + LANDING - 1, "the take-off's 16th tick");
    assert!(t.pos(hornet) != held, "it flies");
    assert!(t.mover(hornet).rise.is_none());
    step_until(&mut t, 200, "there", |sim| {
        sim.object(id(hornet)).unwrap().pos.distance(to) < 2.0
    });
    assert_eq!(ammo(t.sim(), hornet), AMMO, "full");
}

/// 0428's Verification: "Order a Hornet off 5 s into rearming: it leaves with 2 missiles". Out of ammo, it flies home
/// by itself and lands; 80 ticks (5 s) after it is down, two rounds are in (one every 32) and the third half done. A
/// Move then takes it off with the 2: the round under way is lost, and it rearms no more in the air.
#[test]
fn ordered_off_five_seconds_into_rearming_it_leaves_with_two_missiles() {
    let mut t = field(|builder| builder);
    let hornet = one_landed(&mut t);
    next(&mut t, P1, order(&[hornet], OrderKind::AttackGround(SPOT)));
    until_launched(&mut t, hornet, AMMO as usize, 400);
    step_until(&mut t, 2, "out of ammo, home", |sim| {
        doing(sim, hornet) == Some(OrderKind::ReturnToBase)
    });
    let (_, down_at) = until_down(&mut t, hornet, 300);
    t.run_until(down_at + 79);
    assert_eq!(ammo(t.sim(), hornet), 2);
    let round = landing_of(t.sim(), hornet)
        .and_then(|landing| landing.rearm)
        .expect("rearming");
    assert_eq!(
        (round.ticks_done(), round.total),
        (16, REARM),
        "the third round half done"
    );
    next(
        &mut t,
        P1,
        order(&[hornet], OrderKind::Move(Target::Point(Vec2::new(816.0, 760.0)))),
    );
    assert_eq!(t.sim().tick(), down_at + 80, "5 s into rearming");
    assert_eq!(landing_of(t.sim(), hornet), None, "it takes off at once");
    assert_eq!(ammo(t.sim(), hornet), 2, "with the ammo rearmed so far");
    t.run(3 * REARM);
    assert_eq!(ammo(t.sim(), hornet), 2, "none in the air");
}

/// 0423's Verification: "Run Power to about 59% (e.g. 30 generated, 51 consumed): [...] a Hornet's full rearm takes
/// about 17 s" rather than its 10: at the ratio 38 551 of 65 536 (`PowerSummary::ratio`, rounded up), each tick counts
/// that share of a frame of the round, what a round's last tick has left over going on to the next, so that the 5
/// rounds' 160 frames take 272 ticks, 17.0 s. Player 1's Deployment Center 3 (+30) stands on tile (2, 2), its
/// Extraction Facility 4 (−15) on (2, 7), its plates 5 and 6 (−3 each) on the crystal patches 1 and 2, and its Airfield
/// 7 (−30) on (12, 3), whose centre and slots are [`A`]'s, as `tests/defense_system` runs its Missile Turret at 30 for
/// 51.
#[test]
fn at_59_percent_power_a_full_rearm_takes_about_17_s() {
    let builder = ScenarioBuilder::new(&flat(48, 32))
        .spawn(ObjectKind::CrystalPatch, None, tile(7, 8))
        .spawn(ObjectKind::CrystalPatch, None, tile(10, 8))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(2, 7))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(7, 8))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(10, 8))
        .spawn(ObjectKind::Airfield, Some(P1), tile(12, 3))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(40, 24));
    let mut t = started(builder);
    let airfield = 7;
    assert_eq!(object(t.sim(), airfield).pos, A_AT);
    let summary = PowerSummary {
        generated: 30,
        consumed: 51,
    };
    assert_eq!(t.sim().power(P1), summary, "about 59%");
    let hornet = landed_hornets(&mut t, 9, &[Vec2::new(864.0, 760.0)], airfield)[0];
    next(&mut t, P1, order(&[hornet], OrderKind::AttackGround(SPOT)));
    until_launched(&mut t, hornet, AMMO as usize, 400);
    step_until(&mut t, 2, "out of ammo, home", |sim| {
        doing(sim, hornet) == Some(OrderKind::ReturnToBase)
    });
    let (_, down_at) = until_down(&mut t, hornet, 300);
    assert_eq!(t.sim().power(P1), summary);
    step_until(&mut t, 400, "rearmed", |sim| ammo(sim, hornet) == AMMO);
    let ticks = t.sim().tick() - down_at + 1;
    assert_eq!(ticks, 272, "about 17 s, for 5 × 32 = 160 frames at full Power");
}
