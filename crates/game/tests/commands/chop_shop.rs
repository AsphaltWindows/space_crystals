//! The Chop Shop in the client (slice M5-N6; `factions/cults/structures/chop_shop.md > Placeholder stats`: "Panel like
//! the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Build
//! Seeker**, **W: Build Hand of Heaven**, **E: Build Prayer-mobile** (greyed "Requires a Temple" until one stands
//! [...]), **A: Release** in the kind menu"): Recruits right-clicked into it, the selected Chop Shop's door arrows, in at
//! B and out at C, a kind of Recruit first, then KindMenu\[kind\], whose Q builds a Seeker of two Recruits and is
//! greyed while fewer are free; the Seeker driving out of side C, its panel the BasicCombatUnit's, whose W awaits a
//! Reverse's ground point. (M5-N10) Its W builds a Hand of Heaven of three Recruits (design change 0444: "**W: Build
//! Hand of Heaven**"), priced in its hint and greyed while the bank can't pay or fewer are free; the Hand of Heaven's
//! D awaits an Attack Ground's point. (M5-N11) Its E is greyed until a Temple stands, then builds a Prayer-mobile
//! (0444's Verification: Q, W and E). The panel as plain data is the unit tests'
//! (`command_ui/interface/tests/chop_shop.rs`), the training and the exit the sim's (`crates/sim/tests/chop_shop`,
//! `crates/sim/tests/hand_of_heaven`, `crates/sim/tests/prayer_mobile/production.rs`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    BuildingOrientation, Faction, ObjectId, ObjectKind, OrderKind, PlayerId, ProductionCommand, RecruitKind, Rotation,
    SpawnAt, Target, TilePos,
};
use space_crystals::command_ui::feedback::{NO_CRYSTALS_TEXT, NO_RECRUIT_TEXT, NOT_ENOUGH_ENERGY_TEXT};
use space_crystals::command_ui::indicators::{Indicator, MOVE_COLOR, RallyIndicator};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand, PointerDisplay};
use space_crystals::door_arrows::{Way, door_arrows};
use space_crystals::hud::info_panel::hint::CommandHint;
use space_crystals::selection::doors::SelectedDoors;

use super::armory::{buttons, feedback, inside, on_map, until};
use super::indicators::drawn_vertices;
use super::rally::shown;
use super::*;
use KeyCode::{KeyD, KeyE, KeyQ, KeyR, KeyW};
use RecruitKind::{Both, Constitution, Metabolism, Plain};

const P1: PlayerId = PlayerId(1);
const CHOP_SHOP_KIND: ObjectKind = ObjectKind::ChopShop;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const SEEKER: ObjectKind = ObjectKind::Seeker;
const HAND: ObjectKind = ObjectKind::HandOfHeaven;
const PRAYER_MOBILE: ObjectKind = ObjectKind::PrayerMobile;

/// The objects of [`chop_shop_game`], by id, and where they are drawn (the map is flat). The Chop Shop, unturned
/// (3 × 3), has its entrance, side B, to the east and its exit, side C, to the west.
const CHOP_SHOP: u32 = 1;
const CHOP_SHOP_AT: Vec2 = Vec2::new(1120.0, 864.0); // tiles (16, 12) to (18, 14): x 1024 to 1216, y 768 to 960
const RECRUITS: [u32; 3] = [2, 3, 4];
/// The middle of side C, where a Seeker comes out, half inside, facing west.
const EXIT: Vec2 = Vec2::new(1024.0, 864.0);

/// A game of player 1, the Cults, on a flat 64 × 48 map, set up as [`game`] does, with 1000 Space Crystals: a Chop
/// Shop and three Recruits, which a box selects and a right-click on the Chop Shop sends in (the Enter pointer over
/// it: `factions/cults/units/recruit.md > Right-Click Resolution`, "an own structure that can hold units"); returned
/// once they are all inside.
fn chop_shop_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(PlayerId(2), Faction::Cults)
        .spawn(CHOP_SHOP_KIND, Some(P1), SpawnAt::Tile(TilePos::new(16, 12)))
        .spawn(RECRUIT, Some(P1), point(1400.0, 800.0))
        .spawn(RECRUIT, Some(P1), point(1400.0, 864.0))
        .spawn(RECRUIT, Some(P1), point(1400.0, 928.0));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, CHOP_SHOP_AT), Some(PointerDisplay::Enter));
    right_click(&mut app, CHOP_SHOP_AT, &[]);
    let enter = OrderKind::Enter(ObjectId(CHOP_SHOP));
    assert_eq!(orders(&app).last(), Some(&given(&RECRUITS, enter, false)));
    until(&mut app, 300, |app| inside(app, CHOP_SHOP).len() == RECRUITS.len());
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

/// `factions/cults/structures/chop_shop.md > Placeholder stats`: "**Seeker**: 2 Recruits, 75 Space Crystals, 320
/// frames (20 seconds)". Selected, the Chop Shop shows its door arrows, in at the middle of B and out at the middle of
/// C (design change 0394), and the pool buildings' DefaultState: each kind with its count of free Recruits inside, the
/// kinds it has none of greyed, X greyed with nothing to cancel. Q opens KindMenu\[Plain\]: Q Build Seeker, whose hint
/// names it and (M5-N10) its price, W Build Hand of Heaven, (M5-N11) E Build Prayer-mobile, greyed without a Temple
/// ([`e_is_greyed_until_a_temple_stands_and_then_builds_a_prayer_mobile`]), A Release, Z Back. Q queues one Seeker for
/// 75 Space Crystals, reserving two of the three Recruits, the menu staying open; with one free Recruit left Q and W
/// are greyed, as the gate refuses them (0319's Verification: `Reject::NoRecruit`), and Q says why when pressed,
/// giving nothing. The Recruit that went in first comes
/// out 320 frames later a Seeker, half inside the middle of side C, facing west, the second gone into it, and drives
/// out; its panel is the BasicCombatUnit's, whose W, Reverse (slice M5-V2), awaits a point of the ground, and a click
/// there orders it.
#[test]
fn q_q_builds_a_seeker_of_two_recruits_which_drives_out_of_side_c() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, Release, SetRallyPoint, TrainFrom};
    let mut app = chop_shop_game();
    let order_in = inside(&app, CHOP_SHOP);
    click(&mut app, CHOP_SHOP_AT, &[]);
    assert_eq!(selected(&app), [CHOP_SHOP]);
    let doors = app.world().resource::<SelectedDoors>();
    let mut arrows: Vec<(Way, Vec2)> = (doors.shown.iter())
        .flat_map(|(kind, placement)| door_arrows(*kind, placement))
        .map(|arrow| (arrow.way, arrow.at))
        .collect();
    arrows.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
    assert_eq!(arrows, [(Way::Out, EXIT), (Way::In, Vec2::new(1216.0, 864.0))]);
    let c = PanelButton::Command;
    let (x, rally_slot) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(KindMenu(Plain)), Common, Some(3)),
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
    let no_temple = (slot(0, 2), c(TrainFrom(PRAYER_MOBILE, Plain)), Disabled, None);
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(TrainFrom(SEEKER, Plain)), Common, None),
            (slot(0, 1), c(TrainFrom(HAND, Plain)), Common, None),
            no_temple,
            (slot(1, 0), c(Release(Plain)), Common, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );
    assert_eq!(
        hint_over(&mut app, slot(0, 0)).as_deref(),
        Some("Q: Build Seeker (75 Space Crystals)")
    );
    tap(&mut app, &[], KeyQ);
    let train = ProductionCommand::TrainFrom {
        kind: SEEKER,
        from: Plain,
    };
    assert_eq!(productions(&app).last(), Some(&(P1, vec![CHOP_SHOP], train)));
    assert_eq!(bank(&app), 1000 - 75);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain), "the menu stays open");
    updates(&mut app, 2);
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(TrainFrom(SEEKER, Plain)), Disabled, None),
            (slot(0, 1), c(TrainFrom(HAND, Plain)), Disabled, None),
            no_temple,
            (slot(1, 0), c(Release(Plain)), Common, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ],
        "one Recruit free: no Seeker of it, nor a Hand of Heaven"
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(feedback(&app), Some(NO_RECRUIT_TEXT));
    assert_eq!(productions(&app).len(), 1, "nothing given");

    let (first, second) = (order_in[0], order_in[1]);
    until(&mut app, 330, |app| object(app, first).is_some());
    let out = object(&app, first).expect("out");
    assert_eq!((out.kind, out.pos), (SEEKER, EXIT), "half inside side C");
    let facing = sc_sim::math::facing_vector(out.facing);
    assert!(facing.x < -0.999, "facing out: {facing}");
    assert!(
        !on_map(&app, second) && !inside(&app, CHOP_SHOP).contains(&second),
        "gone into it"
    );
    assert_eq!(inside(&app, CHOP_SHOP), [order_in[2]], "the third Recruit stays");
    until(&mut app, 120, |app| {
        let o = object(app, first).expect("out");
        o.mover.as_ref().is_some_and(|mover| mover.exit.is_none()) && o.pos.x <= EXIT.x - 28.0
    });

    let at = pos(&app, first);
    click(&mut app, at, &[]);
    assert_eq!(selected(&app), [first]);
    let shown: Vec<_> = buttons(&mut app)
        .into_iter()
        .map(|(slot, button, look, _)| (slot, button, look))
        .collect();
    assert!(
        shown.contains(&(slot(0, 1), c(PanelCommand::Reverse), Common)),
        "{shown:?}"
    );
    assert!(
        shown.contains(&(slot(1, 0), c(PanelCommand::Attack), Common)),
        "{shown:?}"
    );
    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Reverse));
    let ground = at + Vec2::new(-200.0, 160.0);
    click(&mut app, ground, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let given = orders(&app).pop().expect("an order");
    assert_eq!((given.units, given.queue), (vec![first], false));
    assert!(
        matches!(given.kind, OrderKind::Reverse(p) if near(p, ground)),
        "{:?}",
        given.kind
    );
}

/// (M5-N10) "**W: Build Hand of Heaven**" (design change 0444; `factions/cults/units/hand_of_heaven.md > Placeholder
/// stats`: "Cost: Chop Shop, 3 Recruits, **150 Space Crystals, 480 frames (30 s)**"). In KindMenu\[Plain\], the three
/// Recruits inside, W's hint names it and its price; (M5-N11) E, the Prayer-mobile's, is greyed throughout, no Temple
/// standing. With 100 Space Crystals in the bank W is greyed, as the gate
/// refuses it (`Reject::InsufficientCrystals`), while Q, the Seeker's 75, isn't, and W says why when pressed, giving
/// nothing. With 1000 W queues one for 150, reserving all three Recruits, the menu staying open; then Q, W and A
/// Release are greyed, none free, and W says so. The Recruit that went in first comes out 480 frames after the queue a Hand of
/// Heaven, half inside the middle of side C, facing west, the other two gone into it, and drives out. Its panel is the
/// BasicCombatUnit's, with W Reverse and D Attack Ground (its CanTargetGround), which awaits a point of the ground; a
/// click there orders it.
#[test]
fn w_builds_a_hand_of_heaven_of_three_recruits_which_attacks_the_ground() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{AttackGround, Release, Reverse, TrainFrom};
    let mut app = chop_shop_game();
    let order_in = inside(&app, CHOP_SHOP);
    click(&mut app, CHOP_SHOP_AT, &[]);
    assert_eq!(selected(&app), [CHOP_SHOP]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain));
    let c = PanelButton::Command;
    let menu = |seeker, hand, release| {
        vec![
            (slot(0, 0), c(TrainFrom(SEEKER, Plain)), seeker, None),
            (slot(0, 1), c(TrainFrom(HAND, Plain)), hand, None),
            (slot(0, 2), c(TrainFrom(PRAYER_MOBILE, Plain)), Disabled, None),
            (slot(1, 0), c(Release(Plain)), release, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    };
    assert_eq!(buttons(&mut app), menu(Common, Common, Common));
    assert_eq!(
        hint_over(&mut app, slot(0, 1)).as_deref(),
        Some("W: Build Hand of Heaven (150 Space Crystals)")
    );
    set_crystals(&mut app, 100);
    updates(&mut app, 2);
    let short = menu(Common, Disabled, Common);
    assert_eq!(buttons(&mut app), short, "100 Space Crystals: no Hand of Heaven");
    tap(&mut app, &[], KeyW);
    assert_eq!(feedback(&app), Some(NO_CRYSTALS_TEXT));
    assert_eq!(productions(&app), [], "nothing given");

    set_crystals(&mut app, 1000);
    updates(&mut app, 2);
    tap(&mut app, &[], KeyW);
    let train = ProductionCommand::TrainFrom {
        kind: HAND,
        from: Plain,
    };
    assert_eq!(productions(&app), [(P1, vec![CHOP_SHOP], train)]);
    assert_eq!(bank(&app), 1000 - 150);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain), "the menu stays open");
    updates(&mut app, 2);
    let none_free = menu(Disabled, Disabled, Disabled);
    assert_eq!(
        buttons(&mut app),
        none_free,
        "all three reserved, none to release either"
    );
    tap(&mut app, &[], KeyW);
    assert_eq!(feedback(&app), Some(NO_RECRUIT_TEXT));
    assert_eq!(productions(&app).len(), 1, "nothing more given");

    let first = order_in[0];
    until(&mut app, 500, |app| object(app, first).is_some());
    let out = object(&app, first).expect("out");
    assert_eq!((out.kind, out.pos), (HAND, EXIT), "half inside side C");
    let facing = sc_sim::math::facing_vector(out.facing);
    assert!(facing.x < -0.999, "facing out: {facing}");
    for gone in &order_in[1..] {
        assert!(
            !on_map(&app, *gone) && !inside(&app, CHOP_SHOP).contains(gone),
            "{gone} gone into it"
        );
    }
    until(&mut app, 160, |app| {
        let o = object(app, first).expect("out");
        o.mover.as_ref().is_some_and(|mover| mover.exit.is_none()) && o.pos.x <= EXIT.x - 40.0
    });

    let at = pos(&app, first);
    click(&mut app, at, &[]);
    assert_eq!(selected(&app), [first]);
    let shown: Vec<_> = buttons(&mut app)
        .into_iter()
        .map(|(slot, button, look, _)| (slot, button, look))
        .collect();
    assert!(shown.contains(&(slot(0, 1), c(Reverse), Common)), "{shown:?}");
    assert!(shown.contains(&(slot(1, 2), c(AttackGround), Common)), "{shown:?}");
    tap(&mut app, &[], KeyD);
    assert_eq!(interface(&app), InterfaceState::awaiting(AttackGround));
    // 400 su west, beyond its MinRange of 256 su, edge to edge: the view centred on it, so that the point is on screen.
    app.world_mut().resource_mut::<CameraView>().center = at;
    let ground = at + Vec2::new(-400.0, 0.0);
    click(&mut app, ground, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let given = orders(&app).pop().expect("an order");
    assert_eq!((given.units, given.queue), (vec![first], false));
    assert!(
        matches!(given.kind, OrderKind::AttackGround(p) if near(p, ground)),
        "{:?}",
        given.kind
    );
}

/// (M5-N11) Design change 0444's Verification: "Chop Shop kind menu with 3+ Recruits inside: Q Seeker, W Hand of
/// Heaven, E Prayer-mobile greyed until a Temple exists". With three Plain Recruits inside and no Temple,
/// KindMenu\[Plain\] shows Q Build Seeker and (slice M5-N10) W Build Hand of Heaven lit and E Build Prayer-mobile
/// greyed, its hint naming it and its 125 Space Crystals (`factions/cults/units/prayer_mobile.md > Placeholder stats`: "Cost:
/// Chop Shop, 2 Recruits, **125 Space Crystals, 400 frames (25 s)**; requires a Temple"); E says "Requires a Temple"
/// and sends nothing. Once a Temple stands, E is lit and builds one of two Recruits for 125 Space Crystals, the menu
/// staying open; 400 frames later the Recruit that went in first comes out of side C a Prayer-mobile with its 50
/// energy, whose panel greys R and F, Inspire and Suppress ("Not enough energy").
#[test]
fn e_is_greyed_until_a_temple_stands_and_then_builds_a_prayer_mobile() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{Release, TrainFrom};
    let mut app = chop_shop_game();
    let first = inside(&app, CHOP_SHOP)[0];
    click(&mut app, CHOP_SHOP_AT, &[]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain));
    let c = PanelButton::Command;
    let menu = |e| {
        [
            (slot(0, 0), c(TrainFrom(SEEKER, Plain)), Common, None),
            (slot(0, 1), c(TrainFrom(HAND, Plain)), Common, None),
            (slot(0, 2), c(TrainFrom(PRAYER_MOBILE, Plain)), e, None),
            (slot(1, 0), c(Release(Plain)), Common, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    };
    assert_eq!(buttons(&mut app), menu(Disabled), "no Temple");
    let hint = hint_over(&mut app, slot(0, 2));
    assert_eq!(hint.as_deref(), Some("E: Build Prayer-mobile (125 Space Crystals)"));
    tap(&mut app, &[], KeyE);
    assert_eq!(feedback(&app), Some("Requires a Temple"));
    assert_eq!(productions(&app), []);

    let temple = DebugCommand::Spawn {
        kind: ObjectKind::Temple,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(30, 20)),
        orientation: Default::default(),
    };
    submit(&mut app, P1, sc_sim::Command::Debug(temple));
    updates(&mut app, 2);
    assert_eq!(buttons(&mut app), menu(Common), "a Temple stands");
    tap(&mut app, &[], KeyE);
    let train = ProductionCommand::TrainFrom {
        kind: PRAYER_MOBILE,
        from: Plain,
    };
    assert_eq!(productions(&app), [(P1, vec![CHOP_SHOP], train)]);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain), "the menu stays open");
    updates(&mut app, 2);
    assert_eq!(bank(&app), 1000 - 125);

    until(&mut app, 420, |app| object(app, first).is_some());
    let out = object(&app, first).expect("out");
    assert_eq!((out.kind, out.pos), (PRAYER_MOBILE, EXIT), "half inside side C");
    assert_eq!(out.energy, Some(50 * 65_536), "its 50 energy");
    until(&mut app, 120, |app| {
        let o = object(app, first).expect("out");
        o.mover.as_ref().is_some_and(|mover| mover.exit.is_none()) && o.pos.x <= EXIT.x - 28.0
    });
    let at = pos(&app, first);
    click(&mut app, at, &[]);
    assert_eq!(selected(&app), [first]);
    let casts = [
        (
            slot(0, 3),
            c(PanelCommand::Cast(sc_sim::AreaAbility::Inspire)),
            Disabled,
            None,
        ),
        (
            slot(1, 3),
            c(PanelCommand::Cast(sc_sim::AreaAbility::Suppress)),
            Disabled,
            None,
        ),
    ];
    let shown = buttons(&mut app);
    assert!(casts.iter().all(|cast| shown.contains(cast)), "{shown:?}");
    assert!(
        shown.iter().all(|b| b.0 != slot(1, 0) && b.0 != slot(1, 2)),
        "no A, no D: {shown:?}"
    );
    tap(&mut app, &[], KeyR);
    assert_eq!(feedback(&app), Some(NOT_ENOUGH_ENERGY_TEXT));
}

/// (M5-S8) Design change 0446's Verification, "Place a Chop Shop: selected, it shows a rally point 3 GU out of side
/// C", and 0447's, "Select a new Chop Shop and right-click it: its default rally point disappears". The new Chop Shop
/// starts with a rally point (`core/entities.md > Entrance and exit > Exit Doorway` point 7), which shows as any does
/// while it is selected (design change 0087): a green marker on the ground 3 grid units west of the middle of side C,
/// with its line from the building. A right-click on the Chop Shop itself clears it (`systems/controls/interface.md >
/// CommandPanel > Production Building Default Right-Click`), and nothing shows. One turned a quarter shows it 3 grid
/// units out of side C where that side faces then (north).
#[test]
fn a_new_chop_shop_shows_its_rally_point_out_of_side_c_and_a_right_click_on_it_clears_it() {
    let marker_at = |app: &App| match shown(app)[..] {
        [
            RallyIndicator {
                producer,
                marker: Indicator::Location { at, color },
            },
        ] if producer == ObjectId(CHOP_SHOP) && color == MOVE_COLOR => Some(at),
        _ => None,
    };
    let mut app = chop_shop_game();
    click(&mut app, CHOP_SHOP_AT, &[]);
    assert_eq!(selected(&app), [CHOP_SHOP]);
    assert_eq!(marker_at(&app), Some(EXIT - Vec2::new(192.0, 0.0)), "{:?}", shown(&app));
    assert!(drawn_vertices(&mut app) > 0);
    assert_eq!(pointer_over(&mut app, CHOP_SHOP_AT), Some(PointerDisplay::Move));
    right_click(&mut app, CHOP_SHOP_AT, &[]);
    let clear = ProductionCommand::SetRally(Target::Object(ObjectId(CHOP_SHOP)));
    assert_eq!(productions(&app).last(), Some(&(P1, vec![CHOP_SHOP], clear)));
    assert_eq!(production_of(&app, CHOP_SHOP).rally, None, "cleared");
    assert_eq!((shown(&app), drawn_vertices(&mut app)), (vec![], 0), "nothing shows");

    let mut scenario = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(PlayerId(2), Faction::Cults)
        .spawn(CHOP_SHOP_KIND, Some(P1), SpawnAt::Tile(TilePos::new(16, 12)))
        .build();
    scenario.map.spawns[0].orientation = BuildingOrientation {
        rotation: Rotation::R90,
        flip_h: false,
        flip_v: false,
    };
    let mut app = game_of(scenario);
    click(&mut app, CHOP_SHOP_AT, &[]);
    assert_eq!(selected(&app), [CHOP_SHOP]);
    let north = Vec2::new(CHOP_SHOP_AT.x, 768.0 - 192.0);
    assert_eq!(marker_at(&app), Some(north), "{:?}", shown(&app));
}
