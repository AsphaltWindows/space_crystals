//! Production at the Chop Shop (`factions/cults/structures/chop_shop.md > Production`: "queuing an order deducts its
//! Space Crystals cost and **reserves the vehicle's number of Recruits** from the pool (an order can only be queued if
//! enough unreserved Recruits are inside); when it finishes, they are consumed and the vehicle exits to the
//! RallyPoint"; design change 0444: "**W: Build Hand of Heaven**"; `factions/cults/units/hand_of_heaven.md >
//! Placeholder stats`: "Cost: Chop Shop, 3 Recruits, **150 Space Crystals, 480 frames (30 s)**"), by the pool path the
//! Seeker's tests cover in full (`tests/chop_shop/`): the same path, three Recruits instead of two.

use sc_sim::mover::MoveStatus;
use sc_sim::{Command, DebugCommand, PlayerCommand, ProductionCommand, RecruitKind, Reject, math};

use super::*;

const CHOP_SHOP: ObjectKind = ObjectKind::ChopShop;
/// The Chop Shop of [`workshop`].
const CS: ObjectId = ObjectId(1);
const PRICE: u32 = 150;
const FRAMES: Tick = 480;
/// The Hand of Heaven's MaxHP (`factions/cults/units/hand_of_heaven.md > Placeholder stats`: "MaxHP 200").
const MAX_HP: f32 = 200.0;
/// The middle of [`workshop`]'s Chop Shop's exit side, C, facing west.
const EXIT: Vec2 = Vec2::new(768.0, 736.0);
/// The Space Crystals player 1 has from tick 1.
const BANK: u32 = 1000;

/// Player 1's Chop Shop (1), unturned, on tiles (12, 10) to (14, 12) of a flat 30 × 22 map (its entrance, B, faces
/// east, its exit, C, west, [`EXIT`]), their Armory (2), its TechPrerequisite, and `count` of their Recruits, which
/// are given [`BANK`] Space Crystals in tick 1 and ordered in in tick 2; returned once all are inside, longest inside
/// first, the Chop Shop's default rally point 3 grid units out of side C (design change 0446).
fn workshop(count: u32) -> (TestSim, Vec<ObjectId>) {
    let mut t = cults(30, 22)
        .spawn(CHOP_SHOP, Some(P1), tile(12, 10))
        .spawn(ObjectKind::Armory, Some(P1), tile(2, 2))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(BANK));
    let recruits: Vec<ObjectId> = (0..count).map(|i| id(3 + i)).collect();
    for i in 0..count {
        let at = Vec2::new(1150.0 + 32.0 * (i / 5) as f32, 672.0 + 32.0 * (i % 5) as f32);
        t.spawn_unit_at(1, ObjectKind::Recruit, P1, at);
    }
    t.at(2, P1, enter(&recruits));
    until(&mut t, 300, "the Recruits are in", |t| {
        inside(t).len() == count as usize
    });
    let order = inside(&t);
    (t, order)
}

fn crystals(crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player: P1,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

fn enter(units: &[ObjectId]) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(CS),
        queue: false,
    }
}

fn production(command: ProductionCommand) -> Command {
    Command::Production {
        objects: vec![CS],
        command,
    }
}

/// KindMenu\[`from`\]'s "**W: Build Hand of Heaven**".
fn build(from: RecruitKind) -> Command {
    production(ProductionCommand::TrainFrom { kind: HAND, from })
}

/// The units inside the Chop Shop, longest inside first.
fn inside(t: &TestSim) -> Vec<ObjectId> {
    t.sim().held_in(CS).iter().map(|held| held.object.id).collect()
}

/// The units its queue's entries reserve, entry by entry.
fn reserved(t: &TestSim) -> Vec<Vec<ObjectId>> {
    let o = t.sim().object(CS).expect("the Chop Shop");
    o.production.as_ref().expect("a producer").reserved.clone()
}

fn bank(t: &TestSim) -> u32 {
    t.sim().player(P1).expect("in the game").crystals
}

fn validate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// The units the Chop Shop finished so far, with their ticks.
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::UnitProduced { producer, unit } if producer == CS => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// "W: Build Hand of Heaven" with four Recruits inside: it pays 150 Space Crystals and reserves the three inside the
/// longest, their Unit Control cost; 480 ticks of training, the command's tick counting, and the first comes out a
/// Hand of Heaven, the same unit, at full HP ("a vehicle or aircraft comes out at **full HP**", Q0121), half inside
/// the middle of side C and facing out of it, the other two gone into it and the fourth still inside. It drives out to
/// the Chop Shop's default rally point, and its player's Unit Control counts its 3.
#[test]
fn w_takes_three_recruits_and_150_and_a_hand_of_heaven_comes_out_480_ticks_later() {
    let (mut t, order) = workshop(4);
    let unit_control = t.sim().unit_control(P1);
    let start = t.sim().tick() + 1;
    t.at(start, P1, build(RecruitKind::Plain));
    t.run_until(start);
    assert_eq!(bank(&t), BANK - PRICE);
    assert_eq!(reserved(&t), [order[..3].to_vec()], "the three inside the longest");
    assert_eq!(t.sim().free_units(CS, RecruitKind::Plain), [order[3]]);
    assert_eq!(t.sim().room(CS), 6, "the reserved Recruits are inside");
    let queued = SimEvent::ProductionQueued {
        producer: CS,
        kind: HAND,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == queued));
    t.run_until(start + FRAMES - 2);
    assert_eq!(produced(&t), [], "training");
    t.step();
    let out = start + FRAMES - 1;
    assert_eq!(produced(&t), [(out, order[0])], "480 ticks, the command's first");
    let hand = t.sim().object(order[0]).expect("out");
    assert_eq!((hand.kind, hand.hp, hand.pos), (HAND, Some(MAX_HP), EXIT));
    assert!(
        math::facing_vector(hand.facing).distance(-Vec2::X) < 1e-6,
        "facing out, west"
    );
    for gone in [order[1], order[2]] {
        assert!(
            t.sim().object(gone).is_none() && t.sim().held_unit(gone).is_none(),
            "{gone:?} gone into it"
        );
    }
    assert_eq!((inside(&t), reserved(&t)), (vec![order[3]], Vec::new()));
    assert_eq!(
        t.sim().unit_control(P1),
        unit_control,
        "3 Recruits' Unit Control, now the Hand of Heaven's"
    );
    let rally = EXIT - Vec2::new(3.0 * SU_PER_GU, 0.0);
    until(&mut t, 200, "at the rally point", |t| {
        t.status(order[0].0) == MoveStatus::Arrived
    });
    assert!(t.pos(order[0].0).distance(rally) < 32.0, "at {}", t.pos(order[0].0));
    assert_eq!(bank(&t), BANK - PRICE);
}

/// "an order can only be queued if enough unreserved Recruits are inside": with two inside, W is refused for want of a
/// third (`Reject::NoRecruit`), and Q, the Seeker's two, is taken; then W, its two taken, again.
#[test]
fn with_two_recruits_inside_w_is_refused() {
    let (mut t, _) = workshop(2);
    assert_eq!(
        validate(&t, build(RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
    let seeker = production(ProductionCommand::TrainFrom {
        kind: ObjectKind::Seeker,
        from: RecruitKind::Plain,
    });
    assert_eq!(validate(&t, seeker.clone()), Ok(()));
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, seeker);
    t.run_until(tick);
    assert_eq!(
        validate(&t, build(RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
}

/// "Cancel returns the reserved Recruits to the pool" and refunds the crystals
/// (`factions/cults/structures/chop_shop.md > Placeholder stats`: "**X: Cancel Production** (refunds the crystals and
/// returns the reserved Recruits)"): X during the training gives back the 150 Space Crystals and the three Recruits,
/// free again, and nothing comes out.
#[test]
fn x_refunds_the_150_and_returns_the_three_recruits() {
    let (mut t, order) = workshop(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, build(RecruitKind::Plain));
    t.at(start + 100, P1, production(ProductionCommand::CancelProduction));
    t.run_until(start);
    assert_eq!((bank(&t), t.sim().pool_counts(CS)), (BANK - PRICE, [0; 4]));
    t.run_until(start + 100);
    assert_eq!(bank(&t), BANK, "refunded in full");
    assert_eq!(
        (reserved(&t), t.sim().free_units(CS, RecruitKind::Plain)),
        (Vec::new(), order.clone())
    );
    t.run_until(start + FRAMES + 10);
    assert_eq!(produced(&t), []);
    assert_eq!(inside(&t), order);
}
