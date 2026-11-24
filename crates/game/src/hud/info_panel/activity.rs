//! The InfoPanel's activity area (`core/entities.md > Object Type > InfoPanel > Default InfoPanel for structures`,
//! design change 0105): for one of the local player's own structures, "what it is doing: construction or production
//! with progress, queue length, a building ready to place, and the rally point", under its HP, in a text of its own
//! (`InfoText::Activity`, hidden while it has no line). Another player's structure has no activity area: "name, owner
//! and HP only (no production or queue information)".
//!
//! **The extension point** for the structures' activities: [`lines`] gives the area's lines, one capability of the
//! structure after another, in this order, and the panel shows them as they come:
//! - construction (the constructor capability of slice M2-D1; its lines come with the construction UI, slice
//!   M2-H2): the building under construction and its progress, or the building ready to place;
//! - production (the production capability of slice M2-D2, [`production`]): the unit in production and its
//!   progress, the queue length, and the rally point.
//!
//! Lines read only the sim and the structure, so a test can check them without the UI. Their wording is a
//! placeholder until the panel's layout exists (Q0001).

use sc_sim::{Object, PlayerId, QUEUE_LIMIT, Rally, Sim};

use crate::selection::Relation;

/// What `structure`, one of the local player's own, is doing, one line per item, for the activity area (module docs):
/// empty while it has nothing to show.
pub fn lines(sim: &Sim, structure: &Object) -> Vec<String> {
    production(sim, structure)
}

/// A producer's lines (`factions/gdo/structures.md > Barracks > BarracksInstanceState`): while it has a BuildQueue,
/// the unit in production (CurrentBuild) with its progress, and how many units are queued, that one included, of
/// the most a queue holds (design change 0081); then its RallyPoint, while it has one. A unit that is finished but
/// held inside because the exit doorway is occupied shows at 100% (design change 0085). Nothing for an idle producer
/// without a rally point, nor for a structure that doesn't produce.
pub fn production(sim: &Sim, structure: &Object) -> Vec<String> {
    let Some(production) = &structure.production else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    if let Some(status) = sim.production_status(structure.id) {
        // Whole percent, rounded down: 100% only once the unit is finished.
        let percent = match status.total {
            0 => 100,
            total => u64::from(status.progress) * 100 / u64::from(total),
        };
        lines.push(format!("Training {}: {percent}%", status.kind.def().name));
        lines.push(format!("Queue: {} / {QUEUE_LIMIT}", production.queue.len()));
    }
    if let Some(rally) = production.rally {
        lines.push(rally_line(sim, structure.owner, rally));
    }
    lines
}

/// The RallyPoint of a producer of `owner`: the ground, or the object it is on by its type's name, marked when it is
/// an enemy's (the colours of its indicator: `crate::command_ui::indicators::rally`). An object rally point is on an
/// object its owner sees, or on an own or neutral one (design change 0088), so its name gives nothing away.
fn rally_line(sim: &Sim, owner: Option<PlayerId>, rally: Rally) -> String {
    let target = match rally {
        Rally::Point(_) => None,
        Rally::Object { id, .. } => sim.object(id),
    };
    match target {
        None => "Rally point: ground".to_string(),
        Some(target) => {
            let name = target.def().name;
            match owner.map(|owner| Relation::of(owner, target.owner)) {
                Some(Relation::Enemy) => format!("Rally point: {name} (enemy)"),
                _ => format!("Rally point: {name}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sc_sim::testing::ScenarioBuilder;
    use sc_sim::{
        Command, DebugCommand, ObjectId, ObjectKind, PlayerId, ProductionCommand, SpawnAt, Target, TilePos, Vec2,
    };

    use super::*;

    const P1: PlayerId = PlayerId(1);
    const P2: PlayerId = PlayerId(2);
    const BARRACKS: ObjectId = ObjectId(2);
    const PK: ObjectKind = ObjectKind::Peacekeeper;

    fn tile(x: i32, y: i32) -> SpawnAt {
        SpawnAt::Tile(TilePos::new(x, y))
    }

    fn production(command: ProductionCommand) -> Command {
        Command::Production {
            objects: vec![BARRACKS],
            command,
        }
    }

    /// The Barracks' activity lines.
    fn shown(sim: &Sim) -> Vec<String> {
        lines(sim, sim.object(BARRACKS).unwrap())
    }

    /// `core/entities.md > Object Type > InfoPanel` (design change 0105's first verification: "Select your own
    /// Barracks while it's producing. It shows its name, HP, progress, queue length and rally point"). Player 1's
    /// Deployment Center (1) and Barracks (2, at full power) with a Generator (3) flush against its exit side, so the
    /// first Peacekeeper stays in the doorway and the second waits inside at 100% (design change 0085); player 2's
    /// Peacekeeper (4) in sight.
    #[test]
    fn a_barracks_shows_its_unit_queue_and_rally_point() {
        let mut t = ScenarioBuilder::new(&vec![".".repeat(30); 16].join("\n"))
            .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
            .spawn(ObjectKind::Barracks, Some(P1), tile(8, 4))
            .spawn(ObjectKind::Generator, Some(P1), tile(11, 4))
            .spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(600.0, 520.0)))
            .start();
        assert_eq!(shown(t.sim()), Vec::<String>::new(), "idle, without a rally point");
        t.debug_at(1, DebugCommand::SetAutoTargeting { on: false });
        t.debug_at(
            1,
            DebugCommand::SetResources {
                player: P1,
                crystals: 1000,
                supplies: 0,
            },
        );
        for _ in 0..3 {
            t.at(2, P1, production(ProductionCommand::Train(PK)));
        }
        t.run_until(41);
        assert_eq!(shown(t.sim()), ["Training Peacekeeper: 50%", "Queue: 3 / 20"]);
        t.at(
            42,
            P1,
            production(ProductionCommand::SetRally(Target::Point(Vec2::new(900.0, 320.0)))),
        );
        t.run_until(42);
        assert_eq!(
            shown(t.sim()),
            ["Training Peacekeeper: 51%", "Queue: 3 / 20", "Rally point: ground"]
        );
        t.at(
            43,
            P1,
            production(ProductionCommand::SetRally(Target::Object(ObjectId(4)))),
        );
        t.run_until(43);
        assert_eq!(shown(t.sim())[2], "Rally point: Peacekeeper (enemy)");
        t.at(
            44,
            P1,
            production(ProductionCommand::SetRally(Target::Object(ObjectId(1)))),
        );
        t.run_until(200);
        let status = t.sim().production_status(BARRACKS).unwrap();
        assert!(status.waiting, "the first one blocks the doorway");
        assert_eq!(
            shown(t.sim()),
            [
                "Training Peacekeeper: 100%",
                "Queue: 2 / 20",
                "Rally point: Deployment Center"
            ]
        );
        // The building itself clears the rally point (design change 0086).
        t.at(
            201,
            P1,
            production(ProductionCommand::SetRally(Target::Object(BARRACKS))),
        );
        t.run_until(201);
        assert_eq!(shown(t.sim()), ["Training Peacekeeper: 100%", "Queue: 2 / 20"]);
        let generator = t.sim().object(ObjectId(3)).unwrap();
        assert_eq!(lines(t.sim(), generator), Vec::<String>::new(), "it produces nothing");
    }
}
