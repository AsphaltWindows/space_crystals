//! The Supply Tower in the game: `factions/gdo/structures/deployment_center.md > Constructs` ("Supply Tower: 150 Space
//! Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Radar Station"),
//! `factions/gdo/economy.md > SupplyTower > On Placement` and `factions/gdo/economy.md > SupplyTower > Produces`.

use sc_sim::ProductionCommand;

use super::*;

fn construct_tower() -> Command {
    Command::Construction {
        objects: vec![id(DC)],
        command: ConstructionCommand::Construct(TOWER),
    }
}

/// The Deployment Center constructs the Supply Tower only for a player with a Radar Station (`Reject::Requires`, the
/// client's greyed button), and takes its 150 Space Crystals as it starts.
#[test]
fn the_deployment_center_constructs_a_tower_once_a_radar_station_stands() {
    let mut t = base(|b| b);
    debug(&mut t, DebugCommand::Remove { id: id(3) });
    assert_eq!(
        gate(&t, &construct_tower()),
        Err(Reject::Requires(ObjectKind::RadarStation))
    );
    let radar = DebugCommand::Spawn {
        kind: ObjectKind::RadarStation,
        owner: Some(P1),
        at: tile(8, 5),
        orientation: Default::default(),
    };
    debug(&mut t, radar);
    next(&mut t, construct_tower());
    assert_eq!(banks(&t, P1).0, 5000 - TOWER_COST);
}

/// `factions/gdo/economy.md > SupplyTower > On Placement`: "One Supply Chopper is spawned on the tower at no additional
/// cost (included in tower's construction cost). This chopper is automatically attached to the tower": placed 320
/// frames after its Construct, the Tower comes with a Supply Chopper of player 1's in the same tick, down on its spot
/// (its centre, a placeholder), attached to it, idle, at full HP, carrying nothing; the bank paid the Tower's 150
/// alone. It stays there.
#[test]
fn placing_a_tower_brings_its_free_attached_chopper() {
    let mut t = base(|b| b);
    let (tower, chopper) = placed_tower(&mut t, TilePos::new(2, 8));
    assert_eq!(banks(&t, P1), (5000 - TOWER_COST, 0));
    let o = object(&t, chopper);
    assert_eq!((o.kind, o.owner, o.hp), (CHOPPER, Some(P1), Some(150.0)));
    assert_eq!(o.pos, object(&t, tower).pos, "on its spot, the Tower's centre");
    assert_eq!(landing_of(&t, chopper), Some(landing::down_on(id(tower), LANDING)));
    assert_eq!(
        (attached(&t, chopper), carried(&t, chopper), doing(&t, chopper)),
        (Some(tower), 0, None)
    );
    t.run(100);
    assert!(down_on(&t, chopper, tower), "it stays");
    assert_eq!(t.pos(chopper), spot(&t, tower));
}

/// `factions/gdo/economy.md > SupplyTower > Produces`: "Supply Chopper: 100 Space Crystals, 160 frames (10 seconds)":
/// the Tower trains a chopper for 100 Space Crystals, out 160 frames after its Q, in the air over the Tower's centre (a
/// placeholder), unattached and empty; the free chopper stays down on the Tower. With a rally point on the ground, the
/// next flies there.
#[test]
fn the_tower_trains_choppers_over_its_centre() {
    let mut t = base(|b| b);
    let (tower, free) = placed_tower(&mut t, TilePos::new(2, 8));
    let train = Command::Production {
        objects: vec![id(tower)],
        command: ProductionCommand::Train(CHOPPER),
    };
    let crystals = banks(&t, P1).0;
    let queued_at = next(&mut t, train.clone());
    assert_eq!(banks(&t, P1).0, crystals - CHOPPER_COST);
    let produced = |t: &TestSim| {
        (t.events().iter())
            .filter_map(|(tick, e)| match *e {
                SimEvent::UnitProduced { producer, unit } if producer == id(tower) => Some((*tick, unit.0)),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    step_until(&mut t, 2 * CHOPPER_TICKS, "trained", |t| !produced(t).is_empty());
    let (tick, chopper) = produced(&t)[0];
    assert_eq!(tick, queued_at + CHOPPER_TICKS - 1, "160 frames");
    let o = object(&t, chopper);
    assert_eq!((o.kind, o.owner, o.pos), (CHOPPER, Some(P1), object(&t, tower).pos));
    assert_eq!(
        (landing_of(&t, chopper), attached(&t, chopper), carried(&t, chopper)),
        (None, None, 0)
    );
    assert!(down_on(&t, free, tower), "the free one stays down");

    let rally = object(&t, tower).pos + Vec2::new(640.0, 0.0);
    let set_rally = Command::Production {
        objects: vec![id(tower)],
        command: ProductionCommand::SetRally(sc_sim::Target::Point(rally)),
    };
    next(&mut t, set_rally);
    next(&mut t, train);
    step_until(&mut t, 2 * CHOPPER_TICKS, "trained", |t| produced(t).len() == 2);
    let second = produced(&t)[1].1;
    step_until(&mut t, 400, "at the rally point", |t| {
        object(t, second).pos.distance(rally) < 2.0
    });
}
