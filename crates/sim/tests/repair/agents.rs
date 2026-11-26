//! The Agents' repairs (`factions/syndicate/units/agent.md > Repair`: "Agents can **repair** own **Tunnels** and
//! **Laser Turrets** [...] **Laser Turrets cost Space Crystals to repair; Tunnels are repaired for free**"): the
//! Tunnel's rate by its construction at every tier (design change 0328), and the turret's by its build, lowered or
//! raised, but never once it is neutral (design change 0360).

use sc_sim::popup::PopUp;

use super::*;

/// What an Agent at work on a Laser Turret restores in a tick: its 300 HP in half its build's 320 frames (the
/// Tunnel's ExpandMenu's, design change 0359: 100 Space Crystals and 320 frames).
const TURRET_RATE: f32 = 1.875;

/// The turret's pop-up state.
fn popup(sim: &Sim) -> PopUp {
    sim.object(id(TURRET))
        .expect("on the map")
        .popup
        .expect("a pop-up turret")
}

/// Player 1's Space Crystals and Supplies.
fn bank(sim: &Sim) -> (u32, u32) {
    let player = sim.player(P1).expect("in the game");
    (player.crystals, player.supplies)
}

/// Design change 0328 (`factions/syndicate/units/agent.md > Repair`): "Tunnels are repaired for free"; "a Tunnel's
/// rate uses its construction time, 480 frames, at every Tier". At Tier 1, 600 HP, an Agent at work restores 2.5 HP a
/// tick, half the Tunnel's HP in 120 ticks, neither bank moving, and stops at full HP; at Tier 2, 800 HP, a third more:
/// 800 / 240 a tick.
#[test]
fn an_agent_repairs_its_tunnel_for_free_at_its_constructions_rate_at_every_tier() {
    let mut sim = damaged(syndicate().sim(), T1, 1.0);
    step(&mut sim, vec![repair(&[AGENT], T1)]);
    until_gaining(&mut sim, T1, 600.0 / 240.0, |_| {});
    let mut sim = damaged(&sim, T1, 300.0);
    let free = bank(&sim);
    assert_eq!(restored_to(&mut sim, T1, 600.0, 2.5, |_| {}), 120);
    assert_eq!(bank(&sim), free, "nothing paid");
    assert_eq!(current(&sim, AGENT), None, "it stops at full HP");

    step(
        &mut sim,
        vec![Command::Debug(DebugCommand::SetTier { id: id(T1), tier: 2 })],
    );
    let mut sim = damaged(&sim, T1, 400.0);
    step(&mut sim, vec![repair(&[AGENT], T1)]);
    until_gaining(&mut sim, T1, 800.0 / 240.0, |_| {});
    let before = hp(&sim, T1);
    run(&mut sim, 30);
    assert!(
        about(hp(&sim, T1) - before, 100.0),
        "30 ticks of 800 / 240: {}",
        hp(&sim, T1)
    );
    assert_eq!(bank(&sim), free, "nothing paid");
}

/// Design change 0360's verification: "An Agent repairs a Laser Turret from 150 to 300 in 80 frames for 25 SC, raised
/// or lowered; the repair doesn't raise it." Half its build's 100 Space Crystals and 320 frames from 0 to full, a
/// crystal every 6 HP. Lowered, with no enemy about: it stays down all along, and the Agent stops at full HP.
#[test]
fn an_agent_repairs_a_lowered_turret_for_crystals_and_it_stays_down() {
    let lowered = |sim: &Sim| assert!(matches!(popup(sim), PopUp::Lowered { .. }), "{:?}", popup(sim));
    let mut sim = damaged(syndicate().sim(), TURRET, 1.0);
    step(&mut sim, vec![repair(&[AGENT], TURRET)]);
    until_gaining(&mut sim, TURRET, TURRET_RATE, lowered);
    let mut sim = damaged(&sim, TURRET, 150.0);
    let (crystals, _) = bank(&sim);
    assert_eq!(restored_to(&mut sim, TURRET, 300.0, TURRET_RATE, lowered), 80);
    assert_eq!(crystals - bank(&sim).0, 25);
    assert_eq!(current(&sim, AGENT), None, "it stops at full HP");
    run(&mut sim, 16);
    lowered(&sim);
}

/// The same raised: player 2's Guard, on Hold 4 grid units east of the turret, within the turret's Range of 5 and
/// beyond its own of 3, makes it rise at it and fire (`tests/popup`'s cycle: up 8 frames after, down 32 frames after
/// its last target). The Agent at work on it from before, the turret up, from 150 to 300 HP it takes 80 frames and 25
/// crystals all the same, the turret up all along.
#[test]
fn an_agent_repairs_a_raised_turret_all_the_same() {
    let mut t = syndicate();
    let guard_at = Vec2::new(10.0 * 64.0 + 4.0 * 64.0 + 18.0, TURRET_AT.center().y);
    t.debug_at(
        2,
        DebugCommand::Spawn {
            kind: ObjectKind::Guard,
            owner: Some(P2),
            at: SpawnAt::Point(guard_at),
            orientation: Default::default(),
        },
    );
    t.run(1);
    let guard = t.sim().objects().last().expect("spawned").id;
    let hold = Command::Order {
        units: vec![guard],
        kind: OrderKind::Hold,
        queue: false,
    };
    t.at(3, P2, hold);
    t.run(1);
    let mut sim = damaged(t.sim(), TURRET, 1.0);
    step(&mut sim, vec![repair(&[AGENT], TURRET)]);
    let mut raised = false;
    until_gaining(&mut sim, TURRET, TURRET_RATE, |_| {});
    for _ in 0..WALK {
        if matches!(popup(&sim), PopUp::Raised { .. }) {
            raised = true;
            break;
        }
        run(&mut sim, 1);
    }
    assert!(raised, "it rose at the Guard");
    let up = |sim: &Sim| assert!(matches!(popup(sim), PopUp::Raised { .. }), "{:?}", popup(sim));
    let mut sim = damaged(&sim, TURRET, 150.0);
    let (crystals, _) = bank(&sim);
    assert_eq!(restored_to(&mut sim, TURRET, 300.0, TURRET_RATE, up), 80);
    assert_eq!(crystals - bank(&sim).0, 25);
}

/// Design change 0360: "a neutral turret can't be repaired". The Agent at work on the turret, at 150 HP, when its
/// Tunnel is lost: the turret is nobody's, and the Repair ends; it decays unrepaired, and the gate refuses another.
#[test]
fn a_turret_whose_tunnel_is_lost_is_repaired_no_more() {
    let mut sim = damaged(syndicate().sim(), TURRET, 1.0);
    step(&mut sim, vec![repair(&[AGENT], TURRET)]);
    until_gaining(&mut sim, TURRET, TURRET_RATE, |_| {});
    let mut sim = damaged(&sim, TURRET, 150.0);
    step(&mut sim, vec![Command::Debug(DebugCommand::Remove { id: id(T1) })]);
    run(&mut sim, 1);
    assert_eq!(sim.object(id(TURRET)).expect("on the map").owner, None, "neutral");
    assert_eq!(current(&sim, AGENT), None, "the Repair ended");
    let before = hp(&sim, TURRET);
    run(&mut sim, 16);
    assert!(hp(&sim, TURRET) < before, "it decays, unrepaired");
    assert_eq!(
        gate(&sim, repair(&[AGENT], TURRET)),
        Err(Reject::CantRepair(id(TURRET)))
    );
    assert!(sim.object(id(OTHER_TUNNEL)).is_some(), "player 1 is in the game still");
}
