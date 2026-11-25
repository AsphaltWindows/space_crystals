//! The line at the hatch (`factions/syndicate/units/agent.md > Gathering rules > Waiting at the hatch`, design
//! change 0255: "an Agent waits within reach (or as close as it can get) while another drops off, and the first to
//! arrive delivers next. It waits as long as it takes; it never gives up"), and where an Agent stands to deliver
//! ("The hatch is 1 grid square long, near the middle of its side. An Agent must be within DropOffReach of the hatch
//! to deliver").

use super::*;
use sc_sim::MoveStatus;

/// Design change 0255's Verification: "Six Agents on one Tunnel queue at the hatch and all eventually deliver; none
/// idles with a load." With the patch 3 grid units from the hatch, for ten minutes: every Agent goes on delivering,
/// in the order they come within reach ([`watch`]), and an Agent within reach never stands at a free hatch beyond the
/// tick the one before it finishes in.
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
    assert!(income(&t, TICKS / 2, TICKS) >= 11.5, "the Tunnel is all but saturated");
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

/// The Director's acceptance for 0255: "an Agent that is blocked from the hatch retries on the Drone's rule and
/// never gives up". Five Guards on Hold Position stand along the wall in front of the hatch. The Agent comes back
/// with its load, stops as near as it gets, and sets off again every 32 ticks (`orders::gather::LEG_RETRY_TICKS`),
/// keeping its order and its load; once the Guards are gone it delivers.
#[test]
fn an_agent_blocked_from_the_hatch_retries_and_never_gives_up() {
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
    let standing = |t: &TestSim| phase(t, 3) == Some(GatherPhase::ToDropOff) && t.status(3) != MoveStatus::Moving;
    run_to(&mut t, 600, "it stops short of the hatch", standing);
    assert!(gap_to_hatch(&t, 3, 1) > REACH);
    // A minute of it: it sets off again and again.
    let mut legs = 0;
    for _ in 0..60 * 16 {
        let was = t.status(3);
        t.step();
        legs += u32::from(was != MoveStatus::Moving && t.status(3) == MoveStatus::Moving);
        assert_eq!((order(&t, 3), load(&t, 3)), (Some(OrderKind::Gather(id(2))), LOAD));
        assert_eq!(phase(&t, 3), Some(GatherPhase::ToDropOff), "never within reach");
    }
    assert!((10..=30).contains(&legs), "{legs} legs in a minute");
    assert_eq!(deliveries(&t), []);
    let now = t.sim().tick();
    for &guard in &guards {
        t.debug_at(now + 1, DebugCommand::Remove { id: guard });
    }
    let unloading = run_to(&mut t, 100, "it unloads", |t| is_unloading(t, 3));
    t.run(DROP_OFF);
    assert_eq!(deliveries(&t), [(unloading + DROP_OFF - 1, 3, 1, LOAD)]);
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
