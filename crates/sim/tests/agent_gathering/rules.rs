//! The Drone's job rules on the Agent (`factions/syndicate/units/agent.md > Gathering rules`, design change 0255:
//! "The Drone's job rules apply to Agents (Last load of a patch, Interrupted mining, Other orders keep the load,
//! Gather while carrying a load, Drop Off order, When a Gather ends), with two exceptions: Agents share patches (no
//! occupancy rule), and there is no Set Drop-off command"), and `> Right-Click Resolution`.

use super::*;

const P2: PlayerId = PlayerId(2);

fn command(command: Command) -> PlayerCommand {
    PlayerCommand { player: P1, command }
}

fn order_of(units: &[u32], kind: OrderKind) -> PlayerCommand {
    let units = units.iter().map(|&n| id(n)).collect();
    command(Command::Order {
        units,
        kind,
        queue: false,
    })
}

/// `> Space Crystals > Patch sharing` (slice D's acceptance: "Four Agents mine one patch together"): "any number of
/// Agents may mine the same Space Crystals patch at the same time". None of them waits or looks round.
#[test]
fn four_agents_mine_one_patch_together() {
    // Tunnel 1, patch 2 at tile (13, 8), and an Agent a tile off each of its faces: ids 3 to 6.
    let mut t = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(6, 6))
        .spawn_with(patch_of(1_000_000), tile(13, 8))
        .spawn(AGENT, Some(P1), centre(13, 6))
        .spawn(AGENT, Some(P1), centre(15, 8))
        .spawn(AGENT, Some(P1), centre(13, 10))
        .spawn(AGENT, Some(P1), centre(11, 8))
        .start();
    let agents = agents(4);
    gather(&mut t, 1, &agents, 2);
    run_to(&mut t, 40, "all four mine at once", |t| {
        agents.iter().all(|&n| is_mining(t, n))
    });
    for _ in 0..MINING {
        t.step();
        for &n in &agents {
            assert!(
                !matches!(phase(&t, n), Some(GatherPhase::Waiting { .. })),
                "Agent {n} waits"
            );
            assert_eq!(order(&t, n), Some(OrderKind::Gather(id(2))));
        }
    }
    assert_eq!(remaining(&t, 2), 1_000_000 - 4 * LOAD);
    // And a crowd on one face: four Agents from one side all get to mine, none waits for good.
    let mut t = field(4, 3).start();
    gather(&mut t, 1, &agents, 2);
    t.run_until(1200);
    let delivered = deliveries(&t);
    assert!(
        agents.iter().all(|&n| delivered.iter().any(|d| d.1 == n)),
        "{delivered:?}"
    );
}

/// Design change 0255 point 5: "Agents of any player may share patches"; and the Drones' occupancy is the Drones'
/// alone: an Agent at a patch neither holds it against a Drone nor waits for one.
#[test]
fn an_agent_and_a_drone_mine_one_patch_together() {
    // Tunnel 1, patch 2, Agent 3; player 2's Egress Node 4 and Drone 5, north of the patch.
    let mut t = field(1, 3)
        .faction(P2, Faction::Colonists)
        .spawn(ObjectKind::EgressNode, Some(P2), tile(13, 3))
        .spawn(ObjectKind::Drone, Some(P2), centre(13, 6))
        .start();
    gather(&mut t, 1, &[3], 2);
    run_to(&mut t, 200, "the Agent mines", |t| is_mining(t, 3));
    let now = t.sim().tick();
    t.order_at(now + 1, P2, &[id(5)], OrderKind::Gather(id(2)), false);
    run_to(&mut t, 60, "the Drone mines beside it", |t| {
        is_mining(t, 5) && is_mining(t, 3)
    });
    // And the other way round: the Drone mines on, the Agent comes back and mines.
    run_to(&mut t, 400, "the Agent delivers", |t| !deliveries(t).is_empty());
    run_to(&mut t, 400, "both mine again", |t| is_mining(t, 5) && is_mining(t, 3));
}

/// `systems/mechanics/gathering.md > Other orders keep the load` (0129), `systems/mechanics/gathering.md > Drop Off
/// order` (0127; slice D's acceptance: "A right-click on an own Tunnel while carrying delivers at Side B, then the
/// order completes"), `> Right-Click Resolution`: "Space Crystals patch → Gather crystals", "Own Tunnel (carrying
/// resources) → Drop off resources".
#[test]
fn a_load_is_kept_through_another_order_and_a_drop_off_order_delivers_it() {
    let mut t = field(1, 3).start();
    let start = bank(&t);
    let resolve = |t: &TestSim, target: u32| t.sim().default_order(id(3), Target::Object(id(target)));
    assert_eq!(resolve(&t, 2), Some(OrderKind::Gather(id(2))));
    assert_ne!(resolve(&t, 1), Some(OrderKind::DropOff(id(1))), "nothing to drop off");
    let empty = order_of(&[3], OrderKind::DropOff(id(1)));
    assert_eq!(
        t.sim().validate(&empty),
        Err(Reject::NothingToDeliver),
        "design change 0140"
    );
    gather(&mut t, 1, &[3], 2);
    run_to(&mut t, 200, "it has its load", |t| load(t, 3) == LOAD);
    let now = t.sim().tick();
    let away = OrderKind::Move(Target::Point(TilePos::new(20, 15).center()));
    t.order_at(now + 1, P1, &[id(3)], away, false);
    run_to(&mut t, 300, "it has moved away", |t| order(t, 3).is_none());
    assert_eq!((load(&t, 3), bank(&t)), (LOAD, start), "it keeps its load, idle");
    // The right-click on its Tunnel.
    assert_eq!(resolve(&t, 1), Some(OrderKind::DropOff(id(1))));
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(3)], OrderKind::DropOff(id(1)), false);
    let unloading = run_to(&mut t, 300, "it unloads", |t| is_unloading(t, 3));
    assert!(gap_to_hatch(&t, 3, 1) <= REACH, "at the crystal hatch, Side B");
    t.run(DROP_OFF);
    assert_eq!(deliveries(&t), [(unloading + DROP_OFF - 1, 3, 1, LOAD)]);
    assert_eq!(
        (order(&t, 3), load(&t, 3), bank(&t)),
        (None, 0, start + LOAD),
        "only a delivery"
    );
    assert_eq!(
        object(&t, 3).worker.unwrap().drop_off,
        None,
        "a Drop Off assigns nothing"
    );
}

/// `systems/mechanics/gathering.md > Gather while carrying a load` (0128): the Agent "goes straight to the new
/// pick-up point", skips mining there and returns to deliver; `systems/mechanics/gathering.md > Interrupted mining`
/// (0130): mining cut short collects nothing.
#[test]
fn a_gather_while_carrying_skips_mining_and_interrupted_mining_collects_nothing() {
    // A second patch, 3, south of the first.
    let mut t = field(1, 3).spawn_with(patch_of(1_000_000), tile(13, 12)).start();
    gather(&mut t, 1, &[3], 2);
    run_to(&mut t, 200, "it has its load", |t| load(t, 3) == LOAD);
    let now = t.sim().tick();
    gather(&mut t, now + 1, &[3], 4);
    for _ in 0..600 {
        t.step();
        assert!(!is_mining(&t, 3), "it doesn't mine with a load");
        if !deliveries(&t).is_empty() {
            break;
        }
    }
    assert_eq!(deliveries(&t).len(), 1);
    assert_eq!((remaining(&t, 2), remaining(&t, 4)), (1_000_000 - LOAD, 1_000_000));
    // Then it mines patch 4; ordered away at frame 40 of its 80 it carries nothing.
    let began = run_to(&mut t, 300, "it mines", |t| is_mining(t, 3));
    let away = OrderKind::Move(Target::Point(TilePos::new(20, 15).center()));
    t.order_at(began + 40, P1, &[id(3)], away, false);
    t.run_until(began + 41);
    assert_eq!((load(&t, 3), remaining(&t, 4)), (0, 1_000_000));
}

/// `systems/mechanics/gathering.md > Last load of a patch` (0138) and `systems/mechanics/gathering.md > When a Gather
/// ends` (0123): the last load is what is left, after the full 64 frames; with no patch left in its search radius the
/// Agent delivers it, and the Gather is complete.
#[test]
fn the_last_load_is_what_is_left_and_the_gather_ends_with_no_patch_left() {
    let mut t = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(6, 6))
        .spawn_with(patch_of(LOAD + 20), tile(13, 8))
        .spawn(AGENT, Some(P1), centre(12, 12))
        .start();
    let start = bank(&t);
    gather(&mut t, 1, &[3], 2);
    run_to(&mut t, 2000, "the Gather ends", |t| order(t, 3).is_none());
    let amounts: Vec<u32> = deliveries(&t).iter().map(|d| d.3).collect();
    assert_eq!(amounts, [LOAD, 20]);
    assert_eq!((bank(&t), load(&t, 3)), (start + LOAD + 20, 0));
    assert!(t.sim().object(id(2)).is_none(), "the patch is gone");
}

/// "There is no Set Drop-off command": the gate refuses it for an Agent, and takes a Drop Off on its Tunnel from one
/// that carries a load.
#[test]
fn an_agent_has_no_set_drop_off() {
    let mut t = field(1, 3).start();
    let units = vec![id(3)];
    let ability = Ability::SetDropOff(id(1));
    let set = command(Command::Ability { units, ability });
    assert_eq!(t.sim().validate(&set), Err(Reject::LacksAbility));
    gather(&mut t, 1, &[3], 2);
    run_to(&mut t, 200, "it has its load", |t| load(t, 3) == LOAD);
    assert_eq!(t.sim().validate(&set), Err(Reject::LacksAbility));
    assert_eq!(t.sim().validate(&order_of(&[3], OrderKind::DropOff(id(1)))), Ok(()));
    assert_eq!(
        t.sim().validate(&order_of(&[3], OrderKind::DropOff(id(2)))),
        Err(Reject::NotDropOff(id(2)))
    );
}
