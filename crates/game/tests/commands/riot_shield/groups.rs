//! The Riot Shield's two unit types in a selection (`factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit
//! types`, design change 0173): the Peacekeepers and the Peacekeepers (Shielded) form separate SelectionGroups, the
//! shield command goes to the ActiveGroup, and the units it turns change group as their toggle starts, or, pressed
//! mid-shot, once their shot is done (design change 0172). The ActiveGroup they leave empty gives way to the group
//! they join (`ControlState::set_type`, Q0050).

use sc_sim::combat::AttackPhase;
use sc_sim::{Command, Spawn};
use space_crystals::hud::selection_panel::{PortraitCell, SelectionPanelView};

use super::*;

/// The SelectionPanel's portraits, in their order: each one's type, and whether it is highlighted as the
/// ActiveGroup's.
fn portraits(app: &App) -> Vec<(ObjectKind, bool)> {
    let cells = &app.world().resource::<SelectionPanelView>().cells;
    cells
        .iter()
        .map(|cell| match *cell {
            PortraitCell::Object { kind, active, .. } => (kind, active),
            PortraitCell::More { .. } => panic!("every portrait fits"),
        })
        .collect()
}

/// The 0173 Verification (`> Riot Shield > Two unit types`): "Select 10 Peacekeepers and raise shields on 6. The
/// SelectionPanel shows two groups. Tab to the 4 and press R: all 10 are shielded, in one group. The Peacekeeper
/// panel shows Raise Shield (R); the Shielded panel shows Lower Shield (R) and no Attack." With the ten selected, the
/// four Peacekeepers' group comes first in the panel's order, active; Tab goes to the six, whose panel is the
/// Shielded one, and on to the four again. Their R goes to them alone, a group command; they switch type as their
/// toggle starts, joining the six, and the one group left is active (its old group emptied, Q0050).
#[test]
fn six_shielded_of_ten_are_a_group_of_their_own_until_the_other_four_raise_theirs() {
    const SIX: [u32; 6] = [1, 2, 3, 4, 5, 6];
    const FOUR: [u32; 4] = [7, 8, 9, 10];
    let ten: Vec<u32> = (1..=10).collect();
    let mut scenario = scenarios::empty(64, 48);
    let peacekeeper = |slot, at| Spawn::new(PEACEKEEPER, Some(SlotId(slot)), SpawnAt::Point(at));
    let rows = SIX.map(|n| Vec2::new(1280.0 + 70.0 * n as f32, 800.0));
    let second = FOUR.map(|n| Vec2::new(1280.0 + 70.0 * (n - 6) as f32, 1000.0));
    scenario
        .map
        .spawns
        .extend(rows.into_iter().chain(second).map(|at| peacekeeper(1, at)));
    scenario.map.spawns.push(peacekeeper(2, Vec2::new(3600.0, 2800.0)));
    assert_eq!(scenario.validate(), vec![]);
    let mut app = game_of(scenario);

    select_units(&mut app, &SIX);
    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    select_units(&mut app, &ten);
    assert_eq!(
        groups(&app),
        [(PEACEKEEPER, FOUR.to_vec(), true), (SHIELDED, SIX.to_vec(), false)]
    );
    let cells: Vec<_> = [(PEACEKEEPER, true); 4]
        .into_iter()
        .chain([(SHIELDED, false); 6])
        .collect();
    assert_eq!(portraits(&app), cells, "two groups");
    assert_eq!(
        command(&mut app, r()),
        Some((PanelCommand::RaiseShield, ButtonLook::Group))
    );
    assert_eq!(command(&mut app, a()), Some((PanelCommand::Attack, ButtonLook::Common)));

    tap(&mut app, &[], Tab);
    assert_eq!(active_type(&app), Some(SHIELDED));
    assert_eq!(
        command(&mut app, r()),
        Some((PanelCommand::LowerShield, ButtonLook::Group))
    );
    assert_eq!(button(&mut app, a()), None, "no Attack");
    tap(&mut app, &[], Tab);
    assert_eq!(active_type(&app), Some(PEACEKEEPER), "the 4");

    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app), [used(&SIX, RAISE), used(&FOUR, RAISE)]);
    pause(&mut app);
    assert_eq!(groups(&app), [(SHIELDED, ten.clone(), true)]);
    assert_eq!(portraits(&app), [(SHIELDED, true); 10], "one group");
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    toggle_time(&mut app);
    assert!(ten.iter().all(|&id| state(&app, id) == (SHIELDED, ShieldMode::Raised)));
}

/// `> CommandPanel`: in a mixed selection R is a group command, for the ActiveGroup alone ("the shield command goes
/// to the ActiveGroup", design change 0173), and with the Guards active R is empty. The Peacekeepers raising their
/// shields leave their group, which goes; the group they join becomes the ActiveGroup, in its DefaultState
/// (`ControlState::set_type`: the rule M3-U set for a group that empties as its objects change type, Q0050).
#[test]
fn in_a_mixed_selection_r_goes_to_the_active_group() {
    let mut app = game();
    select_mixed(&mut app);
    assert_eq!(
        command(&mut app, r()),
        Some((PanelCommand::RaiseShield, ButtonLook::Group))
    );
    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app), [used(&[1, 2], RAISE)]);
    assert_eq!(
        groups(&app),
        [(ObjectKind::Guard, vec![3, 4], false), (SHIELDED, vec![1, 2], true)]
    );
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(
        command(&mut app, r()),
        Some((PanelCommand::LowerShield, ButtonLook::Group))
    );

    tap(&mut app, &[], Tab);
    assert_eq!(active_type(&app), Some(ObjectKind::Guard));
    assert_eq!(button(&mut app, r()), None);
    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app).len(), 1);
    toggle_time(&mut app);
    assert_eq!([shield(&app, 1), shield(&app, 2)], [ShieldMode::Raised; 2]);
}

/// The 0172 Verification (`> Riot Shield > During the toggle`): "Peacekeepers fighting, press the shield key: every
/// one raises its shield, some a few frames after the others." Four Peacekeepers attack an enemy Peacekeeper
/// (Shielded), which never fires back, each a tick after the one before; R is pressed as the first fires. It and the
/// others in Firing or Cooldown finish their shot first ("the raise is deferred: the shot finishes, then the shield
/// starts rising immediately"), staying Peacekeepers meanwhile, in their group, which stays the ActiveGroup while it
/// has them (`ControlState::set_type`); the last of them leaving it, the group of the shielded ones takes over.
/// Nobody fires once shielded.
#[test]
fn fighting_peacekeepers_all_raise_some_a_few_frames_later() {
    const FOUR: [u32; 4] = [1, 2, 3, 4];
    const ENEMY: u32 = 5;
    let mut scenario = scenarios::empty(64, 48);
    let unit = |kind, slot, at| Spawn::new(kind, Some(SlotId(slot)), SpawnAt::Point(at));
    let column = FOUR.map(|n| unit(PEACEKEEPER, 1, Vec2::new(1300.0, 740.0 + 60.0 * n as f32)));
    scenario.map.spawns.extend(column);
    scenario.map.spawns.push(unit(SHIELDED, 2, Vec2::new(1500.0, 890.0)));
    assert_eq!(scenario.validate(), vec![]);
    let mut app = game_of(scenario);
    for n in FOUR {
        let attack = OrderKind::Attack(ObjectId(ENEMY));
        let units = vec![ObjectId(n)];
        submit(
            &mut app,
            PlayerId(1),
            Command::Order {
                units,
                kind: attack,
                queue: false,
            },
        );
        app.update();
    }
    select_units(&mut app, &FOUR);
    let phase = |app: &App, n: u32| {
        let sim = app.world().resource::<SimRes>();
        sim.object(ObjectId(n)).unwrap().attack.as_ref().and_then(|a| a.phase)
    };
    for _ in 0..40 {
        if matches!(phase(&app, 1), Some(AttackPhase::Firing { .. })) {
            break;
        }
        app.update();
    }
    assert!(matches!(phase(&app, 1), Some(AttackPhase::Firing { .. })), "fighting");

    input(&mut app).tap_key(KeyR);
    let start = app.world().resource::<SimRes>().tick();
    let mut switched = [None; 4];
    let mut seen_groups = Vec::new();
    let mut shot_shielded: Vec<u32> = Vec::new();
    for _ in 0..12 {
        app.update();
        let sim = app.world().resource::<SimRes>();
        let tick = sim.tick() - start;
        for event in sim.events() {
            if let sc_sim::SimEvent::Attacked { attacker, .. } = event
                && sim.object(*attacker).is_some_and(|o| o.kind == SHIELDED)
                && switched[attacker.0 as usize - 1].is_some()
            {
                shot_shielded.push(attacker.0);
            }
        }
        for n in FOUR {
            if switched[n as usize - 1].is_none() && state(&app, n).0 == SHIELDED {
                switched[n as usize - 1] = Some(tick);
            }
        }
        let now = groups(&app);
        if seen_groups.last() != Some(&now) {
            seen_groups.push(now);
        }
    }
    assert_eq!(abilities(&app), [used(&FOUR, RAISE)]);
    let ticks: Vec<_> = switched
        .iter()
        .map(|t| t.expect("every one raises its shield"))
        .collect();
    // A raise waits at most its unit's Firing and Cooldown frames (1 and 2).
    let weapon = PEACEKEEPER.def().unit.as_ref().and_then(|unit| unit.attack).unwrap();
    let spread = ticks.iter().max().unwrap() - ticks.iter().min().unwrap();
    assert!(
        (1..=weapon.firing_ticks + weapon.cooldown_ticks).contains(&spread),
        "some a few frames after the others: {ticks:?}"
    );
    assert!(
        shot_shielded.is_empty(),
        "nobody fires once shielded: {shot_shielded:?}"
    );
    // Units 3 and 4, aiming, raise at once; 1, firing, and 2, which fires next, once their Cooldown is over.
    assert_eq!(
        seen_groups,
        [
            vec![(PEACEKEEPER, FOUR.to_vec(), true)],
            vec![(PEACEKEEPER, vec![1, 2], true), (SHIELDED, vec![3, 4], false)],
            vec![(PEACEKEEPER, vec![2], true), (SHIELDED, vec![1, 3, 4], false)],
            vec![(SHIELDED, FOUR.to_vec(), true)],
        ]
    );
}
