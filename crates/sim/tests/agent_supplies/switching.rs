//! Switching resources (design change 0256 point 4, `factions/syndicate/units/agent.md > Gathering > Supplies`: "an
//! Agent carrying Space Crystals that is ordered to gather Supplies goes straight to the station; its crystals are
//! lost when it picks up the Supply. (Assumed the same the other way: a carried Supply is lost when it mines
//! crystals.)"). When the old load goes is design change 0292 point 3's: "the old load is lost when the new pick-up or
//! mining ends, both ways", with the new load; an Agent stopped before then keeps it. And design change 0128 (the
//! Drone's rule, design change 0255): an Agent that carries a Supply turns round at the station and delivers it
//! first.

use super::*;

/// `> Gathering > Space Crystals`: CarryCapacity, 50 Space Crystals per load; MiningDuration, 80 frames.
const CRYSTALS: u32 = 50;
const MINING: Tick = 80;

/// [`field`]'s Tunnel (1) and a station (2) of 100, 3 grid units west of the Supplies hatch; a patch (3) on tile
/// (18, 8), 2 grid units east of the crystal hatch; and an Agent (4) south of the Tunnel.
fn both() -> TestSim {
    flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(12, 6))
        .spawn_with(station_of(100), tile(7, 7))
        .spawn_with(patch_of(10_000), tile(18, 8))
        .spawn(AGENT, Some(P1), centre(14, 12))
        .start()
}

const PATCH: u32 = 3;
const A: u32 = 4;

/// Gathers at `target` from the next tick on, until Agent [`A`] carries something.
fn load_up(t: &mut TestSim, target: u32) {
    let next = t.sim().tick() + 1;
    gather(t, next, &[A], target);
    run_to(t, &[A], &[STATION], 400, "a load", |t| load(t, A).0 > 0);
}

/// 0256's Verification: "An Agent carrying 50 crystals, ordered onto a Supply station, picks up a Supply and no
/// longer carries the crystals." It goes straight there, delivering nothing; it carries its crystals through the
/// pick-up and loses them as the pick-up ends, with the Supply in hand; and it delivers the Supply.
#[test]
fn crystals_are_lost_as_the_supply_pick_up_ends() {
    let mut t = both();
    load_up(&mut t, PATCH);
    assert_eq!(load(&t, A), (CRYSTALS, Resource::Crystals));
    let before = banks(&t, P1);
    let next = t.sim().tick() + 1;
    gather(&mut t, next, &[A], STATION);
    let began = run_to(&mut t, &[A], &[STATION], 400, "straight to the station", |t| {
        assert!(deliveries(t).is_empty(), "tick {}: it delivers first", t.sim().tick());
        is_picking_up(t, A)
    });
    while is_picking_up(&t, A) {
        assert_eq!(load(&t, A), (CRYSTALS, Resource::Crystals), "tick {}", t.sim().tick());
        step(&mut t, &[A], &[STATION]);
    }
    assert_eq!(t.sim().tick() - began + 1, PICK_UP);
    assert_eq!(load(&t, A), (CARRY, Resource::Supplies));
    assert_eq!(supplies(&t, STATION), 99);
    let delivered = run_to(&mut t, &[A], &[STATION], 400, "the delivery", |t| {
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, A, T1, CARRY, Resource::Supplies)]);
    assert_eq!(banks(&t, P1), (before.0, before.1 + 1), "the crystals are lost");
}

/// "The same the other way": an Agent carrying a Supply, ordered onto a patch, goes straight there, mines with the
/// Supply in hand, loses it as the mining ends, and delivers 50 Space Crystals at the crystal hatch.
#[test]
fn a_supply_is_lost_as_mining_ends() {
    let mut t = both();
    load_up(&mut t, STATION);
    assert_eq!(load(&t, A), (CARRY, Resource::Supplies));
    let before = banks(&t, P1);
    let next = t.sim().tick() + 1;
    gather(&mut t, next, &[A], PATCH);
    let began = run_to(&mut t, &[A], &[STATION], 400, "straight to the patch", |t| {
        assert!(deliveries(t).is_empty(), "tick {}: it delivers first", t.sim().tick());
        is_picking_up(t, A)
    });
    while is_picking_up(&t, A) {
        assert_eq!(load(&t, A), (CARRY, Resource::Supplies), "tick {}", t.sim().tick());
        step(&mut t, &[A], &[STATION]);
    }
    assert_eq!(t.sim().tick() - began + 1, MINING);
    assert_eq!(load(&t, A), (CRYSTALS, Resource::Crystals));
    let delivered = run_to(&mut t, &[A], &[STATION], 400, "the delivery", |t| {
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, A, T1, CRYSTALS, Resource::Crystals)]);
    assert_eq!(banks(&t, P1), (before.0 + CRYSTALS, before.1), "the Supply is lost");
}

/// Design change 0292's Verification: "An Agent ordered off a station mid-pick-up still carries its crystals" (point 3:
/// the old load goes as the new pick-up ends). Moved away in the middle of its pick-up, it keeps them (and takes no
/// Supply, design change 0130). A Gather on the patch then turns it round there with its crystals, as any load (0128),
/// and they are delivered.
#[test]
fn an_interrupted_pick_up_keeps_the_old_load() {
    let mut t = both();
    load_up(&mut t, PATCH);
    let before = banks(&t, P1);
    let next = t.sim().tick() + 1;
    gather(&mut t, next, &[A], STATION);
    let began = run_to(&mut t, &[A], &[STATION], 400, "the pick-up", |t| is_picking_up(t, A));
    let away = OrderKind::Move(sc_sim::Target::Point(TilePos::new(8, 13).center()));
    t.order_at(began + 20, P1, &[id(A)], away, false);
    run_to(&mut t, &[A], &[STATION], 30, "off the job", |t| phase(t, A).is_none());
    run_to(&mut t, &[A], &[STATION], 120, "off the station", |t| {
        gap(t, A, STATION) > 2.0 * REACH
    });
    t.run(PICK_UP);
    assert_eq!(load(&t, A), (CRYSTALS, Resource::Crystals), "it keeps its crystals");
    assert_eq!(supplies(&t, STATION), 100);
    let next = t.sim().tick() + 1;
    gather(&mut t, next, &[A], PATCH);
    let delivered = run_to(&mut t, &[A], &[STATION], 600, "the delivery", |t| {
        assert!(!is_picking_up(t, A) || deliveries(t).len() == 1, "it mines with a load");
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, A, T1, CRYSTALS, Resource::Crystals)]);
    assert_eq!(banks(&t, P1), (before.0 + CRYSTALS, before.1));
}

/// Design change 0128 ("Gather while carrying a load", one of the Drone's job rules, design change 0255) at a
/// station: an Agent that carries a Supply, ordered to gather there, picks up nothing; it turns round, delivers its
/// Supply at the Supplies hatch, and comes back for the next.
#[test]
fn an_agent_carrying_a_supply_turns_round_at_the_station() {
    let mut t = both();
    load_up(&mut t, STATION);
    let next = t.sim().tick() + 1;
    t.order_at(next, P1, &[id(A)], OrderKind::Stop, false);
    run_to(&mut t, &[A], &[STATION], 30, "stopped", |t| phase(t, A).is_none());
    assert!(gap(&t, A, STATION) <= 2.0 * REACH, "beside the station");
    assert_eq!(load(&t, A), (CARRY, Resource::Supplies), "a Stop keeps the load");
    let next = t.sim().tick() + 1;
    gather(&mut t, next, &[A], STATION);
    let delivered = run_to(&mut t, &[A], &[STATION], 400, "the delivery", |t| {
        assert!(
            !is_picking_up(t, A),
            "tick {}: it picks up with a Supply",
            t.sim().tick()
        );
        assert_eq!(supplies(t, STATION), 99);
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, A, T1, CARRY, Resource::Supplies)]);
    run_to(&mut t, &[A], &[STATION], 400, "the next pick-up", |t| {
        load(t, A) == (CARRY, Resource::Supplies)
    });
    assert_eq!(supplies(&t, STATION), 98);
}
