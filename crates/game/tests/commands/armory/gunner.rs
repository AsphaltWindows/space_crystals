//! The Armory's W in the client (slice M5-G2; `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`:
//! "**W: Train Gunner** [...] and the player owns at least one Gunworks (TechPrerequisite)"): greyed while player 1
//! owns no Gunworks, saying so when pressed; once one stands, W trains a Gunner of a Recruit of the kind for 40 Space
//! Crystals, the menu staying open; and once the last one is lost, W is greyed again while the Gunner queued is still
//! trained (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`: "units already queued or in
//! production continue and complete normally"). The training itself is the sim's (`crates/sim/tests/armory/gunner.rs`).

use super::*;

/// Where player 1's Gunworks stands, spawned complete north of the Armories: tiles (16, 6) to (17, 7). The Recruits'
/// build of one is the sim's (`crates/sim/tests/cults_construction/tech.rs`).
const GUNWORKS_AT: TilePos = TilePos::new(16, 6);

/// Spawns player 1's complete Gunworks at [`GUNWORKS_AT`] (a debug command) and returns its id, two frames later.
fn spawn_gunworks(app: &mut App) -> u32 {
    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::Gunworks,
        owner: Some(P1),
        at: SpawnAt::Tile(GUNWORKS_AT),
        orientation: Default::default(),
    };
    submit(app, P1, Command::Debug(spawn));
    updates(app, 2);
    let sim = app.world().resource::<SimRes>();
    let gunworks = (sim.objects()).find(|o| o.kind == ObjectKind::Gunworks);
    gunworks.expect("spawned").id.0
}

/// The W of the KindMenu shown, as the HUD shows it: its button, its look and its count.
fn w(app: &mut App) -> (PanelButton, ButtonLook, Option<u32>) {
    let shown = buttons(app).into_iter().find(|b| b.0 == slot(0, 1));
    let (_, button, look, count) = shown.expect("W shows");
    (button, look, count)
}

/// The plan's acceptance (addendum G2): "W stays greyed, "Requires a Gunworks", until one is complete. Then a Gunner
/// costs 40 SC and 32 frames"; "When the last Gunworks is lost, queued Gunners complete and no new one can be queued."
/// KindMenu\[Plain\]'s W is greyed and sends nothing; with a Gunworks it sends one TrainFrom, the Plain Recruit that
/// went in first reserved and 40 Space Crystals paid; the Gunworks gone, W is greyed again, and the Recruit comes out a
/// Gunner.
#[test]
fn w_trains_a_gunner_once_a_gunworks_stands_and_greys_again_once_it_is_lost() {
    use ButtonLook::{Common, Disabled};
    let mut app = armory_game();
    let first_in = (inside(&app, ARMORY_1).into_iter())
        .find(|id| PLAIN.contains(id))
        .expect("a Plain Recruit");
    click(&mut app, ARMORY_1_AT, &[]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain));
    let train_gunner = PanelButton::Command(PanelCommand::TrainFrom(GUNNER, Plain));
    assert_eq!(w(&mut app), (train_gunner, Disabled, None));
    tap(&mut app, &[], KeyW);
    assert_eq!(feedback(&app), Some("Requires a Gunworks"));
    assert_eq!(productions(&app), []);

    let gunworks = spawn_gunworks(&mut app);
    assert_eq!(w(&mut app), (train_gunner, Common, None));
    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::KindMenu(Plain), "the menu stays open");
    let train = ProductionCommand::TrainFrom {
        kind: GUNNER,
        from: Plain,
    };
    assert_eq!(productions(&app), [(P1, vec![ARMORY_1], train)]);
    updates(&mut app, 2);
    assert_eq!(bank(&app), 1000 - 40);

    let remove = DebugCommand::Remove { id: ObjectId(gunworks) };
    submit(&mut app, P1, Command::Debug(remove));
    updates(&mut app, 2);
    assert_eq!(w(&mut app), (train_gunner, Disabled, None), "greyed again");
    tap(&mut app, &[], KeyW);
    assert_eq!(feedback(&app), Some("Requires a Gunworks"));
    assert_eq!(productions(&app).len(), 1, "nothing more sent");
    let trained = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        sim.object(ObjectId(first_in)).map(|o| o.kind)
    };
    until(&mut app, 100, |app| trained(app).is_some());
    assert_eq!(trained(&app), Some(GUNNER), "the Gunner queued is trained");
    assert_eq!(bank(&app), 1000 - 40);
}
