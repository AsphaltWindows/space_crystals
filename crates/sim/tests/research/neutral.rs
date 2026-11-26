//! A defeated player's research (design change 0379, answering question Q0142; `core/game_rules.md > On Defeat`:
//! "**Research is kept too** [...]: the defeated player's completed research is **frozen** at the moment of defeat and
//! keeps applying to their former units [...], so nothing changes on them"): a unit that turns neutral keeps the
//! amounts it had (`sc_sim::Object::kept_research`), in a save too.

use sc_sim::{GameSetup, Sim, Slot};

use super::*;

const P3: PlayerId = PlayerId(3);

/// Design change 0379's Verification ("In a 3-player game, research Enhanced Constitution Level 1, augment a Militant,
/// and get defeated. The neutral Militant keeps its MaxHP (+40) and HP"), so that the defeat doesn't end the game:
/// player 2's Temple (1), six Recruits of theirs inside, completes Enhanced Constitution Level 1 and Improved
/// Metabolism Level 1, and their Enhanced Constitution Militant, at full HP, goes from 60/60 to 80/80 (+4
/// Constitution), their Improved Metabolism one from 30/30 to 20/20 (−2), walking at 6 su a frame. Their Temple, their
/// last structure, destroyed, they are defeated in that tick, and their units turn neutral with the research they
/// had: the first Militant stays at 80/80, the second at 20/20 and 6 su a frame, stopped where it walked in that tick;
/// the Plain Recruits evacuated from the Temple, which no research changes, keep none. A save of the game loads with
/// it. Players 1 and 3, each with a Storage, play on.
#[test]
fn a_defeated_players_units_keep_their_research_when_they_turn_neutral() {
    let builder = cults(&flat(30, 22))
        .spawn(TEMPLE, Some(P2), tile(12, 10)) // 1
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2)); // 2
    let mut scenario = builder.build();
    scenario.map.slots.push(Slot::new(TilePos::new(26, 18)));
    scenario.setup = GameSetup::default_for(&scenario.map);
    for player in &mut scenario.setup.players {
        player.faction = Faction::Cults;
    }
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::Storage,
        owner: Some(P3),
        at: tile(26, 18),
        orientation: BuildingOrientation::default(),
    };
    t.debug_at(1, spawn); // 3
    t.debug_at(1, crystals(P2, BANK));
    run_checked(&mut t, 1);
    let first = next_id(&t);
    for i in 0..6 {
        t.spawn_unit_at(2, RECRUIT, P2, Vec2::new(1150.0, 656.0 + 32.0 * i as f32));
    }
    let staff: Vec<ObjectId> = (0..6).map(|i| id(first + i)).collect();
    let (militant, walker) = (id(first + 6), id(first + 7));
    t.spawn_unit_at(2, MILITANT, P2, Vec2::new(1500.0, 300.0));
    t.spawn_unit_at(2, MILITANT, P2, Vec2::new(60.0, 1300.0));
    run_checked(&mut t, 2);
    set_augmentation(&mut t, 3, militant, RecruitKind::Constitution);
    set_augmentation(&mut t, 3, walker, RecruitKind::Metabolism);
    t.at(3, P2, enter(&staff, T1));
    run_checked(&mut t, 3);
    fill(&mut t, T1, 6);
    assert_eq!(hp(&t, militant), (60.0, 60.0));
    let start = t.sim().tick() + 1;
    t.at(start, P2, research(&[T1], EC1));
    t.at(start, P2, research(&[T1], IM1));
    run_checked(&mut t, start + L1_FRAMES / 3 - 1);
    assert_eq!(techs(&t, P2), [EC1, IM1]);
    assert_eq!(hp(&t, militant), (80.0, 80.0), "player 2's research");
    assert_eq!(hp(&t, walker), (20.0, 20.0));
    let walk = OrderKind::Move(Target::Point(Vec2::new(1850.0, 1300.0)));
    t.order_at(start + L1_FRAMES / 3, P2, &[walker], walk, false);
    let gone = start + L1_FRAMES / 3 + 10;
    remove_at(&mut t, gone, T1);
    run_checked(&mut t, gone - 1);
    let speed = |t: &TestSim| t.sim().object(walker).unwrap().max_speed(t.sim().research());
    assert_eq!(speed(&t), 6.0, "+4 Metabolism");
    let was = t.sim().object(walker).unwrap().pos;
    run_checked(&mut t, gone);
    assert_eq!(t.sim().player(P2).unwrap().defeated_at, Some(gone));
    let moved = t.sim().object(walker).unwrap().pos.distance(was);
    assert!((moved - 6.0).abs() < 1e-3, "it walked at its speed: {moved}");
    for unit in [militant, walker] {
        let o = t.sim().object(unit).expect("on the map");
        assert_eq!(
            (o.owner, o.kept_research),
            (None, Some(P2)),
            "neutral, with player 2's research"
        );
    }
    assert_eq!(hp(&t, militant), (80.0, 80.0), "it keeps its MaxHP (+40) and HP");
    assert_eq!(hp(&t, walker), (20.0, 20.0));
    assert_eq!(speed(&t), 6.0, "and its speed");
    let o = t.sim().object(walker).unwrap();
    assert!(
        o.orders.as_ref().is_some_and(|orders| orders.current.is_none()),
        "stopped"
    );
    for unit in staff {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!((o.owner, o.kept_research, hp(&t, unit)), (None, None, (40.0, 40.0)));
    }
    run_checked(&mut t, gone + 20);
    assert_eq!(hp(&t, militant), (80.0, 80.0), "frozen");
    let saved = serde_json::to_string(t.sim()).unwrap();
    let loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(&loaded, t.sim(), "a save loads with it");
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    let o = loaded.object(militant).unwrap();
    assert_eq!(
        (o.kept_research, o.max_hp_with(loaded.research())),
        (Some(P2), Some(80.0))
    );
    assert_eq!(t.sim().outcome(), None, "players 1 and 3 play on");
    let [p1, p3] = [P1, P3].map(|p| t.sim().player(p).unwrap().defeated_at);
    assert_eq!((p1, p3), (None, None));
}
