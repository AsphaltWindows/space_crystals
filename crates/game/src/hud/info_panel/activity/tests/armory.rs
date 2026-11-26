//! The Armory's activity lines (slice M5-F2; placeholder wording): who is inside and free, and its TrainingQueue.

use sc_sim::{Augmentation, HoldCommand, OrderKind, RecruitKind};

use super::*;

const ARMORY: ObjectId = ObjectId(1);
const MILITANT: ObjectKind = ObjectKind::Militant;

/// Player 1's Armory (1) at tiles (6, 4) to (8, 5), its exit, side C, facing west, with 1000 Space Crystals and
/// three Recruits gone in through side B, the last of them with Enhanced Constitution (a debug command stands for the
/// Augmentation Center); returned once they are inside.
fn armory_with_three() -> sc_sim::testing::TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(ObjectKind::Armory, Some(P1), tile(6, 4))
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
    let recruits: Vec<ObjectId> = (0..3)
        .map(|i| {
            let at = Vec2::new(640.0, 300.0 + 32.0 * i as f32);
            t.spawn_unit_at(1, ObjectKind::Recruit, P1, at);
            ObjectId(2 + i)
        })
        .collect();
    let augmentation = Augmentation::of(RecruitKind::Constitution);
    t.debug_at(
        2,
        DebugCommand::SetAugmentation {
            id: recruits[2],
            augmentation,
        },
    );
    let enter = Command::Order {
        units: recruits,
        kind: OrderKind::Enter(ARMORY),
        queue: false,
    };
    t.at(2, P1, enter);
    t.run_until(2);
    while t.sim().held_in(ARMORY).len() < 3 {
        assert!(t.sim().tick() < 300, "the Recruits went in");
        t.step();
    }
    t
}

/// `factions/cults/structures/armory.md > ArmoryInstanceState` (StoredRecruits, TrainingQueue, CurrentTraining): the
/// Armory shows how many are inside of its 10 and, kind by kind, how many are free (design change 0337: "the count
/// inside (unreserved)"); its queue, the entries by name and of its capacity (design change 0347 point 5: the capacity
/// bounds the queue), and the progress of the front one, a training's or a Release's. The capacity counts the reserved
/// Recruits; an Eject All empties the pool.
#[test]
fn an_armory_shows_who_is_inside_and_its_training_queue() {
    let mut t = armory_with_three();
    let shown = |sim: &Sim| lines_of(sim, ARMORY);
    assert_eq!(
        shown(t.sim()),
        ["Inside: 3 / 10", "Free: Plain 2, Constitution 1, Metabolism 0, Both 0"]
    );
    let production = |command| Command::Production {
        objects: vec![ARMORY],
        command,
    };
    let start = t.sim().tick() + 1;
    let train = ProductionCommand::TrainFrom {
        kind: MILITANT,
        from: RecruitKind::Plain,
    };
    t.at(start, P1, production(train));
    t.at(
        start,
        P1,
        production(ProductionCommand::Release(RecruitKind::Constitution)),
    );
    t.run_until(start + 15);
    assert_eq!(
        shown(t.sim()),
        [
            "Inside: 3 / 10",
            "Free: Plain 1, Constitution 0, Metabolism 0, Both 0",
            "Training Militant: 50%",
            "Queue: Militant, Release (2 / 10)"
        ]
    );
    // The Militant out, the Release under way: its Recruit is named.
    let mut releasing = None;
    for _ in 0..40 {
        t.step();
        let lines = shown(t.sim());
        if lines.iter().any(|line| line.starts_with("Releasing")) {
            releasing = Some(lines);
            break;
        }
    }
    let releasing = releasing.expect("a Release under way");
    assert_eq!(
        releasing[..2],
        ["Inside: 2 / 10", "Free: Plain 1, Constitution 0, Metabolism 0, Both 0"]
    );
    assert!(releasing[2].starts_with("Releasing Recruit: "), "{releasing:?}");
    assert_eq!(releasing[3], "Queue: Release (1 / 10)");
    let now = t.sim().tick();
    t.run_until(now + 40);
    assert_eq!(
        shown(t.sim()),
        ["Inside: 1 / 10", "Free: Plain 1, Constitution 0, Metabolism 0, Both 0"]
    );
    let eject = Command::Holding {
        objects: vec![ARMORY],
        command: HoldCommand::EjectAll,
    };
    let now = t.sim().tick() + 1;
    t.at(now, P1, eject);
    t.run_until(now + 40);
    assert_eq!(
        shown(t.sim()),
        ["Inside: 0 / 10", "Free: Plain 0, Constitution 0, Metabolism 0, Both 0"]
    );
}
