//! Reclaiming (`factions/syndicate/tunnels.md > Losing Tunnels`, design change 0258 points 3 and 4): "if a Tunnel is
//! built whose Tunnel Area fully covers a neutral expansion, and that Tunnel's tier meets the expansion's Tier
//! Requirement, the expansion becomes owned by that Tunnel's owner — the original Syndicate player or an enemy
//! Syndicate player — and stops decaying"; "**Reclaiming happens** the moment the covering Tunnel ... **finishes**; a
//! Tunnel under construction reclaims nothing"; "**Partly inside an own Tunnel Area**: until reclaimed, a neutral
//! expansion still **blocks its cells**: nothing can be built there, and they **count as occupied** for that Tunnel's
//! Tunnel Space". A reclaimed expansion is hidden again, and an enemy's record of it drops when the enemy sees its
//! tiles again: through the per-player view, events included.

use sc_sim::targets::TargetView;
use sc_sim::{Command, DebugCommand, TileProblem, TunnelCommand};

use super::*;

/// Player 2's Guard in [`lost`], beside the Hideout.
const WATCHER: ObjectId = ObjectId(5);

/// [`base`] with player 1's second Tunnel (3) far east, its Agent 4 south of [`T1`], and player 2's Guard
/// [`WATCHER`] beside the Hideout, its Agent 6 far south-east; auto-targeting off, both players with 10 Supplies.
/// Player 1's Tunnel 1 is destroyed in the next tick: its Hideout turns neutral, under player 2's eyes. The tick it
/// fell.
fn lost() -> (TestSim, Tick) {
    let mut t = base()
        .spawn(TUNNEL, Some(P1), tile(30, 8)) // 3
        .spawn(AGENT, Some(P1), point(6, 16)) // 4
        .spawn(GUARD, Some(P2), point(10, 12)) // 5
        .spawn(AGENT, Some(P2), point(20, 20)) // 6
        .start();
    t.without_auto_targeting();
    next(&mut t, P1, bank(P1, 0, 10));
    next(&mut t, P2, bank(P2, 0, 10));
    next(&mut t, P1, remove(T1));
    let fell = t.sim().tick();
    assert_eq!(owner(&t, H1), None);
    assert_eq!(t.sim().target_view(P2, H1), TargetView::Visible);
    (t, fell)
}

/// Player `player`'s record of `id`, if it has one.
fn record(t: &TestSim, player: PlayerId, id: ObjectId) -> Option<sc_sim::vision::LastKnown> {
    t.sim().vision().player(player).and_then(|v| v.record(id)).cloned()
}

/// Moves unit `id` to the centre of tile `(x, y)` (`DebugCommand::SetPosition`), in the next tick.
fn teleport(t: &mut TestSim, id: ObjectId, x: i32, y: i32) {
    let pos = at(x, y).center();
    next(t, P1, Command::Debug(DebugCommand::SetPosition { id, pos }));
    assert_eq!(object(t, id).pos, pos);
}

/// H's Acceptance, "A new Tunnel fully covering a neutral Hideout takes it back at completion, never while a site";
/// plan, Vision: "A reclaimed one is hidden" and an enemy's record of it drops when the enemy sees its tiles again.
/// Player 1's Agent builds a Tunnel where its lost one stood. Through the site's 480 ticks the Hideout stays neutral
/// and decays. In the tick the site completes it is player 1's, at the HP the decay left it, and decays no more: from
/// the next tick on it regenerates, its Tunnel alive (slice M5-N8, `factions/syndicate/structures/README.md > Tunnel
/// Expansions`: "1% of MaxHP per second", 0.25 HP a tick of its 400); player 2, whose Guard stands beside it, no
/// longer sees it, and its record goes at the end of that tick: player 2 knows nothing of it from then on. No event of
/// that tick names it. Player 1 has no record of its own Hideout.
#[test]
fn a_new_tunnel_over_a_neutral_hideout_takes_it_back_at_completion_never_while_a_site() {
    let (mut t, fell) = lost();
    next(&mut t, P1, build(&[4], at(4, 8)));
    let (started, site) = until_started(&mut t, 4);
    assert!(object(&t, site).is_site());
    while t.sim().tick() < started + 479 {
        step(&mut t);
        assert_eq!(owner(&t, H1), None, "a site reclaims nothing, tick {}", t.sim().tick());
        assert!(object(&t, H1).decay.is_some());
    }
    step(&mut t);
    let done = completed(&t, site).expect("the site completes");
    assert_eq!(done, t.sim().tick());
    let h = object(&t, H1);
    assert_eq!((h.owner, h.decay), (Some(P1), None));
    let left = 400.0 - (done - fell) as f32 * 0.25;
    assert_eq!(
        h.hp,
        Some(left),
        "the HP the decay left it, which it lost before this tick's step"
    );

    // Hidden again: player 2's Guard stands beside it, and sees it no more.
    assert!(t.sim().hidden_from(h, P2) && !t.sim().is_visible(P2, h));
    assert_eq!(t.sim().target_view(P2, H1), TargetView::Gone);
    assert_eq!(record(&t, P2, H1), None, "player 2 sees its tiles: its record goes");
    assert_eq!(record(&t, P1, H1), None, "a player never remembers its own");
    let named: Vec<SimEvent> = (events_of(&t, done).into_iter()).filter(|e| names(e, H1)).collect();
    assert_eq!(named, [], "no event of the tick names it");

    run_until(&mut t, done + 64);
    assert_eq!(hp(&t, H1), left + 64.0 * 0.25, "it decays no more, and regenerates");
    assert_eq!(record(&t, P2, H1), None);
    assert_eq!(
        gate(&t, P2, order(&[5], OrderKind::Attack(H1), false)),
        Err(Reject::NotVisible(H1))
    );
}

/// Plan, Vision: "an enemy's record of it drops when the enemy sees its tiles again", and the design's "the original
/// Syndicate player or an enemy Syndicate player": player 2's Agent builds a Tunnel over player 1's lost Hideout. Player
/// 1, whose Tunnel's sight lingered over the Hideout as it fell, remembers it neutral; out of its sight when player 2's
/// Tunnel takes it, it remembers it as it last saw it, neutral at its last-known HP, until a unit of its own sees its
/// tiles again: then the record goes, and player 1 knows nothing of it. No event tells player 1 anything meanwhile.
#[test]
fn an_enemys_tunnel_takes_it_and_the_old_owner_forgets_it_once_it_sees_its_tiles() {
    let (mut t, fell) = lost();
    run_until(&mut t, fell + 40);
    let seen = record(&t, P1, H1).expect("player 1 remembers it neutral");
    assert_eq!(seen.owner, None);
    assert!(
        matches!(t.sim().target_view(P1, H1), TargetView::Remembered { .. }),
        "out of its sight"
    );
    // Player 1's Agent goes far away; player 2's Guard too, off the site.
    teleport(&mut t, id(4), 40, 20);
    teleport(&mut t, WATCHER, 20, 16);
    next(&mut t, P2, build(&[6], at(4, 8)));
    let (_, site) = until_started(&mut t, 6);
    step_until(&mut t, 600, "the site completes", |t| completed(t, site).is_some());
    let done = completed(&t, site).expect("completed");
    assert_eq!(owner(&t, H1), Some(P2));
    assert!(t.sim().hidden_from(object(&t, H1), P1));
    let kept = record(&t, P1, H1).expect("player 1 still remembers it");
    assert_eq!(
        (kept.owner, kept.hp, kept.seen),
        (None, seen.hp, seen.seen),
        "as it last saw it"
    );
    assert!(matches!(t.sim().target_view(P1, H1), TargetView::Remembered { .. }));
    assert_eq!(record(&t, P2, H1), None, "its new owner keeps no record of it");
    let named: Vec<SimEvent> = (events_of(&t, done).into_iter()).filter(|e| names(e, H1)).collect();
    assert_eq!(named, []);

    // Player 1's Agent comes back beside it: the record goes.
    run_until(&mut t, done + 16);
    teleport(&mut t, id(4), 11, 12);
    let back = t.sim().tick();
    assert_eq!(record(&t, P1, H1), None, "it sees the tiles, not the Hideout");
    assert_eq!(t.sim().target_view(P1, H1), TargetView::Gone);
    let named: Vec<SimEvent> = (t.events().iter())
        .filter(|(tick, e)| *tick > done && *tick <= back && names(e, H1))
        .map(|(_, e)| e.clone())
        .collect();
    assert_eq!(named, []);
}

/// H's Acceptance, "One covering it partly leaves it neutral. It then blocks its cells (F's placement) and counts as
/// occupied (B's Tunnel Space)": player 1's Agent builds a Tunnel whose Tier 1 area takes the Hideout's east column
/// only. At completion the Hideout stays neutral and keeps decaying. Its two cells in the new Tunnel's region count as
/// occupied: player 1's Tunnel Space is ⌊(48 + 46) / 4⌋ = 23, not 24. And it blocks its cells: a Tunnel ghost over
/// them shows them taken by a structure (the surface rule, design change 0240).
#[test]
fn a_tunnel_covering_it_partly_leaves_it_neutral_blocking_its_cells_and_taking_tunnel_space() {
    let (mut t, _) = lost();
    let space = |t: &TestSim| t.sim().army_cap(P1).expect("a Syndicate player").available;
    assert_eq!(space(&t), 12, "Tunnel 3's region, 48 cells");
    next(&mut t, P1, build(&[4], at(11, 8)));
    let (_, site) = until_started(&mut t, 4);
    step_until(&mut t, 600, "the site completes", |t| completed(t, site).is_some());
    let done = t.sim().tick();
    assert_eq!(owner(&t, H1), None, "partly covered: it stays neutral");
    let before = hp(&t, H1);
    step(&mut t);
    assert_eq!(hp(&t, H1), before - 0.25, "and keeps decaying");
    assert_eq!(space(&t), 23, "two of its cells in the new region count as occupied");

    // Player 1's Agent comes out of the new Tunnel, and puts a ghost of its next Tunnel over the Hideout's tiles:
    // they are taken.
    let eject = Command::Tunnel {
        tunnel: site,
        command: TunnelCommand::Eject { kind: AGENT },
    };
    next(&mut t, P1, eject);
    step_until(&mut t, 100, "the Agent comes out", |t| t.sim().object(id(4)).is_some());
    let orientation = BuildingOrientation::default();
    let check = (t.sim().build_check(id(4), TUNNEL, at(6, 6), orientation, false)).expect("an Agent on the map");
    let taken: Vec<TilePos> = (check.tiles.iter())
        .filter(|(_, problem)| *problem == TileProblem::Structure)
        .map(|(tile, _)| *tile)
        .collect();
    assert_eq!(taken, [at(8, 8), at(9, 8), at(8, 9), at(9, 9)]);
    assert_eq!(owner(&t, H1), None, "still neutral, {} ticks on", t.sim().tick() - done);
}
