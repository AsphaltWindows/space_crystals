//! Cancel Deployment (`factions/colonists/economy.md > Beacon Deployment > Cancel Deployment`): back to the
//! Drone with the fraction of its HP and of its Shields (design change 0111), and its Beacon Capacity taken
//! back even over the cap (0112).

use super::*;

/// Design change 0109 ("On cancel, it turns back into the Drone, standing at the centre of the footprint") and 0111's
/// Verification ("Cancel a deploy at 90/120 → Drone 15/20"; "Cancelling never kills: the result is at least 1 HP"),
/// with `> Cancel Deployment`: "the unit reverts to being a normal, fully-functional Drone". 90/120 is 75%, which a
/// deploying Beacon has at 52.5 of its 70 halfway (24 frames: 20 + 100 × 0.5); its HP is set by editing a save, as no
/// command deals an exact amount. Cancelled there it is a Drone at 15/20; cancelled with 0.5 of 70 it is a Drone at
/// 1; cancelled undamaged it is a Drone at 20/20, "no HP/Shields loss". Each time it is the same object, at the
/// centre of the footprint, idle, its 1 Beacon Capacity back, the footprint free, and nothing is refunded (the
/// deployment cost nothing).
#[test]
fn a_cancel_turns_it_back_into_the_drone_with_the_fraction_of_its_hp() {
    let mut t = base().start();
    let start = deployed(&mut t);
    t.run_until(start + 23);
    let o = object(&t, 2);
    assert_eq!((o.max_hp(), o.hp), (Some(70.0), Some(70.0)), "halfway");
    assert_eq!(capacity(&t, P1), (0, 5));
    let centre = deploy::site(SITE).center();
    for (hp, drone) in [(52.5, 15.0), (0.5, 1.0), (70.0, 20.0)] {
        let mut sim = edited(t.sim(), 2, "hp", serde_json::json!(hp));
        let cmd = PlayerCommand {
            player: P1,
            command: cancel(&[2]),
        };
        assert_eq!(sim.validate(&cmd), Ok(()));
        sim.step(&TickInputs { commands: vec![cmd] });
        assert_eq!(
            sim.events()[..2],
            [
                SimEvent::TransformCancelled {
                    id: id(2),
                    kind: Beacon,
                    refund: 0
                },
                SimEvent::KindChanged {
                    id: id(2),
                    from: Beacon,
                    to: Drone
                },
            ]
        );
        let o = sim.object(id(2)).unwrap();
        assert_eq!(
            (o.kind, o.hp, o.max_hp()),
            (Drone, Some(drone), Some(20.0)),
            "from {hp}"
        );
        assert_eq!((o.pos, o.placement, o.transform.as_ref()), (centre, None, None));
        assert!(o.orders.as_ref().is_some_and(|orders| orders.is_idle()));
        assert!(o.mover.is_some() && o.worker.is_some());
        let cap = sim.army_cap(P1).unwrap();
        assert_eq!((cap.used, cap.available), (1, 5), "the Drone's 1 is back");
        // The footprint's tiles are free again: a unit can be put on each of them.
        for tile in deploy::site(SITE).tiles() {
            let spawn = PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Spawn {
                    kind: Peacekeeper,
                    owner: Some(P2),
                    at: SpawnAt::Point(tile.center()),
                    orientation: Default::default(),
                }),
            };
            assert_ne!(sim.validate(&spawn), Err(Reject::Blocked), "{tile:?}");
        }
    }
}

/// The Shields of a deployment (`crate::shields`, slice M3-B; `> HP/Shields Interpolation`: "Shields stay at 10", the
/// Drone's MaxShields and the Beacon's). Drone 2, its Shields set to 0 a tile from its site's centre (edited into a
/// save, as no command deals an exact amount), begins deploying with what it had, not the Beacon's full 10, and they go
/// on regenerating, 0.25 a frame up to the maximum now (`systems/combat.md > Shields > ShieldRegenRate`, design changes
/// 0110 and 0146). Design change 0111, "Shields by the same fraction (no minimum needed)": cancelled halfway with 4 of
/// its 10, it is a Drone with 4 of its 10, and with none, a Drone with none, unlike the HP's 1. Each is 0.25 more at
/// the end of the cancel's tick, as Shields regenerate after the commands.
#[test]
fn a_deployment_keeps_the_shields_and_a_cancel_their_fraction() {
    let shields = |sim: &Sim| sim.object(id(2)).unwrap().shields.unwrap();
    let mut t = base().start();
    t.step();
    next(&mut t, order(&[2], deploy_at(SITE), false));
    let centre = deploy::site(SITE).center();
    while object(&t, 2).pos.distance(centre) > SU_PER_GU {
        assert!(t.sim().tick() < 600, "it gets there");
        t.step();
    }
    assert_eq!(object(&t, 2).kind, Drone, "still walking");
    let mut sim = edited(t.sim(), 2, "shields", serde_json::json!(0.0));
    let mut before = shields(&sim);
    while sim.object(id(2)).unwrap().kind == Drone {
        assert!(sim.tick() < 600, "it deploys");
        before = shields(&sim);
        sim.step(&TickInputs::default());
    }
    assert!(
        before < 9.75,
        "the walk is shorter than the Shields take to come back: {before}"
    );
    assert_eq!(shields(&sim), before + 0.25, "kept, and regenerating");
    sim.step(&TickInputs::default());
    assert_eq!(shields(&sim), before + 0.5);
    assert_eq!(sim.object(id(2)).unwrap().max_shields(), Some(10.0));

    let mut t = base().start();
    let start = deployed(&mut t);
    t.run_until(start + 23);
    let o = object(&t, 2);
    assert_eq!((o.max_shields(), o.shields), (Some(10.0), Some(10.0)), "halfway");
    for (beacon, drone) in [(4.0, 4.0), (0.0, 0.0)] {
        let mut sim = edited(t.sim(), 2, "shields", serde_json::json!(beacon));
        let cmd = PlayerCommand {
            player: P1,
            command: cancel(&[2]),
        };
        sim.step(&TickInputs { commands: vec![cmd] });
        let o = sim.object(id(2)).unwrap();
        assert_eq!(
            (o.kind, o.shields, o.max_shields()),
            (Drone, Some(drone + 0.25), Some(10.0)),
            "from {beacon}"
        );
    }
}

/// Design change 0112's Verification, as 0143 restates it ("0112's example now reads 'a new Drone starts
/// production'"; the production side is slice M3-P's): at 10/10 a Drone deploys (9/10), a new Drone comes (spawned
/// here: 10/10), then the deploy is cancelled → 11/10. "Cancelling takes the 1 back, even if that puts the player over
/// capacity": the cancel is never refused, and nothing is removed for being over.
#[test]
fn a_cancel_takes_the_drones_capacity_back_even_over_the_cap() {
    let mut t = at_capacity().start();
    let start = deployed(&mut t);
    assert_eq!(capacity(&t, P1), (9, 10));
    t.spawn_unit_at(start + 1, Drone, P1, at(14, 16).center()).step();
    assert_eq!(capacity(&t, P1), (10, 10), "a new Drone");
    let objects = t.sim().objects().count();
    let cmd = PlayerCommand {
        player: P1,
        command: cancel(&[2]),
    };
    assert_eq!(t.sim().validate(&cmd), Ok(()), "never refused for the capacity");
    next(&mut t, cmd.command);
    assert_eq!(object(&t, 2).kind, Drone);
    assert_eq!(capacity(&t, P1), (11, 10), "over capacity");
    t.run(20);
    assert_eq!(t.sim().objects().count(), objects, "nothing removed");
    assert_eq!(capacity(&t, P1), (11, 10));
}
