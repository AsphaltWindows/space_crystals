//! The Defense System from the Deployment Center (`factions/gdo/structures/deployment_center.md > Constructs`:
//! "Defense System: 150 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Radar
//! Station and at least one Arsenal").

use super::*;

/// The plan's acceptance, "the Defense System [is refused] without a Radar Station or without an Arsenal": player 1's
/// Deployment Center and Generator alone can't start one, the Radar Station named first (`Reject::Requires`); with an
/// Arsenal only, the Radar Station is missing; with a Radar Station only, the Arsenal. With both it costs 150 and takes
/// its 320 frames, and placed it is complete and idle, a constructor of Missile Turrets.
#[test]
fn the_deployment_center_constructs_a_defense_system_with_a_radar_station_and_an_arsenal() {
    let scenario = ScenarioBuilder::new(&flat(48, 32))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2)) // 1
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2)); // 2
    let mut t = started(scenario);
    let construct = || construction(DC, Construct(DEFENSE));
    let missing = |kind| Err(Reject::Requires(kind));
    assert_eq!(gate(&t, P1, construct()), missing(ObjectKind::RadarStation));
    let spawn = |kind, at| {
        Command::Debug(DebugCommand::Spawn {
            kind,
            owner: Some(P1),
            at,
            orientation: Default::default(),
        })
    };
    next(&mut t, spawn(ObjectKind::Arsenal, tile(6, 8))); // 3
    assert_eq!(gate(&t, P1, construct()), missing(ObjectKind::RadarStation));
    next(&mut t, Command::Debug(DebugCommand::Remove { id: id(3) }));
    next(&mut t, spawn(ObjectKind::RadarStation, tile(2, 8))); // 4
    assert_eq!(gate(&t, P1, construct()), missing(ObjectKind::Arsenal));
    next(&mut t, spawn(ObjectKind::Arsenal, tile(6, 8))); // 5
    assert_eq!(gate(&t, P1, construct()), Ok(()));
    let before = bank(&t);
    let ordered = next(&mut t, construct());
    assert_eq!(before - bank(&t), 150);
    until_ready(&mut t, DC);
    assert_eq!(completions(&t), [(ordered + 319, DC, DEFENSE)], "320 frames");
    next(&mut t, construction(DC, place(TilePos::new(8, 8))));
    let placed = t.sim().object(id(6)).expect("placed");
    assert_eq!(placed.kind, DEFENSE);
    assert!(placed.is_operational(), "complete as it is placed");
    assert_eq!(placed.constructor, Some(Constructor::Idle));
    assert_eq!(gate(&t, P1, construction(6, Construct(TURRET))), Ok(()));
}
