//! (M6-SB) The Shadebringer's Sludge in the client (`factions/syndicate/units/shadebringer.md > Sludge (toggle)`; `>
//! Placeholder stats`: "Hotkeys: **R** Sludge, **F** Cover"): R switches it on, and once the sim has it on, R switches
//! it off, the unit's Move kept ("Toggles follow the mode-switch rule (keep orders, not queueable)"); its trail shows
//! on the ground under the units, and its tank's bar under its HP bar, lighter while it lays. The panels as plain
//! data are the unit tests' (`command_ui/interface/tests/shadebringer.rs`), the trail's discs
//! `patches_view/tests.rs`', the toggle and the trail the sim's (`crates/sim/tests/sludge`).

use bevy::prelude::*;
use sc_sim::buffs::BuffKind;
use sc_sim::defs::PatchKind;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Ability, Faction, ObjectId, ObjectKind, PlayerId, SpawnAt};
use space_crystals::combat_effects::buffs::buff_marks;
use space_crystals::combat_effects::hp_bars;
use space_crystals::fog::Sight;
use space_crystals::patches_view::PatchesMesh;
use space_crystals::tuning::ClientTuning;

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Player 1's Shadebringer, left of the view's centre; player 2's Guard far right.
const SHADEBRINGER: u32 = 1;

/// A game of player 1's Shadebringer and player 2's Guard, both of the Syndicate, on a flat 48 × 32 map, set up as
/// [`game`] does (the fog of war off).
fn sludge_game() -> App {
    let builder = ScenarioBuilder::new(&vec![".".repeat(48); 32].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(
            ObjectKind::Shadebringer,
            Some(P1),
            SpawnAt::Point(VIEW_CENTER - Vec2::new(400.0, 0.0)),
        )
        .spawn(
            ObjectKind::Guard,
            Some(P2),
            SpawnAt::Point(VIEW_CENTER + Vec2::new(800.0, 300.0)),
        );
    game_of(builder.build())
}

/// The Shadebringer's tanks, as the sim has them.
fn tanks(app: &App) -> sc_sim::patches::Tanks {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(SHADEBRINGER))
        .and_then(|o| o.tanks)
        .expect("its tanks")
}

/// The Shadebringer's tank bars as drawn for player 1 with it selected.
fn tank_bars(app: &App) -> [Option<(f32, bool)>; 2] {
    let sim = app.world().resource::<SimRes>();
    let tuning = app.world().resource::<ClientTuning>();
    let bars = hp_bars(sim, &Sight::new(sim, P1, false), 1.0, tuning, |id| {
        id == ObjectId(SHADEBRINGER)
    });
    let [bar] = bars[..] else {
        panic!("one bar: {bars:?}");
    };
    bar.tanks
}

/// The order the Shadebringer carries out, and since when.
fn current(app: &App) -> Option<(sc_sim::orders::Order, sc_sim::Tick)> {
    let current = object_orders(app, SHADEBRINGER).current;
    current.map(|active| (active.order, active.since))
}

/// Whether the ground patches' mesh shows.
fn trail_shows(app: &mut App) -> bool {
    let mut meshes = app.world_mut().query_filtered::<&Visibility, With<PatchesMesh>>();
    let visibility = meshes.single(app.world()).expect("the patches' mesh");
    *visibility != Visibility::Hidden
}

#[test]
fn r_switches_the_sludge_on_and_off_and_the_trail_shows_under_the_units() {
    let mut app = sludge_game();
    select_units(&mut app, &[SHADEBRINGER]);
    assert_eq!(
        tank_bars(&app),
        [Some((1.0, false)), None],
        "full and off, and no Cover yet"
    );
    assert!(!trail_shows(&mut app));
    let to = VIEW_CENTER + Vec2::new(300.0, 0.0);
    right_click(&mut app, to, &[]);
    tap(&mut app, &[], KeyCode::KeyR);
    let on = Ability::Sludge { on: true };
    assert_eq!(abilities(&app).last(), Some(&(P1, vec![SHADEBRINGER], on)));
    updates(&mut app, 24);
    assert!(tanks(&app).sludge.on, "on");
    let sim = app.world().resource::<SimRes>();
    assert!(sim.patches().count(PatchKind::Sludge) > 3, "its trail");
    assert!(trail_shows(&mut app), "drawn");
    let [Some((left, true)), None] = tank_bars(&app) else {
        panic!("its bar: {:?}", tank_bars(&app));
    };
    assert!(left < 1.0, "draining: {left}");
    let moving = current(&app);
    assert!(moving.is_some(), "still moving");
    tap(&mut app, &[], KeyCode::KeyR);
    let off = Ability::Sludge { on: false };
    assert_eq!(abilities(&app).last(), Some(&(P1, vec![SHADEBRINGER], off)));
    updates(&mut app, 2);
    assert!(!tanks(&app).sludge.on, "off");
    assert_eq!(current(&app), moving, "its Move kept");
    assert!(trail_shows(&mut app), "the trail stays for its 10 s");
}

/// A unit of another player's that the sludge holds back shows its mark to player 1, as every player who sees it
/// does (`combat_effects/buffs.rs`): beside its HP bar's place, right of it, above the unit.
#[test]
fn a_unit_the_sludge_slows_shows_its_mark() {
    let mut app = sludge_game();
    select_units(&mut app, &[SHADEBRINGER]);
    right_click(&mut app, VIEW_CENTER + Vec2::new(300.0, 0.0), &[]);
    tap(&mut app, &[], KeyCode::KeyR);
    updates(&mut app, 40);
    let first = {
        let sim = app.world().resource::<SimRes>();
        sim.patches().iter().next().expect("its trail").centre
    };
    let guard = spawn(&mut app, ObjectKind::Guard, Some(P2), first);
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    let o = sim.object(ObjectId(guard)).expect("the Guard");
    assert!(sc_sim::patches::slowed(o), "slowed: {:?}", o.buffs);
    let tuning = app.world().resource::<ClientTuning>();
    let marks = buff_marks(sim, &Sight::new(sim, P1, false), 1.0, tuning);
    let [mark] = marks[..] else {
        panic!("one mark: {marks:?}");
    };
    assert_eq!(mark.ability, BuffKind::Sludge);
    let beside = mark.center - o.pos;
    assert!(beside.x > 0.0 && beside.y < 0.0 && beside.length() < 64.0, "{beside}");
}
