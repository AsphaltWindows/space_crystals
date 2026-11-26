//! Protection (`factions/cults/structures/garrison.md > Garrisoning`: "**Protection**: units inside cannot be targeted
//! and take no damage while the Garrison stands") and Evacuation on Destruction ("Evacuation on Destruction applies:
//! when the Garrison is destroyed, everyone inside comes out alive inside its former footprint"; F3's,
//! `factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`). Units inside are held off the
//! map (plan decision T72: "F1's held store and protection (held units are off the map, so they can't be targeted or
//! hit)").

use sc_sim::Target;
use sc_sim::combat::weapon;

use super::fire::targets;
use super::*;

/// The HP of the unit `unit` inside.
fn hp_inside(t: &TestSim, unit: ObjectId) -> f32 {
    t.sim().held_unit(unit).expect("inside").object.hp.expect("it has HP")
}

/// Nothing can aim at a unit inside: player 2's Militant beside the Garrison, which sees it and its door, can't be
/// ordered to attack one of them (the gate refuses it: no such object on the map), nor does a right-click on it give an
/// Attack; it can attack the Garrison itself.
#[test]
fn units_inside_cant_be_targeted() {
    let mut t = base().start();
    t.without_auto_targeting();
    let pool = fill(&mut t, &[MILITANT; 4]);
    let enemy = id(next_id(&t));
    let now = t.sim().tick() + 1;
    t.spawn_unit_at(now, MILITANT, P2, DOOR - Vec2::new(64.0, 0.0));
    run_for(&mut t, 2);
    let attack = |target| Command::Order {
        units: vec![enemy],
        kind: OrderKind::Attack(target),
        queue: false,
    };
    let validate_p2 = |t: &TestSim, command| t.sim().validate(&PlayerCommand { player: P2, command });
    for &unit in &pool {
        assert!(validate_p2(&t, attack(unit)).is_err(), "{unit:?}");
        assert_ne!(
            t.sim().default_order(enemy, Target::Object(unit)),
            Some(OrderKind::Attack(unit))
        );
    }
    assert_eq!(validate_p2(&t, attack(G1)), Ok(()));
}

/// "take no damage while the Garrison stands", area damage included (plan §N2's acceptance: "Occupants can't be
/// targeted, even by area damage over the Garrison"): player 2's Grenadier (`factions/syndicate/units/grenadier.md`, a
/// blast of 0.625 grid units) throws its grenades at the Garrison, with four Militants inside (auto-targeting off, so
/// that they don't shoot it first): every blast lands on the footprint and hits the Garrison, and nothing inside.
#[test]
fn a_blast_over_the_garrison_hurts_no_one_inside() {
    let mut t = base().start();
    t.without_auto_targeting();
    let pool = fill(&mut t, &[MILITANT; 4]);
    let grenadier = id(next_id(&t));
    let now = t.sim().tick() + 1;
    t.spawn_unit_at(
        now,
        ObjectKind::Grenadier,
        P2,
        Vec2::new(HI.x + 3.0 * GU + 18.0, DOOR.y),
    );
    t.order_at(now + 1, P2, &[grenadier], OrderKind::Attack(G1), false);
    let blasts = |t: &TestSim| {
        (t.events().iter())
            .filter_map(|(_, e)| match *e {
                SimEvent::Blast { at, .. } => Some(at),
                _ => None,
            })
            .collect::<Vec<Vec2>>()
    };
    while blasts(&t).len() < 3 {
        assert!(t.sim().tick() < now + 400, "throwing\n{}", t.ascii(Some(DOOR)));
        step_checked(&mut t);
    }
    for at in blasts(&t) {
        assert!(at.cmpge(LO).all() && at.cmple(HI).all(), "on the footprint: {at}");
    }
    let struck: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::Hit { target, .. } => Some(target),
            _ => None,
        })
        .collect();
    assert!(struck.contains(&G1), "the Garrison is hit");
    assert!(struck.iter().all(|target| !pool.contains(target)), "{struck:?}");
    for &unit in &pool {
        assert_eq!(hp_inside(&t, unit), 40.0, "{unit:?} unhurt");
    }
    let garrison = t.sim().object(G1).expect("standing");
    assert!(garrison.hp.unwrap() < 400.0);
}

/// "when the Garrison is destroyed, everyone inside comes out alive inside its former footprint" (F3's evacuation, which
/// the Garrison shares: `Sim::evacuate`): a Recruit, a Militant, a Gunner and a Disciple, that were firing from inside
/// (at a target 3 grid units off it), come out at once, at full HP, idle, each with its attack as its type has it,
/// cleared (plan decision T72: "leaving clears it"): no target, no sequence; the Disciple, which fired the Militant's
/// from inside, with none at all. A Storage keeps player 1 in the game.
#[test]
fn destroying_the_garrison_evacuates_them_alive() {
    let mut t = base().spawn(ObjectKind::Storage, Some(P1), tile(2, 2)).start(); // 2
    let pool = fill(&mut t, &[RECRUIT, MILITANT, GUNNER, DISCIPLE]);
    let now = t.sim().tick() + 1;
    targets(&mut t, now, &[Vec2::new(HI.x + 3.0 * GU + HALF.x, DOOR.y)]);
    run_for(&mut t, 12);
    let firing = (t.events().iter()).filter(|(tick, e)| *tick > now && matches!(e, SimEvent::Attacked { .. }));
    assert!(firing.count() > 0, "they fire from inside");
    let gone = t.sim().tick() + 1;
    t.debug_at(gone, DebugCommand::Remove { id: G1 });
    run_checked(&mut t, gone);
    let evacuated: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::Evacuated { unit, building } if building == G1 => Some(unit),
            _ => None,
        })
        .collect();
    let mut sorted = evacuated.clone();
    sorted.sort();
    let mut expected = pool.clone();
    expected.sort();
    assert_eq!(sorted, expected, "everyone inside");
    for &unit in &pool {
        let o = t.sim().object(unit).expect("alive, on the map");
        assert!(
            o.pos.cmpge(LO + HALF).all() && o.pos.cmple(HI - HALF).all(),
            "{unit:?} inside the former footprint: {}",
            o.pos
        );
        assert_eq!(o.hp, Some(40.0), "{unit:?}");
        assert!(o.orders.as_ref().unwrap().is_idle());
        match weapon(o) {
            Some(_) => {
                let attack = o.attack.as_ref().expect("its attack");
                assert_eq!((attack.target, attack.phase), (None, None), "{unit:?}: cleared");
            }
            None => assert_eq!(o.attack, None, "{unit:?}: no attack of its own"),
        }
    }
}
