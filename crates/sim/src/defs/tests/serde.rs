//! The defs are plain data (plan decision T54): every kind's def and every unit base's goes through JSON and comes back
//! the same, so that the data files of playtest can load them without a refactor.

use crate::defs::{ObjectDef, ObjectKind, UnitBase, UnitBaseDef};

#[test]
fn every_kinds_def_round_trips_through_json() {
    for &kind in ObjectKind::ALL {
        let def = kind.def();
        let json = serde_json::to_string(def).unwrap();
        let back: ObjectDef = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{kind:?}: {e} in {json}"));
        assert_eq!(&back, def, "{kind:?}");
        // Nothing is lost on the way: the def read back writes the same text.
        assert_eq!(serde_json::to_string(&back).unwrap(), json, "{kind:?}");
    }
}

#[test]
fn every_unit_bases_def_round_trips_through_json() {
    for base in UnitBase::ALL {
        let def = base.def();
        let json = serde_json::to_string(def).unwrap();
        let back: UnitBaseDef = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{base:?}: {e} in {json}"));
        assert_eq!(&back, def, "{base:?}");
    }
}

/// The data reads as the design's words: a def's fields and variants keep their names in the file.
#[test]
fn a_def_reads_as_data() {
    let json = serde_json::to_value(ObjectKind::Barracks.def()).unwrap();
    assert_eq!(json["name"], "Barracks");
    assert_eq!(json["faction"], "Gdo");
    assert_eq!(json["footprint"]["Tiles"]["w"], 3);
    let recipes = &json["structure"]["production"]["recipes"];
    assert_eq!(recipes[0]["kind"], "Peacekeeper");
    assert_eq!(recipes[0]["cost"], 50);
    assert_eq!(json["structure"]["production"]["exits"]["Side"], "B");
}
