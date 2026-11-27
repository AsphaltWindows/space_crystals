use super::*;
use crate::defs::{Footprint, ProductionDef, Recipe, Tech, TechDef, UnitDef};

fn unit(rules: &mut Ruleset) -> &mut UnitDef {
    rules.defs.get_mut(ObjectKind::Peacekeeper).unit.as_mut().unwrap()
}

fn barracks(rules: &mut Ruleset) -> &mut ProductionDef {
    let structure = rules.defs.get_mut(ObjectKind::Barracks).structure.as_mut().unwrap();
    structure.production.as_mut().unwrap()
}

fn tech(rules: &mut Ruleset) -> &mut TechDef {
    rules.factions.cults.techs.get_mut(Tech::ImprovedMetabolismL2)
}

/// The built-in rules are the def modules' statics and the tuning modules' `BUILTIN`s, and the sim reads them.
#[test]
fn the_builtin_rules_are_the_statics() {
    let builtin = Ruleset::builtin();
    for &kind in ObjectKind::ALL {
        assert_eq!(builtin.def(kind), kind.builtin_def(), "{kind:?}");
        assert_eq!(builtin.defs[kind].kind, kind);
        assert_eq!(rules().def(kind), kind.builtin_def(), "{kind:?}");
    }
    for base in UnitBase::ALL {
        assert_eq!(builtin.bases[base], *base.builtin_def(), "{base:?}");
        assert_eq!(base.def().base, base);
    }
    assert_eq!(builtin.tuning, tuning::BUILTIN);
    assert_eq!(builtin.factions, FactionRules::BUILTIN);
    assert_eq!(Tech::EnhancedConstitutionL2.def().cost, 150);
    assert_eq!(*rules(), builtin);
    assert!(Generation::BUILTIN.is_builtin() && *Generation::BUILTIN.rules() == builtin);
}

/// A ruleset reads back as it was written (plan decision T66's data files, X2's export): the defs and the bases as
/// maps by kind and by base, the rest by field name.
#[test]
fn a_ruleset_reads_back_as_it_was_written() {
    let builtin = Ruleset::builtin();
    let json = serde_json::to_value(&builtin).unwrap();
    assert_eq!(json["defs"]["Peacekeeper"]["max_hp"], 50.0);
    assert_eq!(json["bases"]["Glider"]["base"], "Glider");
    assert_eq!(json["factions"]["gdo"]["unit_control_cap"], 200);
    assert_eq!(
        json["factions"]["cults"]["techs"]["EnhancedConstitutionL1"]["cost"],
        100
    );
    assert_eq!(json["tuning"]["movement"]["stuck_ticks"], 24);
    let back: Ruleset = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(back, builtin);

    // Every kind once.
    let mut missing = json.clone();
    missing["defs"].as_object_mut().unwrap().remove("Guard");
    let error = serde_json::from_value::<Ruleset>(missing).unwrap_err().to_string();
    assert!(error.contains("Guard is missing"), "{error}");
    // No field it doesn't have.
    let mut unknown = json;
    unknown["tuning"]["combat"]["maximum_damage"] = 3.into();
    assert!(serde_json::from_value::<Ruleset>(unknown).is_err());
}

/// `structure`'s rule: numbers apply live, the rest is structure, but for the footprints' sizes (structure) and the
/// fields read only as something starts (not).
#[test]
fn only_numbers_change_without_a_new_game() {
    let builtin = Ruleset::builtin();
    assert_eq!(builtin.structural_diff(&builtin), None);
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    let change = |entry, field| Some(StructuralChange { entry, field });

    // Numbers.
    assert_eq!(changed(&|r| unit(r).attack.as_mut().unwrap().damage = 99.0), None);
    assert_eq!(
        changed(&|r| r.defs.get_mut(ObjectKind::Peacekeeper).max_hp = Some(1.0)),
        None
    );
    assert_eq!(
        changed(&|r| r.defs.get_mut(ObjectKind::Peacekeeper).name = "Keeper"),
        None
    );
    assert_eq!(changed(&|r| r.tuning.movement.stuck_ticks = 2), None);
    assert_eq!(changed(&|r| r.factions.gdo.unit_control_cap = 20), None);
    assert_eq!(
        changed(&|r| r.factions.cults.techs.get_mut(Tech::ImprovedMetabolismL1).frames = 1),
        None
    );
    // What only starts read: a TechPrerequisite, a default rally point, a blast's domain.
    assert_eq!(
        changed(&|r| r.defs.get_mut(ObjectKind::Arsenal).structure.as_mut().unwrap().requires = &[]),
        None
    );
    assert_eq!(changed(&|r| barracks(r).default_rally = None), None);
    let recipes: &'static [Recipe] = Box::leak(
        (barracks(&mut Ruleset::builtin()).recipes.iter())
            .map(|recipe| Recipe {
                requires: &[ObjectKind::Arsenal],
                ..*recipe
            })
            .collect(),
    );
    assert_eq!(changed(&|r| barracks(r).recipes = recipes), None);

    // Structure: a footprint's size, a flag, an optional part, a variant, a list's length or kinds.
    assert_eq!(
        changed(&|r| r.defs.get_mut(ObjectKind::Peacekeeper).footprint = Footprint::Silhouette { w: 30.0, h: 24.0 }),
        change("Peacekeeper", "w")
    );
    assert_eq!(
        changed(&|r| r.bases.get_mut(UnitBase::Mech).crushes = false),
        change("Mech", "crushes")
    );
    assert_eq!(
        changed(&|r| unit(r).riot_shield = None),
        change("Peacekeeper", "riot_shield")
    );
    assert_eq!(
        changed(&|r| r.defs.get_mut(ObjectKind::Disciple).energy = None),
        change("Disciple", "energy")
    );
    assert_eq!(
        changed(&|r| r.tuning.paths.resources_block = false),
        change("tuning", "resources_block")
    );
    assert_eq!(
        changed(&|r| barracks(r).recipes = &barracks(&mut Ruleset::builtin()).recipes[1..]),
        change("Barracks", "kind"),
    );
    assert_eq!(
        changed(&|r| tech(r).requires = None),
        change("ImprovedMetabolismL2", "requires")
    );
}
