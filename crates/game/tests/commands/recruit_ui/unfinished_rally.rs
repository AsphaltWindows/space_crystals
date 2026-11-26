//! (M5-S6) A rally point on an Armory its Recruits are still building (design change 0396): its unfinished panel's C
//! next to X Cancel Construction, a right-click on the ground and the marker it shows, kept through completion, and
//! the Militants trained there walking to it. The gate's rules and the other factions' unfinished buildings are the
//! sim's (`crates/sim/tests/cults_construction/rally.rs`, `crates/sim/tests/warp_in/rally.rs`), the panel as plain
//! data the unit tests' (`command_ui/interface/tests/unfinished.rs`).

use sc_sim::{ProductionCommand, Rally, RecruitKind};
use space_crystals::command_ui::indicators::{Indicator, MOVE_COLOR, RallyIndicator, rally_indicators};
use space_crystals::command_ui::{InterfaceState, PointerDisplay};
use space_crystals::fog::Sight;

use super::super::indicators::drawn_vertices;
use super::*;

/// Where the Recruits build the Armory: tiles (20, 12) to (22, 13), x 1280 to 1472 and y 768 to 896, unturned, its
/// entrance, side B, to the east and its exit, side C, to the west.
const SITE_AT: TilePos = TilePos::new(20, 12);
/// Bare ground east of the site, in the world view, where a right-click sets the rally point.
const RALLY_AT: Vec2 = Vec2::new(1750.0, 850.0);
/// Bare ground south of the site, where C then a left-click moves it.
const MOVED_AT: Vec2 = Vec2::new(1400.0, 1100.0);

/// Object `id`'s rally point, as the sim has it.
fn rally_of(app: &App, id: u32) -> Option<Rally> {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id))?.production.as_ref()?.rally
}

/// Whether object `id`'s rally point is the ground point `at`, give or take the cursor's rounding.
fn rallies_at(app: &App, id: u32, at: Vec2) -> bool {
    matches!(rally_of(app, id), Some(Rally::Point(p)) if near(p, at))
}

/// The rally markers the local player sees now (no fog).
fn markers(app: &App) -> Vec<RallyIndicator> {
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let states = world.resource::<ControlStates>();
    rally_indicators(sim, &Sight::new(sim, local_player(app), false), states)
}

/// Whether the one rally marker shown is producer `id`'s green Location marker at `at`.
fn marked_at(app: &App, id: u32, at: Vec2) -> bool {
    matches!(markers(app)[..], [RallyIndicator { producer, marker: Indicator::Location { at: shown, color } }]
        if producer == ObjectId(id) && near(shown, at) && color == MOVE_COLOR)
}

/// Design change 0396's Verification: "Start an Armory, select it while it's being built, and right-click ground: a
/// rally marker appears. After completion, trained units go there." The four Recruits build an Armory (their Build, as
/// their Construct menu gives it: [`super::placing`]'s), and a click on its site selects it: its panel shows X Cancel
/// Construction and, next to it, C Set Rally Point ("Its unfinished panel adds **C: Set Rally Point**, next to
/// Cancel"). The pointer shows Move over the ground, and a right-click there sets the rally point ("as on the finished
/// building"), whose green marker shows, with its line; C then a left-click moves it. Complete, the Armory keeps it
/// ("The rally point is kept when the building completes"), and the Militant it trains of a Recruit that went in walks
/// there.
#[test]
fn an_armory_being_built_takes_a_rally_point_its_militants_go_to() {
    use PanelCommand::{CancelConstruction, SetRallyPoint};
    let mut app = recruit_game(300);
    let build = Command::Order {
        units: RECRUITS.iter().map(|&n| ObjectId(n)).collect(),
        kind: build_at(ARMORY_KIND, SITE_AT),
        queue: false,
    };
    submit(&mut app, P1, build);
    let site = run_until_site(&mut app);
    click(&mut app, middle(SITE_AT, (3, 2)), &[]);
    assert_eq!(selected(&app), [site]);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (
                CommandSlot::CANCEL_PRODUCTION,
                c(CancelConstruction),
                ButtonLook::Common,
                "Cancel"
            ),
            (CommandSlot::RALLY_POINT, c(SetRallyPoint), ButtonLook::Common, "Rally"),
        ]
    );
    assert_eq!(markers(&app), []);
    assert_eq!(pointer_over(&mut app, RALLY_AT), Some(PointerDisplay::Move));
    right_click(&mut app, RALLY_AT, &[]);
    assert!(rallies_at(&app, site, RALLY_AT), "{:?}", rally_of(&app, site));
    assert!(marked_at(&app, site, RALLY_AT), "{:?}", markers(&app));
    assert!(drawn_vertices(&mut app) > 0, "the marker and its line");
    assert_eq!(orders(&app).len(), 1, "no order, the Build alone");

    tap(&mut app, &[], KeyC);
    assert_eq!(interface(&app), InterfaceState::awaiting(SetRallyPoint));
    click(&mut app, MOVED_AT, &[]);
    assert!(rallies_at(&app, site, MOVED_AT), "{:?}", rally_of(&app, site));
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    let complete = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        sim.object(ObjectId(site)).is_some_and(|o| !o.is_site())
    };
    for _ in 0..400 {
        if complete(&app) {
            break;
        }
        app.update();
    }
    assert!(complete(&app), "built within 400 frames");
    assert_eq!(selected(&app), [site]);
    assert!(rallies_at(&app, site, MOVED_AT), "kept: {:?}", rally_of(&app, site));
    assert!(marked_at(&app, site, MOVED_AT), "{:?}", markers(&app));

    let enter = Command::Order {
        units: vec![ObjectId(RECRUITS[0])],
        kind: OrderKind::Enter(ObjectId(site)),
        queue: false,
    };
    submit(&mut app, P1, enter);
    let inside = |app: &App| !app.world().resource::<SimRes>().held_in(ObjectId(site)).is_empty();
    for _ in 0..300 {
        if inside(&app) {
            break;
        }
        app.update();
    }
    assert!(inside(&app), "the Recruit went in within 300 frames");
    let train = ProductionCommand::TrainFrom {
        kind: ObjectKind::Militant,
        from: RecruitKind::Plain,
    };
    submit(
        &mut app,
        P1,
        Command::Production {
            objects: vec![ObjectId(site)],
            command: train,
        },
    );
    let at_rally = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        (sim.objects())
            .any(|o| o.kind == ObjectKind::Militant && o.owner == Some(P1) && o.pos.distance(MOVED_AT) < 40.0)
    };
    for _ in 0..400 {
        if at_rally(&app) {
            return;
        }
        app.update();
    }
    panic!("no Militant at the rally point within 400 frames");
}
