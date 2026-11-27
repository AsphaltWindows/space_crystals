//! The Aerospace Assembly's production (`factions/syndicate/structures/aerospace_assembly.md`: "Produces (2026-11-25):
//! **Phantom** (requires an Optics Lab)", "**Phantom** 175 SC, 320 f, 2"; plan decision T96: "the Aerospace Assembly is
//! the Hideout's `ExitRule::Network`, and its rally auto-ejects through the doors") and its rally points
//! (`factions/syndicate/structures/README.md > Tunnel Expansions > Rally Point Behavior`, design changes 0246, 0252 and
//! 0253).

use sc_sim::MoveStatus;
use sc_sim::network::Place;

use super::*;

/// The plan's "It trains the Phantom (Q, with an Optics Lab)": without an Optics Lab the Train is refused for it
/// ([`Reject::Requires`]); with one, the click pays 175 Space Crystals and production starts at once, taking the
/// Phantom's Tunnel Space of 2 from that tick. 320 frames on, the Phantom comes into the game inside the network, at
/// its full 70 HP, nowhere on the map, produced, not walked in (design change 0246: "they can exit as soon as they are
/// produced"): an Eject in the next tick takes it out at once, over the air exit door.
#[test]
fn the_aerospace_assembly_trains_a_phantom_into_the_network() {
    let no_lab = started(
        base()
            .spawn(ObjectKind::Forge, Some(P1), SpawnAt::Tile(FORGE_AT))
            .spawn(ObjectKind::AerospaceAssembly, Some(P1), SpawnAt::Tile(AA_AT)),
        3,
    );
    assert_eq!(
        gate(&no_lab, P1, train(id(3), PHANTOM)),
        Err(Reject::Requires(ObjectKind::OpticsLab))
    );
    let mut t = hangar();
    let (used, available) = tunnel_space(&t, P1);
    assert_eq!(used, 0);
    assert_eq!(gate(&t, P1, train(AA, PHANTOM)), Ok(()));
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(AA, PHANTOM));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - PHANTOM_PRICE, "paid at the click");
    assert_eq!(tunnel_space(&t, P1), (2, available), "reserved as production starts");
    let objects = t.sim().objects().count();
    run_checked(&mut t, start + PHANTOM_FRAMES - 2);
    assert!(produced(&t, AA).is_empty(), "not before its 320 frames");
    step_checked(&mut t);
    let done = start + PHANTOM_FRAMES - 1;
    let [(tick, phantom)] = produced(&t, AA)[..] else {
        panic!("one Phantom: {:?}", produced(&t, AA));
    };
    assert_eq!(tick, done);
    assert_eq!((inside(&t), t.sim().objects().count()), (vec![phantom], objects));
    let held = t.sim().held_unit(phantom).expect("in the network");
    assert_eq!((held.place, held.since, held.entered), (Place::Network, done, false));
    assert_eq!(
        (held.object.kind, held.object.owner, held.object.hp),
        (PHANTOM, Some(P1), Some(70.0))
    );
    t.at(done + 1, P1, eject(T1, PHANTOM));
    run_checked(&mut t, done + 1);
    assert_eq!(exits(&t, T1), [(done + 1, phantom)], "out at once");
    assert_eq!(object(&t, phantom).pos, AIR_EXIT, "over the air exit door");
    assert_eq!(tunnel_space(&t, P1), (2, available), "and on the map");
}

/// The plan's acceptance, "A produced Phantom goes into the network and auto-ejects by the air exit door to its rally
/// point" (`factions/syndicate/structures/README.md > Rally Point Behavior`: "**Rally point set on the surface**: Unit
/// auto-ejects from the parent Tunnel (Side A) and moves to the rally point"; design change 0431: "it **rises** out of
/// the exit door over **16 frames (1 s)**"): with a rally point 8 grid units north of the Tunnel, the finished Phantom
/// comes out over the air exit door in the tick it is finished, never by Side A's exit path, its Move to the point
/// given at once; it holds still there through its rise, rising tick by tick, and in the 16th tick after it came out
/// it moves, flying to the point, where it arrives and stops, its order done. The Assembly's next Phantom, rallied to
/// the Tunnel (design change 0252: "the unit stays inside the network"), stays inside.
#[test]
fn a_phantom_rallied_to_the_surface_comes_out_of_the_air_exit_door_and_flies_there_once_risen() {
    let mut t = hangar();
    let point = Vec2::new(948.0, 256.0);
    t.at(2, P1, rally(AA, Target::Point(point)));
    t.at(2, P1, train(AA, PHANTOM));
    let done = 2 + PHANTOM_FRAMES - 1;
    run_checked(&mut t, done);
    let [(_, phantom)] = produced(&t, AA)[..] else {
        panic!("one Phantom: {:?}", produced(&t, AA));
    };
    assert_eq!(
        exits(&t, T1),
        [(done, phantom)],
        "auto-ejected in the tick it is finished"
    );
    assert!(inside(&t).is_empty());
    let o = object(&t, phantom);
    assert_eq!(o.pos, AIR_EXIT, "over the air exit door, not Side A's exit path");
    let given = o.orders.as_ref().and_then(|orders| orders.queue.front());
    assert_eq!(
        given.map(|order| order.kind),
        Some(OrderKind::Move(Target::Point(point))),
        "given at once"
    );
    assert_eq!(
        (rising(&t, phantom), lift(&t, phantom)),
        (Some(0), Some(0.0)),
        "in the door"
    );
    for done_ticks in 1..DOOR_FRAMES {
        step_checked(&mut t);
        assert_eq!(rising(&t, phantom), Some(done_ticks), "tick {}", t.sim().tick());
        assert_eq!(
            object(&t, phantom).pos,
            AIR_EXIT,
            "holding still, tick {}",
            t.sim().tick()
        );
    }
    step_checked(&mut t);
    assert_eq!((rising(&t, phantom), lift(&t, phantom)), (None, None), "risen");
    assert_ne!(
        object(&t, phantom).pos,
        AIR_EXIT,
        "moving in the 16th tick, {}",
        t.sim().tick()
    );
    step_until(&mut t, 400, "the Phantom flies to its rally point", |t| {
        object(t, phantom).pos.distance(point) < 1.0 && t.status(phantom.0) != MoveStatus::Moving
    });
    // Its Move completes in the next tick's behaviours, as any does once its unit stopped at the point.
    step_checked(&mut t);
    let orders = object(&t, phantom).orders.as_ref().unwrap();
    assert!(orders.is_idle(), "{orders:?}");
    // Rallied to the Tunnel, the next stays inside.
    let next = t.sim().tick() + 1;
    t.at(next, P1, rally(AA, Target::Object(T1)));
    t.at(next, P1, train(AA, PHANTOM));
    run_checked(&mut t, next + PHANTOM_FRAMES);
    let second = produced(&t, AA)[1].1;
    assert_eq!(inside(&t), [second]);
    assert_eq!(exits(&t, T1).len(), 1);
}
