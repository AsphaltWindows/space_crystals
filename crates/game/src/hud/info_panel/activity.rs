//! The InfoPanel's activity area (`core/entities.md > Object Type > InfoPanel > Default InfoPanel for structures`,
//! design change 0105): for one of the local player's own structures, "what it is doing: construction or production
//! with progress, queue length, a building ready to place, and the rally point", under its HP, in a text of its own
//! (`InfoText::Activity`, hidden while it has no line). Another player's structure has no activity area: "name, owner
//! and HP only (no production or queue information)".
//!
//! **The extension point** for the structures' activities: [`lines`] gives the area's lines, one capability of the
//! structure after another, in this order, and the panel shows them as they come:
//! - transformation (`Object::transform`, slice M3-E, [`transformation`]): the building being warped in and its
//!   progress, the one thing a transforming structure does (design change 0116);
//! - construction (the constructor capability of slice M2-D1, [`construction`]): the building under construction
//!   and its progress, or the building ready to place;
//! - production (the production capability of slice M2-D2, [`production`]): the unit in production and its
//!   progress, the wait for room under the army cap (design change 0143), the queue length, and the rally point;
//! - a Tunnel's operation (slice M4-F, [`operation`]): the expansion it builds and its progress.
//!
//! Lines read only the sim, the structure and what the player knows of the objects it names (its sight), so a test
//! can check them without the UI. Their wording is a placeholder until the panel's layout exists (Q0001).

use sc_sim::construction::recipe;
use sc_sim::tunnel::Operation;
use sc_sim::{Category, Constructor, Object, PlayerId, QUEUE_LIMIT, Rally, Sim, TargetView};

use crate::fog::Sight;
use crate::selection::Relation;

/// What `structure` is doing, one line per item, for the activity area (module docs): empty while it has nothing to
/// show. It is one of the local player's own, the player whose `sight` this is.
pub fn lines(sim: &Sim, sight: &Sight, structure: &Object) -> Vec<String> {
    let mut lines = transformation(structure);
    lines.extend(construction(structure));
    lines.extend(production(sim, sight, structure));
    lines.extend(operation(sim, structure));
    lines
}

/// A transforming structure's line (`Object::transform`): the building it becomes and its progress, "frames elapsed /
/// WarpInTime" (`factions/colonists/economy.md > Warp In > HP/Shields Interpolation`), in whole percent rounded down,
/// 100% only once complete. "Warping in" from a structure, a Beacon; "Deploying" from a unit, a Drone becoming a
/// Beacon (slice M3-F). Nothing for a structure that isn't transforming.
pub fn transformation(structure: &Object) -> Vec<String> {
    let Some(t) = &structure.transform else {
        return Vec::new();
    };
    let doing = match t.from.def().category {
        Category::Unit => "Deploying",
        _ => "Warping in",
    };
    vec![format!("{doing}: {} {}%", structure.def().name, t.percent())]
}

/// A constructor's line (`factions/gdo/structures/deployment_center.md > DeploymentCenterInstanceState`,
/// `factions/gdo/economy.md > ExtractionFacility`; design change 0105: "construction [...] progress [...] a
/// ready-to-place building"): the building under construction (CurrentConstruction) with its progress
/// (ConstructionProgress, "frames elapsed", of its recipe's frames), or the building ready to place (ReadyToPlace).
/// Nothing for an idle constructor, nor for a structure that constructs nothing.
pub fn construction(structure: &Object) -> Vec<String> {
    match structure.constructor {
        Some(Constructor::Constructing { kind, .. }) => {
            let done = structure.constructor.and_then(|c| c.frames_done()).unwrap_or(0);
            // Whole percent, rounded down: 100% only once it is ready.
            let percent = match recipe(structure.kind, kind).map_or(0, |r| r.frames) {
                0 => 100,
                frames => u64::from(done.min(frames)) * 100 / u64::from(frames),
            };
            vec![format!("Constructing {}: {percent}%", kind.def().name)]
        }
        Some(Constructor::Ready { kind }) => vec![format!("{} ready to place", kind.def().name)],
        Some(Constructor::Idle) | None => Vec::new(),
    }
}

/// A producer's lines (`factions/gdo/structures/barracks.md > BarracksInstanceState`): while it has a BuildQueue,
/// the unit in production (CurrentBuild) with its progress, and how many units are queued, that one included, of
/// the most a queue holds (design change 0081); then its RallyPoint, while it has one. A unit that is finished but
/// held inside because the exit doorway is occupied shows at 100% (design change 0085). A unit at the front that
/// waits at 0% for room under its owner's army cap (design change 0143: "the building waits with the unit at the
/// front (0% progress)") has a line saying so under its own, "Waiting for Unit Control" or "Waiting for Beacon
/// Capacity": placeholder, with the feedback band's message as the wait begins
/// (`crate::command_ui::feedback::army_cap_text`), for the notification 0143 leaves to be designed. Nothing for an
/// idle producer without a rally point, nor for a structure that doesn't produce.
pub fn production(sim: &Sim, sight: &Sight, structure: &Object) -> Vec<String> {
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
        if let Some(cap) = status.blocked {
            lines.push(format!("Waiting for {}", cap.name()));
        }
        lines.push(format!("Queue: {} / {QUEUE_LIMIT}", production.queue.len()));
    }
    if let Some(rally) = production.rally {
        lines.push(rally_line(sim, sight, structure.owner, rally));
    }
    lines
}

/// A Tunnel's line while it performs an operation (`factions/syndicate/tunnels.md > Expansions under construction`:
/// "**The Tunnel meanwhile** shows the progress"; `Sim::operation_progress`): the expansion it builds, its site's
/// progress in whole percent rounded down, 100% only once complete. Nothing for a Tunnel that performs none, nor for
/// another structure.
pub fn operation(sim: &Sim, structure: &Object) -> Vec<String> {
    let Some((Operation::Building(site), progress)) = sim.operation_progress(structure.id) else {
        return Vec::new();
    };
    let Some(site) = sim.object(site) else {
        return Vec::new();
    };
    vec![format!("Building {}: {}%", site.def().name, progress.percent())]
}

/// The RallyPoint of a producer of `owner`: the ground, or the object it is on by its type's name, marked when it is
/// an enemy's (the colours of its indicator: `crate::command_ui::indicators::rally`), as the player whose `sight`
/// this is knows the object (`Sight::target_view`): the object while the player sees it; out of sight, the record it
/// remembers (a structure or a resource, design change 0139: "any rally marker treat[s] the remembered object as
/// present until the player sees otherwise"), whatever became of the object (slice M3-K2a). An enemy unit's rally
/// point turns to the ground when the unit leaves its owner's sight (design change 0088).
fn rally_line(sim: &Sim, sight: &Sight, owner: Option<PlayerId>, rally: Rally) -> String {
    let target = match rally {
        Rally::Point(_) => None,
        Rally::Object { id, .. } => match sight.target_view(sim, id) {
            TargetView::Visible => sim.object(id).map(|o| (o.kind, o.owner)),
            TargetView::Remembered { .. } => sight.remembered_record(id).map(|r| (r.kind, r.owner)),
            TargetView::Gone => None,
        },
    };
    match target {
        None => "Rally point: ground".to_string(),
        Some((kind, target_owner)) => {
            let name = kind.def().name;
            match owner.map(|owner| Relation::of(owner, target_owner)) {
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
        BuildingOrientation, Command, ConstructionCommand, DebugCommand, Faction, ObjectId, ObjectKind, PlayerId,
        ProductionCommand, SpawnAt, Target, TilePos, TunnelCommand, Vec2,
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

    /// The activity lines of player 1's structure `id`, as player 1 sees the world.
    fn lines_of(sim: &Sim, id: ObjectId) -> Vec<String> {
        lines(sim, &Sight::new(sim, P1, true), sim.object(id).unwrap())
    }

    /// The Barracks' activity lines.
    fn shown(sim: &Sim) -> Vec<String> {
        lines_of(sim, BARRACKS)
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
                alloys: 0,
                essence: 0,
                flux: 0,
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
        let generator = ObjectId(3);
        assert_eq!(
            lines_of(t.sim(), generator),
            Vec::<String>::new(),
            "it produces nothing"
        );
    }

    /// Design change 0143: a Barracks whose Peacekeeper waits at 0% for Unit Control says so under the unit, until
    /// it starts. Player 1's Deployment Center (1) and Barracks (2) with 200 Peacekeepers (3 to 202), the cap, far
    /// off; one of them removed gives the room.
    #[test]
    fn a_waiting_barracks_says_what_it_waits_for() {
        let mut builder = ScenarioBuilder::new(&vec![".".repeat(30); 16].join("\n"))
            .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
            .spawn(ObjectKind::Barracks, Some(P1), tile(8, 4));
        for i in 0..200 {
            let at = Vec2::new(1000.0 + 30.0 * (i % 20) as f32, 80.0 + 30.0 * (i / 20) as f32);
            builder = builder.spawn(PK, Some(P1), SpawnAt::Point(at));
        }
        let mut t = builder.start();
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
        t.at(2, P1, production(ProductionCommand::Train(PK)));
        t.at(2, P1, production(ProductionCommand::Train(PK)));
        t.run_until(10);
        assert_eq!(
            shown(t.sim()),
            ["Training Peacekeeper: 0%", "Waiting for Unit Control", "Queue: 2 / 20"]
        );
        t.debug_at(11, DebugCommand::Remove { id: ObjectId(3) });
        t.run_until(11);
        assert_eq!(shown(t.sim()), ["Training Peacekeeper: 1%", "Queue: 2 / 20"]);
    }

    /// Design change 0105: an own constructor shows its construction and its progress, then the building ready to
    /// place. Player 1's Deployment Center (1) constructs a Generator (240 frames at full power: its own +30 power
    /// covers it), which is ready after 240 frames.
    #[test]
    fn a_constructor_shows_its_construction_then_its_ready_building() {
        let dc = ObjectId(1);
        let mut t = ScenarioBuilder::new(&vec![".".repeat(30); 16].join("\n"))
            .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
            .start();
        let shown = |sim: &Sim| lines_of(sim, dc);
        assert_eq!(shown(t.sim()), Vec::<String>::new(), "idle");
        t.at(
            1,
            P1,
            Command::Construction {
                objects: vec![dc],
                command: ConstructionCommand::Construct(ObjectKind::Generator),
            },
        );
        t.run_until(1);
        assert_eq!(shown(t.sim()), ["Constructing Generator: 0%"]);
        t.run_until(121);
        assert_eq!(shown(t.sim()), ["Constructing Generator: 50%"]);
        t.run_until(400);
        assert_eq!(shown(t.sim()), ["Generator ready to place"]);
    }

    /// Slice M3-E: a Beacon warping in a building shows the building and the Warp In's progress, "frames elapsed /
    /// WarpInTime" (`factions/colonists/economy.md > Warp In > HP/Shields Interpolation`) in whole percent rounded
    /// down, and nothing else (design change 0116: functions start at completion); cancelled, a Beacon again, it
    /// shows nothing. Player 1's Beacon (1) warps in a Robo Portal (4 × 4 tiles centred on it, WarpInTime 640 frames)
    /// ordered in tick 1: at the end of tick 288, 288 frames are done, 45%.
    #[test]
    fn a_warp_in_shows_its_building_and_its_progress() {
        let beacon = ObjectId(1);
        let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
            .spawn(ObjectKind::Beacon, Some(P1), tile(8, 6))
            .start();
        let shown = |sim: &Sim| lines_of(sim, beacon);
        assert_eq!(shown(t.sim()), Vec::<String>::new(), "a Beacon has nothing to show");
        let construction = |command| Command::Construction {
            objects: vec![beacon],
            command,
        };
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
        let warp_in = ConstructionCommand::WarpIn {
            kind: ObjectKind::RoboPortal,
            orientation: Default::default(),
        };
        t.at(1, P1, construction(warp_in));
        t.run_until(1);
        assert_eq!(shown(t.sim()), ["Warping in: Robo Portal 0%"]);
        t.run_until(288);
        assert_eq!(shown(t.sim()), ["Warping in: Robo Portal 45%"]);
        t.at(289, P1, construction(ConstructionCommand::CancelTransform));
        t.run_until(289);
        assert_eq!(t.sim().object(beacon).unwrap().kind, ObjectKind::Beacon);
        assert_eq!(shown(t.sim()), Vec::<String>::new(), "a Beacon again");
    }

    /// Slice M4-F, `factions/syndicate/tunnels.md > Expansions under construction`: "**The Tunnel meanwhile** shows
    /// the progress". Player 1's Tunnel (1) builds a Hideout (400 frames) in its area from tick 1: its line names the
    /// expansion and its site's progress, a quarter after 100 frames, until the Tunnel's X cancels it.
    #[test]
    fn a_tunnel_shows_the_expansion_it_builds_and_its_progress() {
        let tunnel = ObjectId(1);
        let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
            .faction(P1, Faction::Syndicate)
            .spawn(ObjectKind::Tunnel, Some(P1), tile(8, 6))
            .start();
        let shown = |sim: &Sim| lines_of(sim, tunnel);
        assert_eq!(shown(t.sim()), Vec::<String>::new(), "no operation");
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
        let command = |command| Command::Tunnel { tunnel, command };
        let expand = TunnelCommand::Expand {
            kind: ObjectKind::Hideout,
            origin: TilePos::new(6, 4),
            orientation: BuildingOrientation::default(),
        };
        t.at(1, P1, command(expand));
        t.run_until(1);
        assert_eq!(shown(t.sim()), ["Building Hideout: 0%"]);
        t.run_until(101);
        assert_eq!(shown(t.sim()), ["Building Hideout: 25%"]);
        t.at(102, P1, command(TunnelCommand::Cancel));
        t.run_until(102);
        assert_eq!(shown(t.sim()), Vec::<String>::new(), "cancelled");
    }
}
