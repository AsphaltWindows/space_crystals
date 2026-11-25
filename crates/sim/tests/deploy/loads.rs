//! A Drone that deploys with a load (design change 0114, `factions/colonists/economy.md > Deploying with a load`): the
//! load is lost at completion, and comes back with the Drone on a cancel.

use super::*;

fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).unwrap().crystals
}

/// [`base`] with a Space Crystals patch 4 at (6, 13), which Drone 2 is sent to gather in tick 1, run until it holds
/// its first load, 8 (`factions/colonists/units/drone.md > Gathering`: CarryCapacity 8), on its way to the Egress
/// Node. Then it is ordered in the next tick to deploy at [`SITE`], replacing its Gather: "A Drone carrying a load
/// deploys without delivering it first" (`factions/colonists/economy.md > Deploying with a load`). Run until it began;
/// returns that tick.
fn deployed_with_a_load(t: &mut TestSim) -> Tick {
    t.order_at(1, P1, &[id(2)], OrderKind::Gather(id(4)), false);
    while object(t, 2).worker.is_some_and(|worker| worker.load == 0) {
        assert!(t.sim().tick() < 400, "it mines");
        t.step();
    }
    assert_eq!(object(t, 2).worker.unwrap().load, 8);
    next(t, order(&[2], deploy_at(SITE), false));
    walk(t, 2);
    t.sim().tick()
}

/// Design change 0114's Verification ("Deploy a carrying Drone: the bank doesn't change, the load is gone at
/// completion"): the load stays with the deploying object (`Transform::load`) and is lost when the deployment
/// completes; nothing was delivered, and nothing is.
#[test]
fn a_load_deployed_with_is_lost_at_completion() {
    let mut t = base().spawn(CrystalPatch, None, tile(6, 13)).start();
    let start = deployed_with_a_load(&mut t);
    let crystals = bank(&t, P1);
    assert_eq!(
        object(&t, 2).transform.as_ref().unwrap().load,
        8,
        "kept while it deploys"
    );
    t.run_until(start + 47);
    assert_eq!(completions(&t), [(start + 47, 2, Beacon)]);
    assert!(
        object(&t, 2).transform.is_none() && object(&t, 2).worker.is_none(),
        "the load is gone"
    );
    t.run(100);
    assert_eq!(bank(&t, P1), crystals, "never delivered");
    let delivered = ticks_of(&t, |e| matches!(e, SimEvent::ResourcesDelivered { .. }));
    assert_eq!(delivered, [] as [Tick; 0]);
}

/// Design change 0114 ("on cancel the Drone returns still carrying it"): the Drone cancelled back carries its 8, and
/// delivers them when sent to Egress Node 1 (a Drop Off order): the bank rises by 8.
#[test]
fn a_load_comes_back_with_the_drone_and_can_be_delivered() {
    let mut t = base().spawn(CrystalPatch, None, tile(6, 13)).start();
    let start = deployed_with_a_load(&mut t);
    let crystals = bank(&t, P1);
    t.run_until(start + 10);
    next(&mut t, cancel(&[2]));
    let o = object(&t, 2);
    assert_eq!((o.kind, o.worker.map(|worker| worker.load)), (Drone, Some(8)));
    next(&mut t, order(&[2], OrderKind::DropOff(id(1)), false));
    while bank(&t, P1) == crystals {
        assert!(t.sim().tick() < start + 400, "it delivers");
        t.step();
    }
    assert_eq!(bank(&t, P1), crystals + 8);
    assert!(t.events().iter().any(|(_, e)| *e
        == SimEvent::ResourcesDelivered {
            worker: id(2),
            drop_off: id(1),
            amount: 8,
            resource: sc_sim::defs::Resource::Crystals,
        }));
    assert_eq!(object(&t, 2).worker.unwrap().load, 0);
}
