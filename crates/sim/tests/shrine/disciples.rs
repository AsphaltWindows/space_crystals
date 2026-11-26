//! Energy regeneration, the Shrine's W (`factions/cults/structures/shrine.md > Abilities`: "Energy regeneration (name
//! TBD) | Own Disciples | Increases Disciple energy regeneration (+1/s, placeholder) for the duration | 50 | radius 4
//! grid units | 20 s"; `factions/cults/structures/shrine.md > Placeholder stats`: "affected Disciples' energy
//! regeneration is **+1 per second for 20 seconds**"), beside the Disciple's own (`factions/cults/units/disciple.md > Placeholder stats > Energy`: "MaxEnergy
//! 200; starts at 50; regenerates 0.5 per second") and Shroud's drain (`factions/cults/units/disciple.md > Placeholder
//! stats > Shroud numbers`: "drains **5 energy per second**; its **own regeneration continues while channelling**").

use sc_sim::testing::invariants::Invariants;
use sc_sim::{Sim, TickInputs};

use super::*;

/// What a Disciple's own regeneration adds in a tick (0.5 a second), what W adds to it (1 a second), and Shroud's
/// drain (5 a second), in `Object::energy`'s units.
const OWN: u32 = 2048;
const BONUS: u32 = 4096;
const DRAIN: u32 = 20_480;

/// The plan's acceptance: "W adds 1 energy a second to Disciples for 20 s". A Disciple in the circle regenerates 1.5 a
/// second from the cast's tick for 320 ticks, 30 points in all, then its own 0.5 again; one outside it, player 2's in it
/// and a Recruit in it get nothing.
#[test]
fn w_adds_1_a_second_to_the_disciples_in_the_circle_for_20_s() {
    let (mut t, _) = manned(8);
    let next = |t: &TestSim| t.sim().tick() + 1;
    let disciple = spawn(&mut t, DISCIPLE, P1, Vec2::new(900.0, 640.0));
    let outside = spawn(&mut t, DISCIPLE, P1, Vec2::new(900.0, 1100.0));
    let recruit = spawn(&mut t, RECRUIT, P1, Vec2::new(900.0, 580.0));
    let enemy = spawn(&mut t, DISCIPLE, P2, Vec2::new(900.0, 760.0));
    charge(&mut t, 50);
    let c = next(&t);
    t.at(c, P1, cast(&[S1], W, AT));
    let (before, outside_before) = (energy(&t, disciple), energy(&t, outside));
    run_checked(&mut t, c);
    assert_eq!(casts(&t), [(c, S1, W, AT)]);
    assert_eq!(buffs(&t, disciple), buff(W, c + W_TICKS));
    for unit in [outside, recruit, enemy] {
        assert_eq!(buffs(&t, unit), [], "{unit:?}");
    }
    assert_eq!(energy(&t, disciple) - before, OWN + BONUS, "in the cast's tick");
    run_checked(&mut t, c + W_TICKS - 1);
    assert_eq!(energy(&t, disciple) - before, W_TICKS * (OWN + BONUS));
    assert_eq!(W_TICKS * BONUS, 20 * ONE, "20 points from W, 30 in all");
    assert_eq!(energy(&t, outside) - outside_before, W_TICKS * OWN);
    assert_eq!(buffs(&t, disciple), [], "over");
    let after = energy(&t, disciple);
    step_checked(&mut t);
    assert_eq!(energy(&t, disciple) - after, OWN, "its own again");
}

/// The coordinator's case: a Disciple channelling Shroud under W nets 0.5 + 1 − 5 a second, −14,336 units a tick,
/// 3.5 points a second; one channelling outside the circle nets 0.5 − 5, 4.5 a second.
#[test]
fn a_disciple_channelling_shroud_under_w_nets_0_5_plus_1_minus_5_a_second() {
    let (mut t, _) = manned(8);
    let next = |t: &TestSim| t.sim().tick() + 1;
    let disciple = spawn(&mut t, DISCIPLE, P1, Vec2::new(900.0, 640.0));
    let outside = spawn(&mut t, DISCIPLE, P1, Vec2::new(900.0, 1100.0));
    charge(&mut t, 50);
    let c = next(&t);
    t.at(c, P1, cast(&[S1], W, AT));
    t.order_at(c, P1, &[disciple, outside], OrderKind::Shroud, false);
    let (before, outside_before) = (energy(&t, disciple), energy(&t, outside));
    run_checked(&mut t, c + 15);
    assert_eq!(t.sim().clouds().list().len(), 2, "both channel");
    assert_eq!(before - energy(&t, disciple), 16 * (DRAIN - OWN - BONUS));
    assert_eq!(16 * (DRAIN - OWN - BONUS), 7 * ONE / 2, "3.5 points in a second");
    assert_eq!(outside_before - energy(&t, outside), 16 * (DRAIN - OWN), "4.5 a second");
}

/// The plan's acceptance: "up to MaxEnergy". A Disciple 3,000 units below its 200 (a save edited to it, and its
/// Shrine's energy to 200) gets 6,144 in the tick of W, and stops at 200: no further while W lasts.
#[test]
fn w_stops_at_the_disciples_max_energy() {
    let t = base()
        .spawn(DISCIPLE, Some(P1), SpawnAt::Point(Vec2::new(640.0, 640.0)))
        .start(); // 3
    let mut json = serde_json::to_value(t.sim()).unwrap();
    assert_eq!(json["objects"][2]["id"], 3);
    json["objects"][0]["energy"] = serde_json::json!(200 * ONE);
    json["objects"][2]["energy"] = serde_json::json!(200 * ONE - 3000);
    let mut sim: Sim = serde_json::from_value(json).unwrap();
    let mut rules = Invariants::new(&sim);
    let mut step = |sim: &mut Sim, commands: Vec<PlayerCommand>| {
        let inputs = TickInputs { commands };
        sim.step(&inputs);
        rules.check(sim, &inputs).unwrap();
    };
    let disciple = |sim: &Sim| sim.object(id(3)).and_then(|o| o.energy).unwrap();
    step(&mut sim, Vec::new());
    assert_eq!(disciple(&sim), 200 * ONE - 3000 + OWN, "its own 0.5 a second");
    let w = PlayerCommand {
        player: P1,
        command: cast(&[S1], W, Vec2::new(640.0, 640.0)),
    };
    step(&mut sim, vec![w]);
    assert_eq!(sim.object(id(3)).unwrap().buffs, buff(W, sim.tick() + W_TICKS));
    assert_eq!(disciple(&sim), 200 * ONE, "up to MaxEnergy");
    for _ in 0..10 {
        step(&mut sim, Vec::new());
        assert_eq!(disciple(&sim), 200 * ONE, "no further");
    }
}
