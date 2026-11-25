//! Attacking a neutral expansion (`factions/syndicate/tunnels.md > Losing Tunnels`, design change 0258 point 2:
//! "Neutral expansions can be attacked **only with explicit Attack**"; `systems/controls/commands.md > Attack`:
//! "Right-click never does this ... Auto-targeting never picks own or neutral objects").

use sc_sim::{Command, DebugCommand, Target, Vec2};

use super::*;

/// The decay's HP after `ticks` ticks of it, from full: what a Hideout nothing hit has.
fn decayed(ticks: u32) -> f32 {
    400.0 - ticks as f32 * 0.25
}

/// Whether anything hit `target` (`SimEvent::Hit`).
fn hit(t: &TestSim, target: ObjectId) -> bool {
    (t.events().iter()).any(|(_, e)| matches!(*e, SimEvent::Hit { target: hit, .. } if hit == target))
}

/// H's Acceptance, "Explicit Attack": player 1's Hideout turns neutral with its Tunnel, while player 1 lives on by its
/// second Tunnel. Player 2's Peacekeeper (debug-spawned, a GDO unit of the Syndicate player's) stands in range of it,
/// auto-targeting on: it never picks the Hideout, which only decays. Its right-click on the Hideout is a Move to it,
/// not an Attack. An explicit Attack passes the gate and hits it.
#[test]
fn only_an_explicit_attack_hits_a_neutral_hideout() {
    let mut t = base().spawn(TUNNEL, Some(P1), tile(30, 8)).start(); // 3
    step(&mut t);
    next(&mut t, P1, remove(T1));
    let fell = t.sim().tick();
    assert_eq!(owner(&t, H1), None);
    let pk = Command::Debug(DebugCommand::Spawn {
        kind: ObjectKind::Peacekeeper,
        owner: Some(P2),
        at: SpawnAt::Point(Vec2::new(11.5 * 64.0, 9.0 * 64.0)),
        orientation: Default::default(),
    });
    next(&mut t, P2, pk); // 4
    let peacekeeper = id(4);
    assert_eq!(object(&t, peacekeeper).kind, ObjectKind::Peacekeeper);
    step(&mut t);
    assert!(t.sim().is_visible(P2, object(&t, H1)));

    // Auto-targeting is on, and the Hideout is in range: the Peacekeeper leaves it alone.
    run_until(&mut t, fell + 64);
    assert!(!hit(&t, H1));
    assert_eq!(hp(&t, H1), decayed(65), "only the decay");
    let engaged = object(&t, peacekeeper).attack.as_ref().and_then(|attack| attack.target);
    assert_eq!(engaged, None, "it engaged nothing");

    // Its right-click moves to it.
    let right_click = t.sim().default_order(peacekeeper, Target::Object(H1));
    assert_eq!(right_click, Some(OrderKind::Move(Target::Object(H1))));

    // An explicit Attack hits it.
    let attack = order(&[4], OrderKind::Attack(H1), false);
    assert_eq!(gate(&t, P2, attack.clone()), Ok(()));
    next(&mut t, P2, attack);
    step_until(&mut t, 64, "the Peacekeeper hits the Hideout", |t| hit(t, H1));
    let ticks = t.sim().tick() - fell + 1;
    assert!(hp(&t, H1) < decayed(ticks), "hit: {} of {}", hp(&t, H1), decayed(ticks));
}
