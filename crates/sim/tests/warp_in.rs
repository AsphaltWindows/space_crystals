//! Warp In (slice M3-E; `factions/colonists/economy.md > Warp In`, `core/entities.md > Placement Validation > Warp
//! In`): a Beacon transforms in place into an Egress Node, a Robo Portal or a Bio Portal. One test per acceptance
//! point of the plan (`artifacts/director/m3-plan.md` §3.3 E): the timings, the interpolated maxima and the damage
//! carried through them (design change 0110), the footprint, its ring and its claim from the first frame (design
//! change 0109), the cancel, the destruction, the gate against `Sim::warp_in_check`, saves and replays; and the gate's
//! rules, fuzzed. The scenarios are built with test maps and spawns, not with the Colonists' Starting Conditions.
//! `TestSim` checks the movement invariants after every tick.

use sc_sim::economy::{RATIO_ONE, Refund};
use sc_sim::placement::{self, Method, Mode, Occupant, Site};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Ability, BuildArea, BuildingOrientation, Command, ConstructionCommand, DebugCommand, Object, ObjectId, ObjectKind,
    OrderKind, PlacementCheck, PlacementProblem, PlayerCommand, PlayerId, ProductionCommand, Reject, Replay, Rng,
    Rotation, SU_PER_GU, Sim, SimConfig, SimEvent, SpawnAt, Target, Tick, TickInputs, TilePos, TileProblem, Transform,
    Vec2, WholeProblem,
};

use ConstructionCommand::{CancelTransform, WarpIn};
use ObjectKind::{Beacon, BioPortal, CrystalPatch, Drone, EgressNode, Generator, Peacekeeper, RoboPortal};

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

/// A Warp In of `kind` with the top-left tile of its footprint at `origin`, unturned.
fn warp(kind: ObjectKind, origin: TilePos) -> ConstructionCommand {
    WarpIn {
        kind,
        origin,
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

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("the object exists")
}

/// Object `n`'s transformation. Panics if it isn't transforming.
fn transform(t: &TestSim, n: u32) -> &Transform {
    object(t, n).transform.as_ref().expect("transforming")
}

/// Schedules player 1's `command` for the next tick and runs it.
fn next(t: &mut TestSim, command: Command) -> &mut TestSim {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).step()
}

/// The transformations completed so far: tick, object, kind.
fn completions(t: &TestSim) -> Vec<(Tick, u32, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::TransformCompleted { id, kind } => Some((*tick, id.0, *kind)),
            _ => None,
        })
        .collect()
}

/// The `TransformCancelled` events so far: tick, object, kind, refund.
fn cancellations(t: &TestSim) -> Vec<(Tick, u32, ObjectKind, u32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::TransformCancelled { id, kind, refund } => Some((*tick, id.0, *kind, *refund)),
            _ => None,
        })
        .collect()
}

/// The `Train` of a unit of `kind`, to the producers `objects`.
fn train(objects: &[u32], kind: ObjectKind) -> Command {
    Command::Production {
        objects: objects.iter().map(|&n| ObjectId(n)).collect(),
        command: ProductionCommand::Train(kind),
    }
}

/// Checks that the gate rejects `player`'s `command` for `reason`, then runs the next tick with it and checks that
/// it changed nothing: the state and the events are those of the same tick without it, but for the
/// `PlacementRefused` of a footprint refused by the placement check, the tick's first event. Returns that event.
fn assert_rejected(t: &mut TestSim, player: PlayerId, command: Command, reason: Reject) -> Option<SimEvent> {
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
    let reported = usize::from(matches!(reason, Reject::Unplaceable(_)));
    assert_eq!(&t.sim().events()[reported..], without.events(), "{reason}");
    (reported == 1).then(|| t.sim().events()[0].clone())
}

/// A 24 × 16 flat map: player 1's Beacons 1 at (8, 6) to (9, 7) and 2 at (16, 6), and Generator 3 at (2, 2); player
/// 2's Beacon 4 at (16, 12) and Generator 5 at (20, 1). A Robo Portal centred on Beacon 1 covers (7, 5) to (10, 8).
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(24, 16))
        .spawn(Beacon, Some(P1), tile(8, 6))
        .spawn(Beacon, Some(P1), tile(16, 6))
        .spawn(Generator, Some(P1), tile(2, 2))
        .spawn(Beacon, Some(P2), tile(16, 12))
        .spawn(Generator, Some(P2), tile(20, 1))
}

/// A building of the Beacon's Constructs List (`factions/colonists/economy.md > Warp In > Constructs List`), as the
/// tests warp it in on Beacon 1.
#[derive(Copy, Clone)]
struct Building {
    kind: ObjectKind,
    /// The top-left tile of its footprint centred on Beacon 1 (`> Warp In Command`: "centered on the Beacon's
    /// location").
    origin: (i32, i32),
    cost: u32,
    /// Its WarpInTime.
    frames: u32,
    /// What it produces, if anything.
    unit: Option<ObjectKind>,
}

const BUILDINGS: [Building; 3] = [
    Building {
        kind: EgressNode,
        origin: (8, 6),
        cost: 75,
        frames: 320,
        unit: None,
    },
    Building {
        kind: RoboPortal,
        origin: (7, 5),
        cost: 400,
        frames: 640,
        unit: Some(Drone),
    },
    Building {
        kind: BioPortal,
        origin: (7, 5),
        cost: 150,
        frames: 480,
        unit: Some(ObjectKind::SabreLegionnaire),
    },
];

/// The plan's acceptance, "Timings" (`ARCHITECTURE.md > Economy > Recipe timing`): for each building, the cost is
/// paid when the command applies; ordered in tick 1, the Warp In is complete at the end of tick 1 + N - 1 = N (its
/// WarpInTime), and the building works from tick N + 1: a portal refuses Train until then and takes it in that
/// tick. From its first frame the object is the building, still transforming, and stays the same object.
#[test]
fn warp_ins_take_their_warp_in_time_and_work_from_the_next_tick() {
    for Building {
        kind,
        origin: (x, y),
        cost,
        frames,
        unit,
    } in BUILDINGS
    {
        let mut t = base().start();
        t.debug_at(1, fund(P1, 1000))
            .at(1, P1, construction(&[1], warp(kind, at(x, y))));
        t.step();
        assert_eq!(bank(&t, P1), 1000 - cost, "{kind:?}: paid at once");
        assert!(
            t.sim().events().contains(&SimEvent::KindChanged {
                id: id(1),
                from: Beacon,
                to: kind
            }),
            "{kind:?}"
        );
        let o = object(&t, 1);
        assert_eq!(o.kind, kind);
        assert_eq!(
            o.placement.map(|p| (p.origin, p.w)),
            Some((at(x, y), kind.def().tile_size().unwrap().0))
        );
        let w = transform(&t, 1);
        assert_eq!((w.from, w.paid, w.total, w.frames_done()), (Beacon, cost, frames, 1));
        assert_eq!(w.from_placement.map(|p| p.origin), Some(at(8, 6)));
        let trained = unit.map(|unit| PlayerCommand {
            player: P1,
            command: train(&[1], unit),
        });
        t.run_until(frames - 1);
        assert!(
            object(&t, 1).is_transforming(),
            "{kind:?}: still at the end of tick {}",
            frames - 1
        );
        assert_eq!(completions(&t), []);
        if let Some(cmd) = &trained {
            assert_eq!(
                t.sim().validate(cmd),
                Err(Reject::NoProducers),
                "{kind:?} doesn't work yet"
            );
        }
        t.step();
        assert_eq!(
            completions(&t),
            [(frames, 1, kind)],
            "{kind:?}: complete at the end of tick {frames}"
        );
        assert!(!object(&t, 1).is_transforming());
        assert_eq!(bank(&t, P1), 1000 - cost);
        if let Some(cmd) = trained {
            assert_eq!(
                t.sim().validate(&cmd),
                Ok(()),
                "{kind:?} works from tick {}",
                frames + 1
            );
            next(&mut t, cmd.command);
            assert!(
                t.sim()
                    .events()
                    .iter()
                    .any(|e| matches!(e, SimEvent::ProductionQueued { producer, .. } if *producer == id(1))),
                "{kind:?}"
            );
        }
    }
}

/// The plan's acceptance, "Interpolation" (`> Warp In > HP/Shields Interpolation`): MaxHP and MaxShields at 0, 25,
/// 50 and 100% of the WarpInTime follow "Beacon MaxHP + (Building MaxHP - Beacon MaxHP) x progress", and the HP of
/// an undamaged building stays at its maximum: a Robo Portal at 50% has 360 HP and 15 Shields. 0% is the Beacon
/// itself; the command's tick is the first frame.
#[test]
fn hp_and_shields_interpolate_by_the_formulas() {
    for Building {
        kind,
        origin: (x, y),
        frames,
        ..
    } in BUILDINGS
    {
        let (to_hp, to_shields) = (kind.def().max_hp.unwrap(), kind.def().max_shields.unwrap());
        let mut t = base().start();
        assert_eq!(
            (object(&t, 1).max_hp(), object(&t, 1).max_shields()),
            (Some(120.0), Some(10.0))
        );
        t.debug_at(1, fund(P1, 1000))
            .at(1, P1, construction(&[1], warp(kind, at(x, y))));
        t.step();
        let first = 120.0 + (to_hp - 120.0) / frames as f32;
        assert!(
            (object(&t, 1).hp.unwrap() - first).abs() < 1e-3,
            "{kind:?}: one frame done"
        );
        for quarter in [1, 2, 4] {
            t.run_until(frames / 4 * quarter);
            let p = quarter as f32 / 4.0;
            let (hp, shields) = (120.0 + (to_hp - 120.0) * p, 10.0 + (to_shields - 10.0) * p);
            let o = object(&t, 1);
            assert_eq!(
                (o.hp, o.max_hp(), o.max_shields()),
                (Some(hp), Some(hp), Some(shields)),
                "{kind:?} at {}%",
                quarter * 25
            );
        }
        assert!(!object(&t, 1).is_transforming());
    }
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], warp(RoboPortal, at(7, 5))));
    t.run_until(320);
    let o = object(&t, 1);
    assert_eq!((o.hp, o.max_shields()), (Some(360.0), Some(15.0)), "the plan's example");
}

/// The plan's acceptance, "With damage taken in between, the current values follow the default" (design change
/// 0110: "current += (newMax − oldMax) each frame; never a fraction-of-max rescale"): the maximum still follows the
/// formula, and the HP rises by what it rises, so the damage taken stays taken, exactly, to the end: player 2's
/// Peacekeeper 6 shoots the warping Robo Portal from tick 100 to 130, 9 HP a hit (its Damage 10 less the portal's
/// PointArmor 1).
#[test]
fn damage_taken_while_warping_in_stays_taken() {
    let mut t = shot_at();
    t.run_until(135);
    let o = object(&t, 1);
    let deficit = o.max_hp().unwrap() - o.hp.unwrap();
    assert!(
        deficit >= 9.0 && deficit % 9.0 == 0.0,
        "the Peacekeeper hit it: {deficit}"
    );
    for tick in [200, 320, 639] {
        t.run_until(tick);
        let o = object(&t, 1);
        let max = 120.0 + 480.0 * tick as f32 / 640.0;
        assert_eq!((o.max_hp(), o.hp), (Some(max), Some(max - deficit)), "tick {tick}");
    }
    t.step();
    assert!(!object(&t, 1).is_transforming());
    assert_eq!(object(&t, 1).hp, Some(600.0 - deficit));
}

/// Design change 0110, for damage "taken before the transformation started too" (its Verification: "Same
/// arithmetic for a Warp In (Beacon 120 → target MaxHP)"): player 2's Peacekeeper 6 shoots Beacon 1 until it is 40 or
/// more short of its 120 HP, 10 HP a hit; the Robo Portal it then warps in is exactly as short of each maximum on
/// the way, and completes that short of its 600.
#[test]
fn a_damaged_beacon_keeps_its_deficit_through_the_warp_in() {
    let mut t = base().start();
    t.without_auto_targeting()
        .debug_at(1, fund(P1, 1000))
        .spawn_unit_at(1, Peacekeeper, P2, at(12, 7).center())
        .order_at(2, P2, &[id(6)], OrderKind::Attack(id(1)), false);
    while object(&t, 1).hp > Some(80.0) {
        assert!(t.sim().tick() < 300, "the Peacekeeper shoots the Beacon");
        t.step();
    }
    let tick = t.sim().tick();
    t.order_at(tick + 1, P2, &[id(6)], OrderKind::Stop, false).step();
    let deficit = 120.0 - object(&t, 1).hp.unwrap();
    assert!(deficit >= 40.0 && deficit % 10.0 == 0.0, "{deficit}");
    let start = tick + 2;
    t.at(start, P1, construction(&[1], warp(RoboPortal, at(7, 5))));
    for frames in [1, 160, 320, 639, 640] {
        t.run_until(start + frames - 1);
        let o = object(&t, 1);
        let max = 120.0 + 480.0 * frames as f32 / 640.0;
        assert_eq!((o.max_hp(), o.hp), (Some(max), Some(max - deficit)), "{frames} frames");
    }
    assert!(!object(&t, 1).is_transforming());
    assert_eq!(object(&t, 1).hp, Some(600.0 - deficit));
}

/// The base with Beacon 1 warping in a Robo Portal from tick 1 on, which player 2's Peacekeeper 6, at the centre of
/// tile (12, 7), shoots from tick 100 to 130; auto-targeting is off.
fn shot_at() -> TestSim {
    let mut t = base().start();
    t.without_auto_targeting()
        .debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], warp(RoboPortal, at(7, 5))))
        .spawn_unit_at(1, Peacekeeper, P2, at(12, 7).center())
        .order_at(100, P2, &[id(6)], OrderKind::Attack(id(1)), false)
        .order_at(130, P2, &[id(6)], OrderKind::Stop, false);
    t
}

/// The plan's acceptance, "Footprints" (`core/entities.md > Placement Validation > Warp In`): a 4 × 4 Robo Portal
/// centred on a 2 × 2 Beacon needs its ring of 12 tiles clear: "All footprint tiles must be Buildable, flat and
/// unoccupied, except for the Beacon's own footprint". A unit (the player's own), a patch or a structure on the ring
/// refuses it, with that tile's problem, and the sim says so (`SimEvent::PlacementRefused`, the Beacon as the
/// constructor); the Beacon's own tiles are ignored, even when the check is shown the Beacon.
#[test]
fn a_robo_portal_needs_its_ring_of_twelve_tiles_clear() {
    let footprint: Vec<TilePos> = (5..9).flat_map(|y| (7..11).map(move |x| at(x, y))).collect();
    let beacon = [at(8, 6), at(9, 6), at(8, 7), at(9, 7)];
    let ring: Vec<TilePos> = footprint.iter().copied().filter(|t| !beacon.contains(t)).collect();
    assert_eq!(ring.len(), 12);
    let t = base().start();
    let check = t
        .sim()
        .warp_in_check(id(1), RoboPortal, at(7, 5), BuildingOrientation::default());
    assert_eq!(check, Some(PlacementCheck::default()), "a clear ring");
    // The pure check shown everything, the Beacon included, says the same.
    let area = BuildArea::default();
    let site = Site {
        map: t.sim().map(),
        area: &area,
        mode: Mode::Layout,
        method: Method::WarpIn {
            beacon: object(&t, 1).placement.unwrap(),
        },
    };
    let all = t.sim().objects().map(Occupant::of);
    assert!(placement::check(&site, all, RoboPortal, at(7, 5), BuildingOrientation::default()).is_valid());

    let blockers = [
        (
            DebugCommand::Spawn {
                kind: Peacekeeper,
                owner: Some(P1),
                at: SpawnAt::Point(at(7, 5).center()),
                orientation: BuildingOrientation::default(),
            },
            at(7, 5),
            TileProblem::Unit,
        ),
        (
            DebugCommand::Spawn {
                kind: CrystalPatch,
                owner: None,
                at: tile(10, 8),
                orientation: BuildingOrientation::default(),
            },
            at(10, 8),
            TileProblem::Resource,
        ),
        (
            DebugCommand::Spawn {
                kind: Generator,
                owner: Some(P2),
                at: tile(10, 4),
                orientation: BuildingOrientation::default(),
            },
            at(10, 5),
            TileProblem::Structure,
        ),
    ];
    for (blocker, on, problem) in blockers {
        let mut t = base().start();
        t.debug_at(1, fund(P1, 1000)).debug_at(1, blocker);
        t.step();
        assert_eq!(t.sim().objects().count(), 6, "{problem:?}: the blocker is there");
        let first = PlacementProblem::Tile(on, problem);
        let check = t
            .sim()
            .warp_in_check(id(1), RoboPortal, at(7, 5), BuildingOrientation::default())
            .unwrap();
        assert_eq!(check.tiles, [(on, problem)], "only the ring tile");
        let refusal = assert_rejected(
            &mut t,
            P1,
            construction(&[1], warp(RoboPortal, at(7, 5))),
            Reject::Unplaceable(first),
        );
        assert_eq!(
            refusal,
            Some(SimEvent::PlacementRefused {
                constructor: id(1),
                kind: RoboPortal,
                origin: at(7, 5),
                orientation: BuildingOrientation::default(),
                problems: check,
            })
        );
        assert_eq!(object(&t, 1).kind, Beacon, "still a Beacon");
        // The smaller Egress Node, on the Beacon's own tiles, still warps in.
        next(&mut t, construction(&[1], warp(EgressNode, at(8, 6))));
        assert_eq!(object(&t, 1).kind, EgressNode, "{problem:?}");
    }
}

/// `> Warp In Command`: the footprint is "centered on the Beacon's location" ([`WholeProblem::NotOnBeacon`]): over the
/// 2 × 2 Beacon 1, a 2 × 2 Egress Node and a 4 × 4 Robo Portal have one footprint each, a 3 × 3 Bio Portal four, the
/// four that cover the Beacon (placeholder: question Q0040 point 5), in any orientation. Anywhere else is refused.
#[test]
fn a_warp_in_is_centred_on_its_beacon() {
    let t = base().start();
    let turnings: Vec<BuildingOrientation> = [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270]
        .into_iter()
        .flat_map(|rotation| {
            [(false, false), (true, false), (false, true), (true, true)].map(|(flip_h, flip_v)| BuildingOrientation {
                rotation,
                flip_h,
                flip_v,
            })
        })
        .collect();
    for (kind, valid) in [
        (EgressNode, vec![at(8, 6)]),
        (RoboPortal, vec![at(7, 5)]),
        (BioPortal, vec![at(7, 5), at(8, 5), at(7, 6), at(8, 6)]),
    ] {
        for &orientation in &turnings {
            for y in 2..11 {
                for x in 4..13 {
                    let check = t.sim().warp_in_check(id(1), kind, at(x, y), orientation).unwrap();
                    if valid.contains(&at(x, y)) {
                        assert!(check.is_valid(), "{kind:?} at ({x}, {y}): {check:?}");
                    } else {
                        assert_eq!(
                            check.first_problem(),
                            Some(PlacementProblem::Whole(WholeProblem::NotOnBeacon)),
                            "{kind:?} at ({x}, {y})"
                        );
                    }
                }
            }
        }
    }
    // An object that doesn't exist has no Warp In to check.
    assert_eq!(
        t.sim()
            .warp_in_check(id(99), EgressNode, at(8, 6), BuildingOrientation::default()),
        None
    );
}

/// Design change 0109 (`> Warp In > The target building from the first frame`), its Verification: "Warp In a 3x3
/// (or larger) building: the whole footprint is solid from the first frame; cancel restores the 2x2 Beacon". For a
/// Bio Portal and a Robo Portal on Beacon 1: from the Warp In's own tick no unit can be put on any tile of the
/// footprint, and player 1's Peacekeeper 6, sent in that tick from the west of it to the east across it, walks
/// around it, its box never on it (`TestSim` also checks that no ground box overlaps a blocked tile after every tick,
/// invariant I2). The cancel brings back the Beacon's own 2 × 2, and the rest of the footprint is free again.
#[test]
fn the_whole_footprint_is_solid_from_the_first_frame() {
    for (kind, origin, size) in [(BioPortal, at(8, 6), 3), (RoboPortal, at(7, 5), 4)] {
        let mut t = base()
            .spawn(Peacekeeper, Some(P1), SpawnAt::Point(at(4, 7).center()))
            .start();
        t.without_auto_targeting().debug_at(1, fund(P1, 1000));
        t.step();
        let beacon = object(&t, 1).placement;
        let footprint: Vec<TilePos> = (0..size)
            .flat_map(|y| (0..size).map(move |x| at(origin.x + x, origin.y + y)))
            .collect();
        let beacon_tiles: Vec<TilePos> = footprint
            .iter()
            .copied()
            .filter(|tile| (8..10).contains(&tile.x) && (6..8).contains(&tile.y))
            .collect();
        let solid = |t: &TestSim| -> Vec<TilePos> {
            let spawn_on = |tile: TilePos| PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Spawn {
                    kind: Peacekeeper,
                    owner: Some(P2),
                    at: SpawnAt::Point(tile.center()),
                    orientation: BuildingOrientation::default(),
                }),
            };
            let blocked = |&tile: &TilePos| t.sim().validate(&spawn_on(tile)) == Err(Reject::Blocked);
            footprint.iter().copied().filter(blocked).collect()
        };
        assert_eq!(solid(&t), beacon_tiles, "{kind:?}: the Beacon's 2 × 2 only");
        let goal = at(13, 7).center();
        let tick = t.sim().tick() + 1;
        t.at(tick, P1, construction(&[1], warp(kind, origin)))
            .order_at(tick, P1, &[id(6)], OrderKind::Move(Target::Point(goal)), false)
            .step();
        assert_eq!(
            solid(&t),
            footprint,
            "{kind:?}: the whole footprint from the first frame"
        );
        let (lo, hi) = (origin.corner(), origin.corner() + Vec2::splat(size as f32 * SU_PER_GU));
        // A Peacekeeper's box is 24 space units across.
        let on_footprint = |p: Vec2| p.x + 12.0 > lo.x && p.x - 12.0 < hi.x && p.y + 12.0 > lo.y && p.y - 12.0 < hi.y;
        for _ in 0..240 {
            t.step();
            assert!(
                !on_footprint(t.pos(6)),
                "{kind:?}: tick {}, at {}",
                t.sim().tick(),
                t.pos(6)
            );
        }
        assert!(t.pos(6).distance(goal) < 8.0, "{kind:?}: around it, to {}", t.pos(6));
        next(&mut t, construction(&[1], CancelTransform));
        let o = object(&t, 1);
        assert_eq!((o.kind, o.placement), (Beacon, beacon), "{kind:?}");
        assert_eq!(solid(&t), beacon_tiles, "{kind:?}: the Beacon's 2 × 2 again");
    }
}

/// Question Q0039 point 8 (placeholder: "its functions (production, drop-off) start at completion"): an Egress Node
/// being warped in takes no drop-off. Player 1's Drone 6 can't be sent to it, nor given it as its AssignedDropOff
/// ([`Reject::NotDropOff`]); sent to gather patch 7 next to it, it mines a load and, with no drop-off to go back to,
/// idles with it (question Q0037 point 8). Once the node is complete, it delivers there.
#[test]
fn an_egress_node_being_warped_in_takes_no_drop_off() {
    let mut t = base()
        .spawn(Drone, Some(P1), SpawnAt::Point(at(6, 9).center()))
        .spawn(CrystalPatch, None, tile(5, 10))
        .start();
    t.debug_at(1, fund(P1, 75))
        .at(1, P1, construction(&[1], warp(EgressNode, at(8, 6))));
    t.step();
    let drone = [id(6)];
    let drop_off = PlayerCommand {
        player: P1,
        command: Command::Order {
            units: drone.to_vec(),
            kind: OrderKind::DropOff(id(1)),
            queue: false,
        },
    };
    let assign = PlayerCommand {
        player: P1,
        command: Command::Ability {
            units: drone.to_vec(),
            ability: Ability::SetDropOff(id(1)),
        },
    };
    assert_eq!(t.sim().validate(&drop_off), Err(Reject::NotDropOff(id(1))));
    assert_eq!(t.sim().validate(&assign), Err(Reject::NotDropOff(id(1))));
    t.order_at(2, P1, &drone, OrderKind::Gather(id(7)), false);
    t.run_until(200);
    let o = object(&t, 6);
    let worker = o.worker.unwrap();
    assert_eq!(
        (worker.load, worker.drop_off),
        (8, None),
        "a load, and nowhere to take it"
    );
    assert!(o.orders.as_ref().unwrap().current.is_none(), "idle");
    t.run_until(320);
    assert!(!object(&t, 1).is_transforming());
    assert_eq!(t.sim().validate(&assign), Ok(()));
    next(&mut t, drop_off.command);
    t.run(200);
    let delivered = t
        .events()
        .iter()
        .any(|(_, e)| matches!(e, SimEvent::ResourcesDelivered { worker, drop_off, amount: 8 } if *worker == id(6) && *drop_off == id(1)));
    assert!(delivered, "delivered at the complete Egress Node");
}

/// The plan's acceptance, "Cancel" (`> Cancel Warp In`: "Cancelling refunds 75% of the building's resource cost
/// (rounded down), and the structure reverts to being a normal, fully-functional Beacon (no HP/Shields loss)"): 75
/// → 56, 150 → 112, 400 → 300 ([`Refund::Ready`]). The Beacon returns on its old tiles, at its full 120 HP, the
/// same object, and can warp in again.
#[test]
fn a_cancel_refunds_75_percent_and_the_beacon_returns() {
    for (b, refund) in BUILDINGS.into_iter().zip([56, 300, 112]) {
        let (kind, (x, y), cost) = (b.kind, b.origin, b.cost);
        assert_eq!(Refund::Ready.of(cost), refund);
        let mut t = base().start();
        let before = object(&t, 1).clone();
        t.debug_at(1, fund(P1, 1000))
            .at(1, P1, construction(&[1], warp(kind, at(x, y))));
        t.run_until(99);
        next(&mut t, construction(&[1], CancelTransform));
        assert_eq!(cancellations(&t), [(100, 1, kind, refund)]);
        assert_eq!(bank(&t, P1), 1000 - cost + refund, "{kind:?}");
        assert!(t.sim().events().contains(&SimEvent::KindChanged {
            id: id(1),
            from: kind,
            to: Beacon
        }));
        let o = object(&t, 1);
        assert_eq!(
            (o.kind, o.placement, o.pos, o.hp, o.owner, &o.transform),
            (Beacon, before.placement, before.pos, Some(120.0), Some(P1), &None),
            "{kind:?}"
        );
        assert_eq!(o.max_hp(), Some(120.0));
        let again = PlayerCommand {
            player: P1,
            command: construction(&[1], warp(kind, at(x, y))),
        };
        assert_eq!(t.sim().validate(&again), Ok(()), "a Beacon again");
    }
}

/// Question Q0039 point 3 (placeholder): a damaged building being warped in returns to a Beacon with the fraction
/// of its HP it had: player 2's Peacekeeper 6 shoots the warping Robo Portal, which is cancelled later.
#[test]
fn a_cancel_keeps_the_fraction_of_hp() {
    let mut t = shot_at();
    t.run_until(299);
    let o = object(&t, 1);
    let fraction = o.hp.unwrap() / o.max_hp().unwrap();
    assert!(fraction < 0.99, "damaged: {fraction}");
    next(&mut t, construction(&[1], CancelTransform));
    let o = object(&t, 1);
    assert_eq!(o.kind, Beacon);
    assert!((o.hp.unwrap() - 120.0 * fraction).abs() < 1e-3, "{:?}", o.hp);
}

/// `ConstructionCommand::CancelTransform` goes to every transforming object it names (plan decision T22), in id
/// order, and to nothing else: a Beacon or a Generator among them is left alone; with nothing transforming it is
/// refused ([`Reject::NotTransforming`]), and anyone else's object refuses it ([`Reject::NotOwned`]).
#[test]
fn a_cancel_goes_to_every_transforming_object_named() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .debug_at(1, fund(P2, 1000))
        .at(1, P1, construction(&[1], warp(EgressNode, at(8, 6))))
        .at(1, P2, construction(&[4], warp(EgressNode, at(16, 12))));
    t.step();
    assert_rejected(
        &mut t,
        P1,
        construction(&[2, 3], CancelTransform),
        Reject::NotTransforming,
    );
    assert_rejected(
        &mut t,
        P1,
        construction(&[1, 4], CancelTransform),
        Reject::NotOwned(id(4)),
    );
    assert_rejected(&mut t, P1, construction(&[], CancelTransform), Reject::NotTransforming);
    next(&mut t, construction(&[1], warp(RoboPortal, at(7, 5))));
    assert_eq!(
        object(&t, 1).kind,
        EgressNode,
        "a building being warped in warps nothing in"
    );
    next(&mut t, construction(&[2], warp(RoboPortal, at(15, 5))));
    assert_eq!(object(&t, 2).kind, RoboPortal);
    next(&mut t, construction(&[3, 2, 1, 2, 99], CancelTransform));
    let tick = t.sim().tick();
    assert_eq!(
        cancellations(&t),
        [(tick, 1, EgressNode, 56), (tick, 2, RoboPortal, 300)]
    );
    assert_eq!((object(&t, 1).kind, object(&t, 2).kind), (Beacon, Beacon));
    assert_eq!(bank(&t, P1), 1000 - 75 - 400 + 56 + 300);
    assert_eq!(object(&t, 4).kind, EgressNode, "player 2's goes on");
}

/// The plan's acceptance, "Destroyed mid-Warp-In" (`> Vulnerability`: "If destroyed mid-Warp-In, the Beacon is
/// destroyed along with it"): the object is removed through the normal path with no refund (placeholder: question
/// Q0039 point 7), and its owner, for whom it was the last structure, is defeated in the same tick
/// (`core/game_rules.md > Defeat Condition`): player 2's Peacekeepers destroy player 1's only structure, a Beacon
/// warping in an Egress Node.
#[test]
fn a_destroyed_warp_in_is_lost_and_its_last_structure_defeats_its_owner() {
    let mut builder = ScenarioBuilder::new(&flat(24, 16))
        .spawn(Beacon, Some(P1), tile(8, 6)) // 1
        .spawn(Generator, Some(P2), tile(20, 12)); // 2
    for (x, y) in [(6, 7), (11, 7), (9, 4), (8, 9)] {
        builder = builder.spawn(Peacekeeper, Some(P2), SpawnAt::Point(at(x, y).center()));
    }
    let mut t = builder.start();
    t.without_auto_targeting()
        .debug_at(1, fund(P1, 75))
        .at(1, P1, construction(&[1], warp(EgressNode, at(8, 6))))
        .order_at(2, P2, &[id(3), id(4), id(5), id(6)], OrderKind::Attack(id(1)), false);
    let mut died = None;
    for _ in 0..300 {
        t.step();
        if t.sim().object(id(1)).is_none() {
            died = Some(t.sim().tick());
            break;
        }
        assert!(object(&t, 1).is_transforming());
    }
    let died = died.expect("destroyed while warping in");
    assert!(died < 320, "before it is complete");
    let events: Vec<&SimEvent> = t
        .events()
        .iter()
        .filter(|(tick, _)| *tick == died)
        .map(|(_, e)| e)
        .collect();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::Died { id: dead, kind: EgressNode, .. } if *dead == id(1)))
    );
    assert!(events.contains(&&SimEvent::Removed(id(1))));
    assert!(events.contains(&&SimEvent::PlayerDefeated { player: P1 }));
    assert!(events.contains(&&SimEvent::GameOver { winners: vec![P2] }));
    assert_eq!(bank(&t, P1), 0, "no refund");
    assert_eq!(cancellations(&t), []);
    assert!(t.sim().player(P1).unwrap().is_defeated());
    assert_eq!(dangling_references(t.sim()), []);
}

/// The gate's other rules for a Warp In (`Sim::validate`): one Beacon of the player's ([`Reject::NoConstructor`],
/// [`Reject::SeveralConstructors`], [`Reject::NotOwned`]), a building on its Constructs List
/// ([`Reject::CannotConstruct`]), a bank that covers the cost as the command applies, after the earlier commands of
/// the tick ([`Reject::InsufficientCrystals`]), and a footprint centred on it. A refused command changes nothing,
/// but a refused footprint is reported.
#[test]
fn refused_warp_ins_change_nothing() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 399));
    t.step();
    let robo = |objects: &[u32]| construction(objects, warp(RoboPortal, at(7, 5)));
    assert_rejected(&mut t, P1, robo(&[]), Reject::NoConstructor);
    assert_rejected(&mut t, P1, robo(&[3]), Reject::NoConstructor);
    assert_rejected(&mut t, P1, robo(&[99]), Reject::NoConstructor);
    assert_rejected(&mut t, P1, robo(&[1, 2]), Reject::SeveralConstructors);
    assert_rejected(&mut t, P1, robo(&[1, 4]), Reject::NotOwned(id(4)));
    assert_rejected(&mut t, P2, robo(&[1]), Reject::NotOwned(id(1)));
    for kind in [ObjectKind::Barracks, Beacon, Generator] {
        let command = construction(&[1], warp(kind, at(8, 6)));
        assert_rejected(&mut t, P1, command, Reject::CannotConstruct(kind));
    }
    assert_rejected(&mut t, P1, robo(&[1]), Reject::InsufficientCrystals);
    let off = construction(&[1], warp(EgressNode, at(9, 6)));
    let refusal = assert_rejected(
        &mut t,
        P1,
        off,
        Reject::Unplaceable(PlacementProblem::Whole(WholeProblem::NotOnBeacon)),
    );
    assert!(matches!(refusal, Some(SimEvent::PlacementRefused { constructor, .. }) if constructor == id(1)));
    // Two Warp Ins in one tick, a bank for one: the second Beacon's is refused, and it stays a Beacon.
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, fund(P1, 200))
        .at(tick, P1, construction(&[1], warp(BioPortal, at(7, 5))))
        .at(tick, P1, construction(&[2], warp(EgressNode, at(16, 6))))
        .step();
    assert_eq!((object(&t, 1).kind, object(&t, 2).kind), (BioPortal, Beacon));
    assert_eq!(bank(&t, P1), 50, "the Egress Node's 75 wasn't there any more");
}

/// A Beacon that is itself transforming (being deployed, slice M3-F) warps nothing in ([`Reject::ConstructorBusy`]).
/// No command makes one yet, so the test edits a saved game: Beacon 1 still deploying from a Drone.
#[test]
fn a_transforming_beacon_warps_nothing_in() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000));
    t.step();
    let mut state = serde_json::to_value(t.sim()).unwrap();
    let deploying = Transform {
        from: Drone,
        from_placement: None,
        progress: 10 * RATIO_ONE,
        total: 48,
        paid: 0,
    };
    let objects = state["objects"].as_array_mut().unwrap();
    let beacon = objects.iter_mut().find(|o| o["id"] == 1).unwrap();
    beacon["transform"] = serde_json::to_value(&deploying).unwrap();
    let sim: Sim = serde_json::from_value(state).unwrap();
    assert!(sim.object(id(1)).unwrap().is_transforming());
    let cmd = PlayerCommand {
        player: P1,
        command: construction(&[1], warp(EgressNode, at(8, 6))),
    };
    assert_eq!(sim.validate(&cmd), Err(Reject::ConstructorBusy(id(1))));
}

/// The plan's acceptance, "Consistency": the gate refuses a Warp In for its footprint exactly when
/// `Sim::warp_in_check` finds a problem, with its first one, for every building, every orientation and every origin
/// around Beacon 1, with a unit, a patch and a structure about.
#[test]
fn the_gate_and_the_warp_in_check_agree() {
    let mut t = base()
        .spawn(Peacekeeper, Some(P2), SpawnAt::Point(at(11, 6).center()))
        .spawn(CrystalPatch, None, tile(6, 9))
        .spawn(Generator, Some(P2), tile(9, 2))
        .start();
    t.debug_at(1, fund(P1, 1000));
    t.step();
    let (mut valid, mut refused) = (0, 0);
    for kind in [EgressNode, RoboPortal, BioPortal] {
        for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
            for (flip_h, flip_v) in [(false, false), (true, true)] {
                let orientation = BuildingOrientation {
                    rotation,
                    flip_h,
                    flip_v,
                };
                for y in 3..10 {
                    for x in 5..12 {
                        let check = t.sim().warp_in_check(id(1), kind, at(x, y), orientation).unwrap();
                        let cmd = PlayerCommand {
                            player: P1,
                            command: construction(
                                &[1],
                                WarpIn {
                                    kind,
                                    origin: at(x, y),
                                    orientation,
                                },
                            ),
                        };
                        let verdict = t.sim().validate(&cmd);
                        match check.first_problem() {
                            None => {
                                valid += 1;
                                assert_eq!(verdict, Ok(()), "{kind:?} at ({x}, {y})");
                            }
                            Some(problem) => {
                                refused += 1;
                                assert_eq!(verdict, Err(Reject::Unplaceable(problem)), "{kind:?} at ({x}, {y})");
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(valid > 0 && refused > valid, "{valid} valid, {refused} refused");
}

/// A game saved in the middle of its Warp Ins loads and goes on exactly as the running one; the replay of its
/// commands, through its file, ends in the same state.
#[test]
fn warp_ins_go_on_alike_after_a_save_and_in_a_replay() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .debug_at(1, fund(P2, 1000))
        .at(1, P1, construction(&[1], warp(RoboPortal, at(7, 5))))
        .at(2, P1, construction(&[2], warp(BioPortal, at(16, 6))))
        .at(3, P2, construction(&[4], warp(EgressNode, at(16, 12))))
        .at(250, P1, construction(&[2], CancelTransform))
        .at(260, P1, construction(&[2], warp(EgressNode, at(16, 6))))
        .at(700, P1, train(&[1], Drone));
    t.run_until(200);
    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(transform(&t, 1).frames_done(), 200);
    while t.sim().tick() < 800 {
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(
        completions(&t),
        [(322, 4, EgressNode), (579, 2, EgressNode), (640, 1, RoboPortal)]
    );
    assert_eq!(cancellations(&t), [(250, 2, BioPortal, 112)]);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}

/// The gate, fuzzed: random Warp Ins and cancels from any player (in the game or not) to any ids, of any kind,
/// anywhere and turned anyhow, with Beacons spawned and objects removed now and then, never panic; a rejected one
/// changes nothing, but a footprint the placement check refuses, which only reports `PlacementRefused`. Each command
/// runs alone in a tick, next to a copy of the game that runs the same tick without it. Twice from the same seed, the
/// runs are identical, and the recorded commands replay to the same state.
#[test]
fn random_warp_ins_never_panic_and_rejections_change_nothing() {
    let (counts, hashes, replay) = warp_in_fuzz(0xbeac);
    assert!(
        counts.accepted > 300 && counts.rejected > 1000 && counts.refused > 50,
        "{counts:?}"
    );
    assert!(counts.completed > 10 && counts.cancelled > 50, "{counts:?}");
    let (_, again, _) = warp_in_fuzz(0xbeac);
    assert_eq!(hashes, again);
    let last = *hashes.last().unwrap();
    let file: Replay = serde_json::from_str(&serde_json::to_string(&replay).unwrap()).unwrap();
    assert_eq!(file.run().state_hash(), last);
}

/// What a fuzz run did.
#[derive(Debug, Default)]
struct Counts {
    accepted: usize,
    rejected: usize,
    /// `SimEvent::PlacementRefused`.
    refused: usize,
    /// `SimEvent::TransformCompleted`.
    completed: usize,
    /// `SimEvent::TransformCancelled`.
    cancelled: usize,
}

fn random_command(rng: &mut Rng, sim: &Sim) -> PlayerCommand {
    let player = PlayerId([1, 1, 1, 2, 2, 2, 0, 3][rng.below(8) as usize]);
    let own: Vec<&Object> = sim.objects().filter(|o| o.owner == Some(player)).collect();
    let beacons: Vec<&Object> = own.iter().copied().filter(|o| o.kind == Beacon).collect();
    let turned = |rng: &mut Rng| BuildingOrientation {
        rotation: [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270][rng.below(4) as usize],
        flip_h: rng.below(2) == 0,
        flip_v: rng.below(2) == 0,
    };
    // Few cancels and removals, so that Warp Ins get to complete.
    let command = match rng.below(40) {
        // Now and then a new Beacon somewhere, or an object removed (a destruction, to the removal path).
        0..2 => {
            return PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Spawn {
                    kind: Beacon,
                    owner: Some(PlayerId(1 + rng.below(2) as u8)),
                    at: tile(rng.below(22) as i32, rng.below(14) as i32),
                    orientation: BuildingOrientation::default(),
                }),
            };
        }
        2 => {
            let ids = sim.objects().map(|o| o.id.0).max().unwrap_or(0) + 1;
            return PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Remove {
                    id: ObjectId(rng.below(ids) + 1),
                }),
            };
        }
        3..6 => {
            // Mostly one of the player's objects, sometimes all of them, else any id.
            let objects = match rng.below(8) {
                _ if own.is_empty() => vec![ObjectId(rng.below(30) + 1)],
                0 => own.iter().map(|o| o.id).collect(),
                1 => vec![ObjectId(rng.below(30) + 1)],
                _ => vec![own[rng.below(own.len() as u32) as usize].id],
            };
            Command::Construction {
                objects,
                command: CancelTransform,
            }
        }
        _ => {
            let kind = if rng.below(6) == 0 {
                ObjectKind::ALL[rng.below(ObjectKind::ALL.len() as u32) as usize]
            } else {
                [EgressNode, RoboPortal, BioPortal][rng.below(3) as usize]
            };
            let (objects, origin) = match beacons.is_empty() || rng.below(5) == 0 {
                true => (
                    vec![ObjectId(rng.below(30) + 1)],
                    at(rng.below(26) as i32 - 2, rng.below(18) as i32 - 2),
                ),
                false => {
                    let beacon = beacons[rng.below(beacons.len() as u32) as usize];
                    let p = beacon.placement.unwrap();
                    let o = at(p.origin.x - rng.below(3) as i32, p.origin.y - rng.below(3) as i32);
                    (vec![beacon.id], o)
                }
            };
            Command::Construction {
                objects,
                command: WarpIn {
                    kind,
                    origin,
                    orientation: turned(rng),
                },
            }
        }
    };
    PlayerCommand { player, command }
}

/// 4000 ticks of one random command each on the base, after a first tick that fills both banks: what happened, the
/// state hash of every tick, and the replay.
fn warp_in_fuzz(seed: u64) -> (Counts, Vec<u64>, Replay) {
    let scenario = base().config(SimConfig::DEBUG).build();
    let mut sim = Sim::new(&scenario);
    let mut replay = Replay::new(scenario);
    let funds = TickInputs {
        commands: [P1, P2]
            .map(|player| PlayerCommand {
                player: P1,
                command: Command::Debug(fund(player, 1_000_000)),
            })
            .to_vec(),
    };
    sim.step(&funds);
    replay.record(funds);
    let mut rng = Rng::from_seed(seed);
    let mut counts = Counts::default();
    let mut hashes = Vec::new();
    for _ in 0..4000 {
        let cmd = random_command(&mut rng, &sim);
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
                            command: WarpIn { .. }, ..
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
                SimEvent::PlacementRefused { .. } => counts.refused += 1,
                SimEvent::TransformCompleted { .. } => counts.completed += 1,
                SimEvent::TransformCancelled { .. } => counts.cancelled += 1,
                _ => {}
            }
        }
        // A transforming object is always the building of its Warp In, standing over where its Beacon stood.
        for o in sim.objects().filter(|o| o.is_transforming()) {
            let (w, p) = (o.transform.as_ref().unwrap(), o.placement.unwrap());
            let from = w.from_placement.unwrap();
            assert!(from.overlaps(&p), "#{}: {p:?} over {from:?}", o.id.0);
            assert_eq!(w.from, Beacon);
            assert!(
                o.production.is_none() && o.constructor.is_none(),
                "#{}: held back",
                o.id.0
            );
        }
        assert_eq!(dangling_references(&sim), []);
        hashes.push(sim.state_hash());
    }
    (counts, hashes, replay)
}
