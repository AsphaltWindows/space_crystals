//! Place, through the placement check (`sc_sim::placement`) as it applies (plan decision T4): a valid Place, a
//! refused one and why, and a Place checked against what the tick's earlier commands left.

use super::*;

/// Plan decision T4: checks that the gate refuses a Place of constructor `n`'s ready building with the top-left tile
/// of its footprint at `origin`, turned by `orientation`, for `first` (the check's first problem); then runs the next
/// tick with it and checks that the building stays ready and that nothing changed but the `PlacementRefused` event,
/// the tick's first, which lists every problem of the check. Returns those.
fn assert_unplaceable(
    t: &mut TestSim,
    n: u32,
    origin: TilePos,
    orientation: BuildingOrientation,
    first: PlacementProblem,
) -> PlacementCheck {
    let kind = state(t, n).ready().expect("a ready building");
    let problems = t.sim().placement_check(P1, kind, origin, orientation);
    assert_eq!(problems.first_problem(), Some(first), "{problems:?}");
    let command = construction(&[n], Place { origin, orientation });
    let cmd = PlayerCommand {
        player: P1,
        command: command.clone(),
    };
    assert_eq!(t.sim().validate(&cmd), Err(Reject::Unplaceable(first)));
    let mut without = t.sim().clone();
    without.step(&TickInputs::default());
    next(t, command);
    assert_eq!(t.sim().state_hash(), without.state_hash(), "{first:?}");
    let refusal = SimEvent::PlacementRefused {
        constructor: id(n),
        kind,
        origin,
        orientation,
        problems: problems.clone(),
    };
    let events: Vec<SimEvent> = std::iter::once(refusal)
        .chain(without.events().iter().cloned())
        .collect();
    assert_eq!(t.sim().events(), events.as_slice());
    assert_eq!(state(t, n), Constructor::Ready { kind }, "still ready to place");
    problems
}

/// The plan's acceptance, a valid Place: the structure is on the map in its orientation, centred on its footprint,
/// with `Spawned` and `StructurePlaced`; the constructor is idle again; power and the build area follow at once
/// (design changes 0089, 0092), the true grids block it in its tick, and so do its owner's planning grids at the end
/// of it (`ARCHITECTURE.md > Navigation > Path knowledge`).
#[test]
fn a_placed_building_is_on_the_map_in_its_orientation() {
    // A scout, Peacekeeper 5, sees the edge of the area (x 21), where the Barracks goes.
    let mut t = ScenarioBuilder::new(&flat(32, 16))
        .spawn(DeploymentCenter, Some(P1), tile(6, 1)) // 1
        .spawn(ExtractionFacility, Some(P1), tile(11, 1)) // 2
        .spawn(Peacekeeper, Some(P1), SpawnAt::Point(at(22, 12).center())) // 3
        .spawn(Peacekeeper, Some(P2), SpawnAt::Point(at(30, 14).center())) // 4
        .start();
    t.without_auto_targeting()
        .debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], Construct(Barracks)));
    until_ready(&mut t, 1);
    let turned = BuildingOrientation {
        rotation: Rotation::R90,
        flip_h: false,
        flip_v: false,
    };
    // Turned, it covers x 21 and 22, y 8 to 10: one column in the area.
    assert!(t.sim().placement_check(P1, Barracks, at(21, 8), turned).is_valid());
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (30, 15));
    let area = t.sim().build_area(P1);
    assert!(area.contains(at(21, 8)) && !area.contains(at(24, 9)));
    let grid = |sim: &Sim| {
        sim.nav()
            .grid(NavClass::GroundRugged, 1)
            .unwrap()
            .is_blocked(at(22, 10))
    };
    assert!(!grid(t.sim()));
    next(
        &mut t,
        construction(
            &[1],
            Place {
                origin: at(21, 8),
                orientation: turned,
            },
        ),
    );
    let barracks = id(5);
    assert_eq!(
        t.sim().events()[..2],
        [
            SimEvent::Spawned(barracks),
            SimEvent::StructurePlaced {
                constructor: id(1),
                structure: barracks
            }
        ]
    );
    let o = t.sim().object(barracks).unwrap();
    assert_eq!((o.kind, o.owner), (Barracks, Some(P1)));
    let placement = Placement {
        origin: at(21, 8),
        w: 2,
        h: 3,
        rotation: Rotation::R90,
        flip_h: false,
        flip_v: false,
    };
    assert_eq!(o.placement, Some(placement));
    assert_eq!(o.pos, placement.center());
    assert!(state(&t, 1).is_idle());
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (30, 45));
    // Its BuildRadiusExtension of 2: x 19 to 24, y 6 to 12.
    let area = t.sim().build_area(P1);
    assert!(area.contains(at(24, 9)) && area.contains(at(24, 12)) && !area.contains(at(25, 9)));
    assert!(grid(t.sim()));
    let known = t.sim().known_nav(P1).unwrap();
    assert!(known.grid(NavClass::GroundRugged, 1).unwrap().is_blocked(at(22, 10)));
    let unseen = t.sim().known_nav(P2).unwrap();
    assert!(
        !unseen.grid(NavClass::GroundRugged, 1).unwrap().is_blocked(at(22, 10)),
        "player 2 hasn't seen it"
    );
}

/// The plan's acceptance, an invalid Place, for each problem of the placement check (`crate::placement`): the
/// command is refused, the building stays ready, and the event lists the problems (plan decision T4). The check's
/// `NotAStructure` never comes up: a ready building is a structure. Then a valid Place of each: the plate goes onto
/// the patch and mines it, 48 ticks after its placement.
#[test]
fn a_refused_place_keeps_the_building_ready_and_says_why() {
    use PlacementProblem::{Tile, Whole};
    use TileProblem::{NotBuildable, NotFlat, NotVisible, OffMap, Resource, Structure, Unit};
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], Construct(Generator)))
        .at(1, P1, construction(&[2], Construct(ExtractionPlate)));
    until_ready(&mut t, 2);
    let unturned = BuildingOrientation::default();
    let cases = [
        (1, at(8, -1), Tile(at(8, -1), OffMap)),
        (1, at(3, 3), Tile(at(3, 4), NotBuildable)),
        (1, at(9, 6), Tile(at(9, 6), NotFlat)),
        (1, at(14, 10), Tile(at(14, 10), NotVisible)),
        (1, at(12, 3), Tile(at(12, 3), Structure)),
        (1, at(5, 6), Tile(at(5, 7), Resource)),
        (1, at(3, 0), Tile(at(3, 1), Unit)),
        (1, at(22, 2), Whole(WholeProblem::OutsideBuildArea)),
        (2, at(5, 5), Whole(WholeProblem::NotOnPatch)),
    ];
    for (n, origin, first) in cases {
        let problems = assert_unplaceable(&mut t, n, origin, unturned, first);
        // Only the problem looked for, but for the unseen tiles that come with a spot outside the build area.
        let alone = match first {
            Tile(_, problem) => problems.whole.is_empty() && problems.tiles.iter().all(|&(_, p)| p == problem),
            Whole(problem) => {
                let outside = problem == WholeProblem::OutsideBuildArea;
                problems.whole == [problem] && (problems.tiles.is_empty() || outside)
            }
        };
        assert!(alone, "{first:?}: {problems:?}");
    }
    let bank_before = bank(&t, P1);
    next(&mut t, construction(&[1], place(6, 5)));
    next(&mut t, construction(&[2], place(5, 7)));
    let placed = t.sim().tick();
    assert!(state(&t, 1).is_idle() && state(&t, 2).is_idle());
    let generator = t.sim().object(id(5)).unwrap();
    assert_eq!(
        (generator.kind, generator.placement.unwrap().origin),
        (Generator, at(6, 5))
    );
    let plate = t.sim().object(id(6)).unwrap();
    assert_eq!(plate.extractor.map(|e| e.patch), Some(id(3)), "it mines the patch");
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (50, 18));
    assert_eq!(bank(&t, P1), bank_before);
    t.run(48);
    assert_eq!(
        bank(&t, P1),
        bank_before + 10,
        "first load 48 ticks after tick {placed}"
    );
}

/// Plan decision T4: a Place is checked when it applies, against the state the tick's earlier commands left. Two
/// Deployment Centers place their Generators on the same spot in one tick: both pass the gate before the tick, the
/// first goes in, and the second is refused, its Generator still ready.
#[test]
fn a_place_is_checked_when_it_applies() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 1
        .spawn(DeploymentCenter, Some(P1), tile(9, 1)) // 2
        .start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], Construct(Generator)))
        .at(1, P1, construction(&[2], Construct(Generator)));
    until_ready(&mut t, 2);
    let commands = [construction(&[1], place(6, 5)), construction(&[2], place(6, 5))];
    for command in &commands {
        let cmd = PlayerCommand {
            player: P1,
            command: command.clone(),
        };
        assert_eq!(t.sim().validate(&cmd), Ok(()));
    }
    let tick = t.sim().tick() + 1;
    for command in commands {
        t.at(tick, P1, command);
    }
    t.step();
    assert!(state(&t, 1).is_idle());
    assert_eq!(state(&t, 2), Constructor::Ready { kind: Generator });
    let refusals: Vec<&SimEvent> = t
        .sim()
        .events()
        .iter()
        .filter(|e| matches!(e, SimEvent::PlacementRefused { .. }))
        .collect();
    let [
        SimEvent::PlacementRefused {
            constructor, problems, ..
        },
    ] = refusals[..]
    else {
        panic!("one refusal: {refusals:?}");
    };
    assert_eq!(*constructor, id(2));
    let covered = [at(6, 5), at(7, 5), at(6, 6), at(7, 6)].map(|tile| (tile, TileProblem::Structure));
    assert_eq!(
        (problems.tiles.as_slice(), problems.whole.as_slice()),
        (&covered[..], &[][..])
    );
}
