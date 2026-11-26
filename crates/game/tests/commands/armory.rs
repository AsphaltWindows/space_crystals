//! The Armory's panel in the client (slice M5-F2; `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`,
//! `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`): a kind of Recruit first, with the
//! count of its free Recruits inside, then KindMenu\[kind\], whose Q trains a Militant and A releases a Recruit, the
//! menu staying open (design change 0337's verification); A ejects every free Recruit; a right-click sets the rally
//! point; and two Armories selected show one panel at a time, each command going to the one Tabbed to (design change
//! 0348's verification). The panel as plain data is the unit tests' (`command_ui/interface/tests/cults.rs`), the
//! training itself the sim's (`crates/sim/tests/armory`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Augmentation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, PlayerId,
    ProductionCommand, Rally, RecruitKind, SimEvent, SpawnAt, Tick, TilePos,
};
use space_crystals::command_ui::feedback::TargetFeedback;
use space_crystals::command_ui::interface::templates::Planned;
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand};
use space_crystals::hud::command_panel::CommandButton;

use super::*;
use KeyCode::{KeyA, KeyC, KeyE, KeyQ, KeyW, KeyX, KeyZ, Tab};
use RecruitKind::{Both, Constitution, Metabolism, Plain};

const P1: PlayerId = PlayerId(1);
const ARMORY: ObjectKind = ObjectKind::Armory;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;

/// The objects of [`armory_game`], by id, and where they are drawn (the map is flat). Each Armory, unturned, has its
/// entrance, side B, to the east and its exit, side C, to the west.
const ARMORY_1: u32 = 1;
const ARMORY_2: u32 = 2;
const ARMORY_1_AT: Vec2 = Vec2::new(1120.0, 832.0); // tiles (16, 12) to (18, 13): x 1024 to 1216
const ARMORY_2_AT: Vec2 = Vec2::new(1120.0, 1088.0); // tiles (16, 16) to (18, 17)
/// Armory 1's Plain Recruits and its Constitution one, and Armory 2's Plain one.
const PLAIN: [u32; 2] = [3, 4];
const CONSTITUTION: u32 = 5;
const OTHER_PLAIN: u32 = 6;
/// Bare ground west of the Armories, beyond their exits, in the world view.
const RALLY_AT: Vec2 = Vec2::new(820.0, 700.0);

/// A game of player 1, the Cults, on a flat 64 × 48 map, set up as [`game`] does, with 1000 Space Crystals: two
/// Armories, the first with two Plain Recruits and one with Enhanced Constitution inside (a debug command stands for the
/// Augmentation Center, slice M5-G's), the second with one Plain Recruit; returned once they are all inside.
fn armory_game() -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(ARMORY, Some(P1), tile(16, 12))
        .spawn(ARMORY, Some(P1), tile(16, 16))
        .spawn(RECRUIT, Some(P1), point(1300.0, 800.0))
        .spawn(RECRUIT, Some(P1), point(1300.0, 832.0))
        .spawn(RECRUIT, Some(P1), point(1300.0, 864.0))
        .spawn(RECRUIT, Some(P1), point(1300.0, 1088.0));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    let augmentation = Augmentation::of(Constitution);
    let id = ObjectId(CONSTITUTION);
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAugmentation { id, augmentation }),
    );
    app.update();
    enter(&mut app, &[PLAIN[0], PLAIN[1], CONSTITUTION], ARMORY_1);
    enter(&mut app, &[OTHER_PLAIN], ARMORY_2);
    until(&mut app, 300, |app| {
        inside(app, ARMORY_1).len() == 3 && inside(app, ARMORY_2).len() == 1
    });
    app
}

/// `units` walk into Armory `armory` (an Enter order).
fn enter(app: &mut App, units: &[u32], armory: u32) {
    let enter = Command::Order {
        units: units.iter().map(|&id| ObjectId(id)).collect(),
        kind: OrderKind::Enter(ObjectId(armory)),
        queue: false,
    };
    submit(app, P1, enter);
}

/// The ids of the units inside `building`, longest inside first.
fn inside(app: &App, building: u32) -> Vec<u32> {
    let sim = app.world().resource::<SimRes>();
    sim.held_in(ObjectId(building))
        .iter()
        .map(|held| held.object.id.0)
        .collect()
}

/// Whether object `id` is on the map.
fn on_map(app: &App, id: u32) -> bool {
    app.world().resource::<SimRes>().object(ObjectId(id)).is_some()
}

/// Runs frames until `done`, `frames` at most.
fn until(app: &mut App, frames: usize, done: impl Fn(&App) -> bool) {
    for _ in 0..frames {
        if done(app) {
            return;
        }
        app.update();
    }
    assert!(done(app), "not within {frames} frames");
}

/// The buttons the HUD shows: slot, button, look and count.
fn buttons(app: &mut App) -> Vec<(CommandSlot, PanelButton, ButtonLook, Option<u32>)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown: Vec<_> = (query.iter(app.world()))
        .map(|b| (b.slot, b.shown.button, b.shown.look, b.shown.count))
        .collect();
    shown.sort_by_key(|(slot, ..)| *slot);
    shown
}

/// The counts drawn on the buttons, in slot order.
fn counts_drawn(app: &mut App) -> Vec<(CommandSlot, String)> {
    let mut query = app.world_mut().query::<(&Name, &Text, &ChildOf)>();
    let mut drawn: Vec<_> = (query.iter(app.world()))
        .filter(|(name, ..)| name.as_str() == "Command button count")
        .map(|(_, text, child_of)| {
            let button = app
                .world()
                .get::<CommandButton>(child_of.parent())
                .expect("on a button");
            (button.slot, text.0.clone())
        })
        .collect();
    drawn.sort();
    drawn
}

/// The feedback message that shows.
fn feedback(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// Every holding command given so far, as the sim received them (like [`orders`]): player, buildings, command.
fn holdings(app: &App) -> Vec<(PlayerId, Vec<u32>, HoldCommand)> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    commands
        .filter_map(|c| match &c.command {
            Command::Holding { objects, command } => {
                Some((c.player, objects.iter().map(|id| id.0).collect(), *command))
            }
            Command::Order { .. }
            | Command::Ability { .. }
            | Command::Production { .. }
            | Command::Construction { .. }
            | Command::Tunnel { .. }
            | Command::Debug(_) => None,
        })
        .collect()
}

/// Whether producer `id`'s rally point is [`RALLY_AT`], give or take the cursor's rounding.
fn rallied(app: &App, id: u32) -> bool {
    let sim = app.world().resource::<SimRes>();
    let rally = sim.object(ObjectId(id)).and_then(|o| o.production.as_ref()?.rally);
    matches!(rally, Some(Rally::Point(at)) if near(at, RALLY_AT))
}

/// The tick of the sim.
fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

/// Design change 0337's verification: "An Armory with 2 Plain and 1 Constitution Recruit: W shows 1. Pressing W, A, A
/// queues one Release; the second press is greyed. The Constitution Recruit walks out after ¼ s." The DefaultState
/// shows each kind with its count (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "it
/// is greyed if there are none"), E and R greyed, saying why; X is greyed with nothing to cancel. W opens
/// KindMenu\[Constitution\]: Q Train Militant, W the Gunner, greyed ("Requires a Gunworks"), A Release, Z Back. The
/// first A sends one Release, free, the menu staying open; the second finds no Constitution Recruit left and says so,
/// sending nothing. The Recruit comes out in the fourth tick of its Release (4 frames, ¼ s at 16 a second), unchanged,
/// and walks to the rally point C set; Z goes back.
#[test]
fn w_shows_one_and_a_releases_the_constitution_recruit_after_a_quarter_second() {
    use ButtonLook::{Back, Common, Disabled, Unavailable};
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, Release, SetRallyPoint, TrainFrom};
    let mut app = armory_game();
    click(&mut app, ARMORY_1_AT, &[]);
    assert_eq!(selected(&app), [ARMORY_1]);
    let c = PanelButton::Command;
    let (x, rally_slot) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(KindMenu(Plain)), Common, Some(2)),
            (slot(0, 1), c(KindMenu(Constitution)), Common, Some(1)),
            (slot(0, 2), c(KindMenu(Metabolism)), Disabled, Some(0)),
            (slot(0, 3), c(KindMenu(Both)), Disabled, Some(0)),
            (slot(1, 0), c(EjectAll), Common, None),
            (x, c(CancelProduction), Disabled, None),
            (rally_slot, c(SetRallyPoint), Common, None),
        ]
    );
    let drawn = ["2", "1", "0", "0"].map(String::from);
    assert_eq!(
        counts_drawn(&mut app),
        [0, 1, 2, 3].map(|column| (slot(0, column), drawn[column as usize].clone()))
    );
    tap(&mut app, &[], KeyE);
    assert_eq!(feedback(&app), Some("No free Recruit of that kind"));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyX);
    assert_eq!(feedback(&app), Some("Nothing to cancel"));
    tap(&mut app, &[], KeyC);
    click(&mut app, RALLY_AT, &[]);
    assert!(rallied(&app, ARMORY_1));

    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Constitution));
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(TrainFrom(MILITANT, Constitution)), Common, None),
            (slot(0, 1), c(PanelCommand::Planned(Planned::Gunner)), Unavailable, None),
            (slot(1, 0), c(Release(Constitution)), Common, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );
    tap(&mut app, &[], KeyW);
    assert_eq!(feedback(&app), Some("Requires a Gunworks"));

    // W, A: one Release, free; the frames from the press until the Recruit is out.
    let bank_before = bank(&app);
    input(&mut app).tap_key(KeyA);
    let mut events = Vec::new();
    let mut frames = 0;
    while !on_map(&app, CONSTITUTION) {
        assert!(frames < 16, "out within a second");
        app.update();
        frames += 1;
        events.extend(
            app.world()
                .resource::<SimRes>()
                .events()
                .iter()
                .cloned()
                .map(|e| (tick(&app), e)),
        );
        if frames == 2 {
            assert_eq!(buttons(&mut app)[2].2, Disabled, "the second press is greyed");
        }
    }
    let queued = SimEvent::ReleaseQueued {
        producer: ObjectId(ARMORY_1),
        unit: ObjectId(CONSTITUTION),
    };
    let left = SimEvent::LeftBuilding {
        building: ObjectId(ARMORY_1),
        unit: ObjectId(CONSTITUTION),
    };
    let at = |wanted: &SimEvent| events.iter().find(|(_, e)| e == wanted).map(|(tick, _)| *tick);
    let (queued_at, left_at) = (at(&queued).expect("queued"), at(&left).expect("out"));
    assert_eq!(left_at - queued_at, 3, "the fourth tick of its Release: ¼ s");
    assert_eq!(
        frames,
        1 + 4,
        "the press's frame, which sends it, then the Release's 4 frames: ¼ s"
    );
    let releases = |app: &App| {
        (productions(app).into_iter())
            .filter(|(_, _, command)| *command == ProductionCommand::Release(Constitution))
            .count()
    };
    assert_eq!(releases(&app), 1);
    tap(&mut app, &[], KeyA);
    assert_eq!(feedback(&app), Some("No free Recruit of that kind"));
    assert_eq!(releases(&app), 1, "one Release queued");
    assert_eq!(
        interface(&app),
        InterfaceState::KindMenu(Constitution),
        "the menu stays open"
    );
    assert_eq!(bank(&app), bank_before, "free");
    let out = app
        .world()
        .resource::<SimRes>()
        .object(ObjectId(CONSTITUTION))
        .cloned()
        .expect("out");
    assert_eq!(
        (out.kind, out.augmentation),
        (RECRUIT, Augmentation::of(Constitution)),
        "unchanged"
    );
    assert!(out.pos.x < 1024.0, "out of side C: {}", out.pos);
    until(&mut app, 200, |app| pos(app, CONSTITUTION).distance(RALLY_AT) < 40.0);
    let mut plain = inside(&app, ARMORY_1);
    plain.sort_unstable();
    assert_eq!(plain, PLAIN, "the Plain Recruits stay inside");
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(buttons(&mut app)[1].3, Some(0), "W shows none left");
}

/// KindMenu\[Plain\]'s Q trains a Militant of the Plain Recruit that went in first, paying 20 Space Crystals, the menu
/// staying open so that Q can be pressed again ("Clicks stay in the menu so they can be repeated"); the kind button's
/// count drops by the Recruits reserved. X cancels the last entry, its Recruit free again and its Space Crystals back.
/// The Armory's A, Eject All, sends every free Recruit out (the sim's `crates/sim/tests/armory/eject.rs` keeps the
/// reserved ones inside), greyed once none is left
/// (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "Only available if StoredRecruits is not
/// empty"); and a right-click on the ground sets the rally point
/// (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory] > Right-click resolution`).
#[test]
fn q_trains_a_militant_x_cancels_and_a_ejects_the_free_recruits() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::EjectAll;
    let mut app = armory_game();
    let first_in = (inside(&app, ARMORY_1).into_iter())
        .find(|id| PLAIN.contains(id))
        .expect("a Plain Recruit");
    click(&mut app, ARMORY_1_AT, &[]);
    right_click(&mut app, RALLY_AT, &[]);
    assert!(rallied(&app, ARMORY_1) && !rallied(&app, ARMORY_2));
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain));
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    assert_eq!(
        interface(&app),
        InterfaceState::KindMenu(Plain),
        "clicks stay in the menu"
    );
    let train = ProductionCommand::TrainFrom {
        kind: MILITANT,
        from: Plain,
    };
    assert_eq!(
        productions(&app)[1..],
        [(P1, vec![ARMORY_1], train), (P1, vec![ARMORY_1], train)]
    );
    assert_eq!(bank(&app), 1000 - 2 * 20);
    tap(&mut app, &[], KeyQ);
    assert_eq!(feedback(&app), Some("No free Recruit of that kind"));
    tap(&mut app, &[], KeyZ);
    assert_eq!(
        buttons(&mut app)[0],
        (
            slot(0, 0),
            PanelButton::Command(PanelCommand::KindMenu(Plain)),
            Disabled,
            Some(0)
        )
    );
    tap(&mut app, &[], KeyX);
    assert_eq!(bank(&app), 1000 - 20, "refunded");
    assert_eq!(buttons(&mut app)[0].3, Some(1), "its Recruit is free again");

    // A: the free Recruits, the Plain one given back and the Constitution one, come out; the reserved one stays.
    assert_eq!(
        buttons(&mut app)[4],
        (slot(1, 0), PanelButton::Command(EjectAll), Common, None)
    );
    tap(&mut app, &[], KeyA);
    assert_eq!(holdings(&app), [(P1, vec![ARMORY_1], HoldCommand::EjectAll)]);
    until(&mut app, 100, |app| inside(app, ARMORY_1).len() == 1);
    assert_eq!(inside(&app, ARMORY_1), [first_in], "the reserved Recruit stays");
    assert_eq!(buttons(&mut app)[4].2, Disabled, "nothing free to eject");
    tap(&mut app, &[], KeyA);
    assert_eq!(feedback(&app), Some("No free Recruit inside"));
    assert_eq!(holdings(&app).len(), 1);
    let trained = |app: &App| {
        app.world()
            .resource::<SimRes>()
            .object(ObjectId(first_in))
            .map(|o| o.kind)
    };
    until(&mut app, 100, |app| trained(app).is_some());
    assert_eq!(trained(&app), Some(MILITANT));
}

/// Design change 0348's verification: "Box-selecting two Armories gives one panel at a time (Tab between them)". A box
/// selects one building (`systems/controls/selection_and_hud.md > BoxSelection > Priority tiers`), so the second
/// Armory is added with Shift + box. Each Armory is a SelectionGroup of its own (point 5: "Cults production buildings
/// aren't groupable"): the panel shows the counts of the one Tabbed to, and its commands, Q, W and on, X and C, go to
/// that one alone; Tab shows the other's, and goes back to DefaultState.
#[test]
fn two_armories_show_one_panel_at_a_time() {
    use ButtonLook::{Disabled, Group};
    use PanelCommand::{CancelProduction, KindMenu};
    let mut app = armory_game();
    click(&mut app, ARMORY_1_AT, &[]);
    add_box(&mut app, ARMORY_2_AT);
    assert_eq!(selected(&app), [ARMORY_1, ARMORY_2]);
    let groups = |app: &App| {
        let states = app.world().resource::<ControlStates>();
        let selection = states.selection(P1);
        let active = selection.active_group().expect("an ActiveGroup");
        (
            selection.groups().len(),
            active.instances.iter().map(|id| id.0).collect::<Vec<_>>(),
        )
    };
    assert_eq!(groups(&app), (2, vec![ARMORY_1]));
    let c = PanelButton::Command;
    let kinds = |app: &mut App| buttons(app)[..2].to_vec();
    assert_eq!(
        kinds(&mut app),
        [
            (slot(0, 0), c(KindMenu(Plain)), Group, Some(2)),
            (slot(0, 1), c(KindMenu(Constitution)), Group, Some(1)),
        ]
    );
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    let train = ProductionCommand::TrainFrom {
        kind: MILITANT,
        from: Plain,
    };
    assert_eq!(productions(&app), [(P1, vec![ARMORY_1], train)]);

    tap(&mut app, &[], Tab);
    assert_eq!(groups(&app), (2, vec![ARMORY_2]));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(
        kinds(&mut app),
        [
            (slot(0, 0), c(KindMenu(Plain)), Group, Some(1)),
            (slot(0, 1), c(KindMenu(Constitution)), Disabled, Some(0)),
        ]
    );
    let cancel = buttons(&mut app)
        .into_iter()
        .find(|b| b.0 == CommandSlot::CANCEL_PRODUCTION);
    assert_eq!(
        cancel.map(|b| (b.1, b.2)),
        Some((c(CancelProduction), Disabled)),
        "nothing queued here"
    );
    tap(&mut app, &[], KeyC);
    click(&mut app, RALLY_AT, &[]);
    assert!(!rallied(&app, ARMORY_1) && rallied(&app, ARMORY_2));
    tap(&mut app, &[], Tab);
    tap(&mut app, &[], KeyX);
    let cancels: Vec<_> = (productions(&app).into_iter())
        .filter(|(_, _, command)| *command == ProductionCommand::CancelProduction)
        .collect();
    assert_eq!(cancels, [(P1, vec![ARMORY_1], ProductionCommand::CancelProduction)]);
}
