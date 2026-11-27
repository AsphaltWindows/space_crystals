//! `factions/gdo/units/drone_operator.md > Drone limits — how a drone ends`: "Leaves the **leash range** around its
//! operator: falls, **explodes** (falls and explodes)"; "Runs out of **battery** (time limit): falls, no explosion";
//! "Its **operator dies**: falls, no explosion" (`crate::tether`; plan decision T91, D33's placeholder: the drone
//! falls in the tick its end comes).

use super::*;

/// Ordered 9 grid units out, edge to edge, the drone flies until it is past its leash of 8 and falls there, in that
/// tick (`Fall::Leash`), exploding where it fell: its blast of 150 over 48 su on the ground units round it, player 1's
/// own Peacekeeper (2) and player 2's (3) alike, each its share by the area rule ([`explosion_share`]), and nothing on
/// player 2's beyond it (4). The operator is free again.
#[test]
fn past_its_leash_the_drone_falls_and_explodes() {
    // Centre to centre, the leash's edge is 536 su east of the operator: its 512 and a Silhouette's 24.
    let edge = LEASH + 24.0;
    let mut t = field(|builder| {
        builder
            .spawn(PK, Some(P1), SpawnAt::Point(from_home(edge + 6.0, 30.0)))
            .spawn(PK, Some(P2), SpawnAt::Point(from_home(edge + 6.0, -30.0)))
            .spawn(PK, Some(P2), SpawnAt::Point(from_home(edge + 120.0, 0.0)))
    });
    let drone = 5;
    next(&mut t, P1, launch(&[1]));
    next(&mut t, P1, move_to(&[drone], from_home(9.0 * SU_PER_GU + 24.0, 0.0)));
    while falls(&t).is_empty() {
        assert!(t.sim().tick() < 200, "no fall\n{}", t.ascii(None));
        t.step();
    }
    let fell_at = t.sim().tick();
    assert_eq!(falls(&t), [(fell_at, id(drone), Fall::Leash)]);
    assert_eq!(died(&t, drone), Some(fell_at));
    assert!(t.sim().object(id(drone)).is_none());
    let [(tick, centre, radius, attacker)] = blasts(&t)[..] else {
        panic!("one blast: {:?}", blasts(&t));
    };
    assert_eq!((tick, radius, attacker), (fell_at, RADIUS, Some(id(drone))));
    let out = centre.x - HOME.x - 24.0;
    assert!(
        out > LEASH && out <= LEASH + 10.0,
        "at the leash's edge, a tick's flight past it: {out}"
    );
    assert!((centre.y - HOME.y).abs() < 1.0, "{centre}");
    for n in [2, 3, 4] {
        let taken: f32 = hits_on(&t, n).iter().map(|&(_, _, damage)| damage).sum();
        let expected = explosion_share(centre, object(t.sim(), n).pos);
        assert!((taken - expected).abs() < 1e-4, "#{n}: {taken}, by the rule {expected}");
    }
    assert!(
        !hits_on(&t, 2).is_empty() && !hits_on(&t, 3).is_empty(),
        "own and enemy"
    );
    assert_eq!(hits_on(&t, 4), [], "beyond the circle");
    assert_eq!(object(t.sim(), 1).drone, None);
    assert_eq!(gate(t.sim(), P1, move_to(&[1], from_home(0.0, 200.0))), Ok(()));
}

/// A drone left hovering falls 320 ticks after its launch, in its battery's tick (`Fall::Battery`), without exploding:
/// player 1's Peacekeeper (2) under it takes nothing. A drone moved past the leash by a debug teleport falls in that
/// tick, and explodes.
#[test]
fn the_battery_runs_out_320_ticks_after_the_launch() {
    let mut t = field(|builder| builder.spawn(PK, Some(P1), SpawnAt::Point(from_home(60.0, 0.0))));
    let drone = 3;
    next(&mut t, P1, launch(&[1]));
    next(&mut t, P1, move_to(&[drone], from_home(60.0, 0.0)));
    t.run_until(2 + BATTERY - 1);
    assert!(t.sim().object(id(drone)).is_some(), "a tick to go");
    t.step();
    assert_eq!(falls(&t), [(2 + BATTERY, id(drone), Fall::Battery)]);
    assert_eq!(died(&t, drone), Some(2 + BATTERY));
    assert_eq!((blasts(&t), hits_on(&t, 2)), (vec![], vec![]), "no explosion");
    assert_eq!(object(t.sim(), 1).drone, None);

    let mut t = field(|builder| builder);
    next(&mut t, P1, launch(&[1]));
    let far = from_home(LEASH + 100.0, 0.0);
    next(
        &mut t,
        P1,
        Command::Debug(DebugCommand::SetPosition { id: id(2), pos: far }),
    );
    assert_eq!(falls(&t), [(3, id(2), Fall::Leash)]);
    assert_eq!(blasts(&t).iter().map(|b| (b.0, b.1)).collect::<Vec<_>>(), [(3, far)]);
}

/// "Its **operator dies**: falls, no explosion": player 2's three Peacekeepers (2 to 4) shoot the operator; the drone
/// (6), hovering over player 2's fourth (5), falls in the tick its operator dies (`Fall::Operator`), and nothing
/// explodes. An operator removed from the game takes its drone along in the same way.
#[test]
fn the_drone_falls_with_its_operator() {
    let mut t = field(|mut builder| {
        for dy in [-40.0, 0.0, 40.0] {
            builder = builder.spawn(PK, Some(P2), SpawnAt::Point(from_home(-150.0, dy)));
        }
        builder.spawn(PK, Some(P2), SpawnAt::Point(from_home(150.0, 0.0)))
    });
    let drone = 6;
    next(&mut t, P1, launch(&[1]));
    next(&mut t, P1, move_to(&[drone], from_home(150.0, 0.0)));
    next(&mut t, P2, order(&[2, 3, 4], OrderKind::Attack(id(1))));
    while died(&t, 1).is_none() {
        assert!(
            t.sim().tick() < 2 + BATTERY - 1,
            "the operator stands\n{}",
            t.ascii(None)
        );
        t.step();
    }
    let gone = died(&t, 1).expect("dead");
    assert_eq!(falls(&t), [(gone, id(drone), Fall::Operator)]);
    assert_eq!(died(&t, drone), Some(gone));
    assert!(t.sim().object(id(drone)).is_none());
    assert_eq!((blasts(&t), hits_on(&t, 5)), (vec![], vec![]), "no explosion");

    let mut t = field(|builder| builder);
    next(&mut t, P1, launch(&[1]));
    next(&mut t, P1, Command::Debug(DebugCommand::Remove { id: id(1) }));
    assert_eq!(falls(&t), [(3, id(2), Fall::Operator)]);
    assert!(t.sim().object(id(2)).is_none());
}
