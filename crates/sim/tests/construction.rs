//! Construction (slice M2-D1; `factions/gdo/structures.md > DeploymentCenter`, `factions/gdo/economy.md >
//! ExtractionFacility`): the Deployment Center and the Extraction Facility build one building at a time and hold it
//! ready to place. One test per acceptance point of the plan (`artifacts/director/m2-plan.md` §3.3 D1) and per
//! verification point of design changes 0084 (a destroyed constructor refunds as if cancelled) and 0089 (the power
//! penalty slows the Extraction Facility, never the Deployment Center); Place through the placement check as it
//! applies (plan decision T4); the gate's rules, fuzzed; saves and replays; and the clean Barracks opening as a
//! mini-oracle. `TestSim` checks the invariants after every tick.

use sc_sim::economy::{RATIO_ONE, Refund};
use sc_sim::nav::NavClass;
use sc_sim::orders::MAX_ORDER_UNITS;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, Constructor, DebugCommand, ObjectId, ObjectKind, OrderKind,
    Placement, PlacementCheck, PlacementProblem, PlayerCommand, PlayerId, Reject, Replay, Rng, Rotation, Sim,
    SimConfig, SimEvent, SpawnAt, Tick, TickInputs, TilePos, TileProblem, WholeProblem,
};

use ConstructionCommand::{CancelConstruction, CancelReady, Construct, Place};
use ObjectKind::{
    Barracks, CrystalPatch, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator, Peacekeeper,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn construction(objects: &[u32], command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: objects.iter().map(|&n| ObjectId(n)).collect(),
        command,
    }
}

fn place(x: i32, y: i32) -> ConstructionCommand {
    Place {
        origin: at(x, y),
        orientation: BuildingOrientation::default(),
    }
}

/// Sets `player`'s bank to `crystals` (no Supplies).
fn fund(player: PlayerId, crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
    }
}

fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).unwrap().crystals
}

/// Constructor `n`'s state. Panics if it is gone or no constructor.
fn state(t: &TestSim, n: u32) -> Constructor {
    t.sim()
        .object(id(n))
        .and_then(|o| o.constructor)
        .expect("a constructor")
}

/// Schedules player 1's `command` for the next tick and runs it.
fn next(t: &mut TestSim, command: Command) -> &mut TestSim {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).step()
}

/// Runs until constructor `n` holds a building ready to place.
fn until_ready(t: &mut TestSim, n: u32) -> ObjectKind {
    for _ in 0..2000 {
        if let Some(kind) = state(t, n).ready() {
            return kind;
        }
        t.step();
    }
    panic!("#{n} never finished: {:?}", state(t, n));
}

/// The constructions completed so far: tick, constructor, building.
fn completions(t: &TestSim) -> Vec<(Tick, u32, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::ConstructionCompleted { constructor, kind } => Some((*tick, constructor.0, *kind)),
            _ => None,
        })
        .collect()
}

/// Checks that the gate rejects `player`'s `command` for `reason`, then runs the next tick with it and checks that
/// it changed nothing: the state and the events are those of the same tick without it.
fn assert_rejected(t: &mut TestSim, player: PlayerId, command: Command, reason: Reject) {
    let cmd = PlayerCommand {
        player,
        command: command.clone(),
    };
    assert_eq!(t.sim().validate(&cmd), Err(reason), "{command:?}");
    let mut without = t.sim().clone();
    without.step(&TickInputs::default());
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command).step();
    assert_eq!(t.sim().state_hash(), without.state_hash(), "{reason}");
    assert_eq!(t.sim(), &without, "{reason}");
}

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

/// 24 × 16 tiles, level 1 but for a pit, corner (10, 7) at level 0, which slopes the 4 tiles around it, (9, 6) to (10,
/// 7); tile (3, 4) is water.
fn pitted_map() -> String {
    let mut rows = vec![".".repeat(24); 16];
    rows[4].replace_range(3..4, "~");
    let heights: Vec<String> = (0..=16)
        .map(|y| (0..=24).map(|x| if (x, y) == (10, 7) { '0' } else { '1' }).collect())
        .collect();
    format!("{}\n\n{}", rows.join("\n"), heights.join("\n"))
}

/// On the pitted map, player 1's Deployment Center 1 at (6, 1) to (9, 4), seeing 6 tiles around (8, 3), and its
/// Extraction Facility 2 at (11, 1) to (13, 3); patch 3 at (5, 7); player 1's Peacekeeper 4 on tile (3, 1). The
/// build area is x 0 to 21, y 0 to 15. Power 30 / 15; no income.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&pitted_map())
        .spawn(DeploymentCenter, Some(P1), tile(6, 1))
        .spawn(ExtractionFacility, Some(P1), tile(11, 1))
        .spawn(CrystalPatch, None, tile(5, 7))
        .spawn(Peacekeeper, Some(P1), SpawnAt::Point(at(3, 1).center()))
}

/// The plan's acceptance, times at full power: ordered in tick 1, a Barracks is ready in tick 161, a Generator in
/// 241, an Extraction Facility in 321, a plate in 401 (160, 240, 320 and 400 frames, counted from the tick after the
/// command). Each cost is paid when the command applies.
#[test]
fn constructions_take_their_frames_at_full_power() {
    let mut t = ScenarioBuilder::new(&flat(40, 12))
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 1
        .spawn(DeploymentCenter, Some(P1), tile(6, 1)) // 2
        .spawn(DeploymentCenter, Some(P1), tile(11, 1)) // 3
        .spawn(ExtractionFacility, Some(P1), tile(16, 1)) // 4
        .start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], Construct(Generator)))
        .at(1, P1, construction(&[2], Construct(Barracks)))
        .at(1, P1, construction(&[3], Construct(ExtractionFacility)))
        .at(1, P1, construction(&[4], Construct(ExtractionPlate)))
        .run_until(1);
    assert_eq!(bank(&t, P1), 1000 - 100 - 200 - 200 - 75);
    assert_eq!(state(&t, 1).current(), Some(Generator));
    assert_eq!(
        state(&t, 1).frames_done(),
        Some(0),
        "the command's own tick doesn't count"
    );
    t.run_until(121);
    assert_eq!(state(&t, 1).frames_done(), Some(120));
    t.run_until(500);
    assert_eq!(
        completions(&t),
        [
            (161, 2, Barracks),
            (241, 1, Generator),
            (321, 3, ExtractionFacility),
            (401, 4, ExtractionPlate)
        ]
    );
    for (n, kind) in [
        (1, Generator),
        (2, Barracks),
        (3, ExtractionFacility),
        (4, ExtractionPlate),
    ] {
        assert_eq!(state(&t, n), Constructor::Ready { kind }, "held until placed");
    }
    assert_eq!(bank(&t, P1), 425, "nothing more paid");
}

/// The plan's acceptance under the penalty and design change 0089's verification: at 30 generated and 51 consumed
/// (the all-in Barracks), the Extraction Facility builds a plate at 30/51, in exactly 680 ticks (400 × 51/30), while
/// the Deployment Center builds a Generator at full speed, in 240.
#[test]
fn negative_power_slows_the_facility_and_not_the_center() {
    let mut t = ScenarioBuilder::new(&flat(16, 10))
        .spawn(CrystalPatch, None, tile(1, 7)) // 1
        .spawn(CrystalPatch, None, tile(3, 7)) // 2
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 3
        .spawn(ExtractionFacility, Some(P1), tile(6, 1)) // 4
        .spawn(ExtractionPlate, Some(P1), tile(1, 7)) // 5
        .spawn(ExtractionPlate, Some(P1), tile(3, 7)) // 6
        .spawn(Barracks, Some(P1), tile(10, 1)) // 7
        .start();
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (30, 51));
    assert_eq!(power.speed(ExtractionFacility), 38_551, "≈ 59%");
    assert_eq!(power.speed(DeploymentCenter), RATIO_ONE);
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[4], Construct(ExtractionPlate)))
        .at(1, P1, construction(&[3], Construct(Generator)))
        .run_until(1000);
    assert_eq!(completions(&t), [(241, 3, Generator), (681, 4, ExtractionPlate)]);
}

/// The plan's acceptance, refunds (`> Cancellation`): cancelling while constructing gives the whole cost back,
/// cancelling a ready building 75% of it, rounded down: 75 → 56, 100 → 75, 200 → 150. The constructor is idle again.
#[test]
fn cancelling_refunds_in_full_while_constructing_and_75_percent_once_ready() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000)).run_until(1);
    let mut refunds = Vec::new();
    for (n, kind) in [
        (1, Generator),
        (1, Barracks),
        (1, ExtractionFacility),
        (2, ExtractionPlate),
    ] {
        for ready in [false, true] {
            let before = bank(&t, P1);
            next(&mut t, construction(&[n], Construct(kind)));
            let paid = before - bank(&t, P1);
            if ready {
                until_ready(&mut t, n);
            } else {
                t.run(20);
            }
            let before = bank(&t, P1);
            let cancel = if ready { CancelReady } else { CancelConstruction };
            next(&mut t, construction(&[n], cancel));
            assert!(state(&t, n).is_idle(), "{kind:?}");
            refunds.push((kind, paid, bank(&t, P1) - before));
        }
    }
    assert_eq!(
        refunds,
        [
            (Generator, 100, 100),
            (Generator, 100, 75),
            (Barracks, 200, 200),
            (Barracks, 200, 150),
            (ExtractionFacility, 200, 200),
            (ExtractionFacility, 200, 150),
            (ExtractionPlate, 75, 75),
            (ExtractionPlate, 75, 56),
        ]
    );
    assert_eq!(bank(&t, P1), 1000 - 25 - 50 - 50 - 19);
    assert!(
        completions(&t).len() == 4 && t.sim().objects().count() == 4,
        "nothing placed"
    );
}

/// The plan's acceptance, refusals: Construct is refused while the constructor constructs or holds a ready building,
/// for a building not on its list and without the crystals; the cancels and Place without their state; anyone
/// else's objects, none of the player's constructors, and two constructors for one Place. The bank stays as it was,
/// and so does everything else.
#[test]
fn refused_construction_commands_change_nothing() {
    let mut t = base()
        .spawn(DeploymentCenter, Some(P2), tile(16, 8)) // 5
        .start();
    t.debug_at(1, fund(P1, 150)).run_until(1);
    let dc = |command| construction(&[1], command);
    assert_rejected(&mut t, P1, dc(Construct(Barracks)), Reject::InsufficientCrystals);
    assert_rejected(&mut t, P1, dc(CancelConstruction), Reject::NotConstructing);
    assert_rejected(&mut t, P1, dc(CancelReady), Reject::NothingReady);
    assert_rejected(&mut t, P1, dc(place(6, 5)), Reject::NothingReady);
    next(&mut t, dc(Construct(Generator)));
    assert_eq!(bank(&t, P1), 50);
    // Busy, whatever the bank holds.
    next(&mut t, Command::Debug(fund(P1, 1000)));
    assert_rejected(&mut t, P1, dc(Construct(Generator)), Reject::ConstructorBusy(id(1)));
    assert_rejected(&mut t, P1, dc(Construct(Barracks)), Reject::ConstructorBusy(id(1)));
    assert_rejected(&mut t, P1, dc(CancelReady), Reject::NothingReady);
    assert_rejected(&mut t, P1, dc(place(6, 5)), Reject::NothingReady);
    until_ready(&mut t, 1);
    assert_rejected(&mut t, P1, dc(Construct(Barracks)), Reject::ConstructorBusy(id(1)));
    assert_rejected(&mut t, P1, dc(CancelConstruction), Reject::NotConstructing);
    // Not on the list.
    let ef = |command| construction(&[2], command);
    assert_rejected(&mut t, P1, ef(Construct(Generator)), Reject::CannotConstruct(Generator));
    assert_rejected(
        &mut t,
        P1,
        ef(Construct(Peacekeeper)),
        Reject::CannotConstruct(Peacekeeper),
    );
    assert_rejected(
        &mut t,
        P1,
        dc(Construct(ExtractionPlate)),
        Reject::CannotConstruct(ExtractionPlate),
    );
    // Whose objects, and which.
    assert_rejected(&mut t, P2, dc(CancelReady), Reject::NotOwned(id(1)));
    assert_rejected(
        &mut t,
        P1,
        construction(&[5], Construct(Generator)),
        Reject::NotOwned(id(5)),
    );
    assert_rejected(
        &mut t,
        P1,
        construction(&[2, 3], Construct(ExtractionPlate)),
        Reject::NotOwned(id(3)),
    );
    assert_rejected(&mut t, P1, construction(&[4], CancelReady), Reject::NoConstructor);
    assert_rejected(&mut t, P1, construction(&[99, 4], CancelReady), Reject::NoConstructor);
    assert_rejected(
        &mut t,
        P1,
        construction(&[], Construct(Generator)),
        Reject::NoConstructor,
    );
    assert_rejected(
        &mut t,
        P1,
        construction(&[1, 2], place(6, 5)),
        Reject::SeveralConstructors,
    );
    assert_rejected(&mut t, PlayerId(9), dc(CancelReady), Reject::UnknownPlayer(PlayerId(9)));
    let many = vec![1; MAX_ORDER_UNITS + 1];
    assert_rejected(&mut t, P1, construction(&many, CancelReady), Reject::TooManyUnits);
    // Repeated ids count once, and ids that are gone are dropped.
    next(&mut t, construction(&[1, 1, 99], CancelReady));
    assert_eq!(bank(&t, P1), 1000 + 75);
}

/// A command goes to every constructor it names that can carry it out, in id order (design change 0071: no
/// smart-casting), each paying from what the ones before it left: with 150 crystals two idle Deployment Centers start
/// one Generator, the lower id's.
#[test]
fn a_command_goes_to_each_constructor_that_can_in_id_order() {
    let mut t = ScenarioBuilder::new(&flat(24, 10))
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 1
        .spawn(DeploymentCenter, Some(P1), tile(6, 1)) // 2
        .spawn(ExtractionFacility, Some(P1), tile(11, 1)) // 3
        .start();
    t.debug_at(1, fund(P1, 150)).run_until(1);
    next(&mut t, construction(&[2, 1], Construct(Generator)));
    assert_eq!(
        (state(&t, 1).current(), state(&t, 2).current()),
        (Some(Generator), None)
    );
    assert_eq!(bank(&t, P1), 50);
    // One busy, the other one starts; the Extraction Facility can't build a Generator and is passed over.
    next(&mut t, Command::Debug(fund(P1, 300)));
    next(&mut t, construction(&[1, 2, 3], Construct(Generator)));
    assert_eq!(state(&t, 2).current(), Some(Generator));
    assert_eq!(bank(&t, P1), 200);
    // A cancel goes to all of them that have something to cancel.
    next(&mut t, construction(&[3, 2, 1], CancelConstruction));
    assert!(state(&t, 1).is_idle() && state(&t, 2).is_idle());
    assert_eq!(bank(&t, P1), 400);
    // Each a building of its own list.
    next(&mut t, construction(&[1, 3], Construct(ExtractionPlate)));
    assert_eq!(
        (state(&t, 1).current(), state(&t, 3).current()),
        (None, Some(ExtractionPlate))
    );
    assert_eq!(bank(&t, P1), 325);
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

/// Design change 0084's verifications 2 and 3 (`> Cancellation`: "Destroyed while constructing or holding a ready
/// building: refunded as if cancelled"): a Deployment Center destroyed while it builds a Generator gives 100 back,
/// one holding a ready Generator 75; an Extraction Facility 75 and 56 for its plate; an idle one nothing. A debug
/// removal stands for a destruction (`Sim::remove_objects`, the one way out of the game).
#[test]
fn a_destroyed_constructor_refunds_as_if_cancelled() {
    let mut t = ScenarioBuilder::new(&flat(32, 12))
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 1
        .spawn(DeploymentCenter, Some(P1), tile(6, 1)) // 2
        .spawn(ExtractionFacility, Some(P1), tile(11, 1)) // 3
        .spawn(ExtractionFacility, Some(P1), tile(15, 1)) // 4
        .spawn(DeploymentCenter, Some(P1), tile(19, 1)) // 5
        .start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1, 2], Construct(Generator)))
        .at(1, P1, construction(&[3, 4], Construct(ExtractionPlate)));
    let removal = |t: &mut TestSim, tick: Tick, n: u32| -> u32 {
        t.run_until(tick - 1);
        let before = bank(t, P1);
        t.debug_at(tick, DebugCommand::Remove { id: id(n) }).step();
        assert!(t.sim().object(id(n)).is_none());
        bank(t, P1) - before
    };
    let refunds = [
        removal(&mut t, 100, 1),
        removal(&mut t, 101, 3),
        removal(&mut t, 300, 2),
        removal(&mut t, 450, 4),
        removal(&mut t, 451, 5),
    ];
    assert_eq!(refunds, [100, 75, Refund::Ready.of(100), Refund::Ready.of(75), 0]);
    assert_eq!(refunds[2..4], [75, 56]);
    assert_eq!(bank(&t, P1), 1000 - 350 + 306);
    assert_eq!(dangling_references(t.sim()), []);
}

/// Design change 0084 in combat: player 2's Extraction Facility, holding a ready plate, is destroyed by player 1's
/// Peacekeepers; player 2 gets 56 back in the tick it dies.
#[test]
fn a_constructor_destroyed_in_combat_refunds_its_owner() {
    let mut builder = ScenarioBuilder::new(&flat(20, 12))
        .spawn(ExtractionFacility, Some(P2), tile(8, 4)) // 1: (8, 4) to (10, 6)
        .spawn(DeploymentCenter, Some(P2), tile(14, 7)); // 2
    for (x, y) in [(6, 3), (6, 5), (6, 7), (8, 8), (10, 8), (12, 5)] {
        builder = builder.spawn(Peacekeeper, Some(P1), SpawnAt::Point(at(x, y).center()));
    }
    let mut t = builder.start();
    t.without_auto_targeting()
        .debug_at(1, fund(P2, 75))
        .at(1, P2, construction(&[1], Construct(ExtractionPlate)));
    until_ready(&mut t, 1);
    assert_eq!(bank(&t, P2), 0);
    let tick = t.sim().tick() + 1;
    let attackers: Vec<ObjectId> = (3..=8).map(id).collect();
    t.order_at(tick, P1, &attackers, OrderKind::Attack(id(1)), false);
    let mut died = None;
    for _ in 0..3000 {
        t.step();
        if t.sim().object(id(1)).is_none() {
            died = Some(t.sim().tick());
            break;
        }
        assert_eq!(bank(&t, P2), 0, "nothing back before it dies");
    }
    let died = died.expect("the Extraction Facility is destroyed");
    assert!(
        t.sim()
            .events()
            .iter()
            .any(|e| matches!(e, SimEvent::Died { id: dead, .. } if *dead == id(1)))
    );
    assert_eq!(bank(&t, P2), 56, "refunded in tick {died}");
}

/// The gate (`ARCHITECTURE.md > Commands, events and replays`), fuzzed: random construction commands from any player
/// (in the game or not) to any ids, of any building and anywhere, never panic; a rejected one changes nothing, but a
/// Place the placement check refuses, which only reports `PlacementRefused`. Each command runs alone in a tick, next
/// to a copy of the game that runs the same tick without it. Twice from the same seed, the runs are identical, and
/// the recorded commands replay to the same state.
#[test]
fn random_construction_commands_never_panic_and_rejections_change_nothing() {
    let (counts, hashes, replay) = construction_fuzz(0xc0de);
    // The stream exercises the valid paths too, not only rejections.
    assert!(counts.accepted > 120 && counts.rejected > 3000, "{counts:?}");
    assert!(
        counts.placed > 10 && counts.refused > 100 && counts.completed > 20,
        "{counts:?}"
    );
    let (_, again, _) = construction_fuzz(0xc0de);
    assert_eq!(hashes, again);
    let last = *hashes.last().unwrap();
    assert_eq!(replay.run().state_hash(), last);
    let file: Replay = serde_json::from_str(&serde_json::to_string(&replay).unwrap()).unwrap();
    assert_eq!(file.run().state_hash(), last);
}

/// What a fuzz run did.
#[derive(Debug, Default)]
struct Counts {
    accepted: usize,
    rejected: usize,
    /// `SimEvent::StructurePlaced`.
    placed: usize,
    /// `SimEvent::PlacementRefused`.
    refused: usize,
    /// `SimEvent::ConstructionCompleted`.
    completed: usize,
}

/// Both players' bases on a 32 × 16 map: player 1's Deployment Center 1 and Extraction Facility 2 in the west,
/// player 2's 3 and 4 in the east, patches 5 to 8, two by each base.
fn fuzz_base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(32, 16))
        .spawn(DeploymentCenter, Some(P1), tile(4, 2))
        .spawn(ExtractionFacility, Some(P1), tile(4, 9))
        .spawn(DeploymentCenter, Some(P2), tile(24, 2))
        .spawn(ExtractionFacility, Some(P2), tile(24, 9))
        .spawn(CrystalPatch, None, tile(9, 3))
        .spawn(CrystalPatch, None, tile(9, 6))
        .spawn(CrystalPatch, None, tile(22, 3))
        .spawn(CrystalPatch, None, tile(22, 6))
        .config(SimConfig::DEBUG)
}

fn random_construction(rng: &mut Rng, sim: &Sim) -> PlayerCommand {
    let player = PlayerId([1, 1, 1, 2, 2, 2, 0, 3][rng.below(8) as usize]);
    // Mostly one of the player's constructors, sometimes two; else any ids, some of which don't exist (any more, or
    // yet).
    let own: Vec<ObjectId> = sim
        .objects()
        .filter(|o| o.owner == Some(player) && o.constructor.is_some())
        .map(|o| o.id)
        .collect();
    let pick = |rng: &mut Rng| own[rng.below(own.len() as u32) as usize];
    let objects = if !own.is_empty() && rng.below(4) != 0 {
        let extra = rng.below(4) == 0;
        let mut objects = vec![pick(rng)];
        if extra {
            objects.push(pick(rng));
        }
        objects
    } else {
        let ids = sim.objects().map(|o| o.id.0).max().unwrap_or(0) + 3;
        (0..rng.below(3) + 1).map(|_| ObjectId(rng.below(ids) + 1)).collect()
    };
    // Few cancels, so that constructions finish and get placed.
    let command = match rng.below(40) {
        0..14 => {
            let kinds = [Generator, Barracks, ExtractionFacility, ExtractionPlate];
            let kind = if rng.below(4) == 0 {
                ObjectKind::ALL[rng.below(ObjectKind::ALL.len() as u32) as usize]
            } else {
                kinds[rng.below(4) as usize]
            };
            Construct(kind)
        }
        14 => CancelConstruction,
        15 => CancelReady,
        _ => {
            // Mostly in sight of the player's Deployment Center or on one of its patches, sometimes anywhere or off
            // the map.
            let (x0, patches) = if player == P2 {
                (20, [(22, 3), (22, 6)])
            } else {
                (0, [(9, 3), (9, 6)])
            };
            let (x, y) = match rng.below(6) {
                0 => (rng.below(40) as i32 - 4, rng.below(24) as i32 - 4),
                1 => patches[rng.below(2) as usize],
                _ => (x0 + rng.below(12) as i32, rng.below(11) as i32),
            };
            let orientation = BuildingOrientation {
                rotation: [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270][rng.below(4) as usize],
                flip_h: rng.below(2) == 0,
                flip_v: rng.below(2) == 0,
            };
            Place {
                origin: at(x, y),
                orientation,
            }
        }
    };
    PlayerCommand {
        player,
        command: Command::Construction { objects, command },
    }
}

/// 6000 ticks of one random construction command each, after a first tick that fills both banks: what happened,
/// the state hash of every tick, and the replay.
fn construction_fuzz(seed: u64) -> (Counts, Vec<u64>, Replay) {
    let scenario = fuzz_base().build();
    let mut sim = Sim::new(&scenario);
    let mut replay = Replay::new(scenario);
    let funds = TickInputs {
        commands: [P1, P2]
            .map(|player| PlayerCommand {
                player: P1,
                command: Command::Debug(fund(player, 100_000)),
            })
            .to_vec(),
    };
    sim.step(&funds);
    replay.record(funds);
    let mut rng = Rng::from_seed(seed);
    let mut counts = Counts::default();
    let mut hashes = Vec::new();
    for _ in 0..6000 {
        let cmd = random_construction(&mut rng, &sim);
        let verdict = sim.validate(&cmd);
        let mut without = sim.clone();
        without.step(&TickInputs::default());
        let inputs = TickInputs {
            commands: vec![cmd.clone()],
        };
        sim.step(&inputs);
        replay.record(inputs);
        match verdict {
            Ok(()) => counts.accepted += 1,
            Err(reason) => {
                counts.rejected += 1;
                assert_eq!(sim.state_hash(), without.state_hash(), "{cmd:?}: {reason}");
                let reported = match (&cmd.command, reason) {
                    (
                        Command::Construction {
                            command: Place { .. }, ..
                        },
                        Reject::Unplaceable(_),
                    ) => 1,
                    _ => 0,
                };
                assert_eq!(&sim.events()[reported..], without.events(), "{cmd:?}: {reason}");
                assert_eq!(sim.events().len(), without.events().len() + reported);
            }
        }
        for e in sim.events() {
            match e {
                SimEvent::StructurePlaced { .. } => counts.placed += 1,
                SimEvent::PlacementRefused { .. } => counts.refused += 1,
                SimEvent::ConstructionCompleted { .. } => counts.completed += 1,
                _ => {}
            }
        }
        assert_eq!(dangling_references(&sim), []);
        hashes.push(sim.state_hash());
    }
    (counts, hashes, replay)
}

/// A game saved in the middle of its constructions loads and goes on exactly as the running one; the replay of
/// its commands, through its file, ends in the same state.
#[test]
fn constructions_go_on_alike_after_a_save_and_in_a_replay() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], Construct(Generator)))
        .at(1, P1, construction(&[2], Construct(ExtractionPlate)))
        .at(
            250,
            P1,
            construction(
                &[1],
                Place {
                    origin: at(6, 5),
                    orientation: BuildingOrientation {
                        rotation: Rotation::R270,
                        flip_h: true,
                        flip_v: false,
                    },
                },
            ),
        )
        .at(251, P1, construction(&[1], Construct(Barracks)))
        .at(300, P1, construction(&[1], CancelConstruction))
        .at(301, P1, construction(&[1], Construct(Generator)))
        .at(420, P1, construction(&[2], place(5, 7)))
        .at(421, P1, construction(&[2], Construct(ExtractionPlate)));
    t.run_until(200);
    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(state(&t, 1).frames_done(), Some(199));
    while t.sim().tick() < 1000 {
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(
        completions(&t),
        [
            (241, 1, Generator),
            (401, 2, ExtractionPlate),
            (541, 1, Generator),
            (821, 2, ExtractionPlate)
        ]
    );
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}

/// The plan's mini-oracle (`factions/gdo/economy.md > Opening Economy Baseline`): from the starting base (a
/// Deployment Center, an Extraction Facility and two plates, 100 Space Crystals; `> Starting Conditions`, set up by
/// hand until slice M2-F1), the clean Barracks opening, Plate, Generator, Generator, Barracks, each ordered as soon
/// as its constructor is free and the bank covers it, each building placed as soon as it is ready. "Fastest clean
/// Barracks ... starts at ~46 seconds and finishes at ~56 seconds", with continuous income; with lumps (design change
/// 0098) about 48 s (the plan's R10). Here it starts in tick 769 (48.1 s) and is ready in tick 929 (58.1 s), and
/// power stays positive throughout.
#[test]
fn the_clean_barracks_opening_starts_the_barracks_at_about_48_seconds() {
    let mut t = ScenarioBuilder::new(&flat(32, 20))
        .spawn(CrystalPatch, None, tile(5, 6)) // 1
        .spawn(CrystalPatch, None, tile(7, 7)) // 2
        .spawn(CrystalPatch, None, tile(9, 7)) // 3
        .spawn(DeploymentCenter, Some(P1), tile(6, 1)) // 4
        .spawn(ExtractionFacility, Some(P1), tile(11, 1)) // 5
        .spawn(ExtractionPlate, Some(P1), tile(5, 6)) // 6
        .spawn(ExtractionPlate, Some(P1), tile(7, 7)) // 7
        .start();
    let (dc, ef) = (4, 5);
    t.debug_at(1, fund(P1, 100))
        .at(1, P1, construction(&[ef], Construct(ExtractionPlate)));
    let mut plan = [Generator, Generator, Barracks].into_iter().peekable();
    let mut spots = [at(9, 7), at(2, 1), at(2, 3), at(10, 5)].into_iter();
    let mut started = Vec::new();
    t.step();
    while t.sim().tick() < 1000 {
        let tick = t.sim().tick() + 1;
        for n in [ef, dc] {
            if state(&t, n).ready().is_some() {
                let spot = spots.next().unwrap();
                t.at(tick, P1, construction(&[n], place(spot.x, spot.y)));
            }
        }
        let free = state(&t, dc).current().is_none();
        if let Some(&kind) = plan.peek()
            && free
            && bank(&t, P1) >= sc_sim::construction::recipe(DeploymentCenter, kind).unwrap().cost
        {
            t.at(tick, P1, construction(&[dc], Construct(kind)));
            started.push((tick, kind));
            plan.next();
        }
        t.step();
        let power = t.sim().power(P1);
        assert!(!power.is_negative(), "tick {}: {power:?}", t.sim().tick());
    }
    let refused = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::PlacementRefused { .. }))
        .count();
    assert_eq!(refused, 0);
    assert_eq!(started, [(193, Generator), (434, Generator), (769, Barracks)]);
    let barracks = completions(&t)
        .into_iter()
        .find(|&(_, _, kind)| kind == Barracks)
        .unwrap()
        .0;
    assert_eq!(barracks, 929);
    // Within the design's tolerance: about 46 to 48 seconds, then 10 more.
    let seconds = |tick: Tick| f64::from(tick) / f64::from(sc_sim::TICKS_PER_SECOND);
    assert!((45.0..=50.0).contains(&seconds(769)) && seconds(barracks) - seconds(769) == 10.0);
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (70, 54));
    let placed = t
        .sim()
        .objects()
        .filter(|o| o.kind == Barracks && o.owner == Some(P1))
        .count();
    assert_eq!(placed, 1);
}
