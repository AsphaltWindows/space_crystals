//! The surfaced Transport in a fight (`factions/syndicate/units/subterranean_transport.md > Above-ground mode`: it
//! "**drives out of the Tunnel in its above-ground mode**, behaving as a tracked vehicle", and "**crushes** enemy Light
//! Infantry like any tracked vehicle"; design change 0419; plan decision T94: its type's Domain, Ground, over its
//! DrillUnit base's): a ground unit that ground attacks hit and anti-air ones can't, on the ground layer, crushing as a
//! tracked vehicle does; its passengers fire nothing (design change 0432 part 1: "**Passengers don't fire out**").

use super::*;

const RECRUIT: ObjectKind = ObjectKind::Recruit;
const PEACEKEEPER: ObjectKind = ObjectKind::Peacekeeper;

/// The attacks `attacker` started so far, with their ticks and targets.
fn attacks_by(t: &TestSim, attacker: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Attacked {
                attacker: by, target, ..
            } if by == attacker => Some((*tick, target)),
            _ => None,
        })
        .collect()
}

/// Every crush so far: its tick, the crushed unit and its crusher.
fn crushes(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Crushed { id, crusher, .. } => Some((*tick, id, crusher)),
            _ => None,
        })
        .collect()
}

/// Design change 0432's Verification, part 1: "a loaded Transport next to enemies: no shots from it". Auto-targeting
/// on, four Guards aboard a Transport, and two of player 2's Peacekeepers spawned 150 su from it, holding there: in 160
/// ticks the Peacekeepers fire at the Transport, and nothing aboard fires back, nor the unarmed Transport. Unloaded,
/// the Guards engage the Peacekeepers at once.
#[test]
fn a_loaded_transport_next_to_enemies_fires_no_shots() {
    let mut scenario = field().spawn(TRANSPORT, Some(P1), point(600.0, 600.0));
    for k in 0..4 {
        scenario = scenario.spawn(GUARD, Some(P1), point(400.0, 520.0 + 60.0 * k as f32));
    }
    let mut t = scenario.start();
    t.run_until(1);
    let transport = id(1);
    let guards: Vec<ObjectId> = (2..6).map(id).collect();
    board(&mut t, &guards, transport);
    let first = next_id(&t);
    let pks = [id(first), id(first + 1)];
    let now = t.sim().tick();
    t.debug_at(now + 1, spawn(PEACEKEEPER, P2, point(750.0, 560.0)));
    t.debug_at(now + 1, spawn(PEACEKEEPER, P2, point(750.0, 640.0)));
    t.at(now + 2, P2, order(&pks, OrderKind::Hold));
    t.run_until(now + 160);
    assert!(
        pks.iter()
            .any(|&pk| attacks_by(&t, pk).iter().any(|&(_, target)| target == transport)),
        "the Peacekeepers fire at the Transport"
    );
    for &unit in guards.iter().chain([&transport]) {
        assert_eq!(attacks_by(&t, unit), [], "{unit:?} fires nothing");
    }
    assert_eq!(aboard(&t, transport).len(), CAPACITY, "all aboard still");
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[transport], OrderKind::UnloadAll));
    step_until(&mut t, 60, "an unloaded Guard fires", |t| {
        guards.iter().any(|&guard| !attacks_by(t, guard).is_empty())
    });
}

/// A ground unit (`combat::kind_domain`: its type's Domain, `UnitDef::domain`, over its DrillUnit base's Underground):
/// player 2's Peacekeeper, whose attack's TargetDomain is Ground, may attack it, and player 2's Twister, whose attack
/// is anti-air only ("TargetDomain: **Air**"), may not ([`Reject::CannotHit`]).
#[test]
fn a_peacekeeper_may_attack_it_and_a_twister_may_not() {
    let scenario = field()
        .spawn(TRANSPORT, Some(P1), point(600.0, 600.0))
        .spawn(PEACEKEEPER, Some(P2), point(800.0, 600.0))
        .spawn(ObjectKind::Twister, Some(P2), point(800.0, 700.0));
    let mut t = started(scenario);
    t.run_until(3);
    let (transport, pk, twister) = (id(1), id(2), id(3));
    let attack = OrderKind::Attack(transport);
    assert_eq!(gate(&t, P2, order(&[pk], attack)), Ok(()));
    assert_eq!(
        gate(&t, P2, order(&[twister], attack)),
        Err(Reject::CannotHit(transport))
    );
    t.at(4, P2, order(&[pk], attack));
    step_until(&mut t, 120, "the Peacekeeper hits it", |t| {
        object(t, transport).hp < Some(MAX_HP)
    });
}

/// It collides with ground units (`movement`'s ground layer, by its type's Domain): driven east at player 2's Twister
/// holding on its way, a wheeled vehicle it can't crush, it is stopped by it and goes round it, never overlapping it
/// (`TestSim`'s no-overlap invariant I1, every tick), its centre never nearer the Twister's than their half widths
/// together; it gets where it was sent, and the Twister is where it was.
#[test]
fn it_collides_with_ground_units_and_goes_round_them() {
    let scenario = field().spawn(TRANSPORT, Some(P1), point(400.0, 600.0)).spawn(
        ObjectKind::Twister,
        Some(P2),
        point(800.0, 600.0),
    );
    let mut t = started(scenario);
    let (transport, twister) = (id(1), id(2));
    t.at(2, P2, order(&[twister], OrderKind::Hold));
    let goal = Vec2::new(1200.0, 600.0);
    t.at(2, P1, order(&[transport], move_to(goal.x, goal.y)));
    let held = object(&t, twister).pos;
    // Half the Twister's 52 × 36 su Silhouette across, and half the Transport's.
    let apart = 18.0 + HALF.y;
    let mut nearest = f32::MAX;
    while t.sim().tick() < 2 || object(&t, transport).pos.distance(goal) >= 1.0 {
        t.step();
        nearest = nearest.min(object(&t, transport).pos.distance(object(&t, twister).pos));
        assert!(t.sim().tick() < 500, "it gets there");
    }
    assert!(nearest >= apart, "it kept clear of the Twister: {nearest} su");
    assert!(crushes(&t).is_empty() && deaths(&t).is_empty());
    assert_eq!(object(&t, twister).pos, held, "the Twister held its ground");
}

/// Design change 0419's Verification, above ground: "a surfaced Transport driven through enemy Recruits kills them".
/// Five of player 2's Recruits stand a tile apart on its way east, from (10, 9) to (14, 9); sent from (6, 9) to
/// (22, 9), it crushes each as it reaches it, in order, and gets there.
#[test]
fn driven_through_enemy_recruits_it_crushes_them() {
    let mut scenario = field().spawn(TRANSPORT, Some(P1), SpawnAt::Point(TilePos::new(6, 9).center()));
    for x in 10..15 {
        scenario = scenario.spawn(RECRUIT, Some(P2), SpawnAt::Point(TilePos::new(x, 9).center()));
    }
    let mut t = started(scenario);
    let goal = TilePos::new(22, 9).center();
    t.at(2, P1, order(&[id(1)], move_to(goal.x, goal.y)));
    step_until(&mut t, 400, "it gets there", |t| {
        object(t, id(1)).pos.distance(goal) < 1.0
    });
    let crushed: Vec<(ObjectId, ObjectId)> = crushes(&t).into_iter().map(|(_, unit, by)| (unit, by)).collect();
    let expected: Vec<(ObjectId, ObjectId)> = (2..7).map(|n| (id(n), id(1))).collect();
    assert_eq!(crushed, expected, "each in its turn");
    assert!((2..7).all(|n| t.sim().object(id(n)).is_none()));
}

/// Design change 0419, "No friendly crushing" (`systems/units.md > UnitCollision > Crushing`: own crushable units
/// "remain hard obstacles and crushers must path around them"): two of player 1's own Recruits holding one behind the
/// other on its way east (holding, they don't make way) stop it, and it goes round them and gets where it was sent.
/// Nobody is crushed, and the Recruits never move.
#[test]
fn its_own_infantry_on_its_way_stop_it_and_it_goes_round_them() {
    let scenario = field()
        .spawn(TRANSPORT, Some(P1), SpawnAt::Point(TilePos::new(4, 9).center()))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(TilePos::new(9, 9).center()))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(TilePos::new(10, 9).center()));
    let mut t = started(scenario);
    let recruits = [id(2), id(3)];
    t.at(2, P1, order(&recruits, OrderKind::Hold));
    let held = [object(&t, recruits[0]).pos, object(&t, recruits[1]).pos];
    let goal = TilePos::new(16, 9).center();
    t.at(2, P1, order(&[id(1)], move_to(goal.x, goal.y)));
    let mut stopped = false;
    while t.sim().tick() < 2 || object(&t, id(1)).pos.distance(goal) >= 1.0 {
        let was = object(&t, id(1)).pos;
        t.step();
        let now = object(&t, id(1)).pos;
        // Stopped by them: a tick it made no way east, with the first Recruit right ahead.
        stopped |= t.sim().tick() > 2 && now.x <= was.x && now.x < held[0].x;
        assert!(t.sim().tick() < 500, "it gets round");
    }
    assert!(stopped, "the Recruits stopped it");
    assert_eq!(crushes(&t), [], "no friendly crushing");
    assert_eq!(
        [object(&t, recruits[0]).pos, object(&t, recruits[1]).pos],
        held,
        "they held their ground"
    );
    assert!(deaths(&t).is_empty());
}

/// Design change 0418 (`systems/units.md > UnitCollision > Crushing`: "**Turning in place crushes**"): player 2's
/// Recruit against the middle of a stopped Transport's south side dies as the Transport, sent due south, spins to face
/// it, before it has moved; it then goes where it was sent.
#[test]
fn an_enemy_recruit_against_its_side_dies_as_it_turns_to_face_it() {
    let at = TilePos::new(6, 4).center();
    let scenario = field().spawn(TRANSPORT, Some(P1), SpawnAt::Point(at)).spawn(
        RECRUIT,
        Some(P2),
        SpawnAt::Point(at + Vec2::new(0.0, HALF.y + 12.0)),
    );
    let mut t = started(scenario);
    let goal = TilePos::new(6, 10).center();
    t.at(2, P1, order(&[id(1)], move_to(goal.x, goal.y)));
    step_until(&mut t, 12, "the Recruit is crushed", |t| {
        t.sim().object(id(2)).is_none()
    });
    assert_eq!(crushes(&t).len(), 1);
    assert_eq!(object(&t, id(1)).pos, at, "in place");
    step_until(&mut t, 300, "it gets there", |t| {
        object(t, id(1)).pos.distance(goal) < 1.0
    });
}
