//! The Riot Shield's two unit types in a selection (`factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit
//! types`, design change 0173): the Peacekeepers and the Peacekeepers (Shielded) form separate SelectionGroups, the
//! shield command goes to the ActiveGroup, and the units it turns change group at the press, also those whose raise
//! waits for their shot (design change 0198: "The type, and so the SelectionGroup, switches at the press"). The group
//! they join becomes the ActiveGroup (`ControlState::toggle`, design changes 0198 and 0199).

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
/// Shielded one, and on to the four again. Their R goes to them alone, a group command; they switch type at the
/// press, joining the six, and the one group left is active (design change 0198: "that merged group becomes the
/// ActiveGroup").
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
/// shields leave their group, which goes; the group they join becomes the ActiveGroup, in its DefaultState (design
/// change 0199: "Its members changed type and joined another group [...] the group they joined becomes active. The
/// command panel resets to that group's default state").
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
/// (Shielded), which never fires back, each a tick after the one before; R is pressed as the first fires. All four are
/// Peacekeepers (Shielded) at the press, before the sim's tick, one group, the ActiveGroup, whose panel shows Lower
/// Shield (design change 0198: "The type, and so the SelectionGroup, switches at the press, including when the raise
/// is deferred behind a shot"; its Verification: "Peacekeepers mid-shot: press R. The panel shows Lower Shield at
/// once"). Those in Firing or Cooldown finish their shot first ("the raise is deferred: the shot finishes, then the
/// shield starts rising immediately"), their shields still down and drawn so meanwhile. Nobody fires once its shield
/// is on its way up.
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

    let mut seen_groups = vec![groups(&app)];
    input(&mut app).tap_key(KeyR);
    app.update();
    assert_eq!(groups(&app), [(SHIELDED, FOUR.to_vec(), true)], "at the press");
    assert_eq!(
        command(&mut app, r()),
        Some((PanelCommand::LowerShield, ButtonLook::Common))
    );
    assert_eq!(state(&app, 1).0, PEACEKEEPER, "the sim's tick is still to come");
    let start = app.world().resource::<SimRes>().tick();
    let mut raised = [None; 4];
    let mut deferred = Vec::new();
    let mut shot_raising: Vec<u32> = Vec::new();
    for _ in 0..12 {
        app.update();
        let sim = app.world().resource::<SimRes>();
        let tick = sim.tick() - start;
        for event in sim.events() {
            if let sc_sim::SimEvent::Attacked { attacker, .. } = event
                && raised[attacker.0 as usize - 1].is_some()
            {
                shot_raising.push(attacker.0);
            }
        }
        let mut waiting = Vec::new();
        for n in FOUR {
            let unit = sim.object(ObjectId(n)).unwrap();
            assert_eq!(unit.kind, SHIELDED, "#{n} from the press");
            if unit.riot_shield.unwrap().up > 0 && raised[n as usize - 1].is_none() {
                raised[n as usize - 1] = Some(tick);
            }
            if shield(&app, n) == ShieldMode::Deferred {
                waiting.push(n);
            }
        }
        for n in waiting {
            let drawn = shield_visual(&mut app, n).map(|(visibility, ..)| visibility);
            assert_eq!(drawn, Some(Visibility::Hidden), "#{n}'s shield is still down");
            deferred.push(n);
        }
        let now = groups(&app);
        if seen_groups.last() != Some(&now) {
            seen_groups.push(now);
        }
    }
    assert_eq!(abilities(&app), [used(&FOUR, RAISE)]);
    let ticks: Vec<_> = raised.iter().map(|t| t.expect("every one raises its shield")).collect();
    // A raise waits at most its unit's Firing and Cooldown frames (1 and 2).
    let weapon = PEACEKEEPER.def().unit.as_ref().and_then(|unit| unit.attack).unwrap();
    let spread = ticks.iter().max().unwrap() - ticks.iter().min().unwrap();
    assert!(
        (1..=weapon.firing_ticks + weapon.cooldown_ticks).contains(&spread),
        "some a few frames after the others: {ticks:?}"
    );
    assert!(!deferred.is_empty(), "some wait for their shot");
    assert!(
        shot_raising.is_empty(),
        "nobody fires once its shield is on its way up: {shot_raising:?}"
    );
    assert_eq!(
        seen_groups,
        [
            vec![(PEACEKEEPER, FOUR.to_vec(), true)],
            vec![(SHIELDED, FOUR.to_vec(), true)],
        ]
    );
}
