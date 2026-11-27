//! What each tier lets through (`factions/syndicate/tunnels.md > Transit Tier Requirements`: "Tier 1: Infantry only",
//! "Tier 2+: Vehicles", "Tier 3+: Air units (Hover Craft, Glider)"; `factions/syndicate/units/phantom.md`: "UnitBase -
//! HoverCraft (enters and exits only through Tier 3 Tunnels' air doors)"), during an upgrade (design change 0249: "the
//! tier it had before the upgrade began"), and when the last Tier 3 Tunnel falls (`factions/syndicate/tunnels.md >
//! Losing Tunnels > Stranded units`).

use sc_sim::Transit;

use super::*;

/// Player 1's Guard, Assault Bike and Phantom, debug-spawned on the map north of [`base`]'s Tunnel in tick 2, which has
/// run, the Tunnel at `tier`: their ids.
fn units_at(tier: u8) -> (TestSim, [ObjectId; 3]) {
    let mut t = started(base(), tier);
    let first = next_id(&t);
    let kinds = [GUARD, ObjectKind::AssaultBike, PHANTOM];
    for (i, kind) in kinds.into_iter().enumerate() {
        let at = Vec2::new(700.0 + 120.0 * i as f32, 400.0);
        t.debug_at(2, spawn(kind, P1, SpawnAt::Point(at)));
    }
    run_checked(&mut t, 2);
    let units = [id(first), id(first + 1), id(first + 2)];
    for (unit, kind) in units.iter().zip(kinds) {
        assert_eq!(object(&t, *unit).kind, kind);
    }
    (t, units)
}

/// The plan's acceptance, "Ejected from a Tier 2 Tunnel, it is refused ("Tier 3")", and slice M6-SK's note on the tier
/// refusals (the brief: "With the doors built, check that each tier's refusal says the right thing"): at each tier,
/// alone, each of a Guard, a Bike and a Phantom has its Enter taken where the tier lists its base and refused for the
/// tier where it doesn't ([`Reject::NoTransit`], which the client words "This Tunnel's tier is too low"), its
/// right-click on the Tunnel an Enter or a Move to it alike; and an Eject of its type, none being inside, is refused
/// for the tier, or as none inside where the tier lets it out ([`Reject::NotInNetwork`]): the Phantom's for the tier at
/// Tiers 1 and 2, as none inside at Tier 3, never for a missing door.
#[test]
fn each_tier_lets_its_units_through_and_refuses_the_others_for_the_tier() {
    for tier in 1..=3u8 {
        let (t, units) = units_at(tier);
        let tunnel = object(&t, T1);
        let row = sc_sim::tunnel::tier_def(tunnel).expect("a tier");
        assert_eq!(row.transit.contains(&Transit::Air), tier == 3, "Tier {tier}");
        for (unit, first) in units.into_iter().zip([1, 2, 3]) {
            let kind = object(&t, unit).kind;
            let passes = tier >= first;
            let enter = gate(&t, P1, order(&[unit], OrderKind::Enter(T1)));
            let refused = Reject::NoTransit { tunnel: T1, kind: None };
            assert_eq!(
                enter,
                if passes { Ok(()) } else { Err(refused) },
                "{kind:?} at Tier {tier}"
            );
            let right_click = t.sim().default_order(unit, Target::Object(T1));
            let expected = match passes {
                true => OrderKind::Enter(T1),
                false => OrderKind::Move(Target::Object(T1)),
            };
            assert_eq!(right_click, Some(expected), "{kind:?} at Tier {tier}");
            let out = gate(&t, P1, eject(T1, kind));
            let refused = match passes {
                true => Reject::NotInNetwork(kind),
                false => Reject::NoTransit {
                    tunnel: T1,
                    kind: Some(kind),
                },
            };
            assert_eq!(out, Err(refused), "{kind:?} out at Tier {tier}");
        }
        // Together, the units the tier lets in take the order, the others ignore it.
        assert_eq!(gate(&t, P1, order(&units, OrderKind::Enter(T1))), Ok(()), "Tier {tier}");
    }
}

/// The plan's acceptance, "A Tunnel being upgraded to Tier 3 admits no air unit (0249: the tier it had before)": a Tier
/// 2 Tunnel upgrading to Tier 3, paid in Supplies, refuses a Phantom's Enter and its Eject for the tier throughout the
/// upgrade's 480 frames, its right-click there a Move; the Guard's and the Bike's are taken. The upgrade complete, the
/// Phantom's Enter is taken and it flies in by the air entry door.
#[test]
fn a_tunnel_upgrading_to_tier_3_admits_no_air_unit_until_it_completes() {
    let (mut t, [guard, bike, phantom]) = units_at(2);
    let upgrade = Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Upgrade,
    };
    assert_eq!(gate(&t, P1, upgrade.clone()), Ok(()));
    t.at(3, P1, upgrade);
    run_checked(&mut t, 3);
    let state = object(&t, T1).tunnel.as_ref().unwrap();
    assert!(
        matches!(
            state.operation,
            Some(sc_sim::tunnel::Operation::Upgrading { to: 3, .. })
        ),
        "{state:?}"
    );
    let refused = Reject::NoTransit { tunnel: T1, kind: None };
    let mut checked = 0;
    while object(&t, T1).tunnel.as_ref().unwrap().tier == 2 {
        assert_eq!(gate(&t, P1, order(&[phantom], OrderKind::Enter(T1))), Err(refused));
        let out = Reject::NoTransit {
            tunnel: T1,
            kind: Some(PHANTOM),
        };
        assert_eq!(gate(&t, P1, eject(T1, PHANTOM)), Err(out));
        let right_click = t.sim().default_order(phantom, Target::Object(T1));
        assert_eq!(right_click, Some(OrderKind::Move(Target::Object(T1))));
        for unit in [guard, bike] {
            assert_eq!(gate(&t, P1, order(&[unit], OrderKind::Enter(T1))), Ok(()));
        }
        step_checked(&mut t);
        checked += 1;
        assert!(checked <= 480, "the upgrade completes");
    }
    assert!(checked >= 479, "{checked} ticks of the upgrade");
    assert_eq!(object(&t, T1).tunnel.as_ref().unwrap().tier, 3);
    assert_eq!(gate(&t, P1, order(&[phantom], OrderKind::Enter(T1))), Ok(()));
    t.at(t.sim().tick() + 1, P1, order(&[phantom], OrderKind::Enter(T1)));
    step_until(&mut t, 400, "the Phantom goes in", |t| !entries(t, T1).is_empty());
    assert_eq!(inside(&t), [phantom]);
}

/// The plan's acceptance, "Losing the last Tier 3 Tunnel destroys the aircraft inside"
/// (`factions/syndicate/tunnels.md > Tunnel Network > Losing Tunnels`, "Stranded units": "any unit inside the Tunnel
/// Network that can no longer exit through any remaining Tunnel (no remaining Tunnel's tier is sufficient for its unit
/// base category) is **destroyed**"): player 1's Tier 3 Tunnel and a second one far east at Tier 2; a Phantom and a
/// Guard go in by the first, which then falls: the Phantom leaves the game in that tick, the Guard stays inside. With
/// the second Tunnel at Tier 3 instead, the Phantom stays too.
#[test]
fn the_last_tier_3_tunnel_lost_strands_the_aircraft_inside() {
    for second_tier in [2, 3] {
        let mut t = started(base().spawn(TUNNEL, Some(P1), tile(30, 10)), 3);
        let t2 = id(2);
        t.debug_at(
            2,
            DebugCommand::SetTier {
                id: t2,
                tier: second_tier,
            },
        );
        let phantom = id(next_id(&t));
        let guard = id(phantom.0 + 1);
        t.debug_at(2, spawn(PHANTOM, P1, SpawnAt::Point(Vec2::new(700.0, 500.0))));
        t.debug_at(2, spawn(GUARD, P1, SpawnAt::Point(Vec2::new(866.0, 560.0))));
        t.at(3, P1, order(&[phantom, guard], OrderKind::Enter(T1)));
        step_until(&mut t, 400, "both go in", |t| entries(t, T1).len() == 2);
        assert_eq!(inside(&t), [phantom, guard]);
        let fell = t.sim().tick() + 1;
        t.debug_at(fell, DebugCommand::Remove { id: T1 });
        run_checked(&mut t, fell);
        let removed = (t.events().iter()).any(|(tick, e)| *tick == fell && *e == SimEvent::Removed(phantom));
        match second_tier {
            2 => {
                assert!(removed, "stranded: no Tier 3 Tunnel left");
                assert_eq!(inside(&t), [guard], "the infantry can still leave");
            }
            _ => {
                assert!(!removed);
                assert_eq!(inside(&t), [phantom, guard], "a Tier 3 Tunnel left");
            }
        }
    }
}
