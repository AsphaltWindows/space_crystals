//! A Tunnel destroyed leaves its expansions neutral (`factions/syndicate/tunnels.md > Losing Tunnels`): visible to
//! everyone and remembered, every kind of them from that tick, its Laser Turrets too (design change 0360), decaying at
//! 1% of their MaxHP a second (design change 0258 point 1), their work in progress lost and their queue refunded; the
//! Syndicate is defeated with its last structure, a turret no longer keeping it in the game, and its network goes with
//! it (design change 0258 point 5). An expansion it was building is destroyed with it, unrefunded (design change 0254).

use sc_sim::targets::TargetView;
use sc_sim::tunnel::Operation;
use sc_sim::{ProductionCommand, Target, TunnelCommand};

use super::*;

/// Player 2's Guards in [`last_stand`], south and east of the Tunnel, in range of its footprint.
const ATTACKERS: [u32; 6] = [5, 6, 7, 8, 9, 10];

/// [`base`], with player 1's Agent 3 on the map far south-east and its Guard 4 at the Tunnel's entry path, and
/// player 2's six Guards around the Tunnel's south and east sides, one of them (10) beside the Hideout; auto-targeting
/// off.
fn last_stand() -> TestSim {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(20, 20)) // 3
        .spawn(GUARD, Some(P1), point(5, 6)) // 4
        .spawn(GUARD, Some(P2), point(4, 13)) // 5
        .spawn(GUARD, Some(P2), point(5, 13)) // 6
        .spawn(GUARD, Some(P2), point(6, 13)) // 7
        .spawn(GUARD, Some(P2), point(7, 13)) // 8
        .spawn(GUARD, Some(P2), point(8, 12)) // 9
        .spawn(GUARD, Some(P2), point(10, 11)) // 10
        .start();
    t.without_auto_targeting();
    step(&mut t);
    t
}

/// H's Acceptance, "The only Tunnel destroyed, with its Hideout": `factions/syndicate/tunnels.md > Losing Tunnels`
/// ("every expansion in its Tunnel Area becomes neutral"; "no longer hidden (visible to everyone, no detection
/// needed)"; "1% per second, placeholder (a Hideout lasts 100 s)") and `core/game_rules.md > On Defeat`. Player 2's
/// Guards destroy player 1's only Tunnel in combat. In that tick the Hideout becomes neutral, seen and remembered by
/// player 2, whose Guard stands beside it; player 1, with no structure left, is defeated: its Agent on the map turns
/// neutral, and its Guard in the network is destroyed. The Hideout loses a quarter of an HP a tick, its first in that
/// tick, 4 a second, and dies, its HP at 0, in its 1600th tick of decay: 100 s.
#[test]
fn the_only_tunnel_falls_its_hideout_turns_neutral_and_decays_and_the_syndicate_falls() {
    let mut t = last_stand();
    enter(&mut t, P1, &[4], T1);
    assert!(t.sim().hidden_from(object(&t, H1), P2), "hidden while player 1 owns it");
    assert_eq!(t.sim().target_view(P2, H1), TargetView::Gone);
    next(&mut t, P2, order(&ATTACKERS, OrderKind::Attack(T1), false));
    step_until(&mut t, 3000, "the Tunnel falls", |t| died(t, T1).is_some());
    let (fell, by) = died(&t, T1).expect("fell");
    assert_eq!((by, t.sim().tick()), (Some(P1), fell));

    let h = object(&t, H1);
    assert_eq!((h.owner, h.hp), (None, Some(400.0 - 0.25)), "neutral, a tick of decay");
    assert!(
        !t.sim().hidden_from(h, P2) && t.sim().is_visible(P2, h),
        "player 2 sees it"
    );
    assert_eq!(t.sim().target_view(P2, H1), TargetView::Visible);
    let record = t
        .sim()
        .vision()
        .player(P2)
        .and_then(|v| v.record(H1))
        .expect("player 2 remembers it");
    assert_eq!((record.kind, record.owner), (HIDEOUT, None));

    assert!(t.sim().player(P1).expect("in the game").is_defeated());
    assert!(events_of(&t, fell).contains(&SimEvent::PlayerDefeated { player: P1 }));
    assert_eq!(owner(&t, id(3)), None, "its Agent on the map turns neutral");
    assert_eq!(removed(&t, id(4)), Some(fell), "its Guard in the network is destroyed");
    assert!(t.sim().held().next().is_none());
    assert_eq!(t.sim().outcome().map(|o| o.winners.clone()), Some(vec![P2]));

    // 4 HP a second, from the tick it fell.
    run_until(&mut t, fell + 16);
    assert_eq!(hp(&t, H1), 400.0 - 17.0 * 0.25);
    run_until(&mut t, fell + 16 * 50);
    assert_eq!(hp(&t, H1), 200.0 - 0.25, "half its MaxHP in 50 s");
    run_until(&mut t, fell + DECAY_TICKS - 2);
    assert_eq!(hp(&t, H1), 0.25, "a tick of it left");
    step(&mut t);
    assert_eq!(
        (died(&t, H1), removed(&t, H1)),
        (Some((fell + DECAY_TICKS - 1, None)), Some(fell + DECAY_TICKS - 1))
    );
}

/// [`base`] with player 1's second Tunnel (3) far east, on tiles (30, 8) to (33, 11), and its Guard 4 at [`T1`]'s
/// entry path; player 1 with 1000 Space Crystals.
fn two_tunnels() -> TestSim {
    let mut t = base()
        .spawn(TUNNEL, Some(P1), tile(30, 8)) // 3
        .spawn(GUARD, Some(P1), point(5, 6)) // 4
        .start();
    t.without_auto_targeting();
    next(&mut t, P1, bank(P1, 1000, 0));
    t
}

fn production(producer: ObjectId, command: ProductionCommand) -> Command {
    Command::Production {
        objects: vec![producer],
        command,
    }
}

/// H's Acceptance, "One of two Tunnels destroyed": its Hideout turns neutral, the infantry in the network survive,
/// and "**In-progress work is destroyed**" (the Agent in production is lost, refunding nothing) and "**Queued work is
/// refunded**" (the two Agents queued behind it, 100 Space Crystals each), with a `SimEvent::ProductionCancelled`
/// each; the rally point is cleared and no unit ever comes of it. Its former owner, whose Tunnel's sight lingers over
/// it in that tick, remembers it neutral.
#[test]
fn one_of_two_tunnels_falls_its_hideout_turns_neutral_its_work_lost_its_queue_refunded() {
    let mut t = two_tunnels();
    enter(&mut t, P1, &[4], T1);
    next(
        &mut t,
        P1,
        production(H1, ProductionCommand::SetRally(Target::Point(at(12, 4).center()))),
    );
    for _ in 0..3 {
        next(&mut t, P1, production(H1, ProductionCommand::Train(AGENT)));
    }
    let queued = t.sim().tick();
    run_until(&mut t, queued + 40);
    let p = object(&t, H1).production.clone().expect("a producer");
    assert!(p.in_production() == Some(AGENT) && p.queue.len() == 3 && p.rally.is_some());
    assert_eq!(t.sim().player(P1).expect("in the game").crystals, 700);

    next(&mut t, P1, remove(T1));
    let fell = t.sim().tick();
    let h = object(&t, H1);
    assert_eq!(h.owner, None);
    let p = h.production.clone().expect("still a producer");
    assert!(p.queue.is_empty() && p.rally.is_none() && p.progress == 0);
    let cancelled: Vec<(ObjectKind, u32)> = (events_of(&t, fell).into_iter())
        .filter_map(|e| match e {
            SimEvent::ProductionCancelled { producer, kind, refund } if producer == H1 => Some((kind, refund)),
            _ => None,
        })
        .collect();
    assert_eq!(cancelled, [(AGENT, 0), (AGENT, 100), (AGENT, 100)]);
    assert_eq!(t.sim().player(P1).expect("in the game").crystals, 900);
    assert!(
        !t.sim().player(P1).expect("in the game").is_defeated(),
        "its other Tunnel stands"
    );
    assert_eq!(inside(&t, P1), [id(4)], "the infantry in the network survive");
    let record = t
        .sim()
        .vision()
        .player(P1)
        .and_then(|v| v.record(H1))
        .expect("its former owner remembers it");
    assert_eq!(record.owner, None);

    // A neutral Hideout takes no command, and produces nothing.
    assert!(gate(&t, P1, production(H1, ProductionCommand::Train(AGENT))).is_err());
    run_until(&mut t, fell + 300);
    let produced = t
        .events()
        .iter()
        .any(|(_, e)| matches!(e, SimEvent::UnitProduced { producer, .. } if *producer == H1));
    assert!(!produced);
    assert_eq!(inside(&t, P1), [id(4)]);
}

/// Design change 0254 (`factions/syndicate/tunnel_menus.md > Expansions under construction`): "**Its Tunnel destroyed
/// mid-build**: the unfinished expansion is destroyed, no refund" (slice M4-F's site, `Builder::Tunnel`). In
/// [`two_tunnels`], [`T1`] builds a second Hideout in its area, paying 200 of player 1's 1000 Space Crystals, and falls
/// halfway: the site leaves the game in that tick with it, as a removal and not a cancel (no
/// `SimEvent::ExpansionCancelled`), refunding nothing, while the finished Hideout [`H1`] turns neutral.
#[test]
fn a_hideout_under_construction_falls_with_its_tunnel_with_no_refund() {
    let mut t = two_tunnels();
    let expand = TunnelCommand::Expand {
        kind: HIDEOUT,
        origin: at(2, 6),
        orientation: BuildingOrientation::default(),
    };
    next(
        &mut t,
        P1,
        Command::Tunnel {
            tunnel: T1,
            command: expand,
        },
    );
    let operation = object(&t, T1).tunnel.as_ref().and_then(|state| state.operation);
    let Some(Operation::Building(site)) = operation else {
        panic!("Tunnel {T1:?} builds a Hideout: {operation:?}");
    };
    let crystals = |t: &TestSim| t.sim().player(P1).expect("in the game").crystals;
    assert_eq!(crystals(&t), 800);
    let started = t.sim().tick();
    run_until(&mut t, started + 200);
    assert!(object(&t, site).is_site(), "halfway");
    next(&mut t, P1, remove(T1));
    let fell = t.sim().tick();
    assert_eq!(removed(&t, site), Some(fell));
    assert!(t.sim().object(site).is_none());
    assert_eq!(crystals(&t), 800, "no refund");
    let cancelled = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::ExpansionCancelled { .. }));
    assert!(!cancelled);
    assert_eq!(owner(&t, H1), None, "the finished Hideout turns neutral");
}

/// Design change 0360 (`factions/syndicate/tunnels.md > Losing Tunnels`: "Neutral expansions are **no longer hidden**
/// (visible to everyone, no detection needed). This holds for **every expansion building of the destroyed Tunnel**, its
/// lowered pop-up turrets included, which become visible at once"): [`base`]'s Tunnel with one of each expansion kind a
/// Tunnel builds in its area (`sc_sim::tunnel::is_expansion`, with its `StructureDef::expansion`; slice M6-SK's three
/// others, which none builds until its slice, plan decision T84, are left out): the Hideout, an Optics Lab, a Chemical
/// Plant, a Laser Turret, lowered, a Vehicle Assembly (9, slice M6-SV), a Forge (10) and an Aerospace Assembly (11,
/// slice M6-SD), whose 3 × 3 needs the Tier 3 area, and a Null Chamber (12, slice M6-SN) (the Tunnel set to Tier 3, and
/// the four spawned in its area, in tick 1); player 1's second Tunnel (3), far east,
/// keeps it in the game. Player 2's two Guards see all their tiles, and none of them while player 1 owns them. In the
/// tick the Tunnel falls every one of them is neutral, hidden from nobody, seen by both players and remembered by both,
/// neutral: player 1 by the sight its lost Tunnel leaves over them in that tick.
#[test]
fn every_expansion_of_a_lost_tunnel_is_seen_by_every_player_at_once() {
    let mut t = base()
        .spawn(TUNNEL, Some(P1), tile(30, 8)) // 3
        .spawn(ObjectKind::OpticsLab, Some(P1), tile(2, 6)) // 4
        .spawn(ObjectKind::ChemicalPlant, Some(P1), tile(2, 12)) // 5
        .spawn(ObjectKind::LaserTurret, Some(P1), tile(9, 12)) // 6
        .spawn(GUARD, Some(P2), point(1, 9)) // 7
        .spawn(GUARD, Some(P2), point(11, 10)) // 8
        .start();
    t.without_auto_targeting();
    t.debug_at(1, DebugCommand::SetTier { id: T1, tier: 3 });
    let spawned = [
        (ObjectKind::VehicleAssembly, tile(4, 12)),   // 9
        (ObjectKind::Forge, tile(1, 10)),             // 10
        (ObjectKind::AerospaceAssembly, tile(9, 13)), // 11
        (ObjectKind::NullChamber, tile(10, 8)),       // 12
    ];
    for (kind, at) in spawned {
        let spawn = DebugCommand::Spawn {
            kind,
            owner: Some(P1),
            at,
            orientation: BuildingOrientation::default(),
        };
        t.debug_at(1, spawn);
    }
    run_until(&mut t, sc_sim::tuning::vision::BUILTIN.fog_delay_ticks + 1);
    let expansions = [H1, id(4), id(5), id(6), id(9), id(10), id(11), id(12)];
    let mut kinds: Vec<ObjectKind> = expansions.iter().map(|&e| object(&t, e).kind).collect();
    kinds.sort();
    let built = |kind: ObjectKind| (kind.def().structure.as_ref()).is_some_and(|s| s.expansion.is_some());
    let every: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|&kind| sc_sim::tunnel::is_expansion(kind) && built(kind))
        .collect();
    assert_eq!(kinds, every, "one of each expansion kind a Tunnel builds");
    for &e in &expansions {
        let o = object(&t, e);
        let origin = o.placement.expect("placed").origin;
        assert_eq!(
            t.sim().visibility(P2, origin),
            sc_sim::vision::VisibilityState::Visible,
            "{:?}",
            o.kind
        );
        assert!(t.sim().hidden_from(o, P2) && !t.sim().is_visible(P2, o), "{:?}", o.kind);
    }
    assert_eq!(object(&t, id(6)).popup, Some(sc_sim::popup::PopUp::LOWERED));

    next(&mut t, P1, remove(T1));
    assert!(!t.sim().player(P1).expect("in the game").is_defeated());
    for &e in &expansions {
        let o = object(&t, e);
        assert_eq!(o.owner, None, "{:?}", o.kind);
        for player in [P1, P2] {
            assert!(!t.sim().hidden_from(o, player), "{:?}, from {player:?}", o.kind);
            assert!(t.sim().is_visible(player, o), "{:?}, by {player:?}", o.kind);
            let record = t.sim().vision().player(player).and_then(|v| v.record(e));
            assert_eq!(
                record.map(|r| (r.kind, r.owner)),
                Some((o.kind, None)),
                "{:?}, {player:?}'s record",
                o.kind
            );
        }
    }
}

/// Design change 0360 (`factions/syndicate/tunnels.md > Losing Tunnels`: "Since turrets and expansions go neutral with
/// their Tunnel, **losing its last Tunnel defeats the Syndicate** unless it owns some other structure"): player 1's
/// only structures are its Tunnel and a Laser Turret in its area, which kept it in the game before (slice M4-S1's
/// upright turret). The Tunnel destroyed, the turret turns neutral with it, and player 1 is defeated in that tick:
/// player 2 wins.
#[test]
fn a_turret_keeps_no_syndicate_player_in_the_game() {
    let mut t = ScenarioBuilder::new(&flat(48, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(4, 8)) // 1
        .spawn(ObjectKind::LaserTurret, Some(P1), tile(9, 10)) // 2
        .spawn(GUARD, Some(P2), point(30, 20)) // 3
        .start();
    step(&mut t);
    assert!(t.sim().is_active_turret(object(&t, id(2))));
    next(&mut t, P1, remove(T1));
    let fell = t.sim().tick();
    assert_eq!(owner(&t, id(2)), None, "the turret turns neutral");
    assert!(t.sim().player(P1).expect("in the game").is_defeated());
    assert!(events_of(&t, fell).contains(&SimEvent::PlayerDefeated { player: P1 }));
    assert_eq!(t.sim().outcome().map(|o| o.winners.clone()), Some(vec![P2]));
}
