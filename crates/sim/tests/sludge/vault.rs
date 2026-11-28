//! The Tunnel Network (`factions/syndicate/structures/vault.md`: "a Vault re-arms and recharges units **anywhere in the
//! Tunnel Network**"; `> Placeholder stats`: "each Vault re-arms or recharges **2 units at a time**; others wait their
//! turn (network-wide, first come first served) ... Shadebringer tanks refill at 5 per second each"; design change
//! 0424: "a unit's Vault turn, Null Chamber turn (and any repair) run **at the same time** in separate queues", "the
//! EjectMenu keeps taking the unit **inside the network longest**"; design change 0425: "entering the Tunnel Network
//! **switches both toggles off**").

use sc_sim::defs::TRANSIT_DELAY_TICKS;
use sc_sim::network::Place;

use super::*;

/// Design change 0425's Verification, "Drive a Shadebringer with both toggles on into a Tunnel, eject it: both toggles
/// are off", with its Sludge (Cover is slice M6-SC's): driving into the Tier 2 Tunnel with its Sludge on, laying its
/// trail and draining on its way, the Shadebringer goes into the network with both its toggles off, its tank as it
/// left the map (no Vault in the game) and its trail ended; its trail stays on the ground for its 10 s. Ejected, it
/// comes out with both off.
#[test]
fn a_shadebringer_driven_in_with_its_toggle_on_comes_out_with_both_off() {
    let (mut t, sbs) = network(&[], 1);
    let sb = sbs[0];
    t.at(2, P1, press(&[sb], sludge(true)));
    let came_in = drive_in(&mut t, sb);
    let held = t.sim().held_unit(sb).expect("held");
    let tanks = held.object.tanks.expect("its tanks");
    assert_eq!(held.place, Place::Network);
    assert_eq!(
        (tanks.sludge.on, tanks.sludge.laid, tanks.smoke.on),
        (false, None, false),
        "both off"
    );
    assert_eq!(tanks.sludge.used, (came_in - 2) * DRAIN, "drained until it went in");
    let trail = laid_by(&t, sb);
    assert!(trail.len() >= 10, "{trail:?}");
    assert_eq!(
        trail.iter().map(|p| p.until).max(),
        Some(came_in - 1 + LIFE),
        "laid until it went in"
    );
    run_until(&mut t, came_in + TRANSIT_DELAY_TICKS);
    let out = eject_out(&mut t);
    assert_eq!(left(&t, sb), Some(out));
    let tanks = object(&t, sb).tanks.expect("its tanks");
    assert_eq!(
        (tanks.sludge.on, tanks.smoke.on, tanks.sludge.used),
        (false, false, (came_in - 2) * DRAIN)
    );
}

/// The plan's acceptance, "in the network with a Vault, tanks refill 5/s. With three Shadebringers and one Vault, two
/// refill at a time": three Shadebringers emptied on the surface go into the network one after another. The first two
/// in each fill a tick's 5 a second from the tick they went in, the third waiting with its tank empty; the first full,
/// 20 s after it came in, the third takes its place in the next tick, and is full 20 s after that.
#[test]
fn a_vault_refills_two_shadebringers_at_a_time_5_a_second_first_come_first_served() {
    let (mut t, sbs) = network(&[(VAULT, VAULT_AT)], 3);
    emptied(&mut t, &sbs);
    let mut came_in = Vec::new();
    for &sb in &sbs {
        came_in.push(drive_in(&mut t, sb));
    }
    assert!(came_in.windows(2).all(|pair| pair[0] < pair[1]), "{came_in:?}");
    assert_eq!(inside(&t), sbs);
    let used = |t: &TestSim| sbs.iter().map(|&sb| tank(t, sb).used).collect::<Vec<u32>>();
    // The first two fill from the tick each came in; the third from the tick after the first is full.
    let filled = |since: Tick, tick: Tick| CAPACITY.saturating_sub((tick + 1).saturating_sub(since) * REFILL);
    let third = came_in[0] + FULL_FILL;
    while t.sim().tick() < third + FULL_FILL + 4 {
        let tick = t.sim().tick();
        let expected = vec![filled(came_in[0], tick), filled(came_in[1], tick), filled(third, tick)];
        assert_eq!(used(&t), expected, "tick {tick}");
        t.step();
    }
    assert_eq!(used(&t), [0, 0, 0], "all full");
    assert!(came_in[2] < third, "the third waited");
}

/// Design change 0424's Verification, "An empty Shadebringer entering a network with a free Vault and Null Chamber can
/// be ejected fully refilled and buffed after 20 s": in every tick from the one it came in, its Vault turn and its
/// Null Chamber turn each go on, the chamber's done 2 s on, its tank full 20 s on; ejected then, it comes out full and
/// buffed (`BuffKind::NullChamber`).
#[test]
fn an_empty_shadebringer_comes_out_full_and_buffed_20_s_after_it_went_in_served_by_both_at_once() {
    let (mut t, sbs) = network(&[(VAULT, VAULT_AT), (ObjectKind::NullChamber, CHAMBER_AT)], 1);
    let sb = sbs[0];
    emptied(&mut t, &sbs);
    let came_in = drive_in(&mut t, sb);
    let turn = |t: &TestSim| (t.sim().held_unit(sb)).and_then(|held| held.service.chamber.map(|p| p.ticks_done()));
    for k in 0..32 {
        assert_eq!(turn(&t), Some(k + 1), "tick {}: the chamber's turn", t.sim().tick());
        assert_eq!(
            tank(&t, sb).used,
            CAPACITY - (k + 1) * REFILL,
            "and the Vault's, at once"
        );
        t.step();
    }
    assert!(t.sim().held_unit(sb).unwrap().service.charged(), "done 2 s on");
    run_until(&mut t, came_in + FULL_FILL - 1);
    assert_eq!(tank(&t, sb).used, 0, "full 20 s on");
    let out = eject_out(&mut t);
    let o = object(&t, sb);
    assert_eq!(o.tanks.unwrap().sludge.used, 0);
    let buffed = o.buffs.iter().any(|buff| buff.ability == BuffKind::NullChamber);
    assert!(buffed, "out buffed in tick {out}: {:?}", o.buffs);
}

/// Design change 0424's Verification, "Two Shadebringers, the first in empty, the second full: eject one: the first
/// comes out, half-filled if ejected early": the EjectMenu takes the one inside longest, whatever its Vault turn, and
/// ejected 10 s after it came in it comes out with its tank half full; the full one stays inside.
#[test]
fn of_two_shadebringers_the_first_in_empty_comes_out_first_half_filled_if_ejected_early() {
    let (mut t, sbs) = network(&[(VAULT, VAULT_AT)], 2);
    let (empty, full) = (sbs[0], sbs[1]);
    emptied(&mut t, &[empty]);
    let came_in = drive_in(&mut t, empty);
    drive_in(&mut t, full);
    assert_eq!(tank(&t, full).used, 0);
    run_until(&mut t, came_in + FULL_FILL / 2 - 2);
    let out = eject_out(&mut t);
    assert_eq!(out, came_in + FULL_FILL / 2 - 1, "out at once");
    assert_eq!(left(&t, empty), Some(out), "the one inside longest");
    assert_eq!(tank(&t, empty).used, CAPACITY / 2, "half full");
    assert_eq!(inside(&t), [full]);
}
