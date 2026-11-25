//! The line at the hatch (`factions/syndicate/units/agent.md > Gathering > Gathering rules`, "Waiting at the hatch",
//! design changes 0255 and 0315: "an Agent waits only once it is within reach of the hatch, while another drops off;
//! the first to arrive within reach delivers next. Until it is within reach it is never 'arrived' [...] It waits as
//! long as it takes; it never gives up"), and where an Agent stands to deliver ("The hatch is 1 grid square long, near
//! the middle of its side. An Agent must be within DropOffReach of the hatch to deliver").

use super::*;
use sc_sim::{MoveStatus, PlayerId};

const P2: PlayerId = PlayerId(2);

/// Design change 0255's Verification: "Six Agents on one Tunnel queue at the hatch and all eventually deliver; none
/// idles with a load." With the patch 3 grid units from the hatch, for ten minutes: every Agent goes on delivering,
/// in the order they come within reach ([`watch`]), and an Agent within reach never stands at a free hatch beyond the
/// tick the one before it finishes in. The Tunnel takes in 11.33 Space Crystals per second over the last five minutes
/// (with 64 frames of mining, design change 0294; 11.5 or more with 80): within the check's 15% of saturated.
#[test]
fn six_agents_queue_at_one_hatch_and_all_deliver() {
    const TICKS: Tick = 10 * 60 * 16;
    let mut t = field(6, 3).start();
    let agents = agents(6);
    gather(&mut t, 1, &agents, 2);
    let watched = watch(&mut t, TICKS, &agents, &[1]);
    assert_ceiling(&t, 1);
    let delivered = deliveries(&t);
    for &n in &agents {
        let mine: Vec<Tick> = delivered.iter().filter(|d| d.1 == n).map(|d| d.0).collect();
        assert!(mine.len() >= 18, "Agent {n} delivered {} loads", mine.len());
        // None is left out for long: each delivers in every stretch of two minutes.
        let longest = mine.windows(2).map(|pair| pair[1] - pair[0]).max().unwrap_or(TICKS);
        assert!(
            longest <= 2 * 60 * 16,
            "Agent {n} went {longest} ticks without a delivery"
        );
        assert_eq!(order(&t, n), Some(OrderKind::Gather(id(2))), "none idles");
    }
    assert!(watched.longest_wait > DROP_OFF, "they did wait in line: {watched:?}");
    assert!(
        watched.longest_idle_wait <= 1,
        "a free hatch is taken at once: {watched:?}"
    );
    let income = income(&t, TICKS / 2, TICKS);
    assert!(income >= 12.5 * 0.85, "the Tunnel is all but saturated: {income}");
}

/// The Director's acceptance for 0255: "an Agent waiting in line at the hatch keeps waiting, and delivers when its
/// turn comes". Three Agents with loads come to the hatch together: the first to come within reach unloads, the
/// others wait their 64 and 128 frames and more without moving, and unload in the order they came.
#[test]
fn an_agent_in_line_keeps_waiting_and_delivers_when_its_turn_comes() {
    // Tunnel 1, a patch a tile off each end of the stretch of wall within reach of the hatch square, and an Agent
    // at each patch and one between: ids 2 and 3, and 4 to 6. All three have their loads within a few ticks of
    // each other and come to the hatch together.
    let mut t = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(6, 6))
        .spawn_with(patch_of(1_000_000), tile(12, 6))
        .spawn_with(patch_of(1_000_000), tile(12, 10))
        .spawn(AGENT, Some(P1), centre(11, 6))
        .spawn(AGENT, Some(P1), centre(11, 10))
        .spawn(AGENT, Some(P1), centre(12, 8))
        .start();
    let agents = [4, 5, 6];
    gather(&mut t, 1, &[4, 6], 2);
    gather(&mut t, 1, &[5], 3);
    let first = run_to(&mut t, 400, "one unloads", |t| {
        agents.iter().any(|&n| is_unloading(t, n))
    });
    let watched = watch(&mut t, 3 * DROP_OFF, &agents, &[1]);
    let delivered = deliveries(&t);
    let ticks: Vec<Tick> = delivered.iter().map(|d| d.0).collect();
    let (second, third) = (first + DROP_OFF, first + 2 * DROP_OFF);
    assert_eq!(
        ticks,
        [second - 1, third - 1, third + DROP_OFF - 1],
        "one after the other without a gap: {delivered:?}"
    );
    assert!(
        watched.longest_wait >= DROP_OFF / 2,
        "they waited their turns: {watched:?}"
    );
    assert!(watched.longest_idle_wait <= 1, "{watched:?}");
    let mut workers: Vec<u32> = delivered.iter().map(|d| d.1).collect();
    workers.sort_unstable();
    assert_eq!(workers, agents, "each delivered its load");
}

/// Design change 0315 (`factions/syndicate/units/agent.md > Gathering > Gathering rules`, "Waiting at the hatch"):
/// "Until it is within reach it is never 'arrived': it keeps actively moving toward the hatch [...] and is exempt from
/// MovingToLocation's 'blocked near the target by standing units counts as arrived' rule [...] It waits as long as it
/// takes; it never gives up". Five Guards on Hold Position stand along the wall in front of the hatch. The Agent comes
/// back with its load and stops against them, 3 grid units or less from where it is going. Its leg never ends as
/// arrived: it presses on, looks for other routes, and when the mover gives it up (104 ticks,
/// `tuning::movement::GIVE_UP_TICKS`) the next leg begins in the next tick, with no wait
/// (`tuning::gathering::LEG_RETRY_TICKS` was the wait until 0315). It keeps its order and its load; once the Guards
/// are gone it delivers.
#[test]
fn an_agent_blocked_from_the_hatch_presses_on_and_never_gives_up() {
    // The wall is at x = 640, the hatch square at y 512 to 576: no Agent fits between the Guards and the wall.
    let mut builder = field(1, 4);
    for i in 0..5 {
        let at = SpawnAt::Point(Vec2::new(659.0, 470.0 + 37.0 * i as f32));
        builder = builder.spawn(ObjectKind::Guard, Some(P1), at);
    }
    let mut t = builder.start();
    let guards: Vec<ObjectId> = (4..9).map(id).collect();
    t.order_at(1, P1, &guards, OrderKind::Hold, false);
    gather(&mut t, 1, &[3], 2);
    // Jammed behind the Guards (`Mover::is_stationary`): until 0315 it had arrived there within a few ticks.
    let blocked = |t: &TestSim| phase(t, 3) == Some(GatherPhase::ToDropOff) && t.mover(3).blocked_ticks >= 8;
    run_to(&mut t, 600, "it is blocked short of the hatch", blocked);
    assert!(gap_to_hatch(&t, 3, 1) > REACH);
    // A minute of it: it is on its way all along, but for the tick each leg fails in.
    let mut legs = 0;
    for _ in 0..60 * 16 {
        let was = t.status(3);
        t.step();
        let now = t.status(3);
        assert_ne!(
            now,
            MoveStatus::Arrived,
            "tick {}: never arrived out of reach",
            t.sim().tick()
        );
        if was != MoveStatus::Moving {
            assert_eq!(
                (was, now),
                (MoveStatus::Failed, MoveStatus::Moving),
                "the next leg begins at once"
            );
            legs += 1;
        }
        assert!(now != MoveStatus::Moving || t.mover(3).presses_on);
        assert_eq!((order(&t, 3), load(&t, 3)), (Some(OrderKind::Gather(id(2))), LOAD));
        assert_eq!(phase(&t, 3), Some(GatherPhase::ToDropOff), "never within reach");
    }
    assert!((8..=10).contains(&legs), "{legs} legs in a minute");
    assert_eq!(deliveries(&t), []);
    let now = t.sim().tick();
    for &guard in &guards {
        t.debug_at(now + 1, DebugCommand::Remove { id: guard });
    }
    let unloading = run_to(&mut t, 100, "it unloads", |t| is_unloading(t, 3));
    t.run(DROP_OFF);
    assert_eq!(deliveries(&t), [(unloading + DROP_OFF - 1, 3, 1, LOAD)]);
}

/// Design change 0315 is a property of one leg, the Agent's leg back to a hatch (`Mover::presses_on`): its leg to the
/// patch doesn't press on (a patch it can't get to is waited at, design change 0141), and neither does a Drone's leg
/// back to its Egress Node, which takes loads anywhere along its edge.
#[test]
fn only_the_leg_to_a_hatch_presses_on() {
    // Tunnel 1, patch 2, Agent 3; player 2's Egress Node 4 and Drone 5, north of the patch.
    let mut t = field(1, 3)
        .faction(P2, Faction::Colonists)
        .spawn(ObjectKind::EgressNode, Some(P2), tile(13, 3))
        .spawn(ObjectKind::Drone, Some(P2), centre(13, 6))
        .start();
    gather(&mut t, 1, &[3], 2);
    t.order_at(1, P2, &[id(5)], OrderKind::Gather(id(2)), false);
    let mut seen = [0; 3];
    for _ in 0..600 {
        t.step();
        for (n, hatch) in [(3, true), (5, false)] {
            if t.status(n) != MoveStatus::Moving {
                continue;
            }
            let back = phase(&t, n) == Some(GatherPhase::ToDropOff);
            assert_eq!(t.mover(n).presses_on, back && hatch, "#{n}, on its way back: {back}");
            seen[usize::from(back) + usize::from(back && hatch)] += 1;
        }
    }
    assert!(
        seen.iter().all(|&ticks| ticks > 20),
        "legs to the patch, to the node and to the hatch: {seen:?}"
    );
}

/// Design change 0255's Verification: "An Agent stands beside the hatch square, not at the side's far end", and
/// 0248: the sides turn and flip with the Tunnel. In each orientation the patch lies 2 grid units out from the
/// crystal hatch; the Agent unloads within DropOffReach of the square, against the wall.
#[test]
fn an_agent_stands_at_the_hatch_square_however_the_tunnel_is_turned() {
    // (rotation, flipped left and right, the hatch tile, the patch tile): the Tunnel is at tiles (6, 6) to (9, 9).
    let cases = [
        (Rotation::R0, false, (9, 8), (12, 8)),
        (Rotation::R90, false, (7, 9), (7, 12)),
        (Rotation::R180, false, (6, 7), (3, 7)),
        (Rotation::R270, false, (8, 6), (8, 3)),
        (Rotation::R0, true, (6, 8), (3, 8)),
        (Rotation::R90, true, (7, 6), (7, 3)),
    ];
    for (rotation, flip_h, hatch_tile, patch) in cases {
        let scenario = flat(30, 20)
            .spawn(TUNNEL, Some(P1), tile(6, 6))
            .spawn_with(patch_of(1_000_000), tile(patch.0, patch.1))
            .spawn(AGENT, Some(P1), centre(14, 14))
            .build();
        let mut t = TestSim::new(turned(scenario, 0, rotation, flip_h));
        let square = TilePos::new(hatch_tile.0, hatch_tile.1);
        assert_eq!(
            hatch(&t, 1),
            Shape::rect(square.center(), Vec2::splat(32.0)),
            "{rotation:?} {flip_h}"
        );
        gather(&mut t, 1, &[3], 2);
        run_to(&mut t, 600, "it unloads", |t| is_unloading(t, 3));
        let gap = gap_to_hatch(&t, 3, 1);
        assert!(gap <= REACH, "{rotation:?} {flip_h}: {gap} su from the hatch");
        // Against the wall, in front of the square or beside it by no more than the reach.
        let off = (t.pos(3) - square.center()).abs();
        assert!(off.max_element() <= 32.0 + REACH + 18.0, "{rotation:?} {flip_h}: {off}");
        t.run(DROP_OFF);
        assert_eq!(deliveries(&t).len(), 1, "{rotation:?} {flip_h}");
    }
}
