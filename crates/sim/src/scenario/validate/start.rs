//! The objects a game starts with (`Scenario::start_objects`): the map's pre-placed objects and the Starting Layouts
//! the players get, checked together, and the starting units' lineage (slice M5-B).

use std::collections::BTreeSet;

use glam::Vec2;

use super::occupied::Occupied;
use super::place_layout_object;
use crate::defs::ObjectKind;
use crate::economy::ArmyResource;
use crate::ids::PlayerId;
use crate::object::Placement;
use crate::paths::misplaced;
use crate::recruitment;
use crate::scenario::Scenario;
use crate::scenario::problems::ScenarioProblem;
use crate::scenario::settings::SpawnSettings;

/// An object a game starts with, checked and placed.
pub(crate) struct StartObject {
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    pub pos: Vec2,
    pub placement: Option<Placement>,
    pub settings: Option<SpawnSettings>,
    /// Its lineage (`Object::lineage`), as indices into the start objects: the Hall each Recruit it embodies belongs
    /// to ([`belong_to_their_hall`]). Empty for everything else.
    pub lineage: Vec<usize>,
}

impl Scenario {
    /// Everything that makes this game unplayable: the map's problems ([`crate::MapDef::validate`], which checks every
    /// Starting Layout on its own), then the setup's, then the overlaps of the Starting Layouts the players get, then
    /// their units on blocked ground or on other units and their footprints on units: the layouts of different
    /// players in each other's way. Empty means [`crate::Sim::try_new`] accepts it. Never panics.
    pub fn validate(&self) -> Vec<ScenarioProblem> {
        self.start_objects().err().unwrap_or_default()
    }

    /// Validates the scenario. If it is valid, returns the objects the game starts with, in order: the pre-placed
    /// objects (those of slots that no player takes are left out), then each player's Starting Layout, player by
    /// player, the objects owned by their players, the units of a layout with a Recruitment Hall belonging to it
    /// ([`belong_to_their_hall`]).
    pub(crate) fn start_objects(&self) -> Result<Vec<StartObject>, Vec<ScenarioProblem>> {
        let mut problems = Vec::new();
        let spots = self.map.check(&mut problems);
        if self.setup.players.is_empty() {
            problems.push(ScenarioProblem::NoPlayers);
        }
        let mut taken = BTreeSet::new();
        for (player, p) in self.setup.player_ids().zip(&self.setup.players) {
            if self.map.slot(p.slot).is_none() {
                problems.push(ScenarioProblem::UnknownSlot { player, slot: p.slot });
            } else if !taken.insert(p.slot) {
                problems.push(ScenarioProblem::SlotTaken { player, slot: p.slot });
            }
        }
        if !problems.is_empty() {
            return Err(problems);
        }

        let mut objects = Vec::new();
        // The Starting Layout object each start object comes from (`None` for a pre-placed one).
        let mut from_layout = Vec::new();
        let mut occupied: Occupied<()> = Occupied::default();
        for (s, spot) in self.map.spawns.iter().zip(spots) {
            let Some((pos, placement)) = spot else {
                continue;
            };
            if let Some(placement) = &placement {
                occupied.take(s.kind, placement, ());
            }
            let owner = match s.owner {
                None => None,
                // No player takes the slot: its objects stay out of the game.
                Some(slot) => match self.setup.player_at(slot) {
                    Some(player) => Some(player),
                    None => continue,
                },
            };
            objects.push(StartObject {
                kind: s.kind,
                owner,
                pos,
                placement,
                settings: s.settings,
                lineage: Vec::new(),
            });
            from_layout.push(None);
        }
        for (player, p) in self.setup.player_ids().zip(&self.setup.players) {
            let Some(slot) = self.map.slot(p.slot) else {
                continue;
            };
            let first = objects.len();
            for (index, o) in slot.layouts.get(p.faction).iter().enumerate() {
                let Ok((pos, placement)) = place_layout_object(&self.map.terrain, o.kind, o.at, o.orientation) else {
                    continue;
                };
                if let Some(placement) = &placement
                    && let Some(((), tile)) = occupied.take(o.kind, placement, ())
                {
                    problems.push(ScenarioProblem::LayoutOverlap {
                        slot: p.slot,
                        faction: p.faction,
                        index,
                        tile,
                    });
                }
                objects.push(StartObject {
                    kind: o.kind,
                    owner: Some(player),
                    pos,
                    placement,
                    settings: None,
                    lineage: Vec::new(),
                });
                from_layout.push(Some((p.slot, p.faction, index)));
            }
            belong_to_their_hall(&mut objects, first);
        }
        // Units. The pre-placed objects passed the same check against all the map's spawns, and each layout against
        // them (`MapDef::check`), so only objects of two players' layouts in each other's way can fail it here.
        let placed = objects.iter().map(|o| (o.kind, o.pos, o.placement.as_ref()));
        for (n, problem) in misplaced(&self.map.terrain, placed) {
            if let (Some(&Some((slot, faction, index))), Some(o)) = (from_layout.get(n), objects.get(n)) {
                problems.push(ScenarioProblem::InvalidLayoutObject {
                    slot,
                    faction,
                    index,
                    kind: o.kind,
                    problem,
                });
            }
        }
        if problems.is_empty() {
            Ok(objects)
        } else {
            Err(problems)
        }
    }
}

/// `factions/cults/economy.md > Starting Conditions`: "Six **Recruits**, all belonging to the starting Recruitment Hall
/// (they count toward its LocalUnitControlUsage — see RecruitmentHall > Unit Control Tracking)". The layout's objects
/// are `objects[first..]`, and its starting Hall is the first Recruitment Hall among them, the one the camera starts on
/// (`Scenario::primary_structure`). Each of its units that takes Cults Unit Control embodies as many Recruits of that
/// Hall as its cost (`Object::lineage`: one entry per Recruit embodied), so each of the six starting Recruits has
/// one entry, the Hall. Wherever the layout puts its Hall, before its units or after them, the lineage is the same.
/// A layout without a Hall (a Custom map's, which may hold anything) gives its units no lineage: no Hall produced
/// them, and their whole cost is orphaned (**Placeholder (Q0101 point 2)**), as a pre-placed Cults unit's is.
fn belong_to_their_hall(objects: &mut [StartObject], first: usize) {
    let layout = objects.get_mut(first..).unwrap_or_default();
    let Some(hall) = layout.iter().position(|o| recruitment::def_of(o.kind).is_some()) else {
        return;
    };
    for o in layout {
        let cost = ArmyResource::CultsUnitControl.cost_of(o.kind);
        o.lineage = (0..cost).map(|_| first + hall).collect();
    }
}
