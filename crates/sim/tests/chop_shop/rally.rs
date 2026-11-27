//! (M5-S8) The Chop Shop's default rally point (`core/entities.md > Entrance and exit > Exit Doorway` point 7, design
//! changes 0446 and 0447, answering Q0175): a building that releases wheeled or tracked vehicles "**starts with a
//! RallyPoint 3 grid units straight out from the middle of its exit side** (placeholder distance)", "set when the
//! building is placed, so it already exists while it is under construction", and "an ordinary rally point: the player
//! can move it, or **clear** it by rallying on the building itself". 0447's own case, a second Seeker stalling behind
//! the first once the rally point is cleared, is `exit.rs`'s.

use sc_sim::production::{Doorway, Rally};
use sc_sim::{BuildingOrientation, Rotation, Sim};

use super::*;

/// "3 grid units" (placeholder distance), space units.
const OUT: f32 = 3.0 * 64.0;
/// [`base`]'s Chop Shop's default rally point: 3 grid units west of [`EXIT`], out of side C.
const DEFAULT: Vec2 = Vec2::new(EXIT.x - OUT, EXIT.y);

/// The rally point of `building`, if it has one.
fn rally_of(sim: &Sim, building: ObjectId) -> Option<Rally> {
    sim.object(building)?.production.as_ref()?.rally
}

/// Every orientation a building can be placed in: four turns, each flipped either way or not.
fn orientations() -> Vec<BuildingOrientation> {
    let mut all = Vec::new();
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        for (flip_h, flip_v) in [(false, false), (true, false), (false, true), (true, true)] {
            all.push(BuildingOrientation {
                rotation,
                flip_h,
                flip_v,
            });
        }
    }
    all
}

/// "Place a Chop Shop: selected, it shows a rally point 3 GU out of side C" (the sim's half; the client shows it,
/// `crates/game/tests/commands/chop_shop.rs`). [`base`]'s, unturned, has it 3 grid units west of the middle of side C,
/// as it comes into the game; and in every orientation, turned or flipped, a Chop Shop has it on the outward normal of
/// the middle of side C, wherever that side faces then (`sc_sim::production::Doorway`, which `exit.rs` turns every
/// way): 3 grid units beyond the footprint's edge there, the footprint 3 × 3 tiles. The gate would take it.
#[test]
fn a_chop_shop_starts_with_a_rally_point_three_grid_units_out_of_side_c() {
    let t = base().start();
    assert_eq!(rally_of(t.sim(), CS), Some(Rally::Point(DEFAULT)));
    for orientation in orientations() {
        let mut scenario = base().build();
        scenario.map.spawns[0].orientation = orientation;
        let t = TestSim::new(scenario);
        let cs = t.sim().object(CS).expect("the Chop Shop");
        let door = Doorway::of(cs).expect("an exit side").doors().next().unwrap();
        let centre = cs.placement.as_ref().unwrap().center();
        let expected = centre + door.outward() * (96.0 + OUT);
        assert_eq!(rally_of(t.sim(), CS), Some(Rally::Point(expected)), "{orientation:?}");
        assert!(
            near(door.middle().distance(centre), 96.0, 1e-3),
            "{orientation:?}: the middle of side C"
        );
        assert_eq!(validate(&t, rally(&[CS], expected)), Ok(()), "{orientation:?}");
    }
}

/// "It is set when the building is placed, so it already exists while it is under construction" (design change 0396:
/// rally points on unfinished buildings). Player 1's three Recruits build a Chop Shop turned a quarter and flipped:
/// its site has the rally point from its first tick, 3 grid units out of side C where that side faces, keeps it
/// through its construction, and keeps it as it completes.
#[test]
fn its_site_has_its_rally_point_from_its_placement_and_keeps_it() {
    let mut t = cults(&flat(30, 22))
        .spawn(ObjectKind::Armory, Some(P1), tile(2, 2))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, 3, 3, Vec2::new(700.0, 300.0));
    let orientation = BuildingOrientation {
        rotation: Rotation::R90,
        flip_h: true,
        flip_v: false,
    };
    let build = OrderKind::Build {
        kind: CHOP_SHOP,
        origin: TilePos::new(12, 10),
        orientation,
    };
    t.order_at(2, P1, &recruits, build, false);
    let site = loop {
        assert!(t.sim().tick() < 300, "no site\n{}", t.ascii(None));
        step_checked(&mut t);
        let site = t.sim().objects().find(|o| o.kind == CHOP_SHOP);
        if let Some(site) = site {
            assert!(site.is_site());
            break site.id;
        }
    };
    let cs = t.sim().object(site).unwrap();
    let door = Doorway::of(cs).unwrap().doors().next().unwrap();
    let expected = Rally::Point(door.middle() + door.outward() * OUT);
    assert_eq!(rally_of(t.sim(), site), Some(expected), "from its first tick");
    while t.sim().object(site).is_some_and(|o| o.is_site()) {
        assert!(t.sim().tick() < 2000, "never complete");
        step_checked(&mut t);
        assert_eq!(rally_of(t.sim(), site), Some(expected), "tick {}", t.sim().tick());
    }
    assert_eq!(rally_of(t.sim(), site), Some(expected), "complete, it keeps it");
}

/// "Build three Seekers without touching the rally: each drives to the rally point area; the doorway never stalls."
/// Six Recruits inside queue three Seekers at once: each comes out 320 frames after the one before, never held back
/// by its doorway (the building never waits, `ProductionStatus::waiting`), and drives to its default rally point:
/// the first stops on it, the second and third as near it as the ones before let them, between it and side C, all
/// out of the doorway (measured: 62 and 126 su east of it, the third's box 38 su clear of side C).
#[test]
fn three_seekers_drive_to_the_rally_point_area_and_the_doorway_never_stalls() {
    let (mut t, _) = filled(6);
    let start = t.sim().tick() + 1;
    for _ in 0..3 {
        t.at(start, P1, train(&[CS], RecruitKind::Plain));
    }
    let mut seekers = Vec::new();
    while seekers.len() < 3 {
        assert!(t.sim().tick() < start + 4 * FRAMES, "three Seekers");
        step_checked(&mut t);
        let stalled = t.sim().production_status(CS).is_some_and(|status| status.waiting);
        assert!(!stalled, "tick {}: stalled", t.sim().tick());
        seekers = produced(&t).into_iter().map(|(tick, _, unit)| (tick, unit)).collect();
    }
    let ticks: Vec<Tick> = seekers.iter().map(|&(tick, _)| tick).collect();
    assert_eq!(
        ticks,
        [ticks[0], ticks[0] + FRAMES, ticks[0] + 2 * FRAMES],
        "one after the other"
    );
    run_for(&mut t, 120);
    assert_eq!(t.sim().production_status(CS), None, "its queue done");
    for (i, &(_, seeker)) in seekers.iter().enumerate() {
        let o = t.sim().object(seeker).expect("on the map");
        assert_eq!(t.mover(seeker.0).exit, None, "Seeker {i}: out of the doorway");
        assert_eq!(t.mover(seeker.0).momentum, Vec2::ZERO, "Seeker {i}: stopped");
        assert!(o.orders.as_ref().is_some_and(|orders| orders.is_idle()), "Seeker {i}");
        assert!(
            o.pos.distance(DEFAULT) < OUT,
            "Seeker {i}: in the rally point's area: {}",
            o.pos
        );
        assert!(
            shape(&t, seeker).bounds().1.x <= EXIT.x,
            "Seeker {i}: clear of the building: {}",
            o.pos
        );
    }
    assert!(pos(&t, seekers[0].1).distance(DEFAULT) < 32.0, "the first at it");
}

/// "Move the rally point: new vehicles go there." The first Seeker drives to the default rally point; the player then
/// moves it (Set Rally Point, as a right-click on the ground does), and the second drives to the new one, not to the
/// default.
#[test]
fn a_moved_rally_point_takes_the_new_vehicles_there() {
    let (mut t, _) = filled(4);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    let (_, first) = until_produced(&mut t, CS, FRAMES + 2);
    let moved = Vec2::new(EXIT.x - 100.0, EXIT.y - 300.0);
    let now = t.sim().tick() + 1;
    t.at(now, P1, rally(&[CS], moved));
    t.at(now, P1, train(&[CS], RecruitKind::Plain));
    run_checked(&mut t, now);
    assert_eq!(rally_of(t.sim(), CS), Some(Rally::Point(moved)));
    let (_, second) = until_produced(&mut t, CS, FRAMES + 2);
    run_for(&mut t, 150);
    assert!(pos(&t, first).distance(DEFAULT) < 32.0, "the first at the default");
    assert!(
        pos(&t, second).distance(moved) < 32.0,
        "the second at the new one: {}",
        pos(&t, second)
    );
}

/// Placeholder (not in the design: a default rally point off the map, or on ground a vehicle can't stand on): a Chop
/// Shop one tile in from the map's west edge, its side C facing it, has its rally point at the map's edge, the point
/// of the map nearest to 3 grid units out, which the gate would take; one with water there has it on the water, as a
/// rally point set by hand may be.
#[test]
fn a_rally_point_off_the_map_is_drawn_in_and_one_on_water_stays() {
    let t = on(&flat(30, 22)).build();
    let mut scenario = t;
    scenario.map.spawns[0].at = tile(1, 10);
    let t = TestSim::new(scenario);
    let edge = Vec2::new(0.0, EXIT.y);
    assert_eq!(rally_of(t.sim(), CS), Some(Rally::Point(edge)), "3 GU out: x = -128");
    assert_eq!(validate(&t, rally(&[CS], edge)), Ok(()));

    let mut rows: Vec<String> = (0..22).map(|_| ".".repeat(30)).collect();
    for row in &mut rows[9..13] {
        row.replace_range(6..11, "~~~~~");
    }
    let t = on(&rows.join("\n")).start();
    assert_eq!(rally_of(t.sim(), CS), Some(Rally::Point(DEFAULT)), "on the water");
    let map = t.sim().map();
    let water = map.tile(TilePos::new(9, 11)).expect("on the map");
    assert!(!map.preset(water.preset).traversible, "{DEFAULT} is on water");
}

/// "Saves and the hash: the rally point is ordinary state" (the brief): a building placed before the default rally
/// point existed has none in its save, and loads as it was, with none. A Chop Shop with its rally point cleared, as
/// every one was then, is saved and loaded: still none, the same game, whose first Seeker parks just clear of side C
/// as before (`exit.rs`).
#[test]
fn a_saved_chop_shop_with_no_rally_point_loads_with_none() {
    let (mut t, _) = filled(2);
    let start = t.sim().tick() + 1;
    t.at(start, P1, clear_rally(CS));
    run_checked(&mut t, start);
    assert_eq!(rally_of(t.sim(), CS), None);
    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(&loaded, t.sim());
    assert_eq!(rally_of(&loaded, CS), None, "loaded with none");
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    let next = t.sim().tick() + 1;
    t.at(next, P1, train(&[CS], RecruitKind::Plain));
    let (_, seeker) = until_produced(&mut t, CS, FRAMES + 2);
    run_for(&mut t, 60);
    for inputs in &t.replay().inputs[loaded.tick() as usize..] {
        loaded.step(inputs);
    }
    assert_eq!(loaded.state_hash(), t.sim().state_hash(), "the same game");
    assert_eq!(pos(&t, seeker), EXIT - Vec2::new(28.0, 0.0), "parked just clear");
}
