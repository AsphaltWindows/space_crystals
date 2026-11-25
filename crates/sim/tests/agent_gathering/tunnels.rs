//! Which Tunnel an Agent delivers to (`factions/syndicate/units/agent.md > Gathering rules > Which Tunnel`, design
//! change 0255): "the nearest own operational Tunnel (straight line to its crystal hatch), regardless of how many
//! other Agents already use it; no load balancing. It is chosen when the Gather's first return starts and kept for
//! that Gather (a new Gather picks again; a lost Tunnel means picking the nearest again)."

use super::*;

/// Two Tunnels facing each other across 9 tiles, each with its crystal hatch towards the other: Tunnel 1 unturned at
/// tiles (6, 6) to (9, 9), hatch (9, 8); Tunnel 4 flipped left and right at (19, 6) to (22, 9), hatch (19, 8). Patch
/// 2 at (11, 8) holds `first` Space Crystals, patch 3 at (16, 8) plenty. Agent 5 starts by patch 2.
fn two_tunnels(first: u32) -> TestSim {
    let scenario = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(6, 6))
        .spawn_with(patch_of(first), tile(11, 8))
        .spawn_with(patch_of(1_000_000), tile(16, 8))
        .spawn(TUNNEL, Some(P1), tile(19, 6))
        .spawn(AGENT, Some(P1), centre(11, 10))
        .build();
    TestSim::new(turned(scenario, 3, Rotation::R0, true))
}

/// Slice D's acceptance: "Each Agent keeps its Tunnel for the whole Gather, even when it ends up nearer the other
/// one". The Agent's first load empties patch 2, by Tunnel 1; the Gather goes on at patch 3 (the Drone's rule, `>
/// A patch that runs out`), by Tunnel 4, and still delivers to Tunnel 1. A new Gather picks again: Tunnel 4.
#[test]
fn an_agent_keeps_its_tunnel_for_the_gather_and_a_new_gather_picks_again() {
    let mut t = two_tunnels(LOAD);
    gather(&mut t, 1, &[5], 2);
    run_to(&mut t, 400, "the first load is in", |t| deliveries(t).len() == 1);
    assert!(t.sim().object(id(2)).is_none(), "the patch ran out");
    assert_eq!(
        order(&t, 5),
        Some(OrderKind::Gather(id(3))),
        "the Gather goes on at the next patch"
    );
    run_to(&mut t, 1500, "three more loads", |t| deliveries(t).len() == 4);
    let at: Vec<u32> = deliveries(&t).iter().map(|d| d.2).collect();
    assert_eq!(at, [1, 1, 1, 1], "its Tunnel, though the other is nearer now");
    assert_eq!(object(&t, 5).worker.unwrap().drop_off, Some(id(1)));
    // The same patch, ordered anew.
    let now = t.sim().tick();
    gather(&mut t, now + 1, &[5], 3);
    t.step();
    assert_eq!(object(&t, 5).worker.unwrap().drop_off, None, "a new Gather picks again");
    run_to(&mut t, 1000, "two more loads", |t| deliveries(t).len() == 6);
    let at: Vec<u32> = deliveries(&t)[4..].iter().map(|d| d.2).collect();
    assert_eq!(at, [4, 4], "the nearest as its first return starts");
}

/// "Straight line to its crystal hatch": Tunnel 1's hatch faces the Agent; Tunnel 2 is nearer centre to centre,
/// but its hatch is on its far side. The Agent delivers to Tunnel 1.
#[test]
fn the_nearest_tunnel_is_the_one_with_the_nearest_hatch() {
    // Both unturned, Side B to the east: Tunnel 1 at (6, 6), Tunnel 2 at (14, 6) with the patch against its west
    // wall.
    let mut t = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(6, 6))
        .spawn(TUNNEL, Some(P1), tile(14, 6))
        .spawn_with(patch_of(1_000_000), tile(13, 8))
        .spawn(AGENT, Some(P1), centre(11, 10))
        .start();
    gather(&mut t, 1, &[4], 3);
    run_to(&mut t, 200, "it has its load", |t| load(t, 4) == LOAD);
    t.step();
    let pos = t.pos(4);
    let centre_of = |n: u32| object(&t, n).pos;
    assert!(
        pos.distance(centre_of(2)) < pos.distance(centre_of(1)),
        "Tunnel 2 is nearer by its centre"
    );
    assert_eq!(
        object(&t, 4).worker.unwrap().drop_off,
        Some(id(1)),
        "Tunnel 1 by its hatch"
    );
    run_to(&mut t, 300, "it delivers", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 1);
}

/// "A lost Tunnel means picking the nearest again" (the Drone's rule, design change 0170: at once for an Agent on
/// its way there); with none left the Agent idles holding its load (0135).
#[test]
fn a_lost_tunnel_means_the_nearest_again() {
    let mut t = two_tunnels(1_000_000);
    gather(&mut t, 1, &[5], 2);
    run_to(&mut t, 200, "it has its load", |t| load(t, 5) == LOAD);
    t.run(2);
    assert_eq!(object(&t, 5).worker.unwrap().drop_off, Some(id(1)));
    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::Remove { id: id(1) });
    t.step();
    assert_eq!(object(&t, 5).worker.unwrap().drop_off, Some(id(4)), "at once");
    run_to(&mut t, 400, "it delivers", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 4);
    // An Agent in the line, or unloading, loses its Tunnel too.
    run_to(&mut t, 600, "it unloads again", |t| is_unloading(t, 5));
    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::Remove { id: id(4) });
    t.run(3);
    assert_eq!((order(&t, 5), load(&t, 5)), (None, LOAD), "it idles holding its load");
    assert_eq!(deliveries(&t).len(), 1);
}
