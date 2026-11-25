//! The Warp In's footprint (`core/entities.md > Placement Validation > Warp In`): derived from the Beacon and
//! its orientation (design change 0121), its ring of tiles clear (0118) and every tile visible (0119), and the
//! whole of it solid from the first frame (0109).

use super::*;

/// The plan's acceptance, "Footprints" (`core/entities.md > Placement Validation > Warp In`): a 4 × 4 Robo Portal
/// centred on a 2 × 2 Beacon needs its ring of 12 tiles clear: "All footprint tiles must be Buildable, flat and
/// unoccupied, except for the Beacon's own footprint". A ground unit of any player (design change 0118: "a ground
/// unit of any player on the footprint makes those tiles invalid"; air and underground units, which never block,
/// don't exist yet), a patch or a structure on the ring refuses it, with that tile's problem, and the sim says so
/// (`SimEvent::PlacementRefused`, the Beacon as the constructor); the Beacon's own tiles are ignored, even when the
/// check is shown the Beacon.
#[test]
fn a_robo_portal_needs_its_ring_of_twelve_tiles_clear() {
    let footprint: Vec<TilePos> = (5..9).flat_map(|y| (7..11).map(move |x| at(x, y))).collect();
    let beacon = [at(8, 6), at(9, 6), at(8, 7), at(9, 7)];
    let ring: Vec<TilePos> = footprint.iter().copied().filter(|t| !beacon.contains(t)).collect();
    assert_eq!(ring.len(), 12);
    let t = base().start();
    let check = t.sim().warp_in_check(id(1), RoboPortal, BuildingOrientation::default());
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
                kind: ObjectKind::SabreLegionnaire,
                owner: Some(P2),
                at: SpawnAt::Point(at(10, 7).center()),
                orientation: BuildingOrientation::default(),
            },
            at(10, 7),
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
            .warp_in_check(id(1), RoboPortal, BuildingOrientation::default())
            .unwrap();
        assert_eq!(check.tiles, [(on, problem)], "only the ring tile");
        let refusal = assert_rejected(
            &mut t,
            P1,
            construction(&[1], warp(RoboPortal)),
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
        next(&mut t, construction(&[1], warp(EgressNode)));
        assert_eq!(object(&t, 1).kind, EgressNode, "{problem:?}");
    }
}

/// Design change 0121 (`> Warp In > Warp In Command`: "an odd-sized footprint (3x3 Bio Portal on a 2x2 Beacon) starts
/// with its extra row/column to the bottom-right. Rotate and flip transform the ghost around the Beacon's centre"),
/// each point of its Verification, with Warp Ins on Beacon 1 at (8, 6): the Bio Portal covers the Beacon plus one
/// extra column right and one extra row below; a horizontal flip moves the column left, a vertical flip the row up;
/// each quarter turn moves the extras to the next corner clockwise, "and exit sides rotate accordingly": its exit
/// sides, the A sides (`factions/colonists/structures/bio_portal.md > Exit Behavior`), are the top and the right
/// unturned, and in every orientation one of them faces the extras and the other faces away from them along the other
/// axis, as the right side and the top do unturned. The Egress Node and the Robo Portal are centred exactly in every
/// orientation. The command names no position: the footprint is `transform::warp_in_footprint`'s, which the check and
/// the ghost share.
#[test]
fn a_warp_in_footprint_is_derived_from_its_beacon_and_its_orientation() {
    use Rotation::{R0, R90, R180, R270};
    let turned = |rotation, flip_h, flip_v| BuildingOrientation {
        rotation,
        flip_h,
        flip_v,
    };
    // Where Beacon 1 warps in `kind` turned by `orientation`.
    let warped = |kind, orientation| -> Placement {
        let mut t = base().start();
        t.debug_at(1, fund(P1, 1000))
            .at(1, P1, construction(&[1], WarpIn { kind, orientation }));
        t.step();
        let o = object(&t, 1);
        assert_eq!(o.kind, kind, "{orientation:?}");
        let placement = o.placement.unwrap();
        assert_eq!(placement.orientation(), orientation);
        let beacon = transform(&t, 1).from_placement.unwrap();
        assert_eq!(warp_in_footprint(&beacon, kind, orientation), Some(placement));
        placement
    };
    // The Bio Portal's top-left tile, by the corner its extras are at.
    let (bottom_right, bottom_left, top_left, top_right) = (at(8, 6), at(7, 6), at(7, 5), at(8, 5));
    let corner = |rotation, flip_h, flip_v| {
        let p = warped(BioPortal, turned(rotation, flip_h, flip_v));
        assert_eq!((p.w, p.h), (3, 3));
        p.origin
    };
    assert_eq!(
        corner(R0, false, false),
        bottom_right,
        "an extra column right, an extra row below"
    );
    assert_eq!(
        corner(R0, true, false),
        bottom_left,
        "H flip: the extra column moves left"
    );
    assert_eq!(corner(R0, false, true), top_right, "V flip: the extra row moves up");
    assert_eq!(
        [R0, R90, R180, R270].map(|rotation| corner(rotation, false, false)),
        [bottom_right, bottom_left, top_left, top_right],
        "each quarter turn clockwise: the next corner clockwise"
    );
    for orientation in every_orientation() {
        let p = warped(BioPortal, orientation);
        // Which way the extras lie from the Beacon's centre, (9, 7), along each axis.
        let (dx, dy) = (
            i64::from((2 * p.origin.x + 3 - 18).signum()),
            i64::from((2 * p.origin.y + 3 - 14).signum()),
        );
        let exits: Vec<(i64, i64)> = DIRECTIONS
            .into_iter()
            .zip(sides(BioPortal, orientation).unwrap())
            .filter(|&(_, letter)| letter == SideLetter::A)
            .map(|(direction, _)| direction)
            .collect();
        let towards = exits.iter().find(|d| [(dx, 0), (0, dy)].contains(d));
        let away = exits.iter().find(|d| [(-dx, 0), (0, -dy)].contains(d));
        let (Some(towards), Some(away)) = (towards, away) else {
            panic!("{orientation:?}: exit sides {exits:?}, extras towards ({dx}, {dy})");
        };
        assert_ne!(towards.0 == 0, away.0 == 0, "{orientation:?}: on the two axes");
        for (kind, origin, size) in [(EgressNode, at(8, 6), 2), (RoboPortal, at(7, 5), 4)] {
            if orientation.rotation == R90 || orientation.flip_h {
                let p = warped(kind, orientation);
                assert_eq!((p.origin, p.w, p.h), (origin, size, size), "{kind:?} {orientation:?}");
            }
        }
    }
    // An object that doesn't exist has no Warp In to check.
    let t = base().start();
    assert_eq!(
        t.sim()
            .warp_in_check(id(99), EgressNode, BuildingOrientation::default()),
        None
    );
}

/// Design change 0119 (`core/entities.md > Placement Validation > Warp In`: "as in Direct Placement, every footprint
/// tile must be Visible to the player"; its Verification: "If any footprint tile is not Visible (test setup), it's red
/// regardless of what stands there"). Beacon 7, which a debug spawn puts down in the tick of the Warp In, hasn't
/// looked around yet: the check takes the sight the latest tick left (`Sim::warp_in_check`), and player 1 then sees
/// none of the Robo Portal's footprint around it. The gate refuses it with Direct Placement's visibility problem
/// ([`TileProblem::NotVisible`]), every tile's only one: player 2's Peacekeeper 6 standing on the footprint, out of
/// player 1's sight, shows as nothing else, so the check gives nothing away. A tick later the Beacon sees its
/// surroundings: the Peacekeeper's tile alone is refused, as occupied ([`TileProblem::Unit`]), and the Egress Node,
/// on the Beacon's own tiles, warps in.
#[test]
fn a_warp_in_needs_every_tile_visible() {
    let mut t = base()
        .spawn(Peacekeeper, Some(P2), SpawnAt::Point(at(2, 13).center()))
        .start();
    t.debug_at(1, fund(P1, 1000));
    t.step();
    let footprint: Vec<TilePos> = (10..14).flat_map(|y| (2..6).map(move |x| at(x, y))).collect();
    assert!(
        footprint
            .iter()
            .all(|&tile| t.sim().visibility(P1, tile) != sc_sim::VisibilityState::Visible),
        "player 1 sees none of it"
    );
    let tick = t.sim().tick() + 1;
    t.debug_at(
        tick,
        DebugCommand::Spawn {
            kind: Beacon,
            owner: Some(P1),
            at: tile(3, 11),
            orientation: BuildingOrientation::default(),
        },
    )
    .at(tick, P1, construction(&[7], warp(RoboPortal)))
    .step();
    assert_eq!((object(&t, 7).kind, bank(&t, P1)), (Beacon, 1000), "refused");
    let refused = t.sim().events().iter().find_map(|e| match e {
        SimEvent::PlacementRefused {
            constructor,
            kind,
            origin,
            problems,
            ..
        } => Some((*constructor, *kind, *origin, problems.clone())),
        _ => None,
    });
    let hidden = PlacementCheck {
        tiles: footprint.iter().map(|&tile| (tile, TileProblem::NotVisible)).collect(),
        whole: Vec::new(),
    };
    assert_eq!(refused, Some((id(7), RoboPortal, at(2, 10), hidden)));
    let check = t
        .sim()
        .warp_in_check(id(7), RoboPortal, BuildingOrientation::default())
        .unwrap();
    assert_eq!(check.tiles, [(at(2, 13), TileProblem::Unit)], "it sees now");
    assert_rejected(
        &mut t,
        P1,
        construction(&[7], warp(RoboPortal)),
        Reject::Unplaceable(PlacementProblem::Tile(at(2, 13), TileProblem::Unit)),
    );
    next(&mut t, construction(&[7], warp(EgressNode)));
    assert_eq!(object(&t, 7).kind, EgressNode);
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
        t.at(tick, P1, construction(&[1], warp(kind)))
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
