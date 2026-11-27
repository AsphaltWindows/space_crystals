//! The Radar Station from the Deployment Center (`factions/gdo/structures/deployment_center.md > Constructs`: "Radar
//! Station: 150 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Generator"),
//! placed complete.

use super::*;

/// The plan's acceptance, "the Radar Station is refused without a Generator": a Deployment Center alone can't start
/// one (`Reject::Requires`), and can once its Generator is placed. The Radar Station costs 150 and takes its 320 frames,
/// ready at the end of the 320th tick from its command's; placed, it is complete, and the next tick it scans
/// (`Object::is_operational`: "GDO buildings are placed complete").
#[test]
fn the_deployment_center_constructs_a_radar_station_once_the_player_has_a_generator() {
    let mut t = started(ScenarioBuilder::new(&flat(48, 32)).spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2)));
    let construct = |kind| construction(DC, ConstructionCommand::Construct(kind));
    assert_eq!(gate(&t, P1, construct(RADAR)), Err(Reject::Requires(GENERATOR)));
    next(&mut t, construct(GENERATOR));
    t.run(240);
    next(&mut t, construction(DC, place(TilePos::new(8, 2))));
    let generator = t.sim().object(id(2)).expect("placed");
    assert_eq!(generator.kind, GENERATOR);
    let before = t.sim().player(P1).unwrap().crystals;
    let ordered = next(&mut t, construct(RADAR));
    assert_eq!(before - t.sim().player(P1).unwrap().crystals, 150);
    let completed = |t: &TestSim| {
        (t.events().iter()).find_map(|(tick, e)| match *e {
            SimEvent::ConstructionCompleted { kind: RADAR, .. } => Some(*tick),
            _ => None,
        })
    };
    t.run_until(ordered + 400);
    assert_eq!(
        completed(&t),
        Some(ordered + 319),
        "320 frames, the command's own tick the first"
    );
    let placed = next(&mut t, construction(DC, place(TilePos::new(2, 8))));
    let station = t.sim().object(id(3)).expect("placed");
    assert_eq!(station.kind, RADAR);
    assert!(station.is_operational(), "complete as it is placed");
    assert_eq!(cooldown(&t, 3), None, "no cooldown to wait out");
    assert_eq!(gate(&t, P1, scan(&[3], FAR)), Ok(()));
    let cast = next(&mut t, scan(&[3], FAR));
    assert_eq!(cast, placed + 1);
    assert_eq!(casts(&t), [(placed + 1, 3, FAR)]);
}
