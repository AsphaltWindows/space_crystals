//! Weapon Conversion in the command flow (slice M3-B; `factions/colonists/units.md > Sabre Legionnaire >
//! ObjectInterfaceState`: "R: Convert to Rifle Legionnaire", `> Weapon Conversion`): R, by hotkey or button, converts
//! the Legionnaires it goes to, one command common to both forms. It is disabled while the gate would refuse it, each
//! of its recipients Firing, in Cooldown or converting. The converted units stay selected as their new kinds and are
//! drawn anew (slice M3-E's response to `SimEvent::KindChanged`).

use bevy::prelude::*;
use sc_sim::conversion::CONVERT_TICKS;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Ability, AttackPhase, Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SpawnAt};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand, PanelSlot};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::object_render::ObjectVisuals;

use super::*;
use KeyCode::{KeyR, ShiftLeft};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SABRE: ObjectKind = ObjectKind::SabreLegionnaire;
const RIFLE: ObjectKind = ObjectKind::RifleLegionnaire;
/// Player 2's Generator in [`legion_game`], which Rifle 3 can shoot at for long.
const GENERATOR: u32 = 4;

/// Player 1, the Colonists, with Sabre Legionnaires 1 and 2 and Rifle Legionnaire 3 in the world view, and player
/// 2's Generator 4 within the Rifle's 4 grid units of it (tiles 21 and 22 of rows 12 and 13, 135 su off), set up as
/// [`game`] does (no auto-targeting: nobody fights unless ordered).
fn legion_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Colonists)
        .spawn(SABRE, Some(P1), point(1000.0, 800.0))
        .spawn(SABRE, Some(P1), point(1000.0, 950.0))
        .spawn(RIFLE, Some(P1), point(1200.0, 800.0))
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(TilePos::new(21, 12)));
    game_of(builder.build())
}

/// Convert's slot, R.
fn r() -> CommandSlot {
    slot(0, 3)
}

/// How the HUD shows the button in R.
fn look(app: &mut App) -> Option<ButtonLook> {
    button(app).map(|b| b.look)
}

/// The button the HUD shows in R.
fn button(app: &mut App) -> Option<PanelSlot> {
    let mut query = app.world_mut().query::<&CommandButton>();
    query.iter(app.world()).find(|b| b.slot == r()).map(|b| b.shown)
}

/// Player 1's Convert of `units`.
fn converted(units: &[u32]) -> (PlayerId, Vec<u32>, Ability) {
    (P1, units.to_vec(), Ability::Convert)
}

/// The kinds of objects `ids`, as the sim has them and as the local player's selection holds them.
fn kinds(app: &App, ids: &[u32]) -> (Vec<ObjectKind>, Vec<ObjectKind>) {
    let sim = app.world().resource::<SimRes>();
    let in_sim = ids.iter().map(|&n| sim.object(ObjectId(n)).unwrap().kind).collect();
    let selection = app.world().resource::<ControlStates>().selection(local_player(app));
    let selected = selection
        .members()
        .filter(|(id, _)| ids.contains(&id.0))
        .map(|(_, kind)| kind)
        .collect();
    (in_sim, selected)
}

/// The entity object `n` is drawn with.
fn visual(app: &App, n: u32) -> Entity {
    app.world().resource::<ObjectVisuals>().by_id[&ObjectId(n)]
}

/// Unit `n`'s attack phase, as the sim has it.
fn phase(app: &App, n: u32) -> Option<AttackPhase> {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(n)).unwrap().attack.as_ref().and_then(|a| a.phase)
}

/// Whether the gate refuses a Convert of a unit in `phase` (`> Eligibility`).
fn refused(phase: Option<AttackPhase>) -> bool {
    matches!(phase, Some(AttackPhase::Firing { .. } | AttackPhase::Cooldown))
}

/// Runs ticks until Rifle 3's phase is `wanted`'s, which it must be within a whole sequence.
fn until(app: &mut App, wanted: fn(Option<AttackPhase>) -> bool) {
    for _ in 0..20 {
        if wanted(phase(app, 3)) {
            return;
        }
        app.update();
    }
    panic!("Rifle 3 never got there: {:?}", phase(app, 3));
}

/// `> Weapon Conversion > Convert Command`: "units may be mixed in selection; each converts to its respective other
/// form". Two Sabres and a Rifle selected, R is one common command, "Convert"; its hotkey sends them all a Convert, an
/// ability and no order, and 4 ticks on the Sabres are Rifles and the Rifle a Sabre, still selected, each drawn anew.
/// Its button converts them back, and Shift with R queues nothing (design change 0151).
#[test]
fn r_converts_each_selected_legionnaire_to_its_other_form() {
    let mut app = legion_game();
    select_units(&mut app, &[1, 2, 3]);
    let shown = button(&mut app).expect("a button in R");
    assert_eq!(
        (shown.button, shown.look, shown.on, shown.label()),
        (
            PanelButton::Command(PanelCommand::Convert),
            ButtonLook::Common,
            false,
            "Convert"
        )
    );
    let drawn = [1, 2, 3].map(|n| visual(&app, n));

    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app), [converted(&[1, 2, 3])]);
    updates(&mut app, CONVERT_TICKS as usize + 2);
    let swapped = vec![RIFLE, RIFLE, SABRE];
    assert_eq!(kinds(&app, &[1, 2, 3]), (swapped.clone(), swapped));
    assert_eq!(selected(&app), [1, 2, 3]);
    for (n, before) in [1, 2, 3].into_iter().zip(drawn) {
        assert_ne!(visual(&app, n), before, "{n} drawn as its new kind");
    }
    assert_eq!(look(&mut app), Some(ButtonLook::Common));

    click_button(&mut app, r(), &[]);
    assert_eq!(abilities(&app)[1..], [converted(&[1, 2, 3])]);
    updates(&mut app, CONVERT_TICKS as usize + 2);
    let back = vec![SABRE, SABRE, RIFLE];
    assert_eq!(kinds(&app, &[1, 2, 3]), (back.clone(), back));

    tap(&mut app, &[ShiftLeft], KeyR);
    assert_eq!(abilities(&app)[2..], [converted(&[1, 2, 3])]);
    assert_eq!(orders(&app), []);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// `> Weapon Conversion > Eligibility`: "may not be issued while the unit is in Firing or Cooldown". Rifle 3, alone
/// selected, shoots Generator 4: through a whole attack sequence (Aim 3, Firing 2, Cooldown 7, Reload 4 frames), R is
/// disabled exactly while it is Firing or in Cooldown, and pressing it then sends nothing. With Sabre 1 selected too,
/// R stays available, and the Convert goes to both, for the gate to leave the Rifle out (question Q0042 point 9).
/// Pressed while the Rifle reloads, R converts it.
#[test]
fn r_is_disabled_while_each_legionnaire_fires_or_cools_down() {
    let mut app = legion_game();
    select_units(&mut app, &[3]);
    let attack = sc_sim::Command::Order {
        units: vec![ObjectId(3)],
        kind: OrderKind::Attack(ObjectId(GENERATOR)),
        queue: false,
    };
    submit(&mut app, P1, attack);
    let mut looks = Vec::new();
    for _ in 0..20 {
        app.update();
        let phase = phase(&app, 3);
        let expected = if refused(phase) {
            ButtonLook::Disabled
        } else {
            ButtonLook::Common
        };
        assert_eq!(look(&mut app), Some(expected), "{phase:?}");
        looks.push(expected);
    }
    assert!(looks.contains(&ButtonLook::Disabled) && looks.contains(&ButtonLook::Common));

    // Pressed in the first frame of Firing, R is pressed in the second: refused.
    until(&mut app, |phase| matches!(phase, Some(AttackPhase::Firing { .. })));
    tap(&mut app, &[], KeyR);
    assert!(refused(phase(&app, 3)));
    assert_eq!(abilities(&app), [], "nothing sent");

    select_units(&mut app, &[1, 3]);
    until(&mut app, |phase| matches!(phase, Some(AttackPhase::Firing { .. })));
    assert_eq!(look(&mut app), Some(ButtonLook::Common), "the Sabre may convert");
    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app), [converted(&[1, 3])]);
    let sim = app.world().resource::<SimRes>();
    let converting = [1, 3].map(|n| sim.object(ObjectId(n)).unwrap().conversion.is_some());
    assert_eq!(converting, [true, false], "the Rifle left out");

    select_units(&mut app, &[3]);
    until(&mut app, |phase| phase == Some(AttackPhase::Reloading));
    assert_eq!(look(&mut app), Some(ButtonLook::Common));
    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app)[1..], [converted(&[3])]);
    updates(&mut app, CONVERT_TICKS as usize + 2);
    assert_eq!(kinds(&app, &[3]).0, [SABRE]);
}
