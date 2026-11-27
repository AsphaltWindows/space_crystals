//! The state hash (`Sim::state_hash`): every field of the state changes it, and nothing else does.

use super::*;
use crate::augmentation::{Augmentation, RecruitKind};
use crate::combat::{TargetLock, TurretState};

/// A named change to one field.
type Mutation = (&'static str, fn(&mut Sim));

/// Gives unit `id` a current order, a queued one, a path, an attack being aimed, a turret turned and locked (any unit
/// may hold one here), (M5-K) a Target Lock under way (the same) and a Riot Shield on its way up, so every field of its
/// orders, mover, attack and shield has a value.
fn make_busy(sim: &mut Sim, id: u32) {
    let order = Order {
        kind: OrderKind::Move(Target::Point(Vec2::new(40.0, 50.0))),
        group: OrderGroup(3),
        slot: Some(Vec2::new(41.0, 51.0)),
    };
    let o = orders(sim, id);
    o.give(order.clone(), false);
    o.current = Some(crate::orders::ActiveOrder {
        order,
        since: 2,
        state: crate::orders::Behaviour::Plain,
    });
    let pos = obj(sim, id).pos;
    let m = mover(sim, id);
    m.go_to(crate::mover::MoveGoal::Point(Vec2::new(41.0, 51.0)), None, 2);
    m.follow(vec![pos, Vec2::new(41.0, 51.0)], Vec2::new(41.0, 51.0), 0);
    m.avoid_tiles.push(TilePos::new(1, 2));
    obj(sim, id).attack = Some(AttackState {
        target: Some(ObjectId(2)),
        phase: Some(AttackPhase::Aiming { target: ObjectId(2) }),
        elapsed: 1,
        location: None,
        turret: Some(TurretState {
            facing: 0.25,
            locked: Some(ObjectId(2)),
        }),
        target_lock: Some(TargetLock {
            target: ObjectId(2),
            elapsed: 5,
            locked: false,
        }),
    });
    obj(sim, id).riot_shield = Some(RiotShield { up: 4 });
}

fn orders(sim: &mut Sim, id: u32) -> &mut Orders {
    obj(sim, id).orders.as_mut().expect("a unit")
}

fn placement(sim: &mut Sim) -> &mut Placement {
    obj(sim, 1).placement.as_mut().expect("patch has a placement")
}

fn constructing(kind: ObjectKind, progress: u32) -> Constructor {
    Constructor::Constructing { kind, progress }
}

#[test]
fn state_hash_covers_every_field() {
    let mut base = small_sim();
    insert_unit(&mut base, Vec2::new(100.0, 100.0)); // id 3
    make_busy(&mut base, 3);
    // A plate of player 1 on the patch: id 4, mining it halfway through a cycle.
    let patch = base.objects[&ObjectId(1)].clone();
    let plate = base.insert_object(
        ObjectKind::ExtractionPlate,
        Some(PlayerId(1)),
        patch.pos,
        patch.placement,
        None,
    );
    obj(&mut base, plate.0).extractor.as_mut().unwrap().cycle = 7;
    // Construction states (any object may hold one here): the unit building a Generator, the plate holding one ready.
    obj(&mut base, 3).constructor = Some(constructing(ObjectKind::Generator, 5));
    obj(&mut base, plate.0).constructor = Some(Constructor::Ready {
        kind: ObjectKind::Generator,
    });
    // A tier (any object may hold one here, plan decision T39).
    obj(&mut base, plate.0).tunnel = Some(crate::tunnel::TunnelState::at(2));
    // Shields and a conversion under way (any unit may hold them here).
    obj(&mut base, 3).shields = Some(7.5);
    obj(&mut base, 3).conversion = Some(Conversion {
        form: ObjectKind::SabreLegionnaire,
        left: Some(2),
    });
    let mutations: Vec<Mutation> = vec![
        ("tick", |s| s.tick += 1),
        ("map", |s| {
            s.map.set_tile(TilePos::new(0, 0), Tile::new(TilePresetId::WATER))
        }),
        ("player id", |s| s.players[0].id = PlayerId(9)),
        ("player name", |s| s.players[0].name.push('!')),
        ("player faction", |s| s.players[0].faction = Faction::Cults),
        ("player crystals", |s| s.players[0].crystals += 1),
        ("player supplies", |s| s.players[0].supplies += 1),
        ("player removed", |s| {
            s.players.pop();
        }),
        ("object removed", |s| {
            s.objects.remove(&ObjectId(3));
        }),
        ("object id", |s| obj(s, 1).id = ObjectId(7)),
        ("kind", |s| obj(s, 1).kind = ObjectKind::SupplyDeliveryStation),
        ("owner", |s| obj(s, 1).owner = Some(PlayerId(2))),
        ("pos x", |s| obj(s, 3).pos.x += 1.0),
        ("pos y", |s| obj(s, 3).pos.y += 1.0),
        ("prev_pos x", |s| obj(s, 3).prev_pos.x += 1.0),
        ("prev_pos y", |s| obj(s, 3).prev_pos.y += 1.0),
        ("facing", |s| obj(s, 3).facing += 0.1),
        ("prev_facing", |s| obj(s, 3).prev_facing += 0.1),
        ("hp value", |s| obj(s, 3).hp = Some(39.5)),
        ("hp none", |s| obj(s, 3).hp = None),
        ("shields value", |s| obj(s, 3).shields = Some(7.25)),
        ("shields none", |s| obj(s, 3).shields = None),
        ("placement none", |s| obj(s, 1).placement = None),
        ("placement origin x", |s| placement(s).origin.x += 1),
        ("placement origin y", |s| placement(s).origin.y += 1),
        ("placement w", |s| placement(s).w += 1),
        ("placement h", |s| placement(s).h += 1),
        ("placement rotation", |s| placement(s).rotation = Rotation::R90),
        ("placement flip_h", |s| placement(s).flip_h = true),
        ("placement flip_v", |s| placement(s).flip_v = true),
        ("resource none", |s| obj(s, 1).resource = None),
        ("patch remaining", |s| {
            obj(s, 1).resource = Some(ResourceState::CrystalPatch { remaining: 1 });
        }),
        ("delivery_size", |s| *station(s).0 += 1),
        ("delivery_interval", |s| *station(s).1 += 1),
        ("current_supplies", |s| *station(s).2 += 1),
        ("next_delivery_in value", |s| *station(s).3 = Some(5)),
        ("next_delivery_in none", |s| *station(s).3 = None),
        ("next_id", |s| s.next_id += 1),
        ("rng", |s| {
            s.rng.next_u32();
        }),
        ("config debug", |s| s.config.debug = !s.config.debug),
        ("next_group", |s| s.next_group += 1),
        ("auto_targeting", |s| s.auto_targeting = !s.auto_targeting),
        ("nav_version", |s| s.nav_version += 1),
        ("path knowledge version", |s| s.path_knowledge.bump(PlayerId(1))),
        ("orders none", |s| obj(s, 3).orders = None),
        ("queue", |s| {
            orders(s, 3).queue.pop_front();
        }),
        ("queued kind", |s| orders(s, 3).queue[0].kind = OrderKind::Hold),
        ("queued group", |s| orders(s, 3).queue[0].group = OrderGroup(4)),
        ("queued slot", |s| orders(s, 3).queue[0].slot = None),
        ("move target", |s| {
            orders(s, 3).queue[0].kind = OrderKind::Move(Target::Object(ObjectId(1)));
        }),
        // (M5-K) A Target Lock order, and its target.
        ("target lock order", |s| {
            orders(s, 3).queue[0].kind = OrderKind::TargetLock(ObjectId(2));
        }),
        ("target lock order's target", |s| {
            orders(s, 3).queue[0].kind = OrderKind::TargetLock(ObjectId(1));
        }),
        ("current none", |s| orders(s, 3).current = None),
        ("current since", |s| orders(s, 3).current.as_mut().unwrap().since += 1),
        ("current order", |s| {
            orders(s, 3).current.as_mut().unwrap().order.kind = OrderKind::Stop;
        }),
        ("mover none", |s| obj(s, 3).mover = None),
        ("goal", |s| mover(s, 3).goal = crate::mover::MoveGoal::None),
        ("status", |s| mover(s, 3).status = crate::mover::MoveStatus::Failed),
        ("locomotion", |s| {
            mover(s, 3).locomotion = crate::mover::Locomotion::Moving
        }),
        ("orientation", |s| {
            mover(s, 3).orientation = crate::mover::Orientation::Turning(Vec2::ONE);
        }),
        ("velocity", |s| mover(s, 3).velocity.x += 1.0),
        ("speed_factor", |s| mover(s, 3).speed_factor = 0.5),
        ("path", |s| mover(s, 3).path[1].y += 1.0),
        ("next", |s| mover(s, 3).next += 1),
        ("path_state", |s| mover(s, 3).path_state = crate::mover::PathState::None),
        ("path group", |s| {
            mover(s, 3).path_state = crate::mover::PathState::Wanted {
                since: 2,
                group: Some(crate::nav::PathGroup {
                    id: 1,
                    anchor: Vec2::ZERO,
                }),
            };
        }),
        ("mover nav_version", |s| mover(s, 3).nav_version += 1),
        ("planned_target", |s| mover(s, 3).planned_target.x += 1.0),
        ("repaths_left", |s| mover(s, 3).repaths_left -= 1),
        ("last_repath", |s| mover(s, 3).last_repath += 1),
        ("avoid_tiles", |s| mover(s, 3).avoid_tiles.clear()),
        ("best_remaining", |s| mover(s, 3).best_remaining = 7.0),
        ("last_progress", |s| mover(s, 3).last_progress += 1),
        ("blocked_ticks", |s| mover(s, 3).blocked_ticks += 1),
        ("avoid_side", |s| mover(s, 3).avoid_side = crate::mover::Side::Left),
        ("avoid_until", |s| mover(s, 3).avoid_until += 1),
        ("sidestep", |s| {
            mover(s, 3).sidestep = Some(crate::mover::Sidestep::new(Vec2::X, 24.0, 4.0, 0, None));
        }),
        // (M5-H1) A Glider's flight (`mover::flight`).
        ("orbit", |s| mover(s, 3).orbit = Some(Vec2::new(64.0, 64.0))),
        ("strafe", |s| mover(s, 3).strafe(192.0)),
        ("strafe pass", |s| {
            mover(s, 3).strafe = Some(crate::mover::Strafe {
                range: 192.0,
                pass: crate::mover::Pass::On,
            });
        }),
        ("attack none", |s| obj(s, 3).attack = None),
        ("attack target", |s| attack(s, 3).target = Some(ObjectId(1))),
        ("attack target none", |s| attack(s, 3).target = None),
        ("attack phase none", |s| attack(s, 3).phase = None),
        ("attack phase", |s| {
            attack(s, 3).phase = Some(AttackPhase::Firing {
                target: Some(ObjectId(2)),
            });
        }),
        ("aimed target", |s| {
            attack(s, 3).phase = Some(AttackPhase::Aiming { target: ObjectId(1) });
        }),
        ("fired target", |s| {
            attack(s, 3).phase = Some(AttackPhase::Firing { target: None });
        }),
        ("cooldown", |s| attack(s, 3).phase = Some(AttackPhase::Cooldown)),
        ("reloading", |s| attack(s, 3).phase = Some(AttackPhase::Reloading)),
        ("attack elapsed", |s| attack(s, 3).elapsed += 1),
        ("attack location", |s| {
            attack(s, 3).location = Some(Vec2::new(70.0, 80.0))
        }),
        ("aiming at", |s| {
            attack(s, 3).phase = Some(AttackPhase::AimingAt {
                at: Vec2::new(70.0, 80.0),
            });
        }),
        ("firing at", |s| {
            attack(s, 3).phase = Some(AttackPhase::FiringAt {
                at: Vec2::new(70.0, 80.0),
                target: None,
            });
        }),
        ("firing at a target", |s| {
            attack(s, 3).phase = Some(AttackPhase::FiringAt {
                at: Vec2::new(70.0, 80.0),
                target: Some(ObjectId(2)),
            });
        }),
        // (M5-U1) A unit's turret (`combat::turret`), each of its fields.
        ("turret none", |s| attack(s, 3).turret = None),
        ("turret facing", |s| attack(s, 3).turret.as_mut().unwrap().facing = -0.5),
        ("turret locked", |s| {
            attack(s, 3).turret.as_mut().unwrap().locked = Some(ObjectId(1))
        }),
        ("turret locked none", |s| {
            attack(s, 3).turret.as_mut().unwrap().locked = None
        }),
        // (M5-K) A Target Lock (`combat::target_lock`), each of its fields.
        ("target lock none", |s| attack(s, 3).target_lock = None),
        ("target lock target", |s| {
            attack(s, 3).target_lock.as_mut().unwrap().target = ObjectId(1)
        }),
        ("target lock elapsed", |s| {
            attack(s, 3).target_lock.as_mut().unwrap().elapsed += 1
        }),
        ("target lock locked", |s| {
            attack(s, 3).target_lock.as_mut().unwrap().locked = true
        }),
        ("a projectile", |s| s.projectiles.push(projectile())),
        ("riot_shield none", |s| obj(s, 3).riot_shield = None),
        ("riot_shield down", |s| {
            obj(s, 3).riot_shield = Some(RiotShield { up: 0 });
        }),
        ("riot_shield up", |s| {
            obj(s, 3).riot_shield = Some(RiotShield { up: 3 });
        }),
        ("riot_shield all the way up", |s| {
            obj(s, 3).riot_shield = Some(RiotShield { up: 6 })
        }),
        ("conversion none", |s| obj(s, 3).conversion = None),
        ("conversion left", |s| {
            obj(s, 3).conversion.as_mut().unwrap().left = Some(1);
        }),
        ("conversion waiting", |s| {
            obj(s, 3).conversion.as_mut().unwrap().left = None;
        }),
        ("conversion form", |s| {
            obj(s, 3).conversion.as_mut().unwrap().form = ObjectKind::RifleLegionnaire;
        }),
        ("extractor none", |s| obj(s, 4).extractor = None),
        ("extractor patch", |s| {
            obj(s, 4).extractor.as_mut().unwrap().patch = ObjectId(2);
        }),
        ("extractor cycle", |s| obj(s, 4).extractor.as_mut().unwrap().cycle += 1),
        ("constructor none", |s| obj(s, 3).constructor = None),
        ("constructor idle", |s| obj(s, 3).constructor = Some(Constructor::Idle)),
        ("constructing kind", |s| {
            obj(s, 3).constructor = Some(constructing(ObjectKind::Barracks, 5));
        }),
        ("constructing progress", |s| {
            obj(s, 3).constructor = Some(constructing(ObjectKind::Generator, 6));
        }),
        ("constructor ready", |s| {
            obj(s, 3).constructor = Some(Constructor::Ready {
                kind: ObjectKind::Generator,
            });
        }),
        ("ready kind", |s| {
            obj(s, 4).constructor = Some(Constructor::Ready {
                kind: ObjectKind::Barracks,
            });
        }),
        ("tunnel none", |s| obj(s, 4).tunnel = None),
        ("tunnel some", |s| {
            obj(s, 3).tunnel = Some(crate::tunnel::TunnelState::at(2));
        }),
        ("tunnel tier", |s| obj(s, 4).tunnel.as_mut().unwrap().tier = 3),
        // (M5-A) A Cults unit's lineage (plan decision T56; any unit may hold one here), one entry per Recruit (M5-C).
        ("lineage", |s| obj(s, 3).lineage = vec![ObjectId(4)]),
        ("lineage entries", |s| {
            obj(s, 3).lineage = vec![ObjectId(4), ObjectId(4)]
        }),
        // (M5-F2) A Cults infantry unit's Augmentations (any unit may hold them here), each of the two.
        ("augmentation constitution", |s| {
            obj(s, 3).augmentation = Augmentation::of(RecruitKind::Constitution)
        }),
        ("augmentation metabolism", |s| {
            obj(s, 3).augmentation = Augmentation::of(RecruitKind::Metabolism)
        }),
    ];
    for (name, mutate) in mutations {
        let mut sim = base.clone();
        mutate(&mut sim);
        assert_ne!(sim, base, "{name}: the mutation changed nothing");
        assert_ne!(sim.state_hash(), base.state_hash(), "{name}");
    }
}

/// (M5-A) A unit's lineage (`Object::lineage`, plan decision T56; one entry per Recruit it embodies, the Director's
/// amendment for design change 0319, slice M5-C) is part of the state while it has an entry: hashed, by the Halls it
/// names, how many and in their order, and saved. While it has none, nothing of it is hashed, and an object's file has
/// no `lineage` key and loads with none, so that the states and the files without one are as they were before it
/// existed (the goldens stand).
#[test]
fn a_lineage_is_hashed_and_saved_only_while_it_is_set() {
    let mut base = small_sim();
    insert_unit(&mut base, Vec2::new(100.0, 100.0)); // id 3
    let with = |halls: &[u32]| {
        let mut sim = base.clone();
        obj(&mut sim, 3).lineage = halls.iter().copied().map(ObjectId).collect();
        sim
    };
    assert_eq!(with(&[]).state_hash(), base.state_hash());
    assert_ne!(with(&[1]).state_hash(), base.state_hash());
    assert_ne!(with(&[1]).state_hash(), with(&[2]).state_hash(), "by its Hall");
    // A Seeker's two Recruits, from one Hall or two (design change 0319).
    assert_ne!(with(&[1]).state_hash(), with(&[1, 1]).state_hash(), "by its entries");
    assert_ne!(with(&[1, 2]).state_hash(), with(&[2, 1]).state_hash(), "in their order");
    for halls in [&[][..], &[1], &[1, 2]] {
        let sim = with(halls);
        let object = &sim.objects[&ObjectId(3)];
        let json = serde_json::to_value(object).unwrap();
        let key = (!halls.is_empty()).then(|| serde_json::json!(halls));
        assert_eq!(json.get("lineage"), key.as_ref(), "{halls:?}");
        let loaded: crate::object::Object = serde_json::from_value(json).unwrap();
        assert_eq!(&loaded, object, "{halls:?}");
    }
}

/// (M5-F2) A unit's Augmentations (`Object::augmentation`, `crate::augmentation`) are part of the state while it has
/// one: hashed, which of the two, and saved. While it has neither, nothing of them is hashed, and an object's file has
/// no `augmentation` key and loads with neither, so that the states and the files without one are as they were before
/// they existed (the goldens stand).
#[test]
fn an_augmentation_is_hashed_and_saved_only_while_it_is_set() {
    let mut base = small_sim();
    insert_unit(&mut base, Vec2::new(100.0, 100.0)); // id 3
    let with = |kind: RecruitKind| {
        let mut sim = base.clone();
        obj(&mut sim, 3).augmentation = Augmentation::of(kind);
        sim
    };
    assert_eq!(with(RecruitKind::Plain).state_hash(), base.state_hash());
    let hashes: std::collections::BTreeSet<u64> =
        RecruitKind::ALL.iter().map(|&kind| with(kind).state_hash()).collect();
    assert_eq!(hashes.len(), 4, "each kind its own");
    for kind in RecruitKind::ALL {
        let sim = with(kind);
        let object = &sim.objects[&ObjectId(3)];
        let json = serde_json::to_value(object).unwrap();
        assert_eq!(
            json.get("augmentation").is_some(),
            kind != RecruitKind::Plain,
            "{kind:?}"
        );
        let loaded: crate::object::Object = serde_json::from_value(json).unwrap();
        assert_eq!(&loaded, object, "{kind:?}");
    }
}

/// (M5-U1) A unit's turret (`AttackState::turret`, `combat::turret`) is part of the state from the first time it locks
/// a target or turns: hashed, its facing and its LockedTarget, and saved. While it has none, nothing of it is hashed,
/// and an attack's file has no `turret` key and loads with none, so that the states and the files without one are as
/// they were before turrets existed (the goldens stand).
#[test]
fn a_turret_is_hashed_and_saved_only_while_it_is_set() {
    let mut base = small_sim();
    insert_unit(&mut base, Vec2::new(100.0, 100.0)); // id 3
    let with = |turret: Option<TurretState>| {
        let mut sim = base.clone();
        attack(&mut sim, 3).turret = turret;
        sim
    };
    assert_eq!(with(None).state_hash(), base.state_hash());
    let turrets = [
        None,
        Some(TurretState::default()),
        Some(TurretState {
            facing: 0.5,
            locked: None,
        }),
        Some(TurretState {
            facing: 0.0,
            locked: Some(ObjectId(1)),
        }),
    ];
    let hashes: std::collections::BTreeSet<u64> = turrets.iter().map(|&turret| with(turret).state_hash()).collect();
    assert_eq!(hashes.len(), turrets.len(), "each its own");
    for turret in turrets {
        let sim = with(turret);
        let attack = sim.objects[&ObjectId(3)].attack.as_ref().unwrap();
        let json = serde_json::to_value(attack).unwrap();
        assert_eq!(json.get("turret").is_some(), turret.is_some(), "{turret:?}");
        let loaded: AttackState = serde_json::from_value(json).unwrap();
        assert_eq!(&loaded, attack, "{turret:?}");
    }
}

/// (M5-K) A Target Lock (`AttackState::target_lock`, `combat::target_lock`) is part of the state while one is under
/// way: hashed, its target, its locking time and whether it is locked, and saved. While there is none, nothing of it is
/// hashed, and an attack's file has no `target_lock` key and loads with none, so that the states and the files without
/// one are as they were before it existed (the goldens stand).
#[test]
fn a_target_lock_is_hashed_and_saved_only_while_one_is_under_way() {
    let mut base = small_sim();
    insert_unit(&mut base, Vec2::new(100.0, 100.0)); // id 3
    let with = |lock: Option<TargetLock>| {
        let mut sim = base.clone();
        attack(&mut sim, 3).target_lock = lock;
        sim
    };
    assert_eq!(with(None).state_hash(), base.state_hash());
    let locks = [
        None,
        Some(TargetLock::new(ObjectId(1))),
        Some(TargetLock {
            target: ObjectId(1),
            elapsed: 48,
            locked: false,
        }),
        Some(TargetLock {
            target: ObjectId(1),
            elapsed: 48,
            locked: true,
        }),
    ];
    let hashes: std::collections::BTreeSet<u64> = locks.iter().map(|&lock| with(lock).state_hash()).collect();
    assert_eq!(hashes.len(), locks.len(), "each its own");
    for lock in locks {
        let sim = with(lock);
        let attack = sim.objects[&ObjectId(3)].attack.as_ref().unwrap();
        let json = serde_json::to_value(attack).unwrap();
        assert_eq!(json.get("target_lock").is_some(), lock.is_some(), "{lock:?}");
        let loaded: AttackState = serde_json::from_value(json).unwrap();
        assert_eq!(&loaded, attack, "{lock:?}");
    }
}

/// (M4-O) A projectile in flight with every field set (`combat::projectiles`).
fn projectile() -> Projectile {
    Projectile {
        shooter: ObjectId(3),
        owner: Some(PlayerId(1)),
        target: Some(ObjectId(2)),
        from: Vec2::new(100.0, 100.0),
        to: Vec2::new(200.0, 120.0),
        lands_at: 30,
        damage: 100.0,
        radius: 40.0,
        domain: crate::defs::TargetDomain::Ground,
        homing: None,
    }
}

/// (M4-O) Every field of a projectile in flight changes the hash, and so does the order of the store; an empty store
/// hashes as a sim without one (`state_hash_covers_every_field`, "a projectile"). (M5-K) So does every field of a
/// homing one's flight, and the flight itself.
#[test]
fn state_hash_covers_every_projectile_field() {
    let mut base = small_sim();
    base.projectiles = vec![
        projectile(),
        Projectile {
            lands_at: 31,
            homing: Some(crate::combat::projectiles::Homing {
                pos: Vec2::new(130.0, 106.0),
                prev: Vec2::new(115.0, 103.0),
                speed: 16.0,
            }),
            ..projectile()
        },
    ];
    let mutations: Vec<Mutation> = vec![
        ("shooter", |s| s.projectiles[0].shooter = ObjectId(4)),
        ("owner", |s| s.projectiles[0].owner = None),
        ("target", |s| s.projectiles[0].target = None),
        ("from", |s| s.projectiles[0].from.x += 1.0),
        ("to", |s| s.projectiles[0].to.y += 1.0),
        ("lands_at", |s| s.projectiles[0].lands_at += 1),
        ("damage", |s| s.projectiles[0].damage += 1.0),
        ("radius", |s| s.projectiles[0].radius += 1.0),
        ("domain", |s| {
            s.projectiles[0].domain = crate::defs::TargetDomain::Universal
        }),
        ("homing", |s| s.projectiles[0].homing = s.projectiles[1].homing),
        ("homing pos", |s| s.projectiles[1].homing.as_mut().unwrap().pos.x += 1.0),
        ("homing prev", |s| {
            s.projectiles[1].homing.as_mut().unwrap().prev.y += 1.0
        }),
        ("homing speed", |s| {
            s.projectiles[1].homing.as_mut().unwrap().speed += 1.0
        }),
        ("order", |s| s.projectiles.swap(0, 1)),
        ("one fewer", |s| s.projectiles.truncate(1)),
    ];
    for (name, mutate) in mutations {
        let mut sim = base.clone();
        mutate(&mut sim);
        assert_ne!(sim, base, "{name}: the mutation changed nothing");
        assert_ne!(sim.state_hash(), base.state_hash(), "{name}");
    }
    base.projectiles.clear();
    assert_eq!(base.state_hash(), small_sim().state_hash());
}

#[test]
fn state_hash_ignores_events_only() {
    let base = small_sim();
    let mut with_event = base.clone();
    with_event.events.push(SimEvent::Removed(ObjectId(1)));
    assert_eq!(with_event.state_hash(), base.state_hash());
    assert_eq!(small_sim().state_hash(), base.state_hash());
}
