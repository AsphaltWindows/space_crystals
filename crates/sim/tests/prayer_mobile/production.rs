//! Its production (`factions/cults/units/prayer_mobile.md > Placeholder stats`: "Cost: Chop Shop, 2 Recruits, **125
//! Space Crystals, 400 frames (25 s)**; requires a Temple"; `> TechPrerequisite - player owns at least one
//! **Temple**`; design change 0444: "**E: Build Prayer-mobile** (greyed "Requires a Temple" until one stands)"), down
//! the Chop Shop's pool path as the Seeker's (`tests/chop_shop/training.rs`).

use sc_sim::{BuildingOrientation, DebugCommand, OrderKind, ProductionCommand, RecruitKind, SpawnAt, TilePos};

use super::*;

/// The Chop Shop (1), player 1's, on tiles (12, 10) to (14, 12): its entrance, B, faces east; its exit, C, west.
const CS: ObjectId = ObjectId(1);
/// The Space Crystals player 1 has from tick 1.
const BANK: u32 = 1000;

/// The Chop Shop's E in KindMenu\[Plain\] (`ProductionCommand::TrainFrom`).
fn train() -> Command {
    Command::Production {
        objects: vec![CS],
        command: ProductionCommand::TrainFrom {
            kind: PRAYER_MOBILE,
            from: RecruitKind::Plain,
        },
    }
}

/// The units its producers finished so far, as `(tick, producer, unit)`.
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::UnitProduced { producer, unit } => Some((*tick, producer, unit)),
            _ => None,
        })
        .collect()
}

/// The plan's acceptance (slice M5-N11): "refused without a Temple; with one, 2 Recruits → Prayer-mobile after 400
/// ticks". Player 1's Chop Shop (1) and Armory (2), its TechPrerequisite, and two of their Recruits inside the Chop
/// Shop: E is refused for want of a Temple (`Reject::Requires`), the Space Crystals and the Recruits there. A Temple
/// of theirs is spawned, and E is taken: it pays 125 and reserves the two. 400 ticks of training, the command's tick
/// counting, and the one that went in first comes out a Prayer-mobile, the other gone into it: at its full 150 HP,
/// with its pool's 50 energy, no augmentation and no buff.
#[test]
fn e_is_refused_without_a_temple_and_with_one_two_recruits_come_out_a_prayer_mobile_400_ticks_later() {
    let mut t = versus(Faction::Cults, 30, 22)
        .spawn(ObjectKind::ChopShop, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(ObjectKind::Armory, Some(P1), SpawnAt::Tile(TilePos::new(2, 2)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(1150.0, 704.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(1150.0, 768.0)))
        .start();
    t.without_auto_targeting();
    let walk_in = [id(3), id(4)];
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: BANK,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t.order_at(2, P1, &walk_in, OrderKind::Enter(CS), false);
    while t.sim().held_in(CS).len() < 2 {
        assert!(t.sim().tick() < 300, "they go in");
        t.step();
    }
    let recruits: Vec<ObjectId> = t.sim().held_in(CS).iter().map(|held| held.object.id).collect();
    assert_eq!(validate(&t, train()), Err(Reject::Requires(ObjectKind::Temple)));
    let temple = id(next_id(&t));
    let tick = t.sim().tick() + 1;
    t.debug_at(
        tick,
        DebugCommand::Spawn {
            kind: ObjectKind::Temple,
            owner: Some(P1),
            at: SpawnAt::Tile(TilePos::new(2, 12)),
            orientation: BuildingOrientation::default(),
        },
    );
    t.run_until(tick);
    assert_eq!(t.sim().object(temple).map(|o| o.kind), Some(ObjectKind::Temple));
    assert_eq!(validate(&t, train()), Ok(()));
    let start = t.sim().tick() + 1;
    t.at(start, P1, train());
    t.run_until(start);
    assert_eq!(t.sim().player(P1).map(|p| p.crystals), Some(BANK - 125), "paid");
    let reserved = t
        .sim()
        .object(CS)
        .and_then(|o| o.production.as_ref())
        .map(|p| p.reserved.clone());
    assert_eq!(reserved, Some(vec![recruits.clone()]), "both reserved");
    t.run_until(start + 398);
    assert_eq!(produced(&t), [], "training");
    t.run_until(start + 399);
    let [(tick, producer, unit)] = produced(&t)[..] else {
        panic!("{:?}", produced(&t))
    };
    assert_eq!(
        (tick, producer, unit),
        (start + 399, CS, recruits[0]),
        "400 ticks, its tick the first"
    );
    let o = t.sim().object(unit).expect("out");
    assert_eq!(o.kind, PRAYER_MOBILE);
    assert_eq!((o.hp, o.max_hp_with(t.sim().research())), (Some(150.0), Some(150.0)));
    assert_eq!(o.energy, Some(50 * ONE), "its pool, from its start");
    assert_eq!((o.augmentation, o.buffs.len()), (sc_sim::Augmentation::NONE, 0));
    let other = recruits[1];
    assert!(
        t.sim().object(other).is_none() && t.sim().held_unit(other).is_none(),
        "gone into it"
    );
}
