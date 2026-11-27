//! The Airstrip in the client (slice M5-N7; `factions/cults/structures/airstrip.md > Placeholder stats`: "Panel like
//! the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Build
//! Sparrow**, **A: Release** in the kind menu"): Recruits right-clicked into it, the selected Airstrip's door arrows
//! along its runway, a kind of Recruit first, then KindMenu\[kind\], whose Q builds a Sparrow, which takes off along the
//! runway and flies, its panel the BasicCombatUnit's. The panel as plain data is the unit tests'
//! (`command_ui/interface/tests/airstrip.rs`), the training and the takeoff the sim's (`crates/sim/tests/airstrip`),
//! the arrows' and the runway's look theirs (`door_arrows/tests.rs`, `object_render/runway.rs`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, ObjectId, ObjectKind, OrderKind, PlayerId, ProductionCommand, RecruitKind, SpawnAt, TilePos};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand, PointerDisplay};
use space_crystals::door_arrows::{Way, door_arrows};
use space_crystals::hud::info_panel::hint::CommandHint;
use space_crystals::selection::doors::SelectedDoors;

use super::armory::{buttons, inside, until};
use super::*;
use KeyCode::KeyQ;
use RecruitKind::{Both, Constitution, Metabolism, Plain};

const P1: PlayerId = PlayerId(1);
const AIRSTRIP_KIND: ObjectKind = ObjectKind::Airstrip;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const SPARROW: ObjectKind = ObjectKind::Sparrow;

/// The objects of [`airstrip_game`], by id, and where they are drawn (the map is flat). The Airstrip, unturned (4 × 2,
/// "Size 2x4 (a runway)" in the default orientation), has its entrance, side B, to the east and the end its Sparrows
/// take off toward, side C, to the west.
const AIRSTRIP: u32 = 1;
const AIRSTRIP_AT: Vec2 = Vec2::new(1152.0, 832.0); // tiles (16, 12) to (19, 13): x 1024 to 1280, y 768 to 896
const RECRUITS: [u32; 2] = [2, 3];

/// A game of player 1, the Cults, on a flat 64 × 48 map, set up as [`game`] does, with 1000 Space Crystals: an
/// Airstrip and two Recruits, which a box selects and a right-click on the Airstrip sends in (the Enter pointer over
/// it: `factions/cults/units/recruit.md > Right-Click Resolution`, "an own structure that can hold units"); returned
/// once they are both inside.
fn airstrip_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(PlayerId(2), Faction::Cults)
        .spawn(AIRSTRIP_KIND, Some(P1), SpawnAt::Tile(TilePos::new(16, 12)))
        .spawn(RECRUIT, Some(P1), point(1400.0, 800.0))
        .spawn(RECRUIT, Some(P1), point(1400.0, 864.0));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, AIRSTRIP_AT), Some(PointerDisplay::Enter));
    right_click(&mut app, AIRSTRIP_AT, &[]);
    let enter = OrderKind::Enter(ObjectId(AIRSTRIP));
    assert_eq!(orders(&app).last(), Some(&given(&RECRUITS, enter, false)));
    until(&mut app, 300, |app| inside(app, AIRSTRIP).len() == RECRUITS.len());
    app
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

/// The object `id`, while it is on the map.
fn object(app: &App, id: u32) -> Option<sc_sim::Object> {
    app.world().resource::<SimRes>().object(ObjectId(id)).cloned()
}

/// `factions/cults/structures/airstrip.md > Placeholder stats`: "**Sparrow**: 1 Recruit, 75 Space Crystals, 320 frames
/// (20 seconds)". Selected, the Airstrip shows its door arrows, in at the middle of B and out at the middle of C, along
/// its runway (design change 0394), and the pool buildings' DefaultState: each kind with its count of free Recruits
/// inside, the kinds it has none of greyed, X greyed with nothing to cancel. Q opens KindMenu\[Plain\]: Q Build Sparrow,
/// whose hint names it and its price, A Release, Z Back. Q queues one Sparrow for 75 Space Crystals, the menu staying
/// open; the Recruit that went in first comes out 320 frames later a Sparrow over the runway's B end, flying toward C,
/// and its panel is the BasicCombatUnit's.
#[test]
fn q_q_builds_a_sparrow_that_takes_off_along_the_runway() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, Release, SetRallyPoint, TrainFrom};
    let mut app = airstrip_game();
    let first_in = inside(&app, AIRSTRIP)[0];
    click(&mut app, AIRSTRIP_AT, &[]);
    assert_eq!(selected(&app), [AIRSTRIP]);
    let doors = app.world().resource::<SelectedDoors>();
    let arrows: Vec<(Way, Vec2)> = (doors.shown.iter())
        .flat_map(|(kind, placement)| door_arrows(*kind, placement))
        .map(|arrow| (arrow.way, arrow.at))
        .collect();
    assert_eq!(
        arrows,
        [
            (Way::In, Vec2::new(1280.0, 832.0)),
            (Way::Out, Vec2::new(1024.0, 832.0))
        ]
    );
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
            (slot(0, 0), c(TrainFrom(SPARROW, Plain)), Common, None),
            (slot(1, 0), c(Release(Plain)), Common, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );
    let hint = hint_over(&mut app, slot(0, 0)).expect("a hint");
    assert!(hint.starts_with("Q: Build Sparrow"), "{hint}");
    tap(&mut app, &[], KeyQ);
    let train = ProductionCommand::TrainFrom {
        kind: SPARROW,
        from: Plain,
    };
    assert_eq!(productions(&app).last(), Some(&(P1, vec![AIRSTRIP], train)));
    assert_eq!(bank(&app), 1000 - 75);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain), "the menu stays open");

    until(&mut app, 330, |app| object(app, first_in).is_some());
    let out = object(&app, first_in).expect("out");
    assert_eq!(out.kind, SPARROW);
    assert_eq!(
        out.pos,
        Vec2::new(1260.0, 832.0),
        "over the B end, wholly on the runway"
    );
    assert_eq!(inside(&app, AIRSTRIP).len(), 1, "the other Recruit stays");
    let mover = out.mover.as_ref().expect("a mover");
    assert!(mover.takeoff.is_some() && mover.momentum.x < 0.0, "taking off toward C");
    select_units(&mut app, &[first_in]);
    let shown: Vec<PanelButton> = buttons(&mut app).into_iter().map(|(_, button, ..)| button).collect();
    assert!(shown.contains(&c(PanelCommand::Attack)), "{shown:?}");
    assert!(shown.contains(&c(PanelCommand::HoldPosition)), "{shown:?}");
}
