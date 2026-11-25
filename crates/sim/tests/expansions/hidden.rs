//! What the other player sees of an expansion (`factions/syndicate/structures/README.md > Tunnel Expansions`:
//! "Expansions are invisible to enemies without detection"; plan decision T32, `vision::hidden_from`): nothing, from
//! the click on; but a Warp In over it is red at once, as a direct placement is (design changes 0240 and 0290).

use sc_sim::vision::VisibilityState;
use sc_sim::{OrderKind, Target};

use super::*;

/// The M4 plan's acceptance, "The enemy never sees the new Hideout or its ghost", through player 2's view: its
/// Peacekeeper stands by the site, the tiles under it Visible to player 2, from the click to completion and after.
/// Player 2 never sees it nor records it, and can't name it in an order; its own Agent's ghost of a Tunnel over it
/// judges by what player 2 knows, without it, where player 1's Agent's has it. Player 2's record of player 1's Tunnel,
/// which it sees, has the tier alone, not the operation (`vision::memory`). The only events that name the site are its
/// `Spawned` and the Tunnel's start and completion of it: no refusal, no attack, no hit. (The ghost itself is the
/// owner's client's alone, never in the sim.)
#[test]
fn the_enemy_never_sees_it() {
    let scenario = base()
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(3, 4)) // 4: by the site
        .spawn(AGENT, Some(P2), point(20, 4)); // 5
    let mut t = started_peaceful(scenario, 1000);
    let hold = Command::Order {
        units: vec![ObjectId(4)],
        kind: OrderKind::Hold,
        queue: false,
    };
    next(&mut t, P2, hold);
    t.run(sc_sim::tuning::vision::FOG_DELAY_TICKS + 1);
    next(&mut t, P1, expand(2, 6));
    let site = building(&t);
    let unseen = |t: &TestSim| {
        let vision = t.sim().vision().player(P2).expect("in the game");
        let o = object(t, site);
        assert!(t.sim().hidden_from(o, P2) && !t.sim().hidden_from(o, P1));
        assert_eq!(t.sim().visibility(P2, at(2, 6)), VisibilityState::Visible);
        assert!(!t.sim().is_visible(P2, o));
        assert!(vision.record(site).is_none());
        let attack = Command::Order {
            units: vec![ObjectId(4)],
            kind: OrderKind::Attack(site),
            queue: false,
        };
        assert_eq!(gate(t, P2, attack), Err(Reject::NotVisible(site)));
        let walk = Command::Order {
            units: vec![ObjectId(4)],
            kind: OrderKind::Move(Target::Object(site)),
            queue: false,
        };
        assert_eq!(gate(t, P2, walk), Err(Reject::NotVisible(site)));
        let record = vision.record(T1).expect("player 2 sees the Tunnel");
        assert_eq!(
            record.tunnel.as_ref().map(|state| (state.tier, state.operation)),
            Some((1, None))
        );
    };
    for _ in 0..420 {
        unseen(&t);
        t.step();
    }
    assert!(!object(&t, site).is_site(), "complete");
    unseen(&t);
    // A Build ghost of a Tunnel on (0, 4) to (3, 7), over the site's (2, 6) to (3, 7): its tiles' problems.
    let over_the_site = |agent: ObjectId| {
        let ghost = t
            .sim()
            .build_check(agent, TUNNEL, at(0, 4), BuildingOrientation::default(), false);
        let ghost = ghost.expect("an Agent of a player's");
        let on_site = |tile: &TilePos| (2..4).contains(&tile.x) && (6..8).contains(&tile.y);
        ghost
            .tiles
            .into_iter()
            .filter(|(tile, _)| on_site(tile))
            .collect::<Vec<_>>()
    };
    assert_eq!(over_the_site(ObjectId(5)), [], "player 2's");
    assert_eq!(over_the_site(AGENT_1).len(), 4, "player 1's");
    // Every event that names the site is about player 1's objects.
    let naming: Vec<&SimEvent> = (t.events().iter())
        .map(|(_, e)| e)
        .filter(|e| match **e {
            SimEvent::Spawned(id) | SimEvent::Removed(id) => id == site,
            SimEvent::ExpansionStarted { site: s, .. } | SimEvent::ExpansionCompleted { site: s, .. } => s == site,
            SimEvent::Attacked { target, .. } | SimEvent::Hit { target, .. } => target == site,
            _ => false,
        })
        .collect();
    assert_eq!(
        naming,
        [
            &SimEvent::Spawned(site),
            &SimEvent::ExpansionStarted { tunnel: T1, site },
            &SimEvent::ExpansionCompleted { tunnel: T1, site },
        ]
    );
}

/// Design change 0290's verification, "A Beacon's Warp In ghost over a hidden enemy expansion is red"
/// (`factions/syndicate/structures/README.md > Tunnel Expansions`: "A Colonists **Warp In** is judged the same way: red
/// at once", as a direct placement, 0240: "which reveals that something is underground there; this is accepted"):
/// player 2's Beacon (10, 12) to (11, 13), by player 1's Hideout on (8, 12) to (9, 13), which player 2 doesn't see on
/// tiles it sees, warps in a Robo Portal on (9, 11) to (12, 14). The ghost (`Sim::warp_in_check`) is red on the
/// Hideout's two tiles under it, by the truth, and the gate refuses the Warp In.
#[test]
fn a_warp_in_over_it_is_red_at_once() {
    let scenario = base()
        .faction(P2, Faction::Colonists)
        .spawn(ObjectKind::Beacon, Some(P2), tile(10, 12)); // 4
    let mut t = started_peaceful(scenario, 1000);
    next_debug(&mut t, set_crystals(P2, 1000));
    let (hideout, beacon) = (ObjectId(2), ObjectId(4));
    let o = object(&t, hideout);
    assert!(t.sim().hidden_from(o, P2) && !t.sim().is_visible(P2, o));
    assert_eq!(t.sim().visibility(P2, at(9, 12)), VisibilityState::Visible);
    let orientation = BuildingOrientation::default();
    let ghost = t.sim().warp_in_check(beacon, ObjectKind::RoboPortal, orientation);
    let under = [(at(9, 12), TileProblem::Structure), (at(9, 13), TileProblem::Structure)];
    assert_eq!(ghost.map(|check| check.tiles), Some(under.to_vec()));
    let warp_in = Command::Construction {
        objects: vec![beacon],
        command: ConstructionCommand::WarpIn {
            kind: ObjectKind::RoboPortal,
            orientation,
        },
    };
    assert_eq!(gate(&t, P2, warp_in), tile_problem(9, 12, TileProblem::Structure));
}
