//! The Cults' panels as plain data (slice M5-C): the Recruitment Hall's, read from a game, whose X waits for a Recruit
//! in production.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, PlayerId, SpawnAt, TilePos};

use super::*;
use crate::command_ui::feedback::unavailable_text;

const HALL: ObjectKind = ObjectKind::RecruitmentHall;

/// `factions/cults/economy.md > RecruitmentHall > ObjectInterfaceState[RecruitmentHall]`: X Cancel Production and C
/// Set Rally Point, and no Train, as the Hall recruits by itself (`factions/cults/economy.md > RecruitmentHall >
/// Production Behavior`). X is "Only available if ProductionProgress is not None": greyed before the Hall's first
/// step, saying there is nothing to cancel, and sent once a Recruit is in production. C awaits its target, and the
/// Hall's right-click sets the rally point (its Production template).
#[test]
fn the_hall_has_x_and_c_and_its_x_waits_for_a_recruit() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, SetRallyPoint};
    let player = PlayerId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(player, Faction::Cults)
        .spawn(HALL, Some(player), SpawnAt::Tile(TilePos::new(8, 8)))
        .start();
    let hall = Selection::new([(id(1), HALL)]);
    let (x, c) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    let idle = panel(
        &hall,
        InterfaceState::DefaultState,
        &Conditions::of(t.sim(), player, &hall),
    );
    let command = PanelButton::Command;
    assert_eq!(
        shown(&idle),
        [
            (x, command(CancelProduction), Disabled),
            (c, command(SetRallyPoint), Common)
        ]
    );
    assert_eq!(
        idle.get(x).and_then(|s| s.why).map(unavailable_text),
        Some("Nothing to cancel")
    );
    assert_eq!(press(&idle, x), SlotPress::Nothing);
    t.run(1);
    let busy = panel(
        &hall,
        InterfaceState::DefaultState,
        &Conditions::of(t.sim(), player, &hall),
    );
    assert_eq!(busy.get(x).map(|s| s.look), Some(Common));
    assert_eq!(
        press(&busy, x),
        SlotPress::Produce(CancelProduction, ProductionCommand::CancelProduction)
    );
    assert_eq!(press(&busy, c), SlotPress::AwaitTarget(SetRallyPoint));
    assert_eq!(InterfaceTemplate::of(HALL), Some(InterfaceTemplate::Production));
}
