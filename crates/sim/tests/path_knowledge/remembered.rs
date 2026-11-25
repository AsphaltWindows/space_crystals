//! Remembered obstacles (the second verifications of design changes 0040 and 0042): a building or a patch the
//! player knows of, gone out of its sight, is routed around until the player sees its spot empty.

use super::*;

/// What stands in the gap of [`remembered_until_seen_gone`].
enum InTheGap {
    /// An enemy Generator the unit scouts first (design change 0040).
    ScoutedGenerator,
    /// Two crystal patches of the map, never seen (design change 0042), known from the start (0159).
    MapPatches,
}

/// The second verification of design changes 0040 and 0042: something the player knows of in the gap goes away out
/// of its sight. Its unit, sent east, still routes around the wall's south end; once it has seen the gap empty,
/// the way back leads through it.
fn remembered_until_seen_gone(in_the_gap: InTheGap) {
    let builder = ScenarioBuilder::new(&map(true));
    let (builder, blockers, start) = match in_the_gap {
        InTheGap::ScoutedGenerator => (
            builder.spawn(ObjectKind::Generator, Some(P2), tile(WALL, GAP)), // 1
            vec![1],
            // In sight of the Generator: it is scouted from the start.
            at(21.5, 7.0),
        ),
        InTheGap::MapPatches => (
            builder
                .spawn(ObjectKind::CrystalPatch, None, tile(WALL, GAP)) // 1
                .spawn(ObjectKind::CrystalPatch, None, tile(WALL, GAP + 1)), // 2
            vec![1, 2],
            point(WEST),
        ),
    };
    let unit = blockers.len() as u32 + 1;
    let builder = builder.spawn(PK, Some(P1), SpawnAt::Point(start));
    let builder = match in_the_gap {
        // Player 2 keeps a building far from the wall, never in player 1's sight: losing the one in the gap doesn't
        // defeat them (plan decision T8), and the game goes on.
        InTheGap::ScoutedGenerator => builder.spawn(ObjectKind::Generator, Some(P2), tile(0, 0)),
        InTheGap::MapPatches => builder,
    };
    let mut t = builder.start();
    t.without_auto_targeting();
    let sight = |t: &TestSim, id: u32| t.sim().vision().player(P1).unwrap().record(ObjectId(id)).is_some();
    let assumed = |t: &TestSim| t.sim().path_knowledge().player(P1).unwrap().assumed().len();
    // The map's patches are known from the start like terrain, by records of them (design change 0159: "pathing and
    // memory"), and nothing is assumed beside the records.
    assert!(blockers.iter().all(|&id| sight(&t, id)) && assumed(&t) == 0);
    assert!(known_blocked(t.sim(), P1, WALL, GAP) && known_blocked(t.sim(), P1, WALL, GAP + 1));

    // Away to the west, out of sight; then the gap is cleared behind the player's back.
    move_to(&mut t, unit, point(WEST));
    walk(&mut t, unit);
    assert_eq!(t.pos(unit), point(WEST));
    let tick = t.sim().tick() + 1;
    for &id in &blockers {
        t.debug_at(tick, DebugCommand::Remove { id: ObjectId(id) });
    }
    t.step();
    assert!(blockers.iter().all(|&id| t.sim().object(ObjectId(id)).is_none()));
    let truth = t.sim().nav().grid(NavClass::GroundRugged, 1).unwrap();
    assert!(!truth.is_blocked(TilePos::new(WALL, GAP)) && !truth.is_blocked(TilePos::new(WALL, GAP + 1)));
    assert!(known_blocked(t.sim(), P1, WALL, GAP) && known_blocked(t.sim(), P1, WALL, GAP + 1));
    let before = version(t.sim(), P1);

    // East: the long way around, though the gap is open.
    move_to(&mut t, unit, point(EAST));
    let track = walk(&mut t, unit);
    assert_eq!((t.status(unit), t.pos(unit)), (MoveStatus::Arrived, point(EAST)));
    assert!(went_south(&track) && !went_through_the_gap(&track));
    assert_eq!(version(t.sim(), P1), before, "it never saw the gap");
    assert!(known_blocked(t.sim(), P1, WALL, GAP));
    assert!(blockers.iter().all(|&id| sight(&t, id)), "still remembered");

    // A look at the gap: it is empty.
    move_to(&mut t, unit, point(BY_THE_GAP));
    walk(&mut t, unit);
    assert!(version(t.sim(), P1) > before);
    assert!(!known_blocked(t.sim(), P1, WALL, GAP) && !known_blocked(t.sim(), P1, WALL, GAP + 1));
    assert!(blockers.iter().all(|&id| !sight(&t, id)) && assumed(&t) == 0);

    // And back west, through it.
    move_to(&mut t, unit, point(WEST));
    let track = walk(&mut t, unit);
    assert_eq!((t.status(unit), t.pos(unit)), (MoveStatus::Arrived, point(WEST)));
    assert!(went_through_the_gap(&track) && !went_south(&track));
    assert_eq!(failed_moves(&t), 0);
}

/// Design change 0040, second verification: "Scout an enemy building, leave, have it destroyed out of sight, then
/// path through its spot: the unit routes around the remembered building until it sees the spot is empty."
#[test]
fn a_scouted_building_destroyed_out_of_sight_is_routed_around_until_seen_gone() {
    remembered_until_seen_gone(InTheGap::ScoutedGenerator);
}

/// Design change 0042, second verification: "A patch mined out in the fog is still routed around until a unit sees
/// its tile." A debug removal stands in for the mining (slice M2-B: a depleted patch goes with its plate).
#[test]
fn a_patch_gone_in_the_fog_is_routed_around_until_its_tile_is_seen() {
    remembered_until_seen_gone(InTheGap::MapPatches);
}
