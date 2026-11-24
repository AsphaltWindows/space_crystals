//! Which patches count: only those the player knows, the map's from the start (design changes 0131 and 0159), never
//! a plated one (0131), and a plate placed on a patch a Drone mines (0137).

use sc_sim::{BuildingOrientation, ConstructionCommand, DebugCommand};

use super::*;

/// What player 1 knows of patch F, in [`field`].
#[derive(Copy, Clone, Debug, PartialEq)]
enum Known {
    /// Nothing: F is placed in the fog in tick 1, and never seen.
    Never,
    /// Placed by the map, never seen: known from the start (design change 0159).
    FromTheStart,
    /// Seen by a Drone that was there for a tick, and remembered since.
    Remembered,
    /// Remembered, and under player 2's plate.
    Plated,
}

/// Patch A (1) holds 8, node 2, Drone 3 south of A; patch F (4), 5.3 grid units from where the Drone mines A: out
/// of its sight (5 grid units, `core/vision.md`: a tile is seen when its centre is within the SightRange) and inside
/// its search radius (6). With [`Known::Never`] a debug command places F in tick 1, else the map does. With
/// [`Known::Plated`], player 2's plate 5 covers F. With [`Known::Remembered`] and [`Known::Plated`], a Drone of
/// player 1's (the next id) stands beside F in tick 1 and is removed in tick 2.
fn field(known: Known) -> TestSim {
    let mut scenario = flat(16, 10)
        .spawn_with(patch_of(8), tile(4, 3))
        .spawn(NODE, Some(P1), tile(4, 6))
        .spawn(DRONE, Some(P1), centre(4, 5));
    if known != Known::Never {
        scenario = scenario.spawn(PATCH, None, tile(9, 2));
    }
    if known == Known::Plated {
        scenario = scenario.spawn(ObjectKind::ExtractionPlate, Some(P2), tile(9, 2));
    }
    let mut t = scenario.start();
    if known == Known::Never {
        let at = tile(9, 2);
        t.debug_at(
            1,
            DebugCommand::Spawn {
                kind: PATCH,
                owner: None,
                at,
                orientation: Default::default(),
            },
        );
    }
    if matches!(known, Known::Remembered | Known::Plated) {
        let scout = t.sim().objects().map(|o| o.id.0).max().expect("objects") + 1;
        t.spawn_unit_at(1, DRONE, P1, TilePos::new(9, 3).center());
        t.debug_at(2, DebugCommand::Remove { id: id(scout) });
    }
    t
}

/// Design change 0131 (`> Patch Occupancy > Search radius`; its Verification: "A patch within 6 GU that the player
/// has never seen is never chosen; a remembered one is. A plated patch is never a redirect target"), with design
/// change 0159 (the map's patches are known from the start, so a patch "never seen" is one placed later): the Drone
/// mines A out and looks round where it stands; with no patch it may go on to, its Gather ends (0123).
#[test]
fn only_a_known_uncovered_patch_is_one_to_go_on_to() {
    for known in [Known::Never, Known::FromTheStart, Known::Remembered, Known::Plated] {
        let mut t = field(known);
        gather(&mut t, 3, P1, &[3], 1);
        run_to(&mut t, 100, "A is mined out", |t| !exists(t, 1));
        let sight = t.sim().vision().player(P1).expect("player 1");
        assert!(!t.sim().is_visible(P1, object(&t, 4)), "{known:?}: F is out of sight");
        assert_eq!(
            sight.remembered_record(id(4)).is_some(),
            known != Known::Never,
            "{known:?}: what player 1 remembers"
        );
        t.step();
        if matches!(known, Known::FromTheStart | Known::Remembered) {
            assert_eq!(
                order(&t, 3),
                Some(OrderKind::Gather(id(4))),
                "{known:?}: on to F, after the delivery"
            );
            run_to(&mut t, 300, "it mines F", |t| mines(t, 3, 4));
            continue;
        }
        assert_eq!(
            (order(&t, 3), phase(&t, 3)),
            (Some(OrderKind::Gather(id(1))), Some(GatherPhase::ToDropOff)),
            "{known:?}: nothing found, it delivers before its Gather ends"
        );
        run_to(&mut t, 200, "it idles", |t| order(t, 3).is_none());
        assert_eq!(bank(&t, P1), BANK + LOAD, "{known:?}");
        t.run(100);
        assert_eq!(order(&t, 3), None, "{known:?}");
    }
}

/// Design change 0137 (`> Patch Occupancy > Patches under a GDO Extraction Plate`; its Verification: "Plate a patch
/// a Colonists Drone is mining: the plate goes down, the Drone stops mid-cycle with no load and moves to another
/// patch"): the Drone beside the patch doesn't stop the placement, and from then on the patch counts as gone for
/// it, as one that ran out (0134); the mining is interrupted and collects nothing (0130).
#[test]
fn a_plate_placed_on_a_patch_a_drone_mines_goes_down_and_the_drone_moves_on() {
    // Player 1: patch A (1), patch B (2) two tiles west of it, node 3, Drone 4. Player 2: a Deployment Center, 5,
    // whose build area and sight take in the patches, and an Extraction Facility, 6.
    let mut t = flat(20, 12)
        .spawn(PATCH, None, tile(8, 2))
        .spawn(PATCH, None, tile(6, 2))
        .spawn(NODE, Some(P1), tile(7, 5))
        .spawn(DRONE, Some(P1), centre(8, 4))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(12, 1))
        .spawn(ObjectKind::ExtractionFacility, Some(P2), tile(16, 7))
        .start();
    let construct = |command| Command::Construction {
        objects: vec![id(6)],
        command,
    };
    t.at(
        1,
        P2,
        construct(ConstructionCommand::Construct(ObjectKind::ExtractionPlate)),
    );
    gather(&mut t, 1, P1, &[4], 1);
    run_to(&mut t, 2000, "the plate is ready", |t| {
        object(t, 6).constructor.and_then(|c| c.ready()).is_some()
    });
    run_to(&mut t, 200, "the Drone is half-way through a mining of A", |t| {
        matches!(phase(t, 4), Some(GatherPhase::Mining { done: 16 })) && order(t, 4) == Some(OrderKind::Gather(id(1)))
    });
    let place = PlayerCommand {
        player: P2,
        command: construct(ConstructionCommand::Place {
            origin: TilePos::new(8, 2),
            orientation: BuildingOrientation::default(),
        }),
    };
    assert_eq!(
        t.sim().validate(&place),
        Ok(()),
        "the Drone beside the patch doesn't stop the plate"
    );
    let (before, delivered) = (remaining(&t, 1), deliveries(&t).len());
    let now = t.sim().tick();
    t.at(now + 1, P2, place.command);
    t.step();
    let plate = t
        .sim()
        .objects()
        .find(|o| o.kind == ObjectKind::ExtractionPlate)
        .expect("the plate went down");
    assert_eq!(plate.extractor.map(|e| e.patch), Some(id(1)), "on A");
    assert!(!is_mining(&t, 4), "the mining is interrupted");
    assert_eq!(load(&t, 4), 0, "and collects nothing");
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(2))), "on to B");
    run_to(&mut t, 100, "it mines B", |t| mines(t, 4, 2));
    assert!(remaining(&t, 1) <= before, "only the plate mines A now");
    assert_eq!(deliveries(&t).len(), delivered);
}
