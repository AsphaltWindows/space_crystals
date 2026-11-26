//! `DebugCommand::SetAugmentation`'s refusals: like every debug command, none while debug commands are off; none for
//! an object that isn't a unit; and none that gives Augmentations to a unit that can't have them
//! (`factions/cults/augmentation.md > Eligibility`: "Only Cults infantry can be augmented").

use sc_sim::{PlayerCommand, Reject, SimConfig};

use super::*;

/// Player 1's Recruit (1) and Armory (2), and player 2's Peacekeeper (3).
fn scenario() -> sc_sim::Scenario {
    cults(16, 10)
        .spawn(ObjectKind::Recruit, Some(P1), point(100.0, 100.0))
        .spawn(ObjectKind::Armory, Some(P1), SpawnAt::Tile(sc_sim::TilePos::new(6, 4)))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(200.0, 100.0))
        .build()
}

fn set(unit: ObjectId, kind: RecruitKind) -> PlayerCommand {
    let augmentation = Augmentation::of(kind);
    PlayerCommand {
        player: P1,
        command: Command::Debug(DebugCommand::SetAugmentation { id: unit, augmentation }),
    }
}

/// With debug commands off, the gate refuses it (`Reject::DebugDisabled`), and the Recruit stays as it was.
#[test]
fn it_is_refused_while_debug_commands_are_off() {
    let mut t = TestSim::with_config(scenario(), SimConfig::default());
    let command = set(id(1), RecruitKind::Constitution);
    assert_eq!(t.sim().validate(&command), Err(Reject::DebugDisabled));
    t.at(next(&t), P1, command.command);
    t.step();
    assert_eq!(object(&t, id(1)).augmentation, Augmentation::NONE);
    assert_eq!(hp(&t, id(1)), (40.0, 40.0));
}

/// A structure is refused as for every unit's debug command (`Reject::TileObject`), a Peacekeeper as a unit that can't
/// be augmented (`Reject::NotAugmentable`), whoever owns it; taking none from it is allowed, and changes nothing.
#[test]
fn it_is_refused_for_what_cannot_be_augmented() {
    let mut t = TestSim::new(scenario());
    let validate = |t: &TestSim, command| t.sim().validate(&command);
    assert_eq!(validate(&t, set(id(1), RecruitKind::Both)), Ok(()));
    assert_eq!(
        validate(&t, set(id(2), RecruitKind::Both)),
        Err(Reject::TileObject(id(2)))
    );
    assert_eq!(
        validate(&t, set(id(3), RecruitKind::Constitution)),
        Err(Reject::NotAugmentable(id(3)))
    );
    assert_eq!(validate(&t, set(id(3), RecruitKind::Plain)), Ok(()));
    augment(&mut t, id(3), RecruitKind::Metabolism);
    augment(&mut t, id(3), RecruitKind::Plain);
    assert_eq!(object(&t, id(3)).augmentation, Augmentation::NONE);
    assert_eq!(hp(&t, id(3)), (50.0, 50.0), "a Peacekeeper's own");
}
