//! The Garrison's activity lines (slice M5-N2; placeholder wording): how many are inside, and who.

use sc_sim::{HoldCommand, OrderKind};

use super::*;

const GARRISON: ObjectId = ObjectId(1);

/// Plan §N2's "the occupants in the InfoPanel": player 1's Garrison (1) at tiles (6, 4) to (7, 5), its door on side B
/// facing west, shows how many are inside of its 4, and its occupants kind by kind, in the order it takes them in (a
/// Recruit, then Militants, then a Gunner): two Militants and a Gunner gone in; nothing but the count while it is
/// empty, before and after an Eject All.
#[test]
fn a_garrison_shows_how_many_are_inside_and_who() {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(ObjectKind::Garrison, Some(P1), tile(6, 4))
        .start();
    t.without_auto_targeting();
    let shown = |sim: &Sim| lines_of(sim, GARRISON);
    assert_eq!(shown(t.sim()), ["Inside: 0 / 4"]);
    let kinds = [ObjectKind::Gunner, ObjectKind::Militant, ObjectKind::Militant];
    let units: Vec<ObjectId> = (kinds.iter().enumerate())
        .map(|(i, &kind)| {
            t.spawn_unit_at(1, kind, P1, Vec2::new(300.0, 340.0 + 32.0 * i as f32));
            ObjectId(2 + i as u32)
        })
        .collect();
    let enter = Command::Order {
        units,
        kind: OrderKind::Enter(GARRISON),
        queue: false,
    };
    t.at(2, P1, enter);
    t.run_until(2);
    while t.sim().held_in(GARRISON).len() < 3 {
        assert!(t.sim().tick() < 300, "the units went in");
        t.step();
    }
    assert_eq!(shown(t.sim()), ["Inside: 3 / 4", "Occupants: Militant 2, Gunner 1"]);
    let eject = Command::Holding {
        objects: vec![GARRISON],
        command: HoldCommand::EjectAll,
    };
    let now = t.sim().tick() + 1;
    t.at(now, P1, eject);
    t.run_until(now);
    assert_eq!(shown(t.sim()), ["Inside: 0 / 4"], "all out at once");
}
