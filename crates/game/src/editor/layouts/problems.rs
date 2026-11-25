//! The Problems list's lines, which say what to fix where (slice M2-I: a new Skirmish Default map can't be played
//! until its layouts are complete, `core/map_editor.md > Starting Layouts > Skirmish Default restriction`: "The
//! editor shows what is still missing, and the map cannot be played until all layouts are complete"). A layout that
//! lacks objects gets one line that names them all and the tool that places them; an object of a layout that breaks
//! a rule gets one line that names the object and the rule; and on a symmetric map, the generated slots' lines that
//! say the same as their editable slot's are folded into it ("S2 follows"), as fixing that slot fixes them. Every
//! other problem keeps the sim's words.

use std::mem::{Discriminant, discriminant};

use sc_sim::{Faction, MapDef, ObjectKind, ScenarioProblem, SlotId};

use super::check::rule_text;
use super::{layout, layout_name, missing_text};

/// One line of the Problems list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProblemLine {
    pub text: String,
    /// The problem a click on the line focuses: an index of the list it was made from.
    pub index: usize,
}

/// What makes two slots' lines say the same thing: the layout's faction and what is wrong with it.
#[derive(Clone, PartialEq)]
enum FoldKey {
    /// The layout lacks these objects.
    Missing(Faction, Vec<(ObjectKind, u32)>),
    /// Object `index` of the layout has this kind of problem.
    Object(Faction, usize, Discriminant<ScenarioProblem>),
}

/// What one layout lacks: its slot and faction, the index of its first `LayoutMissing` problem, and each kind it
/// lacks with how many.
type Missing = ((SlotId, Faction), usize, Vec<(ObjectKind, u32)>);

struct Line {
    slot: Option<SlotId>,
    key: Option<FoldKey>,
    text: String,
    index: usize,
    /// The generated slots whose lines say the same.
    followers: Vec<SlotId>,
}

/// The lines of the Problems list for `problems`, the list [`MapDef::validate`] or `Scenario::validate` gave for
/// `map`, in its order.
pub fn problem_lines(map: &MapDef, problems: &[ScenarioProblem]) -> Vec<ProblemLine> {
    // Each layout's missing objects, in the order the list gives them, with the index of its first such problem.
    let mut missing: Vec<Missing> = Vec::new();
    for (i, problem) in problems.iter().enumerate() {
        if let ScenarioProblem::LayoutMissing {
            slot,
            faction,
            kind,
            missing: n,
        } = *problem
        {
            match missing.iter_mut().find(|(key, ..)| *key == (slot, faction)) {
                Some((_, _, kinds)) => kinds.push((kind, n)),
                None => missing.push(((slot, faction), i, vec![(kind, n)])),
            }
        }
    }
    let mut lines: Vec<Line> = Vec::new();
    for (i, problem) in problems.iter().enumerate() {
        let (slot, key, text) = match *problem {
            ScenarioProblem::LayoutMissing { slot, faction, .. } => {
                // One line per layout, where its first missing kind is listed.
                let Some((_, _, kinds)) = missing
                    .iter()
                    .find(|(key, first, _)| *key == (slot, faction) && *first == i)
                else {
                    continue;
                };
                let text = format!(
                    "{}: add {} (Layout tool, T)",
                    layout_name(slot, faction),
                    missing_text(kinds)
                );
                (Some(slot), Some(FoldKey::Missing(faction, kinds.clone())), text)
            }
            ScenarioProblem::InvalidLayoutObject {
                slot, faction, index, ..
            }
            | ScenarioProblem::LayoutPlacement {
                slot, faction, index, ..
            }
            | ScenarioProblem::LayoutExtra {
                slot, faction, index, ..
            }
            | ScenarioProblem::LayoutOverlap {
                slot, faction, index, ..
            } => {
                let name = layout(map, slot, faction)
                    .get(index)
                    .map_or("object", |o| o.kind.def().name);
                let text = format!("{}, {name}: {}", layout_name(slot, faction), rule_text(problem));
                (
                    Some(slot),
                    Some(FoldKey::Object(faction, index, discriminant(problem))),
                    text,
                )
            }
            _ => (None, None, problem.to_string()),
        };
        if let (Some(slot), Some(key)) = (slot, &key) {
            let source = map.source_slot(slot);
            if source != slot
                && let Some(line) = lines
                    .iter_mut()
                    .find(|l| l.slot == Some(source) && l.key.as_ref() == Some(key))
            {
                line.followers.push(slot);
                continue;
            }
        }
        lines.push(Line {
            slot,
            key,
            text,
            index: i,
            followers: Vec::new(),
        });
    }
    lines
        .into_iter()
        .map(|line| {
            let names: Vec<String> = line.followers.iter().map(|s| format!("S{}", s.0)).collect();
            let text = match names.len() {
                0 => line.text,
                1 => format!("{}; {} follows", line.text, names[0]),
                _ => format!("{}; {} follow", line.text, names.join(", ")),
            };
            ProblemLine {
                text,
                index: line.index,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::{BuildingOrientation, LayoutObject, MapSettings, MapType, SpawnAt, Symmetry, TilePos};

    fn new_map(slots: usize, symmetry: Symmetry) -> MapDef {
        MapSettings {
            map_type: MapType::SkirmishDefault,
            width: 32,
            height: 32,
            slots,
            symmetry,
            name: "t".into(),
            description: String::new(),
        }
        .create()
        .unwrap()
    }

    fn texts(map: &MapDef) -> Vec<String> {
        problem_lines(map, &map.validate())
            .into_iter()
            .map(|l| l.text)
            .collect()
    }

    /// A new Skirmish Default map: one line per edited layout of an implemented faction (the GDO's, the Syndicate's
    /// and the Colonists', plan decision T24, slice M4-B), saying exactly what to add, with its images folded in.
    #[test]
    fn a_new_map_says_what_to_add_where() {
        let map = new_map(2, Symmetry::Rotate180);
        assert_eq!(map.validate().len(), 18, "3 kinds missing per faction at 2 slots");
        assert_eq!(
            texts(&map),
            [
                "S1 GDO layout: add Deployment Center, Extraction Facility, Extraction Plate x2 (Layout tool, T); S2 follows",
                "S1 Syndicate layout: add Tunnel, Hideout, Agent (Layout tool, T); S2 follows",
                "S1 Colonists layout: add Egress Node, Robo Portal, Drone x4 (Layout tool, T); S2 follows"
            ]
        );
        let four = new_map(8, Symmetry::MirrorBothAxes);
        let lines = texts(&four);
        assert_eq!(lines.len(), 6, "{lines:?}");
        for (line, start, follow) in [
            (0, "S1 GDO", "S2, S3, S4"),
            (1, "S1 Syndicate", "S2, S3, S4"),
            (2, "S1 Colonists", "S2, S3, S4"),
            (3, "S5 GDO", "S6, S7, S8"),
            (4, "S5 Syndicate", "S6, S7, S8"),
            (5, "S5 Colonists", "S6, S7, S8"),
        ] {
            let text = &lines[line];
            assert!(text.starts_with(&format!("{start} layout: add ")), "{text}");
            assert!(text.ends_with(&format!("; {follow} follow")), "{text}");
        }
        // A click on a line focuses its first problem: 9 to a slot, 3 to a layout.
        let indices: Vec<usize> = problem_lines(&four, &four.validate()).iter().map(|l| l.index).collect();
        assert_eq!(indices, [0, 3, 6, 36, 39, 42]);
    }

    /// An object that breaks a rule is named with its layout and the rule.
    #[test]
    fn object_problems_name_the_object_and_the_rule() {
        let mut map = new_map(2, Symmetry::Rotate180);
        map.slot_mut(SlotId(1))
            .unwrap()
            .layouts
            .get_mut(Faction::Gdo)
            .push(LayoutObject {
                kind: ObjectKind::ExtractionFacility,
                at: SpawnAt::Tile(TilePos::new(2, 2)),
                orientation: BuildingOrientation::default(),
            });
        map.regenerate();
        let lines = texts(&map);
        assert!(
            lines.contains(
                &"S1 GDO layout, Extraction Facility: no tile is in the build area of the layout's Deployment \
                  Center; S2 follows"
                    .to_string()
            ),
            "{lines:?}"
        );
        assert!(
            lines.contains(
                &"S1 GDO layout: add Deployment Center, Extraction Plate x2 (Layout tool, T); S2 follows".to_string()
            ),
            "{lines:?}"
        );
    }

    /// Slice M4-B: the Tunnel Area's rules in the Problems list, each named with its object and the rule's text. A
    /// Syndicate layout's Tunnel 1 tile from the map's edge, whose Tier 1 area reaches off the map (design change
    /// 0242), and its Hideout outside that area.
    #[test]
    fn a_syndicate_layout_names_the_tunnel_area_rules() {
        let mut map = new_map(2, Symmetry::Rotate180);
        let laid = |kind, x, y| LayoutObject {
            kind,
            at: SpawnAt::Tile(TilePos::new(x, y)),
            orientation: BuildingOrientation::default(),
        };
        *map.slot_mut(SlotId(1)).unwrap().layouts.get_mut(Faction::Syndicate) = vec![
            laid(ObjectKind::Tunnel, 1, 6),
            laid(ObjectKind::Hideout, 12, 6),
            LayoutObject {
                kind: ObjectKind::Agent,
                at: SpawnAt::Point(TilePos::new(8, 14).center()),
                orientation: BuildingOrientation::default(),
            },
        ];
        map.regenerate();
        let lines = texts(&map);
        for expected in [
            "S1 Syndicate layout, Tunnel: the Tunnel Area reaches off the map: a Tunnel's whole Tunnel Area must be on \
             the map; S2 follows",
            "S1 Syndicate layout, Hideout: not wholly inside the Tier 1 Tunnel Area of a Tunnel of the layout; S2 \
             follows",
        ] {
            assert!(lines.contains(&expected.to_string()), "{expected:?} not in {lines:?}");
        }
        assert!(
            !lines.iter().any(|l| l.starts_with("S1 Syndicate layout: add")),
            "{lines:?}"
        );
    }
}
