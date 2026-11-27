//! (M5-N6) The Chop Shop's activity lines (placeholder wording): its queue counts the Recruits its entries reserve.

use sc_sim::{OrderKind, RecruitKind};

use super::*;

const CHOP_SHOP: ObjectId = ObjectId(1);

/// `factions/cults/structures/chop_shop.md > Placeholder stats`: "InternalRecruitCapacity 10, counting reserved
/// Recruits; so at most 5 Seekers queued". Player 1's Chop Shop with four Recruits inside queues two Seekers, each
/// reserving two of them: its queue shows the two entries by name and the four Recruits they reserve of its 10, as the
/// capacity counts them (Q0104), and no Recruit free. (M5-S8) Its rally point shows too: the one on the ground it
/// starts with, out of side C (`core/entities.md > Entrance and exit > Exit Doorway` point 7, design change 0446).
#[test]
fn a_chop_shop_s_queue_counts_the_recruits_its_seekers_reserve() {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(ObjectKind::ChopShop, Some(P1), tile(6, 4))
        .start();
    t.without_auto_targeting();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    let recruits: Vec<ObjectId> = (0..4)
        .map(|i| {
            t.spawn_unit_at(1, ObjectKind::Recruit, P1, Vec2::new(640.0, 260.0 + 40.0 * i as f32));
            ObjectId(2 + i)
        })
        .collect();
    let enter = Command::Order {
        units: recruits,
        kind: OrderKind::Enter(CHOP_SHOP),
        queue: false,
    };
    t.at(2, P1, enter);
    t.run_until(2);
    while t.sim().held_in(CHOP_SHOP).len() < 4 {
        assert!(t.sim().tick() < 300, "the Recruits went in");
        t.step();
    }
    let start = t.sim().tick() + 1;
    let train = Command::Production {
        objects: vec![CHOP_SHOP],
        command: ProductionCommand::TrainFrom {
            kind: ObjectKind::Seeker,
            from: RecruitKind::Plain,
        },
    };
    t.at(start, P1, train.clone());
    t.at(start, P1, train);
    t.run_until(start + 32);
    assert_eq!(
        lines_of(t.sim(), CHOP_SHOP),
        [
            "Inside: 4 / 10",
            "Free: Plain 0, Constitution 0, Metabolism 0, Both 0",
            "Training Seeker: 10%",
            "Queue: Seeker, Seeker (4 / 10)",
            "Rally point: ground"
        ]
    );
}
