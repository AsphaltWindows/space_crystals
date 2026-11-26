//! The Augmentations' state, amounts and effects (`super`), at explicit levels: the amounts at the levels no research
//! gives yet (slice M5-N3's) are the table's (`factions/cults/augmentation.md > Augmentation Effects`). The game's own
//! units, through the gate, are `tests/augmentation/`'s.

use super::stats::{movement_of, unarmed_weapon};
use super::*;
use crate::defs::{AUGMENTATION, MovementAttrs, ObjectKind};
use crate::transform;

fn levels(constitution: Level, metabolism: Level) -> Levels {
    Levels {
        constitution,
        metabolism,
    }
}

/// Design change 0337: the four kinds, from the two Augmentations, both ways.
#[test]
fn the_four_kinds_come_from_the_two_augmentations() {
    for kind in RecruitKind::ALL {
        assert_eq!(Augmentation::of(kind).kind(), kind);
    }
    assert_eq!(Augmentation::NONE.kind(), RecruitKind::Plain);
    assert!(Augmentation::default().is_none() && !Augmentation::of(RecruitKind::Both).is_none());
    let names: Vec<&str> = RecruitKind::ALL.iter().map(|kind| kind.name()).collect();
    assert_eq!(names, ["Plain", "Constitution", "Metabolism", "Both"]);
}

/// Left out of files while it has neither, so that states saved before it existed load.
#[test]
fn an_augmentation_saves_only_what_it_has() {
    let constitution = Augmentation::of(RecruitKind::Constitution);
    let json = serde_json::to_string(&constitution).unwrap();
    assert_eq!(json, r#"{"constitution":true}"#);
    assert_eq!(serde_json::from_str::<Augmentation>(&json).unwrap(), constitution);
    assert_eq!(serde_json::from_str::<Augmentation>("{}").unwrap(), Augmentation::NONE);
}

/// `factions/cults/augmentation.md > Augmentation Effects`: each Augmentation's row at its own line's level, both
/// summed, with the design's two examples ("Both at no research: +1 Constitution / +1 Metabolism"; "Enhanced
/// Constitution at Level 2 + Improved Metabolism at no research: +5 Constitution / −1 Metabolism"). None for a unit
/// with neither, whatever the levels.
#[test]
fn the_increments_are_the_table_s_at_each_line_s_level() {
    let [ec, im] = [RecruitKind::Constitution, RecruitKind::Metabolism].map(Augmentation::of);
    let both = Augmentation::of(RecruitKind::Both);
    let table = [(2, -1), (4, -2), (6, -3)];
    for (level, (plus, minus)) in Level::ALL.into_iter().zip(table) {
        let at = levels(level, level);
        assert_eq!(ec.increments(at), Increments::new(plus, minus), "{level:?}");
        assert_eq!(im.increments(at), Increments::new(minus, plus), "{level:?}");
        assert_eq!(
            both.increments(at),
            Increments::new(plus + minus, plus + minus),
            "{level:?}"
        );
        assert_eq!(Augmentation::NONE.increments(at), Increments::NONE);
    }
    let none = Levels::NONE;
    assert_eq!(both.increments(none), Increments::new(1, 1));
    assert_eq!(
        both.increments(levels(Level::L2, Level::NoResearch)),
        Increments::new(5, -1)
    );
    assert_eq!(
        ec.increments(levels(Level::NoResearch, Level::L2)),
        Increments::new(2, -1),
        "its own line's"
    );
}

/// `factions/cults/augmentation.md > Increments`, at explicit levels: "+10 MaxHP" per Constitution increment for
/// every augmentable unit, "+1 melee Damage" and "−1 frame of FiringDuration" per increment for an unarmed one
/// ("never below 2 frames; a negative increment adds 1 frame"), "+0.5 space units/frame MaxSpeed" per Metabolism
/// increment. The Recruit (MaxHP 40, Damage 3, FiringDuration 8, MaxSpeed 4): Enhanced Constitution at Level 2 is
/// +6/−3 (MaxHP 100, Damage 9, FiringDuration 11, MaxSpeed 2.5); Improved Metabolism at Level 2 is −3/+6 (MaxHP 10,
/// Damage 0, FiringDuration 2, MaxSpeed 7; design change 0331: "No floors on the results"); both at no research,
/// +1/+1 (MaxHP 50, Damage 4, FiringDuration 7, MaxSpeed 4.5).
#[test]
fn a_recruit_s_stats_at_explicit_levels() {
    let recruit = ObjectKind::Recruit.def();
    let unit = recruit.unit.as_ref().unwrap();
    let attack = unit.attack.unwrap();
    let stats = |augmentation: RecruitKind, at: Levels| {
        let increments = Augmentation::of(augmentation).increments(at);
        let weapon = unarmed_weapon(&attack, increments);
        (
            max_hp_of(recruit.max_hp.unwrap(), increments),
            weapon.damage,
            weapon.firing_ticks,
            movement_of(unit.movement, increments).max_speed(),
        )
    };
    let l2 = levels(Level::L2, Level::L2);
    assert_eq!(
        stats(RecruitKind::Plain, l2),
        (40.0, 3.0, 8, 4.0),
        "nothing without an Augmentation"
    );
    assert_eq!(stats(RecruitKind::Constitution, l2), (100.0, 9.0, 11, 2.5));
    assert_eq!(stats(RecruitKind::Metabolism, l2), (10.0, 0.0, 2, 7.0));
    assert_eq!(stats(RecruitKind::Both, Levels::NONE), (50.0, 4.0, 7, 4.5));
    let l1 = levels(Level::L1, Level::L1);
    assert_eq!(stats(RecruitKind::Constitution, l1), (80.0, 7.0, 10, 3.0));
    assert_eq!(stats(RecruitKind::Metabolism, l1), (20.0, 1.0, 4, 6.0));
}

/// "FiringDuration never below 2 frames": a weapon of 3 frames, 6 Metabolism increments faster, fires in 2.
#[test]
fn an_unarmed_unit_s_firing_duration_stops_at_two_frames() {
    let attack = ObjectKind::Recruit.def().unit.as_ref().unwrap().attack.unwrap();
    let quick = crate::defs::AttackAttrs {
        firing_ticks: 3,
        ..attack
    };
    assert_eq!(unarmed_weapon(&quick, Increments::new(0, 6)).firing_ticks, 2);
    assert_eq!(unarmed_weapon(&quick, Increments::new(0, -6)).firing_ticks, 9);
    assert_eq!(AUGMENTATION.increments.min_firing_ticks, 2);
}

/// The top speed of each movement model moves by the increments' amount: the field that is its MaxSpeed, the forward
/// one for a fixed turn radius, and for drag the thrust that balances it at the new top speed. The augmentable kinds
/// all move by turn rate.
#[test]
fn every_movement_model_gets_the_metabolism_speed() {
    let per = AUGMENTATION.increments.max_speed;
    for kind in ObjectKind::ALL {
        let Some(unit) = kind.def().unit.as_ref() else {
            continue;
        };
        for metabolism in [-3, 2, 6] {
            let faster = movement_of(unit.movement, Increments::new(0, metabolism));
            let expected = unit.movement.max_speed() + metabolism as f32 * per;
            assert!((faster.max_speed() - expected).abs() < 1e-4, "{kind:?}: {faster:?}");
            assert_eq!(faster.model(), unit.movement.model());
        }
        if unit.augmentable.is_some() {
            assert!(matches!(unit.movement, MovementAttrs::TurnRate { .. }), "{kind:?}");
        }
    }
}

/// `factions/cults/augmentation.md > Eligibility`: "All four Cults infantry types: Recruit, Militant, Gunner, and
/// Disciple", the Recruit alone unarmed (`> Increments`: "currently only the Recruit"; `factions/cults/units/
/// disciple.md > AttackAttributes`: the Disciple "for Augmentation it is **armed**"). And at every level, with any
/// Augmentations, their MaxHP and their top speed stay above 0 ("An augmentation therefore never kills").
#[test]
fn the_four_cults_infantry_types_can_be_augmented() {
    use ObjectKind::{Disciple, Gunner, Militant, Recruit};
    let mut augmentable: Vec<(ObjectKind, bool)> = (ObjectKind::ALL.iter().copied())
        .filter_map(|kind| Some((kind, kind.def().unit.as_ref()?.augmentable?.unarmed)))
        .collect();
    let mut expected = vec![(Recruit, true), (Militant, false), (Gunner, false), (Disciple, false)];
    expected.sort();
    augmentable.sort();
    assert_eq!(augmentable, expected);
    for (kind, _) in augmentable {
        let def = kind.def();
        for constitution in Level::ALL {
            for metabolism in Level::ALL {
                for augmentation in RecruitKind::ALL.map(Augmentation::of) {
                    let increments = augmentation.increments(levels(constitution, metabolism));
                    let movement = def.unit.as_ref().unwrap().movement;
                    assert!(
                        max_hp_of(def.max_hp.unwrap(), increments) > 0.0,
                        "{kind:?} {increments:?}"
                    );
                    assert!(
                        movement_of(movement, increments).max_speed() > 0.0,
                        "{kind:?} {increments:?}"
                    );
                }
            }
        }
    }
}

/// `factions/cults/augmentation.md > Increments`, "HP when MaxHP changes": up, the HP rises as much (25/40 → 45/60, a
/// full unit stays full); down, it is unchanged unless above the new MaxHP (40/40 → 10/10; 8/40 → 8/10). A
/// transformation's rule (`transform::follow_max`, design change 0110) is the same going up and keeps the damage going
/// down, where this one doesn't: 35/40 → 5/10 there, 10/10 here.
#[test]
fn hp_follows_max_hp_by_the_augmentation_rule() {
    assert_eq!(follow_max(25.0, 40.0, 60.0), 45.0);
    assert_eq!(follow_max(40.0, 40.0, 60.0), 60.0);
    assert_eq!(follow_max(40.0, 40.0, 10.0), 10.0);
    assert_eq!(follow_max(8.0, 40.0, 10.0), 8.0);
    assert_eq!(follow_max(35.0, 40.0, 10.0), 10.0);
    assert_eq!(transform::follow_max(35.0, 40.0, 10.0), 5.0);
    for (hp, before, after) in [(25.0, 40.0, 60.0), (40.0, 40.0, 40.0), (1.0, 40.0, 60.0)] {
        assert_eq!(follow_max(hp, before, after), transform::follow_max(hp, before, after));
    }
    // "No floors on the results" and "never kills": a unit with HP keeps some, down to the smallest MaxHP.
    for hp in [0.5, 5.0, 39.0, 40.0] {
        let after = follow_max(hp, 40.0, 10.0);
        assert!(after > 0.0 && after <= 10.0, "{hp}: {after}");
    }
}

/// Everyone at `levels` reads those levels for any owner, none included; the game's players read theirs, "No research"
/// until the Temple (slice M5-N3), as does an object without an owner or one whose player is gone.
#[test]
fn research_reads_each_owner_s_levels() {
    let l2 = levels(Level::L2, Level::L1);
    let everyone = Research::at(l2);
    assert_eq!(everyone.levels(None), l2);
    assert_eq!(everyone.levels(Some(crate::ids::PlayerId(3))), l2);
    let sim = crate::testing::ScenarioBuilder::new("....\n....").build();
    let sim = crate::sim::Sim::new(&sim);
    let research = sim.research();
    for player in sim.players() {
        assert_eq!(research.levels(Some(player.id)), Levels::NONE);
    }
    assert_eq!(research.levels(None), Levels::NONE);
    assert_eq!(research.levels(Some(crate::ids::PlayerId(99))), Levels::NONE);
}

/// (M5-N3) Design change 0379: a neutral unit reads the levels of the defeated player whose research it keeps
/// (`Object::kept_research`), frozen at their defeat; one that keeps none reads "No research"; an owned one reads its
/// owner's, whatever it might keep.
#[test]
fn a_neutral_unit_reads_the_research_it_keeps() {
    use crate::defs::Tech;
    use crate::ids::PlayerId;
    use crate::player::{Faction, Player};
    let mut p1 = Player::starting(PlayerId(1), "P1".to_string(), Faction::Cults);
    p1.techs.insert(Tech::EnhancedConstitutionL1);
    let players = [p1, Player::starting(PlayerId(2), "P2".to_string(), Faction::Cults)];
    let research = Research::new(&players);
    let l1 = levels(Level::L1, Level::NoResearch);
    let mut o = recruit(RecruitKind::Constitution);
    assert_eq!((research.levels_of(&o), o.max_hp_with(research)), (l1, Some(80.0)));
    o.owner = None;
    assert_eq!(
        (research.levels_of(&o), o.max_hp_with(research)),
        (Levels::NONE, Some(60.0))
    );
    o.kept_research = Some(PlayerId(1));
    assert_eq!((research.levels_of(&o), o.max_hp_with(research)), (l1, Some(80.0)));
    o.owner = Some(PlayerId(2));
    assert_eq!(research.levels_of(&o), Levels::NONE, "its owner's");
    assert_eq!(Research::at(l1).levels_of(&o), l1, "everyone's");
}

/// A Recruit of player 1's with the Augmentations of `kind`, out of a game.
fn recruit(kind: RecruitKind) -> crate::object::Object {
    let builder = crate::testing::ScenarioBuilder::new("....\n....").spawn(
        ObjectKind::Recruit,
        Some(crate::ids::PlayerId(1)),
        crate::scenario::SpawnAt::Point(glam::Vec2::new(64.0, 64.0)),
    );
    let mut o = crate::sim::Sim::new(&builder.build()).objects().next().unwrap().clone();
    o.augmentation = Augmentation::of(kind);
    o
}

/// `factions/cults/structures/garrison.md`: "Augmentations don't change the borrowed attack": a Recruit with both
/// Augmentations strikes with its own attack as they make it, and fires a Militant's (slice M5-N2's Garrison) as it is.
#[test]
fn a_borrowed_weapon_is_left_as_it_is() {
    let o = recruit(RecruitKind::Both);
    let weapon = |kind: ObjectKind| kind.def().unit.as_ref().unwrap().attack.unwrap();
    let (own, militant) = (weapon(ObjectKind::Recruit), weapon(ObjectKind::Militant));
    assert_eq!(augmented(&o, &own, Research::NONE).damage, 4.0, "its own, +1");
    assert_eq!(augmented(&o, &militant, Research::NONE), militant);
}

/// HP restored goes up to the MaxHP with the Augmentations: a Constitution Recruit at 50 HP, above the kind's 40, is
/// below its own 60 to a repair (`repair::damage`, slice M5-N8: what it may restore, and the MaxHP of its rate and its
/// price; no repairer repairs infantry yet) and regenerates up to 60 (`crate::regeneration`, the Tunnel Network's).
#[test]
fn restored_hp_goes_up_to_the_max_hp_with_the_augmentations() {
    let mut o = recruit(RecruitKind::Constitution);
    o.hp = Some(50.0);
    assert_eq!(crate::repair::damage(&o, Research::NONE), Some((50.0, 60.0)));
    for _ in 0..100 {
        crate::regeneration::regenerate(&mut o, 0.5, Research::NONE);
    }
    assert_eq!(o.hp, Some(60.0));
    assert_eq!(crate::repair::damage(&o, Research::NONE), None, "full");
}
