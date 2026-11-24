//! The rules of the Starting Layouts, whoever plays (slice M2-F1; `core/map_editor.md > Starting Layouts`): every
//! layout of every slot and faction is valid on its own (`> Overlay editing`: "each faction's layout only needs to be
//! valid on its own"), with "that faction's normal in-game placement rules, minus visibility" (`> Layout validation`,
//! design change 0097), and on a Skirmish Default map it holds exactly its faction's Starting Conditions (`> Skirmish
//! Default restriction`, design changes 0009 and 0012). Layouts of different factions may overlap. The layouts of the
//! players of one game must also keep apart from each other, which `Scenario::start_objects` checks when it starts.

use glam::Vec2;

use super::{SpawnSpot, place_layout_object};
use crate::defs::{Category, ObjectKind};
use crate::object::Placement;
use crate::paths::misplaced;
use crate::placement::{Occupant, check_layout};
use crate::scenario::mapdef::{MapDef, MapType};
use crate::scenario::problems::ScenarioProblem;
use crate::scenario::starting::restriction;

impl MapDef {
    /// Checks every Starting Layout into `problems`: slot by slot, each slot's layouts in `Faction::ALL` order. For
    /// each layout, its objects' problems in index order, and for each object in this order:
    /// - on a Skirmish Default map, whether it is beyond its faction's Starting Conditions
    ///   ([`ScenarioProblem::LayoutExtra`]);
    /// - whether it fits the map and its ground ([`ScenarioProblem::InvalidLayoutObject`]: on the map, the right kind
    ///   of location, flat ground under a structure, not a resource, which a layout's owner would own), and if it
    ///   does, for a structure, the placement rules in Layout mode (`placement::check_layout`, design change 0097;
    ///   [`ScenarioProblem::LayoutPlacement`]): among the map's pre-placed objects and the rest of its layout, and
    ///   within the build area of the layout's own Deployment Center;
    /// - for a unit, the unit-start rule (`paths::misplaced`, `docs/movement.md > 3`; design change 0097: "starting
    ///   units can't stand on structures or impassable tiles"), against the map's pre-placed objects and the layout's
    ///   objects before it ([`ScenarioProblem::InvalidLayoutObject`]). A structure over a unit is the structure's
    ///   placement problem ([`crate::TileProblem::Unit`]), whatever their order.
    ///
    /// Then, on a Skirmish Default map, the kinds the layout of an implemented faction lacks
    /// ([`ScenarioProblem::LayoutMissing`]). `spots` are where the map's pre-placed objects stand, `None` for those
    /// that can't be placed; all of them count, whoever owns them.
    pub(super) fn check_layouts(&self, spots: &[Option<SpawnSpot>], problems: &mut Vec<ScenarioProblem>) {
        let placed: Vec<(ObjectKind, Vec2, Option<&Placement>)> = self
            .spawns
            .iter()
            .zip(spots)
            .filter_map(|(s, spot)| {
                let (pos, placement) = spot.as_ref()?;
                Some((s.kind, *pos, placement.as_ref()))
            })
            .collect();
        let others: Vec<Occupant> = placed
            .iter()
            .map(|&(kind, pos, placement)| Occupant {
                kind,
                pos,
                placement: placement.copied(),
            })
            .collect();
        let skirmish = self.map_type == MapType::SkirmishDefault;
        for (slot, s) in self.slot_ids().zip(&self.slots) {
            for (faction, objects) in s.layouts.iter() {
                // Each object's problems with its index, in the order they are found; sorted by index below.
                let mut found: Vec<(usize, ScenarioProblem)> = Vec::new();
                let mut missing = Vec::new();
                if skirmish {
                    let (extra, lacking) = restriction(slot, faction, objects);
                    found.extend(extra);
                    missing = lacking;
                }
                // The objects that fit the map, each with its index in the layout.
                let mut laid: Vec<(usize, Occupant)> = Vec::new();
                for (index, o) in objects.iter().enumerate() {
                    match place_layout_object(&self.terrain, o.kind, o.at, o.orientation) {
                        Ok((pos, placement)) => laid.push((
                            index,
                            Occupant {
                                kind: o.kind,
                                pos,
                                placement,
                            },
                        )),
                        Err(problem) => found.push((
                            index,
                            ScenarioProblem::InvalidLayoutObject {
                                slot,
                                faction,
                                index,
                                kind: o.kind,
                                problem,
                            },
                        )),
                    }
                }
                let layout: Vec<Occupant> = laid.iter().map(|&(_, o)| o).collect();
                for (j, &(index, o)) in laid.iter().enumerate() {
                    if let Some(check) = check_layout(&self.terrain, others.iter().copied(), &layout, j)
                        && !check.is_valid()
                    {
                        found.push((
                            index,
                            ScenarioProblem::LayoutPlacement {
                                slot,
                                faction,
                                index,
                                kind: o.kind,
                                check,
                            },
                        ));
                    }
                }
                // Only a layout with units needs the unit-start rule, whose grids cost a build each.
                if laid.iter().any(|(_, o)| is_unit(o.kind)) {
                    let all = placed
                        .iter()
                        .copied()
                        .chain(laid.iter().map(|(_, o)| (o.kind, o.pos, o.placement.as_ref())));
                    for (n, problem) in misplaced(&self.terrain, all) {
                        let Some(&(index, o)) = n.checked_sub(placed.len()).and_then(|j| laid.get(j)) else {
                            continue; // a pre-placed object's, which the map's own check reports
                        };
                        if is_unit(o.kind) {
                            found.push((
                                index,
                                ScenarioProblem::InvalidLayoutObject {
                                    slot,
                                    faction,
                                    index,
                                    kind: o.kind,
                                    problem,
                                },
                            ));
                        }
                    }
                }
                found.sort_by_key(|&(index, _)| index);
                problems.extend(found.into_iter().map(|(_, problem)| problem));
                problems.extend(missing);
            }
        }
    }
}

fn is_unit(kind: ObjectKind) -> bool {
    kind.def().category == Category::Unit
}
