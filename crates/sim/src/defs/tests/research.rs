//! (M5-N3) The types that research (`StructureDef::research`, plan decision T74): what `defs_are_consistent` checks of
//! them.

use crate::defs::{ObjectDef, ObjectKind, StructureDef};
use crate::economy::RATIO_ONE;

/// `defs_are_consistent` for the research of a structure type (`StructureDef::research`): its research runs on its
/// production queue (plan decision T70), which produces no unit: no recipe, no augmentation order, no recruiting. It
/// queues some, and lists each technology once, each technology's previous level among them
/// (`core/tech_and_research.md > Research > Shared Rules` 5: "a multi-level research line is a chain of separate
/// technologies, each requiring the previous level"), each costing something and taking time short enough for its progress in fixed point, behind
/// structures of the type's own faction. A type that holds units for its research takes only Recruits
/// (`core/tech_and_research.md > Research > Faction Exceptions > Cults — research is staffed by Recruits`: "Only
/// Recruits can staff research").
pub(super) fn research_is_consistent(def: &ObjectDef, structure: &StructureDef) {
    let name = def.name;
    let Some(research) = &structure.research else {
        return;
    };
    let production = (structure.production.as_ref()).unwrap_or_else(|| panic!("{name}: researches with no queue"));
    assert!(
        production.recipes.is_empty() && production.augment.is_empty() && production.recruitment.is_none(),
        "{name}: researches and produces"
    );
    assert!(research.queue > 0, "{name}: queues no research");
    assert!(!research.techs.is_empty(), "{name}: researches nothing");
    for (i, &tech) in research.techs.iter().enumerate() {
        let tech_def = tech.def();
        assert!(!research.techs[..i].contains(&tech), "{name} lists {tech:?} twice");
        if let Some(previous) = tech_def.requires {
            assert!(research.offers(previous), "{name}: {tech:?} without {previous:?}");
            assert_eq!(
                previous.def().line,
                tech_def.line,
                "{tech:?}: another line's previous level"
            );
        }
        assert!(tech_def.cost > 0, "{tech:?} is free");
        assert!(
            (1..=u32::MAX / RATIO_ONE).contains(&tech_def.frames),
            "{tech:?}: {tech_def:?}"
        );
        for &required in tech_def.requires_structures {
            let its = required.def();
            assert!(
                its.structure.is_some() && its.faction == def.faction,
                "{name}: {tech:?} requires {required:?}"
            );
        }
    }
    if let Some(holds) = &structure.holds {
        assert_eq!(
            holds.accepts,
            [ObjectKind::Recruit],
            "{name}: staffed by Recruits alone"
        );
    }
}
