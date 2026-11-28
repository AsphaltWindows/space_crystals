//! Its passengers die with it (`factions/syndicate/units/subterranean_transport.md > Destroyed with passengers`: "If
//! the transport is destroyed (above ground or underground), all passengers are lost"; `Sim::passengers_die`,
//! `Sim::remove_held`): on the map, each with a `SimEvent::Died` where it was; held in the network, stranded, with it.
//! Either way their Tunnel Space is free.

use super::*;

/// The plan's acceptance: "with 3 aboard, all 3 die and their Tunnel Space is freed". Three Guards aboard a Transport,
/// and six of player 2's Peacekeepers given an Attack on it: in the tick it dies, each Guard aboard dies with it, a
/// `SimEvent::Died` each, where the Transport was, after the Transport's own. None of them is in the game after, and
/// player 1's Tunnel Space in use, the Transport's 3 and the Guards' 1 each until then, is none.
#[test]
fn with_three_aboard_all_three_die_with_it_and_their_tunnel_space_is_free() {
    let mut t = transport_and_guards(3);
    let transport = id(1);
    let guards: Vec<ObjectId> = (2..5).map(id).collect();
    board(&mut t, &guards, transport);
    assert_eq!(space_used(&t, P1), TRANSPORT_SPACE + 3 * GUARD_SPACE);
    let first = next_id(&t);
    let pks: Vec<ObjectId> = (first..first + 6).map(id).collect();
    let now = t.sim().tick();
    for k in 0..6 {
        let at = Vec2::new(800.0, 450.0 + 60.0 * k as f32);
        t.debug_at(now + 1, spawn(ObjectKind::Peacekeeper, P2, SpawnAt::Point(at)));
    }
    t.at(now + 2, P2, order(&pks, OrderKind::Attack(transport)));
    step_until(&mut t, 900, "the Transport dies", |t| {
        t.sim().object(transport).is_none()
    });
    let fell = t.sim().tick();
    let died = deaths(&t);
    let [(when, first_died, at), passengers @ ..] = &died[..] else {
        panic!("deaths: {died:?}");
    };
    assert_eq!((*when, *first_died), (fell, transport), "the Transport first");
    let lost: Vec<ObjectId> = passengers.iter().map(|&(_, unit, _)| unit).collect();
    assert_eq!(lost, guards, "each passenger, in id order");
    for &(when, unit, pos) in passengers {
        assert_eq!((when, pos), (fell, *at), "{unit:?} dies with it, where it was");
        assert!(
            t.sim().held_unit(unit).is_none() && t.sim().object(unit).is_none(),
            "{unit:?} is out of the game"
        );
    }
    assert_eq!(space_used(&t, P1), 0, "their Tunnel Space is free");
    assert!(t.sim().held().next().is_none());
}

/// "above ground or underground": a loaded Transport held in the network when the last Tier 2 Tunnel falls, a Tier 1
/// Tunnel standing, is stranded (`factions/syndicate/tunnels.md > Losing Tunnels > Stranded units`: "no remaining
/// Tunnel's tier is sufficient for its unit base category"), and its passengers leave the game with it, in that tick,
/// a `SimEvent::Removed` each, though a Guard could leave by a Tier 1 Tunnel; a Guard in the network beside it stays.
/// Player 1's Tunnel Space in use is then the lone Guard's.
#[test]
fn stranded_in_the_network_its_passengers_go_with_it() {
    let (mut t, transport, guards) = network(&[(TUNNEL, TilePos::new(24, 10))], 3, 2);
    let (aboard_it, alone) = (&guards[..2], guards[2]);
    board(&mut t, aboard_it, transport);
    drive_in(&mut t, transport, T1);
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[alone], OrderKind::Enter(T1)));
    step_until(&mut t, 300, "the Guard goes in", |t| inside(t).contains(&alone));
    assert_eq!(inside(&t), [transport, alone]);
    assert_eq!(space_used(&t, P1), TRANSPORT_SPACE + 3 * GUARD_SPACE);
    let fell = t.sim().tick() + 1;
    t.debug_at(fell, DebugCommand::Remove { id: T1 });
    t.run_until(fell);
    let removed: Vec<ObjectId> = (t.events().iter())
        .filter(|&&(tick, _)| tick == fell)
        .filter_map(|(_, e)| match *e {
            SimEvent::Removed(id) if id != T1 => Some(id),
            _ => None,
        })
        .collect();
    let mut gone = vec![transport];
    gone.extend(aboard_it);
    assert_eq!(removed, gone, "the Transport and its passengers");
    for &unit in &gone {
        assert!(t.sim().held_unit(unit).is_none(), "{unit:?} is out of the game");
    }
    assert_eq!(inside(&t), [alone], "the Guard beside it stays");
    assert_eq!(space_used(&t, P1), GUARD_SPACE);
}
