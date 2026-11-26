//! The Recruit's UI in the client (slice M5-J1; design changes 0322, 0327, 0341 and 0347, and 0072's Verifications):
//! its panel's letters; its two Construct menus, each building in its fixed slot and greyed until it can be built; the
//! Assist pointer over a site, which no one enters, and S Assist Construction; the Enter pointer over an Armory, the
//! gathering pointers and a load carried; and Tab, which stops once on the Recruits. A Construct menu's placement for
//! the whole group, the Shift chain and a Hall refused at the map's edge are [`placing`]'s; the Repair pointer, D
//! Repair and the right-click on a damaged building [`repairing`]'s (slice M5-N8).

mod placing;
mod repairing;

use sc_sim::defs::Resource;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{BuildingOrientation, Faction, SU_PER_GU};
use space_crystals::command_ui::feedback::{NOT_YET_TEXT, TargetFeedback};
use space_crystals::command_ui::{BuildMenu, ButtonLook, PanelButton, PanelCommand, Planned};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::hud::info_panel::hint::CommandHint;
use space_crystals::object_render::LoadMarker;
use space_crystals::placement::{Ghost, LingeringGhosts, PlacementGhost};

use super::*;
use KeyCode::{KeyA, KeyC, KeyD, KeyF, KeyS, KeyV, KeyW, KeyX, KeyZ, Tab};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const STORAGE_KIND: ObjectKind = ObjectKind::Storage;
const ARMORY_KIND: ObjectKind = ObjectKind::Armory;
const HALL_KIND: ObjectKind = ObjectKind::RecruitmentHall;

/// The objects of [`recruit_game`], by id, and where they are (the map is flat: where they are drawn).
const STORAGE: u32 = 1;
const ARMORY: u32 = 2;
const RECRUITS: [u32; 4] = [3, 4, 5, 6];
const MILITANTS: [u32; 2] = [7, 8];
const PATCH: u32 = 9;
/// The Storage's tiles (14, 9) to (16, 10), the Armory's (30, 9) to (32, 10), the patch's (24, 17).
const STORAGE_AT: TilePos = TilePos::new(14, 9);
const ARMORY_AT: TilePos = TilePos::new(30, 9);
const PATCH_AT: TilePos = TilePos::new(24, 17);
const RECRUITS_AT: [Vec2; 4] = [
    Vec2::new(900.0, 1000.0),
    Vec2::new(960.0, 1000.0),
    Vec2::new(1020.0, 1000.0),
    Vec2::new(1080.0, 1000.0),
];
const MILITANTS_AT: [Vec2; 2] = [Vec2::new(900.0, 1200.0), Vec2::new(960.0, 1200.0)];

/// A game of player 1, the Cults, against player 2, the GDO, on a flat 64 × 48 map, as [`game`] sets it up (no fog,
/// no auto-targeting, the camera at [`VIEW_CENTER`]: tiles x 11 to 39, y 8.4 to 19.75): player 1's Storage and Armory
/// (in operation, so the Armory's TechPrerequisite, a Storage, is met), four Recruits and, south of them, two Militants
/// in the south-west of the world view, a Space Crystals patch, and player 2's Deployment Center far off. Player 1's bank is
/// `crystals` Space Crystals.
fn recruit_game(crystals: u32) -> App {
    let tile = SpawnAt::Tile;
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Gdo)
        .spawn(STORAGE_KIND, Some(P1), tile(STORAGE_AT))
        .spawn(ARMORY_KIND, Some(P1), tile(ARMORY_AT));
    for at in RECRUITS_AT {
        builder = builder.spawn(RECRUIT, Some(P1), SpawnAt::Point(at));
    }
    for at in MILITANTS_AT {
        builder = builder.spawn(ObjectKind::Militant, Some(P1), SpawnAt::Point(at));
    }
    let builder = builder.spawn(ObjectKind::CrystalPatch, None, tile(PATCH_AT)).spawn(
        ObjectKind::DeploymentCenter,
        Some(P2),
        tile(TilePos::new(56, 40)),
    );
    let mut app = game_of(builder.build());
    set_crystals(&mut app, crystals);
    app
}

/// Sets player 1's Space Crystals (a debug command, applied at the next frame's tick).
fn set_crystals(app: &mut App, crystals: u32) {
    let set = DebugCommand::SetResources {
        player: P1,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(app, P1, Command::Debug(set));
    app.update();
}

/// Player 1's Space Crystals.
fn crystals(app: &App) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.player(P1).expect("in the game").crystals
}

/// The middle of a `w × h` footprint whose top-left tile is `origin`: where the cursor goes to put it there.
fn middle(origin: TilePos, (w, h): (u32, u32)) -> Vec2 {
    origin.corner() + Vec2::new(w as f32, h as f32) * SU_PER_GU / 2.0
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

/// A Build of a `kind` at `origin`, unturned.
fn build_at(kind: ObjectKind, origin: TilePos) -> OrderKind {
    OrderKind::Build {
        kind,
        origin,
        orientation: BuildingOrientation::default(),
    }
}

/// Every Build given so far, as the sim received them.
fn builds(app: &App) -> Vec<Given> {
    let build = |given: &Given| matches!(given.kind, OrderKind::Build { .. });
    orders(app).into_iter().filter(build).collect()
}

/// The placement ghost as it shows now.
fn ghost(app: &App) -> Option<Ghost> {
    app.world().resource::<PlacementGhost>().shown.clone()
}

/// The lingering ghosts shown: their footprints' top-left tiles and sizes.
fn lingering(app: &App) -> Vec<(TilePos, u32, u32)> {
    let shown = &app.world().resource::<LingeringGhosts>().shown;
    shown.iter().map(|site| (site.origin, site.w, site.h)).collect()
}

/// Centres the camera on the view point `view`, as far as the map lets it (a frame).
fn look_at(app: &mut App, view: Vec2) {
    app.world_mut().resource_mut::<CameraView>().center = view;
    app.update();
}

/// The type of the local player's ActiveGroup.
fn active_type(app: &App) -> Option<ObjectKind> {
    let states = app.world().resource::<ControlStates>();
    states.selection(local_player(app)).active().map(|active| active.kind)
}

/// Player 1's site, if one stands.
fn site_of(app: &App) -> Option<u32> {
    let sim = app.world().resource::<SimRes>();
    let site = sim.objects().find(|o| o.is_site() && o.owner == Some(P1));
    site.map(|o| o.id.0)
}

/// Runs frames until one of player 1's sites stands, 400 at most, and returns its id.
fn run_until_site(app: &mut App) -> u32 {
    for _ in 0..400 {
        if let Some(site) = site_of(app) {
            return site;
        }
        app.update();
    }
    panic!("no site within 400 frames");
}

/// What worker `id` carries, if anything.
fn carried(app: &App, id: u32) -> Option<Resource> {
    let sim = app.world().resource::<SimRes>();
    let worker = sim.object(ObjectId(id))?.worker?;
    (worker.load > 0).then_some(worker.resource)
}

/// Which of object `id`'s load markers show, by resource.
fn markers_shown(app: &mut App, id: u32) -> Vec<(Resource, bool)> {
    let mut query = app.world_mut().query::<(&LoadMarker, &Visibility)>();
    let mut shown: Vec<(Resource, bool)> = (query.iter(app.world()))
        .filter(|(marker, _)| marker.0 == ObjectId(id))
        .map(|(marker, visibility)| (marker.1, *visibility != Visibility::Hidden))
        .collect();
    shown.sort();
    shown
}

/// The slots of the Recruit's letters and of its menus'.
fn q() -> CommandSlot {
    slot(0, 0)
}
fn w() -> CommandSlot {
    slot(0, 1)
}
fn a() -> CommandSlot {
    slot(1, 0)
}
fn s() -> CommandSlot {
    slot(1, 1)
}
fn d() -> CommandSlot {
    slot(1, 2)
}
fn f() -> CommandSlot {
    slot(1, 3)
}
fn x() -> CommandSlot {
    slot(2, 1)
}
fn c() -> CommandSlot {
    slot(2, 2)
}
fn v() -> CommandSlot {
    slot(2, 3)
}
fn z() -> CommandSlot {
    CommandSlot::BACK
}

/// Design change 0322's Verification (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState`): "Select
/// Recruits: the panel shows Q, A, S, D and X", and W since design change 0327: Q Move, W Advanced Construct, A Basic
/// Construct, S Assist Construction, D Repair, X Stop, each hint naming it in full. D (slice M5-N8) awaits its target,
/// and gives no order until it has one.
#[test]
fn the_recruits_panel_shows_every_letter() {
    use ButtonLook::Common;
    let mut app = recruit_game(0);
    select_units(&mut app, &RECRUITS);
    let cmd = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), cmd(PanelCommand::Move), Common, "Move"),
            (w(), cmd(PanelCommand::BuildMenu(BuildMenu::Advanced)), Common, "Adv."),
            (a(), cmd(PanelCommand::BuildMenu(BuildMenu::Basic)), Common, "Basic"),
            (s(), cmd(PanelCommand::Assist), Common, "Assist"),
            (d(), cmd(PanelCommand::Repair), Common, "Repair"),
            (x(), cmd(PanelCommand::Stop), Common, "Stop"),
        ]
    );
    let hints = [
        (q(), "Q: Move"),
        (w(), "W: Advanced Construct"),
        (a(), "A: Basic Construct"),
        (s(), "S: Assist Construction"),
        (d(), "D: Repair"),
        (x(), "X: Stop"),
    ];
    for (at, hint) in hints {
        assert_eq!(hint_over(&mut app, at).as_deref(), Some(hint));
    }
    tap(&mut app, &[], KeyD);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Repair));
    assert_eq!(orders(&app), []);
}

/// Design change 0327 (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`,
/// BuildMenu[Basic] and BuildMenu[Advanced]: "Fixed slots: every building keeps its key and is greyed until its
/// TechPrerequisite is met, so no hotkey ever moves"; "**Z** returns to DefaultState"). With 250 Space Crystals: A
/// opens the Basic menu, Q Recruitment Hall greyed for the bank, W Storage, A Armory (a Storage stands), S Gunworks
/// (an Armory stands; slice M5-G2), D Garrison (an Armory stands; slice M5-N2) and X Altar (a Storage stands; slice
/// M5-N1b), whose placement X enters and Z leaves for the menu; each Build's hint names its price. Z goes back; W opens
/// the Advanced menu: S Machine Shop and F Scrap Hangar (slice M5-G2), X Temple (slice M5-N3), C Cathedral (slice
/// M5-N4a) and V Shrine (slice M5-N5) greyed by the gate for their TechPrerequisites, a Chop Shop, an Airstrip, an
/// Altar and a Temple, which they name when pressed; A Chop Shop and D Airstrip greyed as not in the game yet (0327's
/// Verification: "W then A places a Chop Shop (greyed without an Armory)": its Build comes with its slice).
#[test]
fn the_construct_menus_keep_their_slots_greying_what_cant_be_built() {
    use ButtonLook::{Back, Common, Disabled, Unavailable};
    use PanelCommand::Build;
    let mut app = recruit_game(250);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Basic));
    let cmd = PanelButton::Command;
    let planned = |entry| cmd(PanelCommand::Planned(entry));
    assert_eq!(
        buttons(&mut app),
        [
            (q(), cmd(Build(HALL_KIND)), Disabled, "RH"),
            (w(), cmd(Build(STORAGE_KIND)), Common, "SE"),
            (a(), cmd(Build(ARMORY_KIND)), Common, "AY"),
            (s(), cmd(Build(ObjectKind::Gunworks)), Common, "GS"),
            (d(), cmd(Build(ObjectKind::Garrison)), Common, "GA"),
            (z(), PanelButton::Back, Back, "Back"),
            (x(), cmd(Build(ObjectKind::Altar)), Common, "AR"),
        ]
    );
    let hints = [
        (q(), "Q: Build Recruitment Hall (300 Space Crystals)"),
        (w(), "W: Build Storage (100 Space Crystals)"),
        (a(), "A: Build Armory (200 Space Crystals)"),
        (s(), "S: Build Gunworks (150 Space Crystals)"),
        (d(), "D: Build Garrison (150 Space Crystals)"),
        (x(), "X: Build Altar (150 Space Crystals)"),
    ];
    for (at, hint) in hints {
        assert_eq!(hint_over(&mut app, at).as_deref(), Some(hint));
    }
    tap(&mut app, &[], KeyX);
    let placing = interface(&app).placing().map(|placing| placing.kind);
    assert_eq!(placing, Some(ObjectKind::Altar));
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Basic));
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    let entries = [
        (a(), planned(Planned::ChopShop), Unavailable, "CS"),
        (s(), cmd(Build(ObjectKind::MachineShop)), Disabled, "MS"),
        (d(), planned(Planned::Airstrip), Unavailable, "AP"),
        (f(), cmd(Build(ObjectKind::ScrapHangar)), Disabled, "SH"),
        (x(), cmd(Build(ObjectKind::Temple)), Disabled, "TE"),
        (c(), cmd(Build(ObjectKind::Cathedral)), Disabled, "CL"),
        (v(), cmd(Build(ObjectKind::Shrine)), Disabled, "SR"),
    ];
    let mut expected = entries.to_vec();
    expected.insert(4, (z(), PanelButton::Back, Back, "Back"));
    assert_eq!(buttons(&mut app), expected);
    assert_eq!(hint_over(&mut app, a()).as_deref(), Some("A: Build Chop Shop"));
    let hints = [
        (s(), "S: Build Machine Shop (150 Space Crystals)"),
        (f(), "F: Build Scrap Hangar (200 Space Crystals)"),
        (x(), "X: Build Temple (200 Space Crystals)"),
        (c(), "C: Build Cathedral (250 Space Crystals)"),
        (v(), "V: Build Shrine (200 Space Crystals)"),
    ];
    for (at, hint) in hints {
        assert_eq!(hint_over(&mut app, at).as_deref(), Some(hint));
    }
    tap(&mut app, &[], KeyA);
    assert_eq!(said(&app), Some(NOT_YET_TEXT));
    tap(&mut app, &[], KeyS);
    assert_eq!(said(&app), Some("Requires a Chop Shop"));
    tap(&mut app, &[], KeyF);
    assert_eq!(said(&app), Some("Requires an Airstrip"));
    tap(&mut app, &[], KeyX);
    assert_eq!(said(&app), Some("Requires an Altar"));
    tap(&mut app, &[], KeyC);
    assert_eq!(said(&app), Some("Requires a Temple"));
    tap(&mut app, &[], KeyV);
    assert_eq!(said(&app), Some("Requires a Temple"));
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    assert_eq!(orders(&app), []);
}

/// Where Recruit 3 builds the Armory (3 × 2) of [`the_assist_pointer_shows_over_a_site_which_no_one_enters`]: tiles
/// (16, 12) to (18, 13), by the Recruits, off their spots.
const NEAR_SITE: TilePos = TilePos::new(16, 12);

/// Design change 0322's Verification: "Hover over a site under construction: the Assist pointer shows", here an
/// Armory's, which no one enters until it stands (design change 0347 point 4: "a building under construction can't
/// be entered", and a Recruit's right-click on an own site Assists). Recruit 3 alone starts the site; the three
/// others, selected, show Assist over it, and their right-click there gives them an Assist. S awaits the site
/// ("Left-click an in-progress own building to join construction"): Assist over it, Inactive over the Storage, and a
/// click on the site sends the Assist and ends the targeting. The Militants build nothing: a Move.
#[test]
fn the_assist_pointer_shows_over_a_site_which_no_one_enters() {
    let mut app = recruit_game(200);
    let build = Command::Order {
        units: vec![ObjectId(RECRUITS[0])],
        kind: build_at(ARMORY_KIND, NEAR_SITE),
        queue: false,
    };
    submit(&mut app, P1, build);
    let site = run_until_site(&mut app);
    updates(&mut app, 4);
    assert_eq!(object_kind(&app, site), ARMORY_KIND);
    let site_at = pos(&app, site);
    let helpers = &RECRUITS[1..];
    select_units(&mut app, helpers);
    assert_eq!(pointer_over(&mut app, site_at), Some(PointerDisplay::Assist));
    let assist = OrderKind::Assist(ObjectId(site));
    right_click(&mut app, site_at, &[]);
    assert_eq!(orders(&app).last(), Some(&given(helpers, assist, false)));

    tap(&mut app, &[], KeyS);
    let awaiting = InterfaceState::AwaitingTarget {
        pending: PanelCommand::Assist,
        queued: false,
    };
    assert_eq!(interface(&app), awaiting);
    let storage_at = middle(STORAGE_AT, (2, 2));
    assert_eq!(pointer_over(&mut app, storage_at), Some(PointerDisplay::Inactive));
    assert_eq!(pointer_over(&mut app, site_at), Some(PointerDisplay::Assist));
    let given_before = orders(&app).len();
    click(&mut app, site_at, &[]);
    assert_eq!(orders(&app).len(), given_before + 1);
    assert_eq!(orders(&app).last(), Some(&given(helpers, assist, false)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    select_units(&mut app, &MILITANTS);
    assert_eq!(pointer_over(&mut app, site_at), Some(PointerDisplay::Move));
}

/// The kind of object `id`.
fn object_kind(app: &App, id: u32) -> ObjectKind {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id)).expect("in the game").kind
}

/// `factions/cults/units/recruit.md > Right-Click Resolution` under the pointer
/// (`systems/controls/hud.md > PointerDisplayType`): "Space Crystal patch → Gather" shows
/// GatherResources; "Own Storage (carrying Space Crystals) → Drop off resources" shows ReturnResources for Recruit 3
/// with its load (which shows on its body, the crystal shard), whose right-click sends it there, and a Move to it for
/// the others, so Mixed for all four;
/// "Own Armory → Enter Armory" shows Enter, and the right-click sends the four in.
#[test]
fn the_enter_gather_and_return_pointers() {
    let mut app = recruit_game(0);
    select_units(&mut app, &RECRUITS);
    let (patch_at, storage_at) = (pos(&app, PATCH), middle(STORAGE_AT, (2, 2)));
    let armory_at = middle(ARMORY_AT, (3, 2));
    assert_eq!(pointer_over(&mut app, patch_at), Some(PointerDisplay::GatherResources));
    assert_eq!(pointer_over(&mut app, storage_at), Some(PointerDisplay::Move));

    let carrier = RECRUITS[0];
    let gather = Command::Order {
        units: vec![ObjectId(carrier)],
        kind: OrderKind::Gather(ObjectId(PATCH)),
        queue: false,
    };
    submit(&mut app, P1, gather);
    for _ in 0..600 {
        if carried(&app, carrier).is_some() {
            break;
        }
        app.update();
    }
    let stop = Command::Order {
        units: vec![ObjectId(carrier)],
        kind: OrderKind::Stop,
        queue: false,
    };
    submit(&mut app, P1, stop.clone());
    updates(&mut app, 2);
    assert_eq!(carried(&app, carrier), Some(Resource::Crystals));
    assert_eq!(markers_shown(&mut app, carrier), [(Resource::Crystals, true)]);
    let at = pos(&app, carrier);
    click(&mut app, at, &[]);
    assert_eq!(selected(&app), [carrier]);
    assert_eq!(
        pointer_over(&mut app, storage_at),
        Some(PointerDisplay::ReturnResources)
    );
    right_click(&mut app, storage_at, &[]);
    let drop_off = OrderKind::DropOff(ObjectId(STORAGE));
    assert_eq!(orders(&app).last(), Some(&given(&[carrier], drop_off, false)));
    submit(&mut app, P1, stop);
    updates(&mut app, 2);
    assert_eq!(carried(&app, carrier), Some(Resource::Crystals), "still carried");
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, storage_at), Some(PointerDisplay::Mixed));

    assert_eq!(pointer_over(&mut app, armory_at), Some(PointerDisplay::Enter));
    right_click(&mut app, armory_at, &[]);
    let enter = OrderKind::Enter(ObjectId(ARMORY));
    assert_eq!(orders(&app).last(), Some(&given(&RECRUITS, enter, false)));
}

/// Design change 0072's Verification 4: "Pressing Tab with Recruits and Soldiers selected stops once on the Recruits as
/// a group". Four Recruits and two Militants are two SelectionGroups: Tab goes from one to the other and back, and the
/// panel follows, the Recruits' Construct menus a group command.
#[test]
fn tab_stops_once_on_the_recruits() {
    let mut app = recruit_game(0);
    let all = [RECRUITS.as_slice(), MILITANTS.as_slice()].concat();
    select_units(&mut app, &all);
    let mut stops = vec![active_type(&app)];
    for _ in 0..3 {
        tap(&mut app, &[], Tab);
        stops.push(active_type(&app));
    }
    let first = stops[0].expect("an ActiveGroup");
    let second = if first == RECRUIT {
        ObjectKind::Militant
    } else {
        RECRUIT
    };
    assert_eq!(stops, [first, second, first, second].map(Some));
    while active_type(&app) != Some(RECRUIT) {
        tap(&mut app, &[], Tab);
    }
    let basic = (buttons(&mut app).into_iter()).find(|(at, ..)| *at == a());
    let menu = PanelButton::Command(PanelCommand::BuildMenu(BuildMenu::Basic));
    assert_eq!(basic, Some((a(), menu, ButtonLook::Group, "Basic")));
}
