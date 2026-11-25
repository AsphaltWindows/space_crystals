//! Stranded units (`factions/syndicate/tunnels.md > Losing Tunnels > Stranded units`): "whenever a Tunnel is
//! destroyed, any unit inside the Tunnel Network that can no longer exit through any remaining Tunnel (no remaining
//! Tunnel's tier is sufficient for its unit base category) is **destroyed**"; "A Tunnel that is still under
//! construction does not count as an exit". And the free rebuild (`factions/syndicate/tunnels.md > Losing Tunnels >
//! Defeat`: "A Syndicate player who survives this way can rebuild with a surviving Agent: with no Tunnels owned, a new
//! Tunnel costs 0 Supplies").

use sc_sim::{Command, DebugCommand, Vec2};

use super::*;

/// A debug spawn of a `kind` of `owner`'s at `at`.
fn spawn(kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> Command {
    Command::Debug(DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation: Default::default(),
    })
}

/// The design's example: "if the last Tier 2+ Tunnel is destroyed, every vehicle inside the network is destroyed,
/// while infantry survive (they can still exit through Tier 1 Tunnels)". Player 1's Tunnel 1 is at Tier 2, its Tunnel
/// 3 at Tier 1; a vehicle (a Drone of player 1's, as in `tests/tunnel_network`) and a Guard go in at Tunnel 1, which
/// is then destroyed: the Drone goes, in that tick, the Guard stays.
#[test]
fn the_last_tier_2_tunnel_lost_strands_the_vehicles_and_not_the_infantry() {
    let mut t = base()
        .spawn(TUNNEL, Some(P1), tile(30, 8)) // 3
        .spawn(GUARD, Some(P1), point(5, 6)) // 4
        .start();
    t.without_auto_targeting();
    next(&mut t, P1, Command::Debug(DebugCommand::SetTier { id: T1, tier: 2 }));
    next(
        &mut t,
        P1,
        spawn(ObjectKind::Drone, P1, SpawnAt::Point(Vec2::new(420.0, 380.0))),
    ); // 5
    enter(&mut t, P1, &[4, 5], T1);
    assert_eq!(inside(&t, P1), [id(4), id(5)]);

    next(&mut t, P1, remove(T1));
    let fell = t.sim().tick();
    assert_eq!(
        removed(&t, id(5)),
        Some(fell),
        "the vehicle can't leave by a Tier 1 Tunnel"
    );
    assert_eq!(removed(&t, id(4)), None);
    assert_eq!(inside(&t, P1), [id(4)], "the infantry can");
    assert_eq!(owner(&t, H1), None);
}

/// [`base`] with player 1's Agent 3 far south-east and its Guard 4 at [`T1`]'s entry path, and a Barracks of player
/// 1's (debug-spawned, as H's Acceptance asks), far east: a structure that keeps player 1 in the game when its last
/// Tunnel falls.
fn with_barracks() -> TestSim {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(20, 20)) // 3
        .spawn(GUARD, Some(P1), point(5, 6)) // 4
        .start();
    t.without_auto_targeting();
    next(&mut t, P1, spawn(ObjectKind::Barracks, P1, tile(40, 16))); // 5
    assert_eq!(object(&t, id(5)).kind, ObjectKind::Barracks);
    t
}

/// "If the last Tunnel of all is destroyed and the player is not defeated (they still own Laser Turrets, or a custom
/// scenario), every unit inside the network is destroyed", and a Tunnel under construction is no exit: player 1's
/// Agent builds a Tunnel far east, and its Guard is in the network when its only Tunnel in operation falls. The Guard
/// goes in that tick; the Agent, held at its site, stays there and comes into the network when the site completes.
/// Player 1 lives on by its Barracks.
#[test]
fn the_last_tunnel_lost_strands_every_unit_in_the_network_though_a_site_stands() {
    let mut t = with_barracks();
    enter(&mut t, P1, &[4], T1);
    next(&mut t, P1, bank(P1, 0, 1));
    next(&mut t, P1, build(&[3], at(20, 8)));
    let (_, site) = until_started(&mut t, 3);

    next(&mut t, P1, remove(T1));
    let fell = t.sim().tick();
    assert_eq!(removed(&t, id(4)), Some(fell), "a site is no exit");
    assert_eq!(t.sim().held_unit(id(3)).map(|held| held.place), Some(Place::Site(site)));
    assert!(!t.sim().player(P1).expect("in the game").is_defeated());
    step_until(&mut t, 600, "the site completes", |t| completed(t, site).is_some());
    assert_eq!(inside(&t, P1), [id(3)]);
}

/// The free rebuild ("with no Tunnels owned, a new Tunnel costs 0 Supplies", `factions/syndicate/tunnels.md > Cost
/// Scaling > Construction Cost`): player 1, alive by its Barracks, loses its only Tunnel with no Supplies in the bank.
/// Its next Tunnel costs nothing: its surviving Agent's Build is taken, and its site starts.
#[test]
fn with_no_tunnel_left_the_next_tunnel_costs_nothing() {
    let mut t = with_barracks();
    next(&mut t, P1, bank(P1, 0, 0));
    assert_eq!(t.sim().build_price(P1, TUNNEL), Some(1));
    next(&mut t, P1, remove(T1));
    assert!(!t.sim().player(P1).expect("in the game").is_defeated());
    assert_eq!(t.sim().build_price(P1, TUNNEL), Some(0));
    assert_eq!(gate(&t, P1, build(&[3], at(20, 8))), Ok(()));
    next(&mut t, P1, build(&[3], at(20, 8)));
    let (_, site) = until_started(&mut t, 3);
    assert_eq!(object(&t, site).kind, TUNNEL);
    assert_eq!(t.sim().player(P1).expect("in the game").supplies, 0);
}
