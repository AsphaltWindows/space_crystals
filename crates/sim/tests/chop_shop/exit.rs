//! The Seeker's way out (`factions/cults/structures/chop_shop.md`: "EntranceSide B, ExitSide C (like the Armory)";
//! `core/entities.md > Structure Type > Exit Doorway`): "half inside the building, at the middle of the exit side",
//! facing out, or as far in as it takes to stand clear of a unit right in front of the doorway; then straight out by
//! its FixedTurnRadiusMovement (slice M5-V2, `movement::doorway`), or to the rally point. Its 56 × 40 Silhouette turns
//! with its facing (slice M5-V1), so out of a side that faces north or south it is 56 su deep and 40 su across. (M5-S8)
//! A Chop Shop starts with a rally point (`rally.rs`, design change 0446): the tests of a Seeker with none clear it
//! first, as the player does by rallying on the building itself (design change 0447).

use sc_sim::production::Doorway;
use sc_sim::{BuildingOrientation, Rotation, math};

use super::*;

/// [`filled_on`] with player 1's Chop Shop turned by `rotation`.
fn turned(rotation: Rotation, count: u32) -> (TestSim, Vec<ObjectId>) {
    let mut scenario = base().build();
    scenario.map.spawns[0].orientation = BuildingOrientation {
        rotation,
        flip_h: false,
        flip_v: false,
    };
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, count, 5, Vec2::new(1150.0, 1100.0));
    t.at(2, P1, enter(&recruits, CS));
    run_checked(&mut t, 2);
    fill(&mut t, CS, recruits.len());
    let order = inside(&t, CS);
    (t, order)
}

/// (M5-S8) `building`'s rally point, if it has one.
fn rally_of(t: &TestSim, building: ObjectId) -> Option<sc_sim::production::Rally> {
    t.sim().object(building)?.production.as_ref()?.rally
}

/// The exit side's middle and the way out of it, as the Chop Shop of `t` is placed.
fn door(t: &TestSim) -> (Vec2, Vec2) {
    let doorway = Doorway::of(t.sim().object(CS).expect("the Chop Shop")).expect("its exit doorway");
    let door = doorway.doors().next().expect("one door");
    (door.middle(), door.outward())
}

/// The doorway's rule with something in its way (`core/entities.md > Structure Type > Exit Doorway`, design change
/// 0080; `Sim::doorway_spot`), in the four rotations of the Chop Shop, whose exit side, C, faces west, north, east and
/// south in turn: an idle Recruit of player 1's stands right in front of the middle of side C, 36 su out, its box 24 su
/// from the side. The Seeker comes out facing out of C, moved in from half inside (its centre on the side) as far as
/// its 28 su half length and a sixty-fourth of a su take it to stand clear of the Recruit, 4 su in: the same in every
/// rotation, its box turned with it (40 su across the side, 56 su out of it), and touching the Recruit's, never
/// overlapping it. It then drives out, the Recruit stepping aside for it (`systems/units.md`, design change 0353: own
/// idle infantry make way for a vehicle).
#[test]
fn the_seeker_comes_out_of_side_c_turned_with_it_whichever_way_the_chop_shop_faces() {
    let mut ways = Vec::new();
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        let (mut t, order) = turned(rotation, 2);
        let (middle, outward) = door(&t);
        ways.push(outward);
        let blocker = id(next_id(&t));
        t.spawn_unit_at(t.sim().tick() + 1, RECRUIT, P1, middle + outward * 36.0);
        let start = t.sim().tick() + 2;
        t.at(start, P1, train(&[CS], RecruitKind::Plain));
        let (_, seeker) = until_produced(&mut t, CS, FRAMES + 3);
        assert_eq!(seeker, order[0], "{rotation:?}");
        let o = t.sim().object(seeker).expect("out");
        let expected = middle + outward * (36.0 - 12.0 - 28.0 - 1.0 / 64.0);
        assert!(
            o.pos.distance(expected) < 1e-3,
            "{rotation:?}: at {}, not {expected}",
            o.pos
        );
        assert!(
            math::facing_vector(o.facing).distance(outward) < 1e-6,
            "{rotation:?}: facing out"
        );
        let box_half = shape(&t, seeker).bound_half();
        let along = outward.abs();
        assert!(near(box_half.dot(along), 28.0, 1e-3) && near(box_half.dot(Vec2::ONE - along), 20.0, 1e-3));
        assert!(
            !shape(&t, seeker).overlaps(&shape(&t, blocker)),
            "{rotation:?}: clear of the Recruit in front"
        );
        let out = t.sim().tick();
        while t.mover(seeker.0).exit.is_some() {
            assert!(
                t.sim().tick() < out + 120,
                "{rotation:?}: out of the doorway\n{}",
                t.ascii(None)
            );
            step_checked(&mut t);
        }
    }
    let mut sorted: Vec<(i32, i32)> = ways.iter().map(|v| (v.x as i32, v.y as i32)).collect();
    sorted.sort_unstable();
    assert_eq!(sorted, [(-1, 0), (0, -1), (0, 1), (1, 0)], "C faced each way: {ways:?}");
}

/// `core/entities.md > Structure Type > Exit Doorway` 2, "with no rally point the unit walks straight out of the exit
/// side", which a wheeled one does by its model (slice M5-V2, `movement::doorway::leave`: "drives straight out from
/// where it was released, facing out, speeding up and braking to rest just clear of the building"): the Seeker trained
/// with no rally point (its default one cleared, design change 0447) drives west out of side C along the doorway's
/// middle line, never turning, and stops with its box just clear of the footprint.
#[test]
fn with_no_rally_point_it_drives_straight_out_and_stops_just_clear() {
    let (mut t, order) = filled(2);
    let start = t.sim().tick() + 1;
    t.at(start, P1, clear_rally(CS));
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    let (out, seeker) = until_produced(&mut t, CS, FRAMES + 2);
    assert_eq!((seeker, pos(&t, seeker)), (order[0], EXIT));
    while t.mover(seeker.0).momentum != Vec2::ZERO || t.sim().tick() == out {
        assert!(t.sim().tick() < out + 60, "{:?}", t.mover(seeker.0));
        step_checked(&mut t);
        let o = t.sim().object(seeker).expect("on the map");
        assert!(near(o.pos.y, EXIT.y, 1e-3), "straight out: {}", o.pos);
        assert!(math::facing_vector(o.facing).distance(-Vec2::X) < 1e-6, "never turning");
    }
    let front = pos(&t, seeker).x + 28.0;
    assert!(
        front <= EXIT.x + 1e-3 && front > EXIT.x - 4.0,
        "just clear of side C: {}",
        pos(&t, seeker)
    );
    assert_eq!(t.mover(seeker.0).exit, None);
    assert!(
        t.sim()
            .object(seeker)
            .and_then(|o| o.orders.as_ref())
            .is_some_and(|orders| orders.is_idle())
    );
}

/// Two Seekers with no rally point (a reading of the rules as they stand, for the Director: `core/entities.md >
/// Structure Type > Exit Doorway`, "If the way is blocked, it stops in the doorway" and "While a unit still stands in
/// it, the building pauses"; design change 0353, an idle vehicle never makes way). The first parks just clear of side C
/// (above); the second comes out behind it, fully inside the doorway, its box touching the first's, and stands there
/// for good, the doorway occupied: a third waits inside until the player moves the first. (M5-S8) Design change 0447's
/// Verification: "Select a new Chop Shop and right-click it: its default rally point disappears; a second Seeker then
/// stalls behind the first, as before 0446" ("A cleared vehicle building leaves the stall to the player",
/// `core/entities.md > Entrance and exit > Exit Doorway` point 7): the new Chop Shop's rally point, cleared by Set
/// Rally Point on itself, is gone before any Seeker is trained.
#[test]
fn a_second_seeker_with_no_rally_point_stands_in_the_doorway_behind_the_first() {
    let (mut t, _) = filled(6);
    let start = t.sim().tick() + 1;
    assert!(rally_of(&t, CS).is_some(), "a new Chop Shop's default rally point");
    t.at(start, P1, clear_rally(CS));
    run_checked(&mut t, start);
    assert_eq!(rally_of(&t, CS), None, "cleared");
    let start = t.sim().tick() + 1;
    for _ in 0..3 {
        t.at(start, P1, train(&[CS], RecruitKind::Plain));
    }
    let (_, first) = until_produced(&mut t, CS, FRAMES + 2);
    let (_, second) = until_produced(&mut t, CS, FRAMES + 2);
    assert_eq!(
        pos(&t, first),
        EXIT - Vec2::new(28.0, 0.0),
        "parked just clear of side C"
    );
    assert!(
        pos(&t, second).distance(EXIT + Vec2::new(28.0 + 1.0 / 64.0, 0.0)) < 1e-3,
        "fully inside, touching the first: {}",
        pos(&t, second)
    );
    run_for(&mut t, FRAMES + 40);
    assert_eq!(pos(&t, first), EXIT - Vec2::new(28.0, 0.0), "the first never makes way");
    assert_eq!(t.mover(second.0).exit, Some(CS), "the second still in the doorway");
    assert_eq!(produced(&t).len(), 2, "the third waits inside");
    let status = t.sim().production_status(CS).expect("a producer");
    assert!(status.waiting, "{status:?}");
    t.order_at(
        t.sim().tick() + 1,
        P1,
        &[first],
        OrderKind::Move(Target::Point(RALLY)),
        false,
    );
    let (_, third) = until_produced(&mut t, CS, 120);
    assert!(![first, second].contains(&third));
}
