//! Buildings that hold units, where a test needs the sim's inside: an Armory under construction (no Cults site exists
//! before slice M5-E), and the mark Eject All leaves on a unit inside (`Held::leaving`, slice M5-F2's).

use glam::Vec2;

use crate::command::{Command, HoldCommand, PlayerCommand};
use crate::construction::site::{Builder, Construction};
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::progress::Progress;
use crate::scenario::SpawnAt;
use crate::sim::Reject;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);
const ARMORY: ObjectId = ObjectId(1);
const RECRUIT: ObjectId = ObjectId(2);

/// Player 1's Armory (1) on tiles (12, 10) to (14, 11) of a 30 × 22 map, unturned: its entrance, B, faces east at
/// x = 960, its entry path's spot at (960, 704). A Recruit (2) of player 1's stands east of it.
fn base() -> TestSim {
    let mut t = ScenarioBuilder::new(&"..............................\n".repeat(22))
        .faction(P1, Faction::Cults)
        .spawn(ObjectKind::Armory, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Recruit, P1, Vec2::new(1100.0, 704.0));
    t.run_until(1);
    t
}

fn enter() -> PlayerCommand {
    let command = Command::Order {
        units: vec![RECRUIT],
        kind: OrderKind::Enter(ARMORY),
        queue: false,
    };
    PlayerCommand { player: P1, command }
}

/// Placeholder (Q0104 point 4): "an Armory under construction can't be entered"; "A right-click on one Assists"
/// (slice M5-E), which until then is a Move to it. In operation it takes the Recruit, by the gate and the right-click.
#[test]
fn an_armory_under_construction_refuses_enter() {
    let t = base();
    assert_eq!(t.sim().validate(&enter()), Ok(()));
    let right_click = |sim: &crate::sim::Sim| sim.default_order(RECRUIT, Target::Object(ARMORY));
    assert_eq!(right_click(t.sim()), Some(OrderKind::Enter(ARMORY)));
    let mut sim = t.sim().clone();
    let armory = sim.objects.get_mut(&ARMORY).unwrap();
    armory.construction = Some(Construction {
        progress: Progress::new(480),
        builder: Builder::Held(ObjectId(99)),
    });
    assert!(armory.is_site());
    assert_eq!(sim.validate(&enter()), Err(Reject::UnderConstruction(ARMORY)));
    assert_eq!(right_click(&sim), Some(OrderKind::Move(Target::Object(ARMORY))));
    assert_eq!(sim.room(ARMORY), 10, "an empty pool, all the same");
}

/// The mark Eject All leaves on a unit inside (`Held::leaving`, the tick it is ready to leave from) is part of the
/// state, hashed and saved only while set: a game that never ejects hashes and saves as it did before the mark
/// existed.
#[test]
fn the_eject_mark_is_hashed_and_saved_only_while_set() {
    let mut t = base();
    t.at(2, P1, enter().command);
    t.run(60);
    assert_eq!(t.sim().held_in(ARMORY).len(), 1, "the Recruit is inside");
    assert_eq!((t.sim().room(ARMORY), t.sim().room(RECRUIT)), (9, 0));
    let unmarked = t.sim().state_hash();
    let json = serde_json::to_value(t.sim()).unwrap();
    assert!(json["held"][0].get("leaving").is_none(), "{}", json["held"][0]);
    let mut sim = t.sim().clone();
    sim.apply_holding(&[ARMORY], HoldCommand::EjectAll);
    assert_eq!(sim.held[&RECRUIT].leaving, Some(sim.tick()));
    assert_ne!(sim.state_hash(), unmarked);
    let json = serde_json::to_value(&sim).unwrap();
    assert_eq!(json["held"][0]["leaving"], sim.tick());
    let loaded: crate::sim::Sim = serde_json::from_value(json).unwrap();
    assert_eq!(loaded.state_hash(), sim.state_hash());
    sim.held.get_mut(&RECRUIT).unwrap().leaving = None;
    assert_eq!(sim.state_hash(), unmarked);
}
