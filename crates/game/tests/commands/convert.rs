//! Weapon Conversion in the command flow (slice M3-B; `factions/colonists/units/sabre_legionnaire.md >
//! ObjectInterfaceState`: "R: Convert to Rifle Legionnaire", `factions/colonists/units/weapon_conversion.md`), as
//! design change 0198 has it: R, by hotkey or button, goes to the ActiveGroup alone, and flips its units' target form.
//! Their type, and so their SelectionGroup, switches at the press, before the sim's next tick, also for a unit whose
//! conversion waits for its shot; they join the other form's group, which becomes the ActiveGroup, so a second press
//! reaches them all. R is never disabled. A unit is drawn in the form it is in, the weapon it has, until its conversion
//! is over.

use bevy::prelude::*;
use sc_sim::conversion;
use sc_sim::defs::CONVERT_TICKS;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Ability, AttackPhase, Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand, PanelSlot};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::object_render::ObjectVisuals;
use space_crystals::sim_driver::SimControl;

use super::*;
use KeyCode::{KeyR, ShiftLeft, Tab};

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

/// Player 1, the Colonists, with Sabre Legionnaires 1 to 4 and Rifle Legionnaires 5 to 10 in the world view: 0198's
/// example.
fn ten_legionnaires() -> App {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n")).faction(P1, Faction::Colonists);
    for n in 1..=10 {
        let kind = if n <= 4 { SABRE } else { RIFLE };
        let at = Vec2::new(800.0 + 100.0 * n as f32, if n <= 4 { 800.0 } else { 950.0 });
        builder = builder.spawn(kind, Some(P1), SpawnAt::Point(at));
    }
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

/// The command the HUD shows in R, and how.
fn shown_r(app: &mut App) -> (PanelButton, ButtonLook) {
    let shown = button(app).expect("a button in R");
    assert_eq!(shown.label(), "Conv.");
    (shown.button, shown.look)
}

/// Player 1's Convert of `units`.
fn converted(units: &[u32]) -> (PlayerId, Vec<u32>, Ability) {
    (P1, units.to_vec(), Ability::Convert)
}

/// The types of objects `ids`, as the sim has them and as the local player's selection holds them.
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

/// The forms objects `ids` are in (`conversion::form`): the weapons they have.
fn forms(app: &App, ids: &[u32]) -> Vec<ObjectKind> {
    let sim = app.world().resource::<SimRes>();
    ids.iter()
        .map(|&n| conversion::form(sim.object(ObjectId(n)).unwrap()))
        .collect()
}

/// The local player's SelectionGroups, in their order: type, objects, and whether it is the ActiveGroup.
fn groups(app: &App) -> Vec<(ObjectKind, Vec<u32>, bool)> {
    let selection = app.world().resource::<ControlStates>().selection(local_player(app));
    let active = selection.active();
    let groups = selection.groups().into_iter();
    groups
        .map(|g| {
            (
                g.kind(),
                g.instances.iter().map(|id| id.0).collect(),
                Some(g.key) == active,
            )
        })
        .collect()
}

/// The entity object `n` is drawn with.
fn visual(app: &App, n: u32) -> Entity {
    app.world().resource::<ObjectVisuals>().by_id[&ObjectId(n)]
}

/// The kind object `n` is drawn as: its visual's name (`crate::object_render`).
fn drawn_as(app: &App, n: u32) -> String {
    let name = app.world().get::<Name>(visual(app, n)).expect("a named visual");
    name.as_str().split(' ').next().unwrap().to_string()
}

/// Unit `n`'s attack phase, as the sim has it.
fn phase(app: &App, n: u32) -> Option<AttackPhase> {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(n)).unwrap().attack.as_ref().and_then(|a| a.phase)
}

/// Unit `n`'s conversion: `None` without one, `Some(true)` while its Convert waits for the unit's Cooldown to end
/// (design change 0153), `Some(false)` under way.
fn waits(app: &App, n: u32) -> Option<bool> {
    let sim = app.world().resource::<SimRes>();
    let o = sim.object(ObjectId(n)).unwrap();
    match conversion::waits_to_convert(o) {
        true => Some(true),
        false => o.conversion.map(|_| false),
    }
}

/// R pressed in one frame, which sends its Convert for the sim's next tick (two presses in a row are two frames).
fn press_r(app: &mut App) {
    input(app).tap_key(KeyR);
    app.update();
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

/// `factions/colonists/units/weapon_conversion.md > Convert Command`: R "goes only to the ActiveGroup" (design change
/// 0198). Two Sabres and a Rifle selected, the Sabres active (their group comes first), R is the Sabres' "Convert to
/// Rifle Legionnaire", a group command; its hotkey sends the two a Convert, an ability and no order. At the press,
/// before the sim's tick, all three are one Rifle group, the ActiveGroup (`> A toggle sets a target form`: "The pressed
/// units join the group of that form [...] and that merged group becomes the ActiveGroup"), whose R, "Convert to Sabre
/// Legionnaire", is common to the three; the sim has the two as Rifles from its next tick. They are drawn as the Sabres
/// they are until their 4-frame conversion is over, and as Rifles from then. The button converts all three to Sabres,
/// each drawn anew, and Shift with R queues nothing (design change 0151).
#[test]
fn r_converts_the_active_group_and_the_other_forms_group_becomes_active() {
    let mut app = legion_game();
    select_units(&mut app, &[1, 2, 3]);
    assert_eq!(groups(&app), [(SABRE, vec![1, 2], true), (RIFLE, vec![3], false)]);
    let c = PanelButton::Command;
    assert_eq!(shown_r(&mut app), (c(PanelCommand::Convert(RIFLE)), ButtonLook::Group));
    let drawn = [1, 2, 3].map(|n| visual(&app, n));

    press_r(&mut app);
    assert_eq!(abilities(&app), [converted(&[1, 2])]);
    assert_eq!(groups(&app), [(RIFLE, vec![1, 2, 3], true)], "at the press");
    assert_eq!(
        kinds(&app, &[1, 2, 3]).0,
        [SABRE, SABRE, RIFLE],
        "the sim's tick is still to come"
    );
    assert_eq!(shown_r(&mut app), (c(PanelCommand::Convert(SABRE)), ButtonLook::Common));
    app.update();
    assert_eq!(kinds(&app, &[1, 2, 3]), (vec![RIFLE; 3], vec![RIFLE; 3]));
    assert_eq!(forms(&app, &[1, 2, 3]), [SABRE, SABRE, RIFLE], "converting");
    assert_eq!([1, 2].map(|n| drawn_as(&app, n)), ["SabreLegionnaire"; 2]);
    assert_eq!([1, 2].map(|n| visual(&app, n)), [drawn[0], drawn[1]]);
    updates(&mut app, CONVERT_TICKS as usize);
    assert_eq!(forms(&app, &[1, 2, 3]), [RIFLE; 3]);
    assert_eq!([1, 2, 3].map(|n| drawn_as(&app, n)), ["RifleLegionnaire"; 3]);
    for n in [1, 2] {
        assert_ne!(visual(&app, n), drawn[n as usize - 1], "{n} drawn anew");
    }
    assert_eq!(visual(&app, 3), drawn[2], "the Rifle as ever");
    assert_eq!(selected(&app), [1, 2, 3]);

    click_button(&mut app, r(), &[]);
    assert_eq!(abilities(&app)[1..], [converted(&[1, 2, 3])]);
    assert_eq!(groups(&app), [(SABRE, vec![1, 2, 3], true)]);
    updates(&mut app, CONVERT_TICKS as usize + 2);
    assert_eq!(kinds(&app, &[1, 2, 3]), (vec![SABRE; 3], vec![SABRE; 3]));
    assert_eq!(forms(&app, &[1, 2, 3]), [SABRE; 3]);
    assert_ne!(visual(&app, 3), drawn[2], "the Rifle drawn anew");

    tap(&mut app, &[ShiftLeft], KeyR);
    assert_eq!(abilities(&app)[2..], [converted(&[1, 2, 3])]);
    assert_eq!(orders(&app), []);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// 0198's Verification: "4 Sabres (active) and 6 Rifles: press R. Only the Sabres convert, and all 10 show as one
/// active Rifle group. Press R again: all 10 become Sabres." The second R, a tick later, goes to all ten: the six
/// Rifles convert, and the four, a frame into their conversion, finish it and convert back (`> A toggle sets a target
/// form`: "mid-conversion: it finishes the 4 frames (Completion), then converts back"). Pressed twice before the
/// sim's next tick (the sim held meanwhile), the second R reaches all ten as well, and the four never convert. One
/// group throughout.
#[test]
fn four_sabres_and_six_rifles_r_gives_ten_rifles_and_r_again_ten_sabres() {
    let all: Vec<u32> = (1..=10).collect();
    let sabres = [1, 2, 3, 4];
    for held in [false, true] {
        let mut app = ten_legionnaires();
        select_units(&mut app, &all);
        assert_eq!(
            groups(&app),
            [(SABRE, sabres.to_vec(), true), (RIFLE, all[4..].to_vec(), false)]
        );
        app.world_mut().resource_mut::<SimControl>().paused = held;
        press_r(&mut app);
        assert_eq!(abilities(&app), [converted(&sabres)]);
        assert_eq!(groups(&app), [(RIFLE, all.clone(), true)], "held {held}");
        if held {
            updates(&mut app, 3);
            assert_eq!(groups(&app), [(RIFLE, all.clone(), true)], "until the sim has it");
        } else {
            app.update();
        }
        press_r(&mut app);
        assert_eq!(abilities(&app)[1..], [converted(&all)], "all ten");
        assert_eq!(groups(&app), [(SABRE, all.clone(), true)], "held {held}");
        app.world_mut().resource_mut::<SimControl>().paused = false;
        app.update();
        assert_eq!(kinds(&app, &all), (vec![SABRE; 10], vec![SABRE; 10]), "held {held}");
        let four = if held { None } else { Some(false) };
        assert_eq!(waits(&app, 1), four, "held {held}: the four's conversion");
        for _ in 0..2 * CONVERT_TICKS {
            app.update();
            assert_eq!(groups(&app), [(SABRE, all.clone(), true)], "held {held}");
        }
        assert_eq!(forms(&app, &all), [SABRE; 10], "held {held}");
        assert!(all.iter().all(|&n| waits(&app, n).is_none()), "held {held}");
    }
}

/// `factions/colonists/units/weapon_conversion.md > Eligibility`, as design changes 0153 and 0198 have it: pressed
/// during Firing or Cooldown, Convert "is deferred, not rejected", and the unit's type switches at the press all the
/// same. Rifle 3, alone selected, shoots Generator 4: through a whole attack sequence (Aim 3, Firing 2, Cooldown 7,
/// Reload 4 frames), R stays available. With Sabre 1 selected too, Tabbed to the Rifle, R pressed while the Rifle is
/// Firing goes to it alone, and at once it is in the Sabres' group, the ActiveGroup, with the panel's R available,
/// while it keeps its rifle and is drawn as a Rifle: its shot lands, a rifle's as before (`> Eligibility`: "The shot
/// already under way still finishes"), and it waits for its Cooldown. R again before that goes to both: the Sabre
/// converts, and the Rifle, its target its weapon again, doesn't convert at all ("it just takes the new target; if that
/// is its current weapon again, it doesn't convert at all"); it fights on with its Attack (design change 0150). Both
/// are Rifles, one group.
#[test]
fn r_mid_attack_switches_the_group_at_once_and_r_again_drops_the_waiting_conversion() {
    let mut app = legion_game();
    select_units(&mut app, &[3]);
    let attack = sc_sim::Command::Order {
        units: vec![ObjectId(3)],
        kind: OrderKind::Attack(ObjectId(GENERATOR)),
        queue: false,
    };
    submit(&mut app, P1, attack);
    let mut mid_attack = Vec::new();
    let mut rifle_shots = Vec::new();
    for _ in 0..20 {
        rifle_shots.extend(shots_by_3(&mut app, 1));
        let phase = phase(&app, 3);
        assert_eq!(look(&mut app), Some(ButtonLook::Common), "{phase:?}");
        if matches!(phase, Some(AttackPhase::Firing { .. } | AttackPhase::Cooldown)) {
            mid_attack.push(phase);
        }
    }
    assert!(
        !mid_attack.is_empty() && !rifle_shots.is_empty(),
        "the Rifle fired and cooled down"
    );

    select_units(&mut app, &[1, 3]);
    tap(&mut app, &[], Tab);
    assert_eq!(groups(&app), [(SABRE, vec![1], false), (RIFLE, vec![3], true)]);
    until(&mut app, |phase| matches!(phase, Some(AttackPhase::Aiming { .. })));
    until(&mut app, |phase| matches!(phase, Some(AttackPhase::Firing { .. })));
    press_r(&mut app);
    assert!(
        matches!(phase(&app, 3), Some(AttackPhase::Firing { .. })),
        "pressed mid-shot"
    );
    assert_eq!(abilities(&app), [converted(&[3])], "the ActiveGroup alone");
    assert_eq!(groups(&app), [(SABRE, vec![1, 3], true)], "at the press");
    assert_eq!(
        shots_by_3(&mut app, 1),
        [rifle_shots[0]],
        "its shot lands, with its rifle"
    );
    assert!(
        matches!(phase(&app, 3), Some(AttackPhase::Cooldown)),
        "{:?}",
        phase(&app, 3)
    );
    assert_eq!(
        (waits(&app, 1), waits(&app, 3)),
        (None, Some(true)),
        "Rifle 3 waits for its Cooldown"
    );
    assert_eq!(kinds(&app, &[1, 3]), (vec![SABRE; 2], vec![SABRE; 2]));
    assert_eq!(forms(&app, &[3]), [RIFLE]);
    assert_eq!(drawn_as(&app, 3), "RifleLegionnaire");
    let c = PanelButton::Command;
    assert_eq!(shown_r(&mut app), (c(PanelCommand::Convert(RIFLE)), ButtonLook::Common));

    press_r(&mut app);
    assert_eq!(abilities(&app)[1..], [converted(&[1, 3])]);
    assert_eq!(groups(&app), [(RIFLE, vec![1, 3], true)]);
    for _ in 0..2 * CONVERT_TICKS {
        app.update();
        assert_eq!(waits(&app, 3), None, "Rifle 3 has nothing to do");
        assert_eq!(drawn_as(&app, 3), "RifleLegionnaire");
        assert_eq!(groups(&app), [(RIFLE, vec![1, 3], true)]);
    }
    assert_eq!(kinds(&app, &[1, 3]), (vec![RIFLE; 2], vec![RIFLE; 2]));
    assert_eq!(forms(&app, &[1, 3]), [RIFLE; 2]);
    let orders = object_orders(&app, 3);
    let attacking = Some(OrderKind::Attack(ObjectId(GENERATOR)));
    assert_eq!(orders.current.map(|a| a.order.kind), attacking);
    assert!(!shots_by_3(&mut app, 16).is_empty(), "it fights on");
}

/// The damage of each hit Rifle 3 lands in the next `n` ticks.
fn shots_by_3(app: &mut App, n: usize) -> Vec<f32> {
    let events = run_collecting(app, n);
    events
        .into_iter()
        .filter_map(|event| match event {
            SimEvent::Hit {
                attacker: ObjectId(3),
                damage,
                ..
            } => Some(damage),
            _ => None,
        })
        .collect()
}
