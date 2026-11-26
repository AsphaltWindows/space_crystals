//! A defeated player's research (`core/game_rules.md > On Defeat`: their units become neutral;
//! `core/tech_and_research.md > Research > Shared Rules` 1: research "applies to all of that player's eligible
//! objects"): a neutral unit is no one's, so it has no one's research, and its HP follows its MaxHP
//! (`factions/cults/augmentation.md > Increments`: "HP when MaxHP changes"). **Placeholder (Q0142)**: option (a), the
//! literal reading; option (b) would keep the amounts it had.

use sc_sim::{GameSetup, Slot};

use super::*;

const P3: PlayerId = PlayerId(3);

/// Question Q0142, option (a), in a three-player game, so that the defeat doesn't end it: player 2's Temple (1), six
/// Recruits of theirs inside, completes Enhanced Constitution Level 1 and Improved Metabolism Level 1, and their
/// Enhanced Constitution Militant, at full HP, goes from 60/60 to 80/80 (+4 Constitution), their Improved Metabolism
/// one from 30/30 to 20/20 (−2), walking at 6 su a frame. Their Temple, their last structure, destroyed, they are
/// defeated in that tick, and their units turn neutral with the amounts of no research again: the first Militant
/// 60/60 (a decrease of MaxHP only caps HP), the second 30/30 (an increase raises HP as much), stopped where it walked
/// at its old speed in that tick (`TestSim`'s I3 holds); the Plain Recruits evacuated from the Temple don't change.
/// Players 1 and 3, each with a Storage, play on.
#[test]
fn a_defeated_players_units_lose_their_research_when_they_turn_neutral() {
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
    let moved = t.sim().object(walker).unwrap().pos.distance(was);
    assert!(
        (moved - 6.0).abs() < 1e-3,
        "it walked at its speed of the tick before: {moved}"
    );
    assert_eq!(speed(&t), 5.0, "+2 Metabolism, no one's research");
    assert_eq!(hp(&t, walker), (30.0, 30.0));
    assert_eq!(t.sim().player(P2).unwrap().defeated_at, Some(gone));
    let o = t.sim().object(militant).expect("on the map");
    assert_eq!(o.owner, None, "neutral");
    assert_eq!(hp(&t, militant), (60.0, 60.0), "no one's research");
    for unit in staff {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!((o.owner, hp(&t, unit)), (None, (40.0, 40.0)));
    }
    assert_eq!(t.sim().outcome(), None, "players 1 and 3 play on");
    let [p1, p3] = [P1, P3].map(|p| t.sim().player(p).unwrap().defeated_at);
    assert_eq!((p1, p3), (None, None));
}
