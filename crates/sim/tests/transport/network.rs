//! The Transport in the Tunnel Network
//! (`factions/syndicate/units/subterranean_transport.md > Entering the Tunnel Network loaded`: it "can enter the Tunnel
//! Network **loaded**", "its passengers stay inside it", "it **counts as one unit** in the EjectMenu"; design change
//! 0432 answering Q0166, part 2: "Passengers aren't in the network: no regeneration, no Null Chamber turn", "only the
//! Transport regenerates"): a vehicle, through a Tier 2+ Tunnel, its passengers held aboard it there
//! (`network::Place::Carrier`), out of every service's reach, their Tunnel Space counted throughout.

use sc_sim::TICKS_PER_SECOND;
use sc_sim::network::Place;

use super::*;

const CHAMBER: ObjectKind = ObjectKind::NullChamber;

/// The plan's acceptance: "it enters a Tier 2 Tunnel loaded; the EjectMenu lists only the Transport; ejected, it is
/// still loaded; Tunnel Space counts the passengers throughout". Three Guards aboard: a Tier 1 Tunnel won't take the
/// Transport ([`Reject::NoTransit`]: "Tier 2+: Vehicles"); at Tier 2 it drives in with them, the EjectMenu lists it
/// alone, an Eject of a Guard is refused ([`Reject::NotInNetwork`]: none is in the network), and the Guards stay
/// aboard, held in their carrier. Ejected, it comes out loaded, and Unload All lets them out round it. Player 1's
/// Tunnel Space in use is the same in every tick.
#[test]
fn it_drives_into_a_tier_2_tunnel_loaded_and_comes_out_loaded() {
    let (mut t, transport, guards) = network(&[], 3, 1);
    let space = TRANSPORT_SPACE + 3 * GUARD_SPACE;
    board(&mut t, &guards, transport);
    let loaded = aboard(&t, transport);
    assert_eq!(
        gate(&t, P1, order(&[transport], OrderKind::Enter(T1))),
        Err(Reject::NoTransit { tunnel: T1, kind: None }),
        "not at Tier 1"
    );
    let tier = t.sim().tick() + 1;
    t.debug_at(tier, DebugCommand::SetTier { id: T1, tier: 2 });
    t.run_until(tier);
    t.at(tier + 1, P1, order(&[transport], OrderKind::Enter(T1)));
    step_until(&mut t, 400, "it drives in", |t| {
        assert_eq!(space_used(t, P1), space, "tick {}", t.sim().tick());
        inside(t).contains(&transport)
    });
    assert_eq!(inside(&t), [transport], "the EjectMenu lists it alone");
    assert_eq!(aboard(&t, transport), loaded, "they stay aboard");
    for &guard in &guards {
        assert_eq!(
            t.sim().held_unit(guard).map(|held| held.place),
            Some(Place::Carrier(transport))
        );
    }
    assert_eq!(gate(&t, P1, eject(T1, GUARD)), Err(Reject::NotInNetwork(GUARD)));
    let came_in = entered(&t, transport);
    t.run_until(came_in + 60);
    assert_eq!(space_used(&t, P1), space);
    let out = eject_out(&mut t, T1, TRANSPORT, transport);
    assert_eq!(aboard(&t, transport), loaded, "out loaded");
    assert_eq!(space_used(&t, P1), space);
    let unload = out + 1;
    t.at(unload, P1, order(&[transport], OrderKind::UnloadAll));
    step_until(&mut t, 60, "they come out", |t| aboard(t, transport).is_empty());
    assert_eq!(unloaded(&t, transport).len(), 3);
    assert_eq!(space_used(&t, P1), space);
}

/// Design change 0432's Verification, part 2: "a loaded Transport, wounded, a minute in the network with a Null
/// Chamber: the Transport comes out healed and buffed, its passengers wounded and unbuffed". The Transport and two
/// Guards are wounded, and the Guards board it; inside, the Transport regains 1% of its MaxHP a second
/// (`factions/syndicate/tunnels.md > Regeneration inside`), 3 HP, to its MaxHP, and takes its 2 s Null Chamber turn;
/// its passengers regain nothing and take no turn (`Held::service` stays empty), their HP the same in every tick.
/// Ejected after a minute, the Transport holds the Null Chamber's exit buff and its passengers don't; unloaded, they
/// come out as wounded as they went in, and unbuffed.
#[test]
fn a_minute_inside_heals_and_buffs_it_and_not_its_passengers() {
    let (mut t, transport, guards) = network(&[(CHAMBER, CHAMBER_AT)], 2, 2);
    let wounded = wound(&mut t, transport);
    let theirs: Vec<f32> = guards.iter().map(|&guard| wound(&mut t, guard)).collect();
    assert!(wounded <= MAX_HP - 3.0 * 2.0, "wounded: {wounded}");
    assert!(theirs.iter().all(|&hp| hp < GUARD_HP), "wounded: {theirs:?}");
    board(&mut t, &guards, transport);
    let came_in = drive_in(&mut t, transport, T1);
    let unchanged = |t: &TestSim| {
        for (&guard, &was) in guards.iter().zip(&theirs) {
            let held = t.sim().held_unit(guard).expect("held");
            assert_eq!(held.place, Place::Carrier(transport));
            assert_eq!(hp(t, guard), was, "{guard:?} regains nothing, tick {}", t.sim().tick());
            assert!(held.service.is_none(), "{guard:?} takes no turn");
            assert!(!buffed(t, guard));
        }
    };
    let at_entry = hp(&t, transport);
    while t.sim().tick() < came_in + TICKS_PER_SECOND {
        t.step();
        unchanged(&t);
    }
    let gain = hp(&t, transport) - at_entry;
    assert!((gain - 3.0).abs() < 0.01, "1% of 300 a second: {gain}");
    while t.sim().tick() < came_in + 60 * TICKS_PER_SECOND {
        t.step();
        unchanged(&t);
    }
    assert_eq!(hp(&t, transport), MAX_HP, "healed");
    assert!(
        t.sim().held_unit(transport).expect("held").service.charged(),
        "its turn done"
    );
    eject_out(&mut t, T1, TRANSPORT, transport);
    assert!(buffed(&t, transport), "it leaves buffed");
    unchanged(&t);
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[transport], OrderKind::UnloadAll));
    step_until(&mut t, 60, "they come out", |t| aboard(t, transport).is_empty());
    for (&guard, &was) in guards.iter().zip(&theirs) {
        assert_eq!(hp(&t, guard), was, "{guard:?} as wounded as it went in");
        assert!(!buffed(&t, guard), "{guard:?} unbuffed");
    }
}
