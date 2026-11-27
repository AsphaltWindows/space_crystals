//! (M5-S2b) The threat its units inside make of a building (design change 0377, answering Q0139;
//! `systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` 1, which BaseAutoTargeting
//! shares; `factions/cults/structures/garrison.md > Garrisoning`: "**Seen as a threat**"): what a Garrison threatens by
//! who is inside, and 0377's Verification through the whole sim, the units' scan. A structure's own pick is
//! `combat/structures/tests.rs`'s.

use std::collections::BTreeMap;

use glam::Vec2;

use super::{P1, objects};
use crate::combat::Threats;
use crate::command::{Command, DebugCommand, HoldCommand};
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, TargetDomain};
use crate::ids::{ObjectId, PlayerId};
use crate::network::{Held, Place};
use crate::orders::OrderKind;
use crate::scenario::SpawnAt;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::{TilePos, gu};

const P2: PlayerId = PlayerId(2);

/// Design change 0377: "In target scans, a Garrison is a **threatening** target while any occupant's weapon, as fired
/// from inside, could attack the scanner's domain. This includes the Militant stand-in for Recruits [...]. An empty
/// Garrison, or one with no weapon that can reach the scanner's domain, is not a threat." What it threatens: the
/// Militant's TargetDomain, Universal, with a Militant inside, and with a Recruit, who fires the Militant's attack;
/// (M6-CM) nothing with a Disciple, who fires nothing (design change 0450: "a **Disciple or illusion inside doesn't
/// attack**"), alone or with another Disciple, and what the others threaten with others; Ground with a Gunner alone (no
/// threat to an air scanner); Universal with a Gunner and a Recruit; nothing while it is empty. A unit held anywhere
/// else, in the Tunnel Network, adds nothing, nor does one in a building that isn't in the game.
#[test]
fn a_garrison_threatens_what_its_units_inside_could_attack() {
    let t = objects();
    let object = |id: u32| t.sim().object(ObjectId(id)).expect("spawned").clone();
    let garrison = ObjectId(1);
    let (recruit, militant, gunner, disciple) = (2, 3, 4, 5);
    let buildings = BTreeMap::from([(garrison, object(1))]);
    let held = |ids: &[u32], place: Place| -> BTreeMap<ObjectId, Held> {
        (ids.iter())
            .map(|&id| {
                let held = Held {
                    object: object(id),
                    place,
                    since: 1,
                    entered: true,
                    leaving: None,
                    service: Default::default(),
                };
                (ObjectId(id), held)
            })
            .collect()
    };
    let threat = |ids: &[u32]| Threats::of(&buildings, &held(ids, Place::Building(garrison))).get(garrison);
    use TargetDomain::{Ground, Universal};
    assert_eq!(threat(&[]), None, "empty");
    assert_eq!(threat(&[militant]), Some(Universal), "a Militant");
    assert_eq!(threat(&[recruit]), Some(Universal), "a Recruit, the stand-in's attack");
    assert_eq!(threat(&[disciple]), None, "a Disciple, who fires nothing (0450)");
    assert_eq!(threat(&[gunner, disciple]), Some(Ground), "a Gunner and a Disciple");
    assert_eq!(threat(&[gunner]), Some(Ground), "a Gunner alone");
    assert_eq!(threat(&[gunner, recruit]), Some(Universal));
    assert_eq!(threat(&[recruit, militant, gunner, disciple]), Some(Universal), "full");
    let networked = Threats::of(&buildings, &held(&[militant], Place::Network));
    assert_eq!(networked.get(garrison), None, "in the network");
    let gone = Threats::of(&BTreeMap::new(), &held(&[militant], Place::Building(garrison)));
    assert_eq!(gone.get(garrison), None, "no such building");
}

/// Design change 0377's Verification: "An enemy Peacekeeper sees an armed enemy unit that can't hit it, and a Garrison
/// with a Militant inside. It targets the Garrison first. Empty the Garrison: it is no longer preferred." Player 1's
/// Garrison, with a Militant inside, and player 1's Guard, given a weapon that targets the air only
/// (`defs::test_defs`: no type of the game has one), stand in a row west of player 2's idle Peacekeeper, the Guard
/// nearer, both within its Range: it engages the Garrison, which threatens it, though the Guard is nearer in the same
/// rotation step; a Stop makes it look again, and it takes the Garrison again. Eject All empties the Garrison, the
/// Militant is taken off the map once out (an enemy that threatens it, which would be first), and after another Stop
/// it engages the Guard: the Garrison is no longer preferred.
#[test]
fn an_enemy_targets_a_manned_garrison_first_and_no_longer_once_it_is_empty() {
    let _air_only = TestDef::unit(ObjectKind::Guard, |unit| {
        unit.attack.as_mut().expect("armed").target_domain = TargetDomain::Air;
    });
    let (garrison, militant, guard, peacekeeper) = (ObjectId(1), ObjectId(2), ObjectId(3), ObjectId(4));
    // The Garrison covers x 384..512 and y 320..448, unturned: its door, side B, faces west, its middle at (384, 384).
    let mut t = ScenarioBuilder::new(&"..............................\n".repeat(14))
        .spawn(ObjectKind::Garrison, Some(P1), SpawnAt::Tile(TilePos::new(6, 5)))
        .start();
    let row = 384.0;
    // The Guard (36 × 36) 1 grid unit east of the Garrison; the Peacekeeper (24 × 24) 116 su east of the Guard and 216
    // su of the Garrison: both within its SightRange of 5 grid units and its Range of 4.
    t.spawn_unit_at(1, ObjectKind::Militant, P1, Vec2::new(320.0, row))
        .spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(512.0 + gu(1.0) + 18.0, row))
        .order_at(2, P1, &[militant], OrderKind::Enter(garrison), false);
    while t.sim().held_in(garrison).is_empty() {
        assert!(t.sim().tick() < 200, "the Militant walks in");
        t.step();
    }
    let spawned = t.sim().tick() + 1;
    t.spawn_unit_at(spawned, ObjectKind::Peacekeeper, P2, Vec2::new(740.0, row));
    let engaged = |t: &TestSim| t.sim().object(peacekeeper).and_then(|o| o.attack.as_ref()?.target);
    while engaged(&t).is_none() {
        assert!(t.sim().tick() < spawned + 20, "it looks round");
        t.step();
    }
    assert_eq!(t.sim().object(guard).map(|o| o.kind), Some(ObjectKind::Guard));
    assert_eq!(engaged(&t), Some(garrison), "the Garrison first");
    let stop = |t: &mut TestSim| {
        let now = t.sim().tick() + 1;
        t.order_at(now, P2, &[peacekeeper], OrderKind::Stop, false)
            .run_until(now);
    };
    stop(&mut t);
    assert_eq!(engaged(&t), Some(garrison), "looking again, still the Garrison");
    let eject = Command::Holding {
        objects: vec![garrison],
        command: HoldCommand::EjectAll,
    };
    let ejected = t.sim().tick() + 1;
    t.at(ejected, P1, eject);
    while t.sim().object(militant).is_none() {
        assert!(t.sim().tick() < ejected + 30, "the Militant comes out");
        t.step();
    }
    assert_eq!(t.sim().held_in(garrison).len(), 0, "empty");
    let gone = t.sim().tick() + 1;
    t.debug_at(gone, DebugCommand::Remove { id: militant }).run_until(gone);
    assert_eq!(engaged(&t), Some(garrison), "it keeps what it engages");
    stop(&mut t);
    assert_eq!(engaged(&t), Some(guard), "the Garrison is no longer preferred");
}
