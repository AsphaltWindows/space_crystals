//! The Cults' panels as plain data (slice M5-C): the Recruitment Hall's, read from a game, which has no X (design change
//! 0342).

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, PlayerId, SpawnAt, TilePos};

use super::*;

const HALL: ObjectKind = ObjectKind::RecruitmentHall;

/// `factions/cults/economy.md > RecruitmentHall > ObjectInterfaceState[RecruitmentHall]`: C Set Rally Point alone,
/// with no Train, as the Hall recruits by itself (`factions/cults/economy.md > RecruitmentHall > Production
/// Behavior`), and no X (design change 0342: "Recruits can't be cancelled. The X button is removed"), before its first
/// step or with a Recruit in production: X's slot is empty and its key does nothing. C awaits its target, and the
/// Hall's right-click sets the rally point (its Production template).
#[test]
fn the_hall_has_c_alone_and_no_x() {
    use ButtonLook::Common;
    use PanelCommand::SetRallyPoint;
    let player = PlayerId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(player, Faction::Cults)
        .spawn(HALL, Some(player), SpawnAt::Tile(TilePos::new(8, 8)))
        .start();
    let hall = Selection::new([(id(1), HALL)]);
    let (x, c) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    let command = PanelButton::Command;
    for ticks in [0, 1, 100] {
        t.run(ticks);
        let shown_now = panel(
            &hall,
            InterfaceState::DefaultState,
            &Conditions::of(t.sim(), player, &hall),
        );
        assert_eq!(
            shown(&shown_now),
            [(c, command(SetRallyPoint), Common)],
            "tick {}",
            t.sim().tick()
        );
        assert_eq!(shown_now.get(x), None);
        assert_eq!(press(&shown_now, x), SlotPress::Nothing);
        assert_eq!(press(&shown_now, c), SlotPress::AwaitTarget(SetRallyPoint));
    }
    assert!(t.sim().production_status(id(1)).is_some(), "a Recruit in production");
    assert_eq!(InterfaceTemplate::of(HALL), Some(InterfaceTemplate::Production));
}
