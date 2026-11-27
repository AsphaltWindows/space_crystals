//! Slice M5-G2's three buildings that only unlock, the Gunworks (`factions/cults/structures/gunworks.md`), the Machine
//! Shop (`factions/cults/structures/machine_shop.md`) and the Scrap Hangar (`factions/cults/structures/scrap_hangar.md`),
//! built by Recruits as every Cults building is (`factions/cults/units/recruit.md > Constructs`): each at its price and
//! in its time, behind its TechPrerequisite (`Reject::Requires`), and a right-click on one with Recruits a Move
//! (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState > Right-Click Resolution`: "Own Gunworks / Machine
//! Shop → Move (not enterable until their research is designed, 0319)").

use super::*;
use sc_sim::{ProductionCommand, RecruitKind, Target};

const GUNWORKS: ObjectKind = ObjectKind::Gunworks;
const MACHINE_SHOP: ObjectKind = ObjectKind::MachineShop;
const SCRAP_HANGAR: ObjectKind = ObjectKind::ScrapHangar;
const CHOP_SHOP: ObjectKind = ObjectKind::ChopShop;
const AIRSTRIP: ObjectKind = ObjectKind::Airstrip;
const GUNNER: ObjectKind = ObjectKind::Gunner;

/// Each building with the one its TechPrerequisite names, its price, its BaseConstructionTime, its MaxHP and its
/// side, tiles (`factions/cults/units/recruit.md > Constructs`: "Gunworks: 150 Space Crystals, 480 frames (30 seconds)
/// BaseConstructionTime (placeholder). Requires: Armory."; "Machine Shop: 150 Space Crystals, 480 frames [...]
/// Requires: Chop Shop."; "Scrap Hangar: 200 Space Crystals, 640 frames [...] Requires: Airstrip."; and their design
/// files' Placeholder stats: MaxHP 300, 300 and 450, Size 2x2, 2x2 and 3x3).
const TECH: [(ObjectKind, ObjectKind, u32, u32, f32, i32); 3] = [
    (GUNWORKS, ARMORY, 150, 480, 300.0, 2),
    (MACHINE_SHOP, CHOP_SHOP, 150, 480, 300.0, 2),
    (SCRAP_HANGAR, AIRSTRIP, 200, 640, 450.0, 3),
];

/// [`cults`] with player 1's complete `prerequisite` on tiles (4, 20) on (id 2), and four Recruits of theirs, ids 3 to
/// 6, one off each side of a `side` × `side` building at [`SITE`], 150 su from it (west, east, north, south), so that
/// they get there together; player 1's bank is [`BANK`] from tick 1.
fn four_round(prerequisite: ObjectKind, side: i32) -> TestSim {
    let half = 32.0 * side as f32;
    let centre = Vec2::new(64.0 * SITE.x as f32 + half, 64.0 * SITE.y as f32 + half);
    let mut scenario = cults().spawn(prerequisite, Some(P1), tile(4, 20));
    for out in [Vec2::NEG_X, Vec2::X, Vec2::NEG_Y, Vec2::Y] {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(centre + out * (half + 150.0)));
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    t
}

/// A complete `kind` of `owner`'s, spawned at `origin` in `tick` (a debug spawn, quicker than a build: the Recruits
/// build the Chop Shop and the Airstrip since slices M5-N6 and M5-N7, `tests/chop_shop/normal_game.rs`).
fn spawn(t: &mut TestSim, tick: Tick, kind: ObjectKind, owner: PlayerId, origin: TilePos) {
    let spawn = DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at: SpawnAt::Tile(origin),
        orientation: BuildingOrientation::default(),
    };
    t.debug_at(tick, spawn);
}

/// Plan addendum G2's acceptance: "Four Recruits build each simple building from the Construct menu, at 0319's cost
/// and time, scaled by E's rule." Each, its prerequisite standing, is paid once as the order is given; its site starts
/// at 10% of its MaxHP as the four get to it (`factions/cults/economy.md > Cults Building Mechanics`: sites "start at
/// 10% of their maximum HP"), and it is complete a quarter of its BaseConstructionTime later
/// (`factions/cults/economy.md > BaseConstructionTime Scaling`), at its MaxHP, holding no one and producing nothing.
#[test]
fn four_recruits_build_each_at_its_price_in_a_quarter_of_its_time() {
    for (kind, prerequisite, price, base_frames, max_hp, side) in TECH {
        let mut t = four_round(prerequisite, side);
        let group = [3, 4, 5, 6];
        t.at(2, P1, build(&group, kind, SITE, false));
        step(&mut t);
        step(&mut t);
        assert_eq!(crystals(&t), BANK - price, "{kind:?} paid as the order is given");
        let start = run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
        let site = sites(&t)[0];
        assert_eq!(object(&t, site).kind, kind);
        assert_eq!(object(&t, site).hp, Some(max_hp / 10.0), "{kind:?}: 10% of its MaxHP");
        assert_eq!(
            builders(&t, site).len(),
            4,
            "{kind:?}: all four within reach as it starts"
        );
        let done = run_to(&mut t, 400, "complete", |t| !built(t).is_empty());
        assert_eq!(built(&t), [(done, site)]);
        assert_eq!(
            done - start,
            base_frames / 4,
            "{kind:?}: {base_frames} frames over four"
        );
        let building = object(&t, site);
        assert!(building.is_operational() && building.production.is_none(), "{kind:?}");
        assert_eq!(building.hp, Some(max_hp));
        assert_eq!(t.sim().held_in(site).len(), 0);
        assert_eq!(crystals(&t), BANK - price, "{kind:?} paid once");
        assert!(t.sim().requirement_met(P1, kind));
    }
}

/// Plan addendum G2's acceptance: "The Machine Shop is refused without a complete Chop Shop, and the Scrap Hangar
/// without an Airstrip. The test spawns a complete Chop Shop and Airstrip: their kinds exist." The gate refuses each of
/// the three (`Reject::Requires`, `core/tech_and_research.md > Tech Prerequisites`) while player 1 owns none of the
/// building it requires, another player's not counting, and takes it once theirs stands; the Recruits then build it.
#[test]
fn each_is_refused_until_its_prerequisite_stands() {
    let mut t = world(4);
    let group = recruits(4);
    step(&mut t);
    let orders = TECH.map(|(kind, prerequisite, ..)| (build(&group, kind, SITE, false), prerequisite));
    for (order, prerequisite) in &orders {
        assert_eq!(gate(&t, P1, order.clone()), Err(Reject::Requires(*prerequisite)));
    }
    let origins = [TilePos::new(4, 20), TilePos::new(10, 20), TilePos::new(16, 20)];
    for (&(_, prerequisite), &origin) in orders.iter().zip(&origins) {
        spawn(&mut t, 2, prerequisite, P2, origin);
    }
    run(&mut t, 1);
    for (order, prerequisite) in &orders {
        assert_eq!(
            gate(&t, P1, order.clone()),
            Err(Reject::Requires(*prerequisite)),
            "player 2's"
        );
    }
    // Player 1's Chop Shop and Airstrip, complete, the Machine Shop's and the Scrap Hangar's; the Gunworks still waits
    // for an Armory.
    spawn(&mut t, 3, CHOP_SHOP, P1, TilePos::new(4, 24));
    spawn(&mut t, 3, AIRSTRIP, P1, TilePos::new(10, 24));
    run(&mut t, 1);
    assert_eq!(gate(&t, P1, orders[0].0.clone()), Err(Reject::Requires(ARMORY)));
    assert_eq!(gate(&t, P1, orders[1].0.clone()), Ok(()));
    assert_eq!(gate(&t, P1, orders[2].0.clone()), Ok(()));
    spawn(&mut t, 4, ARMORY, P1, TilePos::new(16, 24));
    run(&mut t, 1);
    assert_eq!(gate(&t, P1, orders[0].0.clone()), Ok(()));
    let now = t.sim().tick() + 1;
    t.at(now, P1, orders[1].0.clone());
    run_to(&mut t, 1000, "the Machine Shop is complete", |t| !built(t).is_empty());
    assert_eq!(object(&t, built(&t)[0].1).kind, MACHINE_SHOP);
    assert_eq!(crystals(&t), BANK - 150);
}

/// Plan addendum G2's acceptance: "W stays greyed, "Requires a Gunworks", until one is complete. Then a Gunner costs
/// 40 SC and 32 frames." (`factions/cults/structures/armory.md > Training`: "TechPrerequisite: player owns at least one
/// Gunworks"; `core/tech_and_research.md > Tech Prerequisites > Counting Rule`: a structure counts "once it is **fully
/// constructed and present on the map**".) Four Recruits build a Gunworks while a fifth is in the Armory: the gate
/// refuses its Gunner while the Gunworks is a site, and takes it once it is complete; the Recruit comes out a Gunner.
#[test]
fn a_gunworks_unlocks_the_gunner_once_it_is_complete() {
    let mut scenario = cults().spawn(ARMORY, Some(P1), tile(4, 20)); // 2: tiles (4, 20) to (6, 21)
    for i in 0..4 {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(start_of(i, 4))); // 3 to 6
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(600.0, 1344.0)); // 7, east of the Armory's entrance, side B
    let (armory, trainee) = (id(2), 7);
    let gunner = Command::Production {
        objects: vec![armory],
        command: ProductionCommand::TrainFrom {
            kind: GUNNER,
            from: RecruitKind::Plain,
        },
    };
    t.at(2, P1, build(&[3, 4, 5, 6], GUNWORKS, SITE, false));
    t.at(2, P1, order(&[trainee], OrderKind::Enter(armory), false));
    run_to(&mut t, WALK, "in the Armory", |t| t.sim().held_in(armory).len() == 1);
    assert_eq!(gate(&t, P1, gunner.clone()), Err(Reject::Requires(GUNWORKS)));
    run_to(&mut t, WALK, "the Gunworks' site starts", |t| !sites(t).is_empty());
    assert_eq!(
        gate(&t, P1, gunner.clone()),
        Err(Reject::Requires(GUNWORKS)),
        "a site is no Gunworks yet"
    );
    run_to(&mut t, 400, "the Gunworks is complete", |t| !built(t).is_empty());
    assert_eq!(gate(&t, P1, gunner.clone()), Ok(()));
    let queued = t.sim().tick() + 1;
    t.at(queued, P1, gunner);
    run(&mut t, 1);
    assert_eq!(crystals(&t), BANK - 150 - 40);
    run_to(&mut t, 40, "the Gunner is out", |t| {
        t.sim().object(id(trainee)).is_some()
    });
    assert_eq!(t.sim().tick(), queued + 31, "32 frames, the command's first");
    assert_eq!(object(&t, id(trainee)).kind, GUNNER);
}

/// `factions/cults/structures/gunworks.md`: "**Until its research is designed it has no panel commands and Recruits
/// can't enter it** (a right-click on it with Recruits is a Move; Q0093)", and the Machine Shop alike; the Scrap Hangar
/// holds no one either. A right-click on each, complete and their own, is a Move (`Sim::default_order`, the holders'
/// rule: an own building that holds nothing); the Recruits walk to it and stay out, and the gate refuses an Enter of
/// one, which holds no one (`Reject::NotATunnel`).
#[test]
fn a_right_click_on_one_with_recruits_is_a_move() {
    let mut t = world(4);
    let origins = [TilePos::new(20, 4), TilePos::new(26, 4), TilePos::new(20, 22)];
    for ((kind, ..), origin) in TECH.into_iter().zip(origins) {
        spawn(&mut t, 1, kind, P1, origin);
    }
    run(&mut t, 2);
    let ids: Vec<ObjectId> = (TECH.iter())
        .map(|&(kind, ..)| t.sim().objects().find(|o| o.kind == kind).expect("spawned").id)
        .collect();
    for &building in &ids {
        let right_click = t.sim().default_order(id(2), Target::Object(building));
        assert_eq!(right_click, Some(OrderKind::Move(Target::Object(building))));
        let enter = order(&[2], OrderKind::Enter(building), false);
        assert_eq!(gate(&t, P1, enter), Err(Reject::NotATunnel(building)));
    }
    let gunworks = ids[0];
    let now = t.sim().tick() + 1;
    t.at(
        now,
        P1,
        order(&recruits(4), OrderKind::Move(Target::Object(gunworks)), false),
    );
    run(&mut t, WALK);
    let works = object(&t, gunworks);
    let footprint = Shape::footprint(&works.placement.expect("a footprint"));
    for n in recruits(4) {
        let recruit = object(&t, id(n));
        assert!(
            Shape::of(recruit).gap(&footprint) < 96.0,
            "#{n} walked to it: {}",
            recruit.pos
        );
    }
    assert_eq!(t.sim().held_in(gunworks).len(), 0, "no one went in");
}
