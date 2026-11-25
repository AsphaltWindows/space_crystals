//! Deploy in sight (`factions/colonists/economy.md > Beacon Deployment > Deploy Command`): the Drone walks to
//! the centre of its site and begins deploying on arrival, as a Beacon whose timing, HP and Beacon Capacity
//! follow the design; and a Deploy queued behind a Move runs after it.

use super::*;

/// The `KindChanged` events so far: tick, object, from, to.
fn kind_changes(t: &TestSim) -> Vec<(Tick, u32, ObjectKind, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::KindChanged { id, from, to } => Some((*tick, id.0, *from, *to)),
            _ => None,
        })
        .collect()
}

/// The plan's acceptance, "Deploy in sight", at Beacon Capacity 10/10:
/// - `> Deploy Command`: "The Drone walks to the location, then begins deploying in place"; design change 0117: "the
///   Drone walks to the centre of the Beacon's footprint and starts deploying when it gets there". Drone 2 stood on
///   the centre of its site the tick before it began.
/// - Design change 0109: "The moment deployment begins, the Drone becomes a Beacon under deployment", "the same
///   object", "a structure on its 2x2 footprint": the object is the Beacon on the site, transforming from the Drone
///   (no footprint to go back to, nothing paid), with neither orders nor a move nor a worker.
/// - Design change 0112: "The Drone's 1 Beacon Capacity is freed as soon as deployment starts", "The Beacon's 5 is
///   provided only on completion": 9/10 at once, 9/15 at completion.
/// - `> Deployment Duration` (48 frames) and `ARCHITECTURE.md > Economy > Recipe timing`: the first frame is the
///   tick it began, and it is complete at the end of that tick + 47.
/// - `> HP/Shields Interpolation`: "HP = Drone MaxHP + (Beacon MaxHP - Drone MaxHP) x progress", 20 to 120, and an
///   undamaged one at its maximum; its maximum Shields 10 throughout ("Shields stay at 10", 10 to 10).
#[test]
fn a_drone_walks_to_the_centre_of_its_site_and_deploys_on_arrival() {
    let mut t = at_capacity().start();
    assert_eq!(capacity(&t, P1), (10, 10));
    t.step();
    next(&mut t, order(&[2], deploy_at(SITE), false));
    assert_eq!(object(&t, 2).kind, Drone, "it walks there first");
    let centre = deploy::site(SITE).center();
    let stood = walk(&mut t, 2);
    let start = t.sim().tick();
    assert_eq!(stood, centre, "it got to the centre of its site");
    assert_eq!(kind_changes(&t), [(start, 2, Drone, Beacon)]);
    let o = object(&t, 2);
    assert_eq!(
        (o.owner, o.placement, o.pos),
        (Some(P1), Some(deploy::site(SITE)), centre)
    );
    assert!(o.orders.is_none() && o.mover.is_none() && o.worker.is_none());
    assert!(o.is_transforming() && o.is_deploying());
    let deploying = o.transform.as_ref().unwrap();
    assert_eq!(
        (
            deploying.from,
            deploying.from_placement,
            deploying.progress.total,
            deploying.paid,
            deploying.load
        ),
        (Drone, None, DEPLOYMENT_FRAMES, 0, 0)
    );
    assert_eq!(DEPLOYMENT_FRAMES, 48);
    assert_eq!(capacity(&t, P1), (9, 10), "the Drone's 1 freed, the Beacon's 5 not yet");
    assert_eq!(
        pending_deploys(t.sim(), P1),
        [],
        "the site is built on, no longer pending"
    );
    for frames in 1..DEPLOYMENT_FRAMES {
        t.run_until(start + frames - 1);
        let o = object(&t, 2);
        assert_eq!(o.transform.as_ref().unwrap().frames_done(), frames);
        let max = 20.0 + 100.0 * frames as f32 / 48.0;
        assert_eq!(
            (o.max_hp(), o.hp, o.max_shields()),
            (Some(max), Some(max), Some(10.0)),
            "{frames} frames"
        );
        assert_eq!(capacity(&t, P1), (9, 10));
    }
    assert_eq!(completions(&t), []);
    t.step();
    assert_eq!(completions(&t), [(start + 47, 2, Beacon)]);
    let o = object(&t, 2);
    assert!(!o.is_transforming() && !o.is_deploying());
    assert_eq!((o.kind, o.hp, o.max_hp()), (Beacon, Some(120.0), Some(120.0)));
    assert_eq!(capacity(&t, P1), (9, 15));
    assert_eq!(kind_changes(&t).len(), 1, "one object throughout");
}

/// The plan's acceptance, "Shift" (`systems/controls/commands.md > Unit Command`: "the new command is appended to the
/// end of the queue"): a Deploy queued behind a Move runs after it. Drone 2 is sent to (6, 15), and the Deploy is
/// queued in the same tick: it is pending from then on (design change 0125), the Drone stands on (6, 15) before it
/// sets off for its site, and it deploys there.
#[test]
fn a_deploy_queued_behind_a_move_runs_after_it() {
    let mut t = base().start();
    let first = at(6, 15).center();
    t.order_at(1, P1, &[id(2)], OrderKind::Move(Target::Point(first)), false)
        .order_at(1, P1, &[id(2)], deploy_at(SITE), true)
        .step();
    assert_eq!(pending_deploys(t.sim(), P1), [(id(2), deploy::site(SITE))]);
    let current = |t: &TestSim| {
        object(t, 2)
            .orders
            .as_ref()
            .and_then(|orders| orders.current.as_ref())
            .map(|active| active.order.kind)
    };
    let mut stood_there = false;
    while current(&t) != Some(deploy_at(SITE)) {
        assert!(t.sim().tick() < 300, "the Move ends");
        stood_there |= t.pos(2) == first;
        t.step();
    }
    assert!(stood_there, "the Move first");
    walk(&mut t, 2);
    assert_eq!(object(&t, 2).placement, Some(deploy::site(SITE)));
}
