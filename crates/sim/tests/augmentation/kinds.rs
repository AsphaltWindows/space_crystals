//! `factions/cults/augmentation.md > Persistence`: "a trained unit inherits its Recruit's AugmentationState
//! unchanged" (`factions/cults/structures/armory.md > Training`), and keeps their effects: plan decision T73,
//! `Sim::change_kind` keeps a unit's Augmentations through a change to a kind that can have them. "If augmented
//! Recruits are consumed to produce a vehicle, their Augmentations are lost": no vehicle is trained of Recruits yet,
//! so that is the sim's unit tests' (`sim/kinds/tests.rs`, `sim/producers/pool/tests.rs`).

use sc_sim::{ProductionCommand, Research};

use super::*;

/// Player 1's Armory (1), unturned on tiles (12, 10) to (14, 11), its entrance facing east, a complete Gunworks of
/// theirs north of it (2), and two Recruits east of the Armory (3 and 4), with 1000 Space Crystals.
fn armory() -> TestSim {
    let mut t = cults(30, 22)
        .spawn(
            ObjectKind::Armory,
            Some(P1),
            SpawnAt::Tile(sc_sim::TilePos::new(12, 10)),
        )
        .spawn(
            ObjectKind::Gunworks,
            Some(P1),
            SpawnAt::Tile(sc_sim::TilePos::new(20, 2)),
        )
        .spawn(ObjectKind::Recruit, Some(P1), point(1150.0, 688.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(1150.0, 720.0))
        .start();
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(next(&t), crystals);
    t
}

fn train(kind: ObjectKind, from: RecruitKind) -> Command {
    Command::Production {
        objects: vec![id(1)],
        command: ProductionCommand::TrainFrom { kind, from },
    }
}

/// A Recruit with Enhanced Constitution and one with both, trained at the Armory into a Militant and a Gunner, come
/// out with their Augmentations and their effects, unhurt: the Militant of MaxHP 60, the Gunner of MaxHP 50 and
/// MaxSpeed 4.5 (+1/+1 at no research), its weapon its kind's.
#[test]
fn a_recruit_trained_into_infantry_keeps_its_augmentations() {
    let mut t = armory();
    let (constitution, both) = (id(3), id(4));
    augment(&mut t, constitution, RecruitKind::Constitution);
    augment(&mut t, both, RecruitKind::Both);
    let enter = Command::Order {
        units: vec![constitution, both],
        kind: OrderKind::Enter(id(1)),
        queue: false,
    };
    t.at(next(&t), P1, enter);
    while t.sim().held_in(id(1)).len() < 2 {
        assert!(t.sim().tick() < 300, "never went in");
        t.step();
    }
    t.at(next(&t), P1, train(ObjectKind::Militant, RecruitKind::Constitution));
    t.at(next(&t), P1, train(ObjectKind::Gunner, RecruitKind::Both));
    while of_kind(&t, ObjectKind::Militant).len() + of_kind(&t, ObjectKind::Gunner).len() < 2 {
        assert!(t.sim().tick() < 600, "never trained");
        t.step();
    }
    let research = t.sim().research();
    let militant = object(&t, constitution);
    assert_eq!(
        (militant.kind, militant.augmentation),
        (ObjectKind::Militant, Augmentation::of(RecruitKind::Constitution))
    );
    assert_eq!(hp(&t, constitution), (60.0, 60.0));
    let gunner = object(&t, both);
    assert_eq!(
        (gunner.kind, gunner.augmentation.kind()),
        (ObjectKind::Gunner, RecruitKind::Both)
    );
    assert_eq!((hp(&t, both), gunner.max_speed(research)), ((50.0, 50.0), 4.5));
    let own = ObjectKind::Gunner.def().unit.as_ref().unwrap().attack;
    assert_eq!(sc_sim::combat::weapon(gunner, Research::NONE), own);
}
