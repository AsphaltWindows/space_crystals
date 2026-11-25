//! `systems/combat.md > ValidTarget` 3 for the Agent's pistol (`factions/syndicate/units.md > Agent > Attack`:
//! TargetDomain Ground): an Agent never shoots an air unit, even one in its reach. No air unit exists yet, so a
//! Peacekeeper on a Hover Craft base stands in (`defs::test_defs`); a Guard's laser rifle (Universal, design change
//! 0186) shoots it, which shows it is a target for a weapon that may hit air.

use glam::Vec2;

use crate::combat;
use crate::defs::test_defs::TestDef;
use crate::defs::{Domain, ObjectKind, UnitBase};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::testing::ScenarioBuilder;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectId = ObjectId(1);
const GUARD: ObjectId = ObjectId(2);
const FLYER: ObjectId = ObjectId(3);

/// Player 1's Agent and Guard on Hold, player 2's flying Peacekeeper 64 su from the Agent's edge (within its Range of
/// 2 grid units) and the Guard's: the Guard shoots it from tick 6 on (its Aim of 4 frames), every 24 ticks; the Agent never.
#[test]
fn an_agent_never_shoots_an_air_unit() {
    let _flyer = TestDef::unit(ObjectKind::Peacekeeper, |unit| unit.base = UnitBase::HoverCraft);
    let mut t = ScenarioBuilder::new(&"............\n".repeat(6)).start();
    t.spawn_unit_at(1, ObjectKind::Agent, P1, Vec2::new(100.0, 96.0))
        .spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(100.0, 160.0))
        .spawn_unit_at(
            1,
            ObjectKind::Peacekeeper,
            P2,
            Vec2::new(100.0 + 18.0 + 64.0 + 12.0, 128.0),
        )
        .order_at(2, P1, &[AGENT, GUARD], OrderKind::Hold, false)
        .order_at(2, P2, &[FLYER], OrderKind::Hold, false)
        .run_until(30);
    let sim = t.sim();
    let (agent, guard, flyer) = [AGENT, GUARD, FLYER].map(|id| sim.object(id).expect("exists")).into();
    assert_eq!(combat::object_domain(flyer), Domain::Air, "the test's flyer");
    assert!(!combat::can_attack_now(sim.map(), sim.vision(), agent, flyer));
    assert!(combat::can_attack_now(sim.map(), sim.vision(), guard, flyer));
    let shots = |attacker| -> Vec<u32> {
        let events = t.events().iter();
        let hits = events.filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == attacker));
        hits.map(|&(tick, _)| tick).collect()
    };
    assert_eq!(shots(AGENT), [] as [u32; 0]);
    assert_eq!(shots(GUARD), [6, 30]);
}
