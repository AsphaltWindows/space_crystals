//! The Garrison's golden game (slice M5-N2, `factions/cults/structures/garrison.md`): Militants and a Gunner inside
//! it against Agents and Grenadiers.

use sc_sim::network::Place;

use super::*;

/// The Garrison (id 1), player 1's, on tiles (8, 5) to (9, 6), its door on side B, west.
const GARRISON: ObjectId = ObjectId(1);
/// The units that go in: three Militants and a Gunner.
const INSIDE: [ObjectId; 4] = [ObjectId(2), ObjectId(3), ObjectId(4), ObjectId(5)];
/// Player 2's Grenadiers.
const GRENADIERS: [ObjectId; 2] = [ObjectId(10), ObjectId(11)];
/// The four go in at tick 1; player 2's units attack from this tick on, with everyone inside.
const ATTACK: Tick = 120;

/// Player 1 plays the Cults, player 2 the Syndicate.
fn cults_p1_syndicate_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.build();
    scenario.setup.players[0].faction = Faction::Cults;
    scenario.setup.players[1].faction = Faction::Syndicate;
    scenario
}

/// Three Militants and a Gunner of player 1's (ids 2–5) go into their Garrison (id 1) through its door; at [`ATTACK`]
/// four Agents and two Grenadiers of player 2's (ids 6–11), 12 tiles east of it, AttackMove onto it. The four fire
/// from its footprint (`combat::occupants`), the Militants 5 grid units off it and the Gunner 6 (`> Garrisoning`'s +1
/// grid unit of Range), farther than the Grenadiers' 4 and the Agents' 2, and kill an Agent while it stands. The
/// Grenadiers' blasts land on it and hurt no one inside (`> Garrisoning > Protection`); one brings it down, and the four
/// come out alive and unhurt (`Sim::evacuate`) and kill the rest. Player 1's Storage (id 12), in a corner, keeps it in
/// the game once the Garrison is gone.
pub(crate) fn cults_garrison() -> Game {
    let mut builder =
        ScenarioBuilder::new(&open(28, 12)).spawn(ObjectKind::Garrison, Some(P1), SpawnAt::Tile(TilePos::new(8, 5)));
    let inside = [
        ObjectKind::Militant,
        ObjectKind::Militant,
        ObjectKind::Militant,
        ObjectKind::Gunner,
    ];
    for (y, kind) in [4, 5, 6, 7].into_iter().zip(inside) {
        builder = builder.spawn(kind, Some(P1), at(5, y));
    }
    for y in [4, 5, 6, 7] {
        builder = builder.spawn(ObjectKind::Agent, Some(P2), at(21, y));
    }
    for y in [5, 6] {
        builder = builder.spawn(ObjectKind::Grenadier, Some(P2), at(22, y));
    }
    builder = builder.spawn(ObjectKind::Storage, Some(P1), SpawnAt::Tile(TilePos::new(1, 1)));
    Game {
        scenario: cults_p1_syndicate_p2(builder),
        commands: vec![
            (1, order(P1, 2..=5, OrderKind::Enter(GARRISON))),
            (ATTACK, order(P2, 6..=11, OrderKind::AttackMove(point(9, 6)))),
        ],
        script: None,
        end: 640,
        during: |tick, sim| {
            for held in sim.held() {
                let unit = &held.object;
                assert_eq!(held.place, Place::Building(GARRISON), "tick {tick}: #{}", unit.id.0);
                assert_eq!(unit.hp, unit.max_hp(), "tick {tick}: #{} is hurt inside", unit.id.0);
            }
            if tick == ATTACK {
                assert_eq!(sim.held_in(GARRISON).len(), 4, "the four went in");
            }
            let events = sim.events();
            if events
                .iter()
                .any(|e| matches!(e, SimEvent::Died { id, .. } if *id == GARRISON))
            {
                assert!(
                    events.iter().any(|e| matches!(e, SimEvent::Hit { attacker, target, .. }
                        if *target == GARRISON && GRENADIERS.contains(attacker))),
                    "tick {tick}: a Grenadier's blast brings the Garrison down"
                );
                let evacuated = events
                    .iter()
                    .filter(|e| matches!(e, SimEvent::Evacuated { .. }))
                    .count();
                assert_eq!(evacuated, 4, "tick {tick}: the four come out");
                for id in INSIDE {
                    let unit = sim.object(id).unwrap_or_else(|| panic!("#{} came out alive", id.0));
                    assert_eq!(unit.hp, unit.max_hp(), "tick {tick}: #{} came out unhurt", id.0);
                }
                assert!(units_of(sim, P2).len() < 6, "tick {tick}: they killed from inside");
            }
        },
        expect: |sim| {
            assert!(sim.object(GARRISON).is_none(), "the Garrison fell");
            assert!(units_of(sim, P2).is_empty(), "player 2's units are all dead");
            assert!(
                INSIDE.iter().any(|&id| sim.object(id).is_some()),
                "a unit of the four survives"
            );
        },
    }
}
