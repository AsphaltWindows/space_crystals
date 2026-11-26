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
//! - recruitment (a Recruitment Hall's, slice M5-C, [`recruitment`]): its Effectiveness, by the tiles it claims, and
//!   its LocalUnitControlUsage over its LocalUnitControlCapacity;
//! - production (the production capability of slice M2-D2, [`production`]): the unit in production and its
//!   progress, the wait for room under the army cap (design change 0143), the queue length (not a Hall's, which has
//!   no queue to fill), and the rally point;
//! - a Tunnel's operation (slice M4-F, [`operation`]): the expansion it builds and its progress, or the tier it
//!   upgrades to and its progress (slice M4-G).
//!
//! Lines read only the sim, the structure and what the player knows of the objects it names (its sight), so a test
//! can check them without the UI. Their wording is a placeholder until the panel's layout exists (Q0001).

use sc_sim::construction::recipe;
use sc_sim::recruitment;
use sc_sim::tunnel::Operation;
use sc_sim::{Category, Constructor, Object, PlayerId, QUEUE_LIMIT, Rally, Sim, TargetView};

use crate::fog::Sight;
use crate::selection::Relation;

/// What `structure` is doing, one line per item, for the activity area (module docs): empty while it has nothing to
/// show. It is one of the local player's own, the player whose `sight` this is.
pub fn lines(sim: &Sim, sight: &Sight, structure: &Object) -> Vec<String> {
    let mut lines = transformation(structure);
    lines.extend(construction(structure));
    lines.extend(recruitment(sim, structure));
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

/// A Recruitment Hall's lines (`factions/cults/economy.md > RecruitmentHall > RecruitmentHallInstanceState`, slice
/// M5-C; `Sim::hall_state`): its Effectiveness in whole percent rounded down, with the Recruitable tiles it claims of
/// its area's, and its Unit Control, LocalUnitControlUsage, its Recruit in production included (design change 0342),
/// over LocalUnitControlCapacity rounded down (design change 0341: "Capacity is rounded down everywhere"). A Hall under
/// construction claims nothing (design change 0341: "A Hall under construction does nothing until it is complete").
/// Nothing for another structure. Its ProductionProgress is [`production`]'s line.
pub fn recruitment(sim: &Sim, structure: &Object) -> Vec<String> {
    let Some(hall) = sim.hall_state(structure.id) else {
        return Vec::new();
    };
    vec![
        format!(
            "Effectiveness: {}% ({}/{} tiles)",
            hall.effectiveness_percent(),
            hall.claimed,
            hall.tiles
        ),
        format!("Unit Control: {} / {}", hall.usage, hall.capacity.floor()),
    ]
}

/// A producer's lines (`factions/gdo/structures/barracks.md > BarracksInstanceState`): while it has a BuildQueue,
/// the unit in production (CurrentBuild) with its progress, and how many units are queued, that one included, of
/// the most a queue holds (design change 0081); then its RallyPoint, while it has one. A unit that is finished but
/// held inside because the exit doorway is occupied shows at 100% (design change 0085). A unit at the front that
/// waits at 0% for room under its owner's army cap (design change 0143: "the building waits with the unit at the
/// front (0% progress)") has a line saying so under its own, "Waiting for Unit Control" or "Waiting for Beacon
/// Capacity": placeholder, with the feedback band's message as the wait begins
/// (`crate::command_ui::feedback::army_cap_text`), for the notification 0143 leaves to be designed. Nothing for an
/// idle producer without a rally point, nor for a structure that doesn't produce. A Recruitment Hall's Recruit is
/// "Recruiting", and the Hall shows no queue: it makes one Recruit at a time, by itself (slice M5-C).
pub fn production(sim: &Sim, sight: &Sight, structure: &Object) -> Vec<String> {
    let Some(production) = &structure.production else {
        return Vec::new();
    };
    let recruits = recruitment::def_of(structure.kind).is_some();
    let mut lines = Vec::new();
    if let Some(status) = sim.production_status(structure.id) {
        // Whole percent, rounded down: 100% only once the unit is finished.
        let percent = match status.total {
            0 => 100,
            total => u64::from(status.progress) * 100 / u64::from(total),
        };
        let doing = if recruits { "Recruiting" } else { "Training" };
        lines.push(format!("{doing} {}: {percent}%", status.kind.def().name));
        if let Some(cap) = status.blocked {
            lines.push(format!("Waiting for {}", cap.name()));
        }
        if !recruits {
            lines.push(format!("Queue: {} / {QUEUE_LIMIT}", production.queue.len()));
        }
    }
    if let Some(rally) = production.rally {
        lines.push(rally_line(sim, sight, structure.owner, rally));
    }
    lines
}

/// A Tunnel's line while it performs an operation (`factions/syndicate/tunnels.md > Expansions under construction`:
/// "**The Tunnel meanwhile** shows the progress"; `Sim::operation_progress`): the expansion it builds, or the tier it
/// upgrades to (slice M4-G: "Upgrading to Tier 2", placeholder wording), and the progress in whole percent rounded
/// down, 100% only once complete. Nothing for a Tunnel that performs none, nor for another structure.
pub fn operation(sim: &Sim, structure: &Object) -> Vec<String> {
    let Some((operation, progress)) = sim.operation_progress(structure.id) else {
        return Vec::new();
    };
    let doing = match operation {
        Operation::Building(site) => match sim.object(site) {
            Some(site) => format!("Building {}", site.def().name),
            None => return Vec::new(),
        },
        Operation::Upgrading { to, .. } => format!("Upgrading to Tier {to}"),
    };
    vec![format!("{doing}: {}%", progress.percent())]
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
mod tests;
