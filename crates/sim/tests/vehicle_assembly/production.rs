//! The Vehicle Assembly's production (`factions/syndicate/structures/vehicle_assembly.md > Placeholder stats`:
//! "**Assault Bike** 125 SC, 240 f, 2", "Panel as the Hideout"; plan decision T100: "Syndicate vehicles go into the
//! network (`ExitRule::Network`)") and its rally points (`factions/syndicate/structures/README.md > Tunnel Expansions
//! > Rally Point Behavior`, design changes 0246, 0252 and 0253).

use sc_sim::network::Place;
use sc_sim::{ArmyResource, MoveStatus};

use super::*;

/// The plan's acceptance, "It is produced into the network, reserving Tunnel Space 2 at the start": the Train pays
/// its 125 Space Crystals at the click, and production starts at once, taking the Bike's Tunnel Space of 2 from that
/// tick (`factions/syndicate/overview.md > SyndicateResources > Tunnel Space`, design change 0143: "Reserved when
/// production starts"). 240 frames on, the Bike comes into the game inside the network, at its full 100 HP, nowhere
/// on the map, produced, not walked in (design change 0246: "they can exit as soon as they are produced"), and keeps
/// its 2: an Eject in the next tick takes it out.
#[test]
fn the_vehicle_assembly_trains_a_bike_into_the_network() {
    let mut t = assembly();
    let (used, available) = tunnel_space(&t, P1);
    assert_eq!(used, 0);
    assert_eq!(gate(&t, P1, train_bike(VA)), Ok(()));
    let start = t.sim().tick() + 1;
    t.at(start, P1, train_bike(VA));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - BIKE_PRICE, "paid at the click");
    assert_eq!(tunnel_space(&t, P1), (2, available), "reserved as production starts");
    let status = t.sim().production_status(VA).expect("a producer");
    assert_eq!((status.entry.unit(), status.blocked), (Some(BIKE), None));
    let objects = t.sim().objects().count();
    run_checked(&mut t, start + BIKE_FRAMES - 2);
    assert!(produced(&t, VA).is_empty(), "not before its 240 frames");
    step_checked(&mut t);
    let done = start + BIKE_FRAMES - 1;
    let [(tick, bike)] = produced(&t, VA)[..] else {
        panic!("one Bike: {:?}", produced(&t, VA));
    };
    assert_eq!(tick, done);
    assert_eq!((inside(&t), t.sim().objects().count()), (vec![bike], objects));
    let held = t.sim().held_unit(bike).expect("in the network");
    assert_eq!((held.place, held.since, held.entered), (Place::Network, done, false));
    assert_eq!(
        (held.object.kind, held.object.owner, held.object.hp),
        (BIKE, Some(P1), Some(100.0))
    );
    assert_eq!(tunnel_space(&t, P1), (2, available), "the Bike's 2, inside");
    assert!(!(t.events().iter()).any(|(_, e)| *e == SimEvent::Spawned(bike)));
    t.at(done + 1, P1, eject(T1, BIKE));
    run_checked(&mut t, done + 1);
    assert_eq!(exits(&t, T1), [(done + 1, bike)], "out at once");
    assert_eq!(tunnel_space(&t, P1), (2, available), "and on the map");
}

/// Design change 0143 for the Bike: "it does **not** check or reserve" Tunnel Space when queued, but production
/// starts only when the Bike's 2 fit. With player 1's units on the map using all of their Tunnel Space but 1, the
/// Bike is queued and paid for, and waits at 0% for Tunnel Space (`ProductionStatus::blocked`); one Agent gone, it
/// starts, and is finished 240 frames later.
#[test]
fn a_bike_starts_only_when_its_tunnel_space_of_2_fits() {
    let mut t = assembly();
    let (_, available) = tunnel_space(&t, P1);
    let first = next_id(&t);
    for i in 0..available - 1 {
        let at = Vec2::new(320.0 + 48.0 * (i % 12) as f32, 1100.0 + 48.0 * (i / 12) as f32);
        t.spawn_unit_at(2, ObjectKind::Agent, P1, at);
    }
    t.at(3, P1, train_bike(VA));
    run_checked(&mut t, 40);
    assert_eq!(tunnel_space(&t, P1), (available - 1, available));
    assert_eq!(bank(&t, P1), BANK - BIKE_PRICE, "queued and paid");
    let status = t.sim().production_status(VA).expect("a producer");
    assert_eq!(
        (status.entry.unit(), status.progress, status.blocked),
        (Some(BIKE), 0, Some(ArmyResource::TunnelSpace))
    );
    t.debug_at(41, DebugCommand::Remove { id: id(first) });
    run_checked(&mut t, 41);
    assert_eq!(tunnel_space(&t, P1), (available, available), "the Bike's 2 reserved");
    run_checked(&mut t, 41 + BIKE_FRAMES);
    assert_eq!(produced(&t, VA).len(), 1);
}

/// `factions/syndicate/structures/README.md > Tunnel Expansions > Rally Point Behavior`, for the Bike: "**No rally
/// point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network"; "**Rally point on any own
/// Tunnel**: the unit stays inside the network" (design change 0252); "**Rally point set on the surface**: Unit
/// auto-ejects from the parent Tunnel (Side A) and moves to the rally point"; "**Rally point on an expansion**: treated
/// as the surface location above it" (a Move to its centre). Out of the parent Tunnel, the Tier 2 [`T1`], in the tick
/// it is finished, on the exit path's spot; then it drives there and stops.
#[test]
fn the_bikes_rally_point_keeps_it_inside_or_sends_it_out() {
    let other = id(4);
    let surface = Vec2::new(400.0, 300.0);
    let plant_centre = object(&assembly(), CP).pos;
    let cases = [
        (None, None),
        (Some(Target::Object(T1)), None),
        (Some(Target::Object(other)), None),
        (Some(Target::Point(surface)), Some(surface)),
        (Some(Target::Object(CP)), Some(plant_centre)),
    ];
    for (target, goes_to) in cases {
        let mut t = started(
            plant()
                .spawn(ASSEMBLY, Some(P1), SpawnAt::Tile(VA_AT))
                .spawn(TUNNEL, Some(P1), tile(30, 4)),
            2,
        );
        if let Some(target) = target {
            t.at(2, P1, rally(VA, target));
        }
        t.at(3, P1, train_bike(VA));
        let done = 3 + BIKE_FRAMES - 1;
        run_checked(&mut t, done);
        let [(_, bike)] = produced(&t, VA)[..] else {
            panic!("{target:?}: one Bike");
        };
        match goes_to {
            None => {
                assert_eq!(inside(&t), [bike], "{target:?}: inside");
                assert_eq!(exits(&t, T1), [], "{target:?}");
            }
            Some(to) => {
                assert_eq!(exits(&t, T1), [(done, bike)], "{target:?}: out of the parent Tunnel");
                assert_eq!(object(&t, bike).pos, BIKE_OUT, "{target:?}");
                run_checked(&mut t, done + 1);
                let current = object(&t, bike).orders.clone().unwrap().current.unwrap();
                assert_eq!(current.order.kind, OrderKind::Move(Target::Point(to)), "{target:?}");
                step_until(&mut t, 400, "the Bike stops", |t| {
                    t.status(bike.0) != MoveStatus::Moving && t.sim().tick() > done + 2
                });
                let o = object(&t, bike);
                assert!(o.pos.distance(to) < 8.0, "{target:?}: at {}, not {to}", o.pos);
            }
        }
    }
}
