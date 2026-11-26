//! The Field Engineer's UI in the client (slice M5-N9; design changes 0328 and 0398,
//! `factions/gdo/units/field_engineer.md`): its panel, Q Move, W Construct, D Repair and X Stop, each hint naming it;
//! W's menu, a build menu of its own whose button reads "Constr" (the Recruit's still read "Basic" and "Adv."), with Q
//! Bunker and W First Aid Tent greyed as not in the game yet, and Z back; and the Repair pointer over a damaged own
//! Barracks, whose right-click repairs it
//! for Space Crystals until it is full. The Barracks' E, which trains it once a Career Campus stands, is
//! `production`'s; the Deployment Center's D, the Career Campus, `construction`'s.

use sc_sim::testing::ScenarioBuilder;
use space_crystals::command_ui::feedback::{NOT_YET_TEXT, TargetFeedback};
use space_crystals::command_ui::{BuildMenu, ButtonLook, PanelButton, PanelCommand, Planned};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::hud::info_panel::hint::CommandHint;

use super::*;
use KeyCode::{KeyA, KeyD, KeyQ, KeyW, KeyZ};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// The objects of [`field_engineer_game`], by id: 1 below, 2 player 1's Generator on tiles (18, 9) to (19, 10), 3 to 5
// below, 6 player 2's Deployment Center far off.
/// Player 1's Deployment Center, on tiles (12, 9) to (15, 12).
const DC: u32 = 1;
const DC_AT: Vec2 = Vec2::new(896.0, 704.0);
/// Player 1's Barracks, on tiles (22, 10) to (24, 11).
const BARRACKS: u32 = 3;
const BARRACKS_AT: TilePos = TilePos::new(22, 10);
/// Player 1's two Field Engineers, south of the Barracks.
const ENGINEERS: [u32; 2] = [4, 5];
const ENGINEERS_AT: [Vec2; 2] = [Vec2::new(1440.0, 1000.0), Vec2::new(1520.0, 1000.0)];

/// A game of the GDO against the GDO on a flat 64 × 48 map, as [`game`] sets it up (no fog, no auto-targeting, the
/// camera at [`VIEW_CENTER`]: tiles x 11 to 39, y 8.4 to 19.75): player 1's Deployment Center, Generator, Barracks
/// and two Field Engineers, with 1000 Space Crystals, and player 2's Deployment Center far off.
fn field_engineer_game() -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(12, 9))
        .spawn(ObjectKind::Generator, Some(P1), tile(18, 9))
        .spawn(ObjectKind::Barracks, Some(P1), SpawnAt::Tile(BARRACKS_AT));
    for at in ENGINEERS_AT {
        builder = builder.spawn(ObjectKind::FieldEngineer, Some(P1), SpawnAt::Point(at));
    }
    let builder = builder.spawn(ObjectKind::DeploymentCenter, Some(P2), tile(56, 40));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    app
}

/// Where the cursor points at the Barracks (3 × 2): the middle of its footprint.
fn barracks_at() -> Vec2 {
    BARRACKS_AT.corner() + Vec2::new(3.0, 2.0) * 32.0
}

/// The buttons the HUD shows: slot, button, look and label.
fn buttons(app: &mut App) -> Vec<(CommandSlot, PanelButton, ButtonLook, &'static str)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown: Vec<_> = (query.iter(app.world()))
        .map(|b| (b.slot, b.shown.button, b.shown.look, b.shown.label()))
        .collect();
    shown.sort_by_key(|(slot, ..)| *slot);
    shown
}

/// The feedback message that shows.
fn said(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// What the InfoPanel's command hint says over the button of `slot`, a frame after the pointer moved onto it.
fn hint_over(app: &mut App, slot: CommandSlot) -> Option<String> {
    let surface = app.world().resource::<SurfaceSize>().logical();
    input(app).cursor = Some(CommandGrid::on(surface).slot_rect(slot).center());
    app.update();
    let mut query = app.world_mut().query_filtered::<(&Text, &Node), With<CommandHint>>();
    let (text, node) = query.single(app.world()).expect("one hint");
    (node.display != Display::None).then(|| text.0.clone())
}

/// Player 1's Space Crystals.
fn crystals(app: &App) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.player(P1).expect("in the game").crystals
}

/// The slots of the Field Engineer's letters and of its menu's.
fn q() -> CommandSlot {
    slot(0, 0)
}
fn w() -> CommandSlot {
    slot(0, 1)
}
fn e() -> CommandSlot {
    slot(0, 2)
}
fn d() -> CommandSlot {
    slot(1, 2)
}
fn x() -> CommandSlot {
    slot(2, 1)
}
fn z() -> CommandSlot {
    CommandSlot::BACK
}

/// `factions/gdo/units/field_engineer.md > Placeholder stats`: "Panel (rearranged 2026-11-27 [...]): **Q** Move; **W**
/// Construct (a menu: **Q** Bunker, **W** First Aid Tent, both greyed until designed; **Z** back); **D** Repair [...];
/// **X** Stop" (design change 0398). The selected Field Engineers show those four and no A, carrying no weapon (design
/// change 0399: "Only when **no** selected unit can attack is A not shown"); W is labelled "Constr", its hint naming it
/// Construct; W opens the menu, whose Q Build Bunker and W Set up First Aid Tent are greyed as not in the game yet and
/// say so when pressed, the menu staying open, and Z goes back. D awaits its target and gives no order until it has
/// one.
#[test]
fn the_field_engineers_panel_and_its_construct_menu() {
    use ButtonLook::{Back, Common, Unavailable};
    let mut app = field_engineer_game();
    select_units(&mut app, &ENGINEERS);
    let cmd = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), cmd(PanelCommand::Move), Common, "Move"),
            (
                w(),
                cmd(PanelCommand::BuildMenu(BuildMenu::Construct)),
                Common,
                "Constr"
            ),
            (d(), cmd(PanelCommand::Repair), Common, "Repair"),
            (x(), cmd(PanelCommand::Stop), Common, "Stop"),
        ]
    );
    let hints = [
        (q(), "Q: Move"),
        (w(), "W: Construct"),
        (d(), "D: Repair"),
        (x(), "X: Stop"),
    ];
    for (at, hint) in hints {
        assert_eq!(hint_over(&mut app, at).as_deref(), Some(hint));
    }
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::DefaultState, "no A");

    tap(&mut app, &[], KeyW);
    let menu = InterfaceState::BuildMenu(BuildMenu::Construct);
    assert_eq!(interface(&app), menu);
    let planned = |entry| cmd(PanelCommand::Planned(entry));
    assert_eq!(
        buttons(&mut app),
        [
            (q(), planned(Planned::Bunker), Unavailable, "BKR"),
            (w(), planned(Planned::FirstAidTent), Unavailable, "FAT"),
            (z(), PanelButton::Back, Back, "Back"),
        ]
    );
    let hints = [
        (q(), "Q: Build Bunker"),
        (w(), "W: Set up First Aid Tent"),
        (z(), "Z: Back"),
    ];
    for (at, hint) in hints {
        assert_eq!(hint_over(&mut app, at).as_deref(), Some(hint));
    }
    for key in [KeyQ, KeyW] {
        tap(&mut app, &[], key);
        assert_eq!(said(&app), Some(NOT_YET_TEXT));
        assert_eq!(interface(&app), menu, "the menu stays open");
    }
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    tap(&mut app, &[], KeyD);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Repair));
    assert_eq!(orders(&app), []);
}

/// The Field Engineer's W opens a build menu of its own (`BuildMenu::Construct`), which no other type has: its button
/// reads "Constr", the six letters a label keeps its size with (slice M3-K2a), and its hint "W: Construct". A Recruit
/// of player 1's (a debug spawn) still shows "Basic" in W and "Adv." in E, its two Construct menus (design change
/// 0398), and the Deployment Center the same in Q and W, its two build menus.
#[test]
fn the_field_engineers_w_reads_constr_and_the_recruits_basic() {
    let mut app = field_engineer_game();
    let recruit = spawn(&mut app, ObjectKind::Recruit, Some(P1), Vec2::new(1700.0, 1000.0));
    let shown = |app: &mut App, at: CommandSlot| (buttons(app).into_iter()).find(|b| b.0 == at).map(|b| (b.1, b.3));
    let menu = |menu| PanelButton::Command(PanelCommand::BuildMenu(menu));
    select_units(&mut app, &ENGINEERS);
    assert_eq!(shown(&mut app, w()), Some((menu(BuildMenu::Construct), "Constr")));
    assert_eq!(hint_over(&mut app, w()).as_deref(), Some("W: Construct"));
    select_units(&mut app, &[recruit]);
    assert_eq!(shown(&mut app, w()), Some((menu(BuildMenu::Basic), "Basic")));
    assert_eq!(shown(&mut app, e()), Some((menu(BuildMenu::Advanced), "Adv.")));
    click(&mut app, DC_AT, &[]);
    assert_eq!(selected(&app), [DC]);
    assert_eq!(shown(&mut app, q()), Some((menu(BuildMenu::Basic), "Basic")));
    assert_eq!(shown(&mut app, w()), Some((menu(BuildMenu::Advanced), "Adv.")));
}

/// `factions/gdo/units/field_engineer.md > Placeholder stats`: "Right-click a damaged own building or vehicle: Repair"
/// (design change 0328). Over player 1's Barracks at its full 300 HP the Field Engineers' pointer is Move; at half its
/// HP it is Repair, and a right-click gives the two a Repair of it. They walk to it and restore it, paid for as it is
/// restored (`> Abilities`: "costs **half its price** [...] paid gradually as HP is restored"): half the Barracks' 200
/// Space Crystals from 0 to full, so 50 for its last 150 HP. They stop once it is full ("repairers **stop once the
/// target is at full HP**"), and the pointer over it is Move again.
#[test]
fn a_right_click_on_a_damaged_barracks_repairs_it() {
    let mut app = field_engineer_game();
    select_units(&mut app, &ENGINEERS);
    assert_eq!(pointer_over(&mut app, barracks_at()), Some(PointerDisplay::Move));
    set_hp(&mut app, BARRACKS, 150.0);
    select_units(&mut app, &ENGINEERS);
    assert_eq!(pointer_over(&mut app, barracks_at()), Some(PointerDisplay::Repair));
    right_click(&mut app, barracks_at(), &[]);
    let repair = OrderKind::Repair(ObjectId(BARRACKS));
    assert_eq!(orders(&app), [given(&ENGINEERS, repair, false)]);
    let mut restored = vec![hp(&app, BARRACKS)];
    while restored.last() != Some(&Some(300.0)) && restored.len() < 600 {
        app.update();
        restored.push(hp(&app, BARRACKS));
    }
    assert_eq!(restored.last(), Some(&Some(300.0)), "full");
    assert!(restored.windows(2).all(|w| w[0] <= w[1]), "it only goes up");
    assert_eq!(crystals(&app), 1000 - 50);
    for engineer in ENGINEERS {
        assert_eq!(object_orders(&app, engineer).current, None, "{engineer} stopped");
    }
    assert_eq!(pointer_over(&mut app, barracks_at()), Some(PointerDisplay::Move));
}
