//! The Cathedral's panel in the client (slice M5-N4a; `factions/cults/structures/cathedral.md > Placeholder stats`:
//! "Panel like the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject
//! All): **Q: Train Disciple**, **A: Release** in the kind menu"): Recruits right-clicked into it, a kind of Recruit
//! first, with the count of its free Recruits inside, then KindMenu\[kind\], whose Q trains a Disciple, the menu staying
//! open; and the Disciple that comes out: its panel (`factions/cults/units/disciple.md > Placeholder stats`:
//! "BasicCombatUnitInterfaceState, plus **R: Shroud**"), its InfoPanel's energy, and its right-click on an enemy, a
//! move, as it has no weapon. The panels as plain data are the unit tests' (`command_ui/interface/tests/cathedral.rs`),
//! the training itself the sim's (`crates/sim/tests/cathedral`), the energy bar `combat`'s.

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Faction, ObjectId, ObjectKind, OrderKind, PlayerId, ProductionCommand, RecruitKind, SpawnAt, Target, TilePos,
};
use space_crystals::command_ui::feedback::{NOT_FOR_TYPE_TEXT, NOT_YET_TEXT};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand, Planned, PointerDisplay};
use space_crystals::hud::info_panel::InfoText;

use super::armory::{buttons, feedback, inside, until};
use super::*;
use KeyCode::{KeyA, KeyQ, KeyR};
use RecruitKind::{Both, Constitution, Metabolism, Plain};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const CATHEDRAL_KIND: ObjectKind = ObjectKind::Cathedral;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const DISCIPLE: ObjectKind = ObjectKind::Disciple;

/// The objects of [`cathedral_game`], by id, and where they are drawn (the map is flat). The Cathedral, unturned, has
/// its entrance, side B, to the east and its exit, side C, to the west, as the Armory's
/// (`factions/cults/structures/cathedral.md > Placeholder stats`: "EntranceSide B, ExitSide C (like the Armory)").
const CATHEDRAL: u32 = 1;
const CATHEDRAL_AT: Vec2 = Vec2::new(1120.0, 832.0); // tiles (16, 12) to (18, 13): x 1024 to 1216
const RECRUITS: [u32; 2] = [2, 3];

/// A game of player 1, the Cults, on a flat 64 × 48 map, set up as [`game`] does, with 1000 Space Crystals: a
/// Cathedral and two Recruits, which a box selects and a right-click on the Cathedral sends in
/// (`factions/cults/structures/cathedral.md > Training`: "Recruits enter the Cathedral and join an internal pool"; the
/// Enter pointer over it); returned once they are both inside. Player 2, an enemy, has nothing yet.
fn cathedral_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(CATHEDRAL_KIND, Some(P1), SpawnAt::Tile(TilePos::new(16, 12)))
        .spawn(RECRUIT, Some(P1), point(1300.0, 800.0))
        .spawn(RECRUIT, Some(P1), point(1300.0, 864.0));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, CATHEDRAL_AT), Some(PointerDisplay::Enter));
    right_click(&mut app, CATHEDRAL_AT, &[]);
    let enter = OrderKind::Enter(ObjectId(CATHEDRAL));
    assert_eq!(orders(&app).last(), Some(&given(&RECRUITS, enter, false)));
    until(&mut app, 300, |app| inside(app, CATHEDRAL).len() == RECRUITS.len());
    app
}

/// The kind of object `id`, while it is on the map.
fn kind_of(app: &App, id: u32) -> Option<ObjectKind> {
    app.world().resource::<SimRes>().object(ObjectId(id)).map(|o| o.kind)
}

/// The InfoPanel's lines, while it shows them.
fn info_lines(app: &mut App) -> Vec<String> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    (texts.iter(app.world()))
        .find(|(which, _, node)| **which == InfoText::Body && node.display != Display::None)
        .map(|(_, text, _)| text.0.lines().map(String::from).collect())
        .unwrap_or_default()
}

/// `factions/cults/structures/cathedral.md > Training`: "**Disciple**: 1 Recruit, 75 Space Crystals, 320 frames (20
/// seconds)". The Cathedral's DefaultState shows each kind with its count of free Recruits inside, the kinds it has none
/// of greyed, and X greyed with nothing to cancel; Q opens KindMenu\[Plain\]: Q Train Disciple, A Release, Z Back. Q
/// queues one Disciple for 75 Space Crystals, the menu staying open; the Recruit that went in first comes out of side C
/// 320 frames later a Disciple, with its 50 energy of 200 (`factions/cults/units/disciple.md > Placeholder stats`:
/// "Energy - MaxEnergy 200; starts at 50"), and its panel and its InfoPanel show it.
#[test]
fn q_q_trains_a_disciple_that_comes_out_with_its_energy() {
    use ButtonLook::{Back, Common, Disabled, Unavailable};
    use PanelCommand::{
        Attack, AttackGround, CancelProduction, EjectAll, HoldPosition, KindMenu, Move, Patrol, Release, Reverse,
        SetRallyPoint, Stop, TrainFrom,
    };
    let mut app = cathedral_game();
    let first_in = inside(&app, CATHEDRAL)[0];
    click(&mut app, CATHEDRAL_AT, &[]);
    assert_eq!(selected(&app), [CATHEDRAL]);
    let c = PanelButton::Command;
    let (x, rally_slot) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(KindMenu(Plain)), Common, Some(2)),
            (slot(0, 1), c(KindMenu(Constitution)), Disabled, Some(0)),
            (slot(0, 2), c(KindMenu(Metabolism)), Disabled, Some(0)),
            (slot(0, 3), c(KindMenu(Both)), Disabled, Some(0)),
            (slot(1, 0), c(EjectAll), Common, None),
            (x, c(CancelProduction), Disabled, None),
            (rally_slot, c(SetRallyPoint), Common, None),
        ]
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain));
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(TrainFrom(DISCIPLE, Plain)), Common, None),
            (slot(1, 0), c(Release(Plain)), Common, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );
    tap(&mut app, &[], KeyQ);
    let train = ProductionCommand::TrainFrom {
        kind: DISCIPLE,
        from: Plain,
    };
    assert_eq!(productions(&app).last(), Some(&(P1, vec![CATHEDRAL], train)));
    assert_eq!(bank(&app), 1000 - 75);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain), "the menu stays open");

    until(&mut app, 330, |app| kind_of(app, first_in) == Some(DISCIPLE));
    let out = (app.world().resource::<SimRes>().object(ObjectId(first_in)).cloned()).expect("out");
    assert!(out.pos.x <= 1024.0, "out of side C, its west edge: {}", out.pos);
    assert_eq!(sc_sim::energy::of(&out), Some((50.0, 200.0)));
    assert_eq!(inside(&app, CATHEDRAL).len(), 1, "the other Recruit stays");

    // The Disciple: BasicCombatUnitInterfaceState, A greyed for want of a weapon, R holding Shroud's slot.
    select_units(&mut app, &[first_in]);
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Move), Common, None),
            (slot(0, 1), c(Reverse), Unavailable, None),
            (slot(0, 2), c(HoldPosition), Common, None),
            (slot(0, 3), c(PanelCommand::Planned(Planned::Shroud)), Unavailable, None),
            (slot(1, 0), c(Attack), Disabled, None),
            (slot(1, 1), c(Patrol), Common, None),
            (slot(1, 2), c(AttackGround), Unavailable, None),
            (slot(2, 1), c(Stop), Common, None),
        ]
    );
    tap(&mut app, &[], KeyA);
    assert_eq!(feedback(&app), Some(NOT_FOR_TYPE_TEXT));
    assert_eq!(interface(&app), InterfaceState::DefaultState, "no target awaited");
    tap(&mut app, &[], KeyR);
    assert_eq!(feedback(&app), Some(NOT_YET_TEXT));
    let lines = info_lines(&mut app);
    assert_eq!(lines[1..], ["HP 40 / 40", "Energy 50 / 200"], "{lines:?}");
}

/// `factions/cults/units/disciple.md > Placeholder stats`: "AttackAttributes - None". A Disciple never attacks: over an
/// enemy its pointer is Move, and its right-click there moves it to the enemy (design change 0062: "what a unit does on a
/// target it can't act on: it moves"), as the sim's right-click resolution has it (`crates/sim/tests/cathedral`).
#[test]
fn a_disciple_s_right_click_on_an_enemy_moves_it() {
    let mut app = cathedral_game();
    let disciple = spawn(&mut app, DISCIPLE, Some(P1), Vec2::new(900.0, 1000.0));
    let enemy = spawn(&mut app, ObjectKind::Militant, Some(P2), Vec2::new(1050.0, 1000.0));
    select_units(&mut app, &[disciple]);
    let enemy_at = pos(&app, enemy);
    assert_eq!(pointer_over(&mut app, enemy_at), Some(PointerDisplay::Move));
    right_click(&mut app, enemy_at, &[]);
    let to_enemy = OrderKind::Move(Target::Object(ObjectId(enemy)));
    assert_eq!(orders(&app).last(), Some(&given(&[disciple], to_enemy, false)));
}
