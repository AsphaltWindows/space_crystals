use super::*;
use crate::defs::{Footprint, ProductionDef, Recipe, StockDef, Tech, TechDef, UnitDef};

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
    assert_eq!(builtin.non_finite(), None);
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    let change = |entry, field| Some(RuleField { entry, field });

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

/// (M6-GD) The Drone Operator's stock and its drone's tether (`crate::stock`, `crate::tether`): their numbers apply
/// live (the stock's size, start, price and frames, the leash and the battery); having one, and what a stock holds, are
/// structure.
#[test]
fn a_stock_and_a_tether_are_structure_and_their_numbers_live() {
    let builtin = Ruleset::builtin();
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    let change = |entry, field| Some(RuleField { entry, field });
    fn operator(rules: &mut Ruleset) -> &mut UnitDef {
        rules.defs.get_mut(ObjectKind::DroneOperator).unit.as_mut().unwrap()
    }
    fn drone(rules: &mut Ruleset) -> &mut UnitDef {
        rules.defs.get_mut(ObjectKind::OperatorDrone).unit.as_mut().unwrap()
    }

    assert_eq!(
        changed(&|r| {
            let stock = operator(r).stock.as_mut().unwrap();
            (stock.max, stock.start, stock.price, stock.frames) = (3, 2, 75, 200);
        }),
        None
    );
    assert_eq!(
        changed(&|r| {
            let tether = drone(r).tether.as_mut().unwrap();
            (tether.leash, tether.battery) = (10.0, 400);
        }),
        None
    );
    assert_eq!(changed(&|r| operator(r).stock = None), change("DroneOperator", "stock"));
    assert_eq!(
        changed(&|r| operator(r).stock.as_mut().unwrap().item = ObjectKind::Peacekeeper),
        change("DroneOperator", "item")
    );
    assert_eq!(changed(&|r| drone(r).tether = None), change("OperatorDrone", "tether"));
    let stock = StockDef {
        item: ObjectKind::OperatorDrone,
        max: 1,
        start: 1,
        price: 50,
        frames: 160,
    };
    assert_eq!(
        changed(&|r| unit(r).stock = Some(stock)),
        change("Peacekeeper", "stock")
    );
}

/// (M6-GA) A vehicle deploy is structure, the type it leads into included (`crate::vehicle_deploy`: a type's order and
/// its panel's R follow it), and its frames are a number that applies live.
#[test]
fn a_vehicle_deploy_is_structure_and_its_frames_live() {
    let builtin = Ruleset::builtin();
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    fn rig(rules: &mut Ruleset) -> &mut UnitDef {
        rules.defs.get_mut(ObjectKind::ArtilleryRig).unit.as_mut().unwrap()
    }
    let change = |field| {
        Some(RuleField {
            entry: "ArtilleryRig",
            field,
        })
    };
    assert_eq!(changed(&|r| rig(r).vehicle_deploy.as_mut().unwrap().frames = 96), None);
    assert_eq!(changed(&|r| rig(r).vehicle_deploy = None), change("vehicle_deploy"));
    assert_eq!(
        changed(&|r| rig(r).vehicle_deploy.as_mut().unwrap().into = ObjectKind::Peacekeeper),
        change("into")
    );
}

/// (M5-X1b) `Ruleset::non_finite` names the first number that isn't finite: an attack's NaN Damage, an infinite
/// MaxHP, a tunable; none of the built-in rules'.
#[test]
fn a_number_that_isnt_finite_is_named() {
    assert_eq!(Ruleset::builtin().non_finite(), None);
    let named = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        rules.non_finite().map(|at| (at.entry, at.field))
    };
    assert_eq!(
        named(&|r| unit(r).attack.as_mut().unwrap().damage = f32::NAN),
        Some(("Peacekeeper", "damage"))
    );
    assert_eq!(
        named(&|r| r.defs.get_mut(ObjectKind::Guard).max_hp = Some(f32::INFINITY)),
        Some(("Guard", "max_hp"))
    );
    assert_eq!(
        named(&|r| r.tuning.vision.elevation_modifier = f32::NEG_INFINITY),
        Some(("tuning", "elevation_modifier"))
    );
}

/// (M6-GH1) The Hornet's ammo and the Airfield's assignments (`crate::aircraft`): their numbers apply live (the
/// rounds, the rearm frames and the capacity, and (M6-GH2) the landing's frames and the slots' layout,
/// `crate::landing`); having them, the Airfield's pads (`ExitRule::Pad`) and the HoverCraft's braking by drag alone
/// (`UnitBaseDef::brakes_by_drag`) are structure.
#[test]
fn ammo_and_an_airfield_are_structure_and_their_numbers_live() {
    use crate::defs::{AirfieldDef, AmmoDef, ExitRule, StructureDef};
    use crate::sides::SideLetter;
    let builtin = Ruleset::builtin();
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    let change = |entry, field| Some(RuleField { entry, field });
    fn hornet(rules: &mut Ruleset) -> &mut UnitDef {
        rules.defs.get_mut(ObjectKind::Hornet).unit.as_mut().unwrap()
    }
    fn airfield(rules: &mut Ruleset) -> &mut StructureDef {
        rules.defs.get_mut(ObjectKind::Airfield).structure.as_mut().unwrap()
    }
    assert_eq!(
        changed(&|r| {
            let ammo = hornet(r).ammo.as_mut().unwrap();
            (ammo.max, ammo.rearm_ticks_per_ammo) = (3, 20);
        }),
        None
    );
    assert_eq!(changed(&|r| airfield(r).airfield.as_mut().unwrap().capacity = 6), None);
    assert_eq!(
        changed(&|r| {
            let support = airfield(r).airfield.as_mut().unwrap();
            (support.landing_ticks, support.slot_offset_su) = (8, 40.0);
        }),
        None
    );
    assert_eq!(changed(&|r| hornet(r).ammo = None), change("Hornet", "ammo"));
    let ammo = AmmoDef {
        max: 5,
        rearm_ticks_per_ammo: 32,
    };
    assert_eq!(changed(&|r| unit(r).ammo = Some(ammo)), change("Peacekeeper", "ammo"));
    assert_eq!(
        changed(&|r| airfield(r).airfield = None),
        change("Airfield", "airfield")
    );
    let room = AirfieldDef {
        capacity: 4,
        landing_ticks: 16,
        slot_offset_su: 48.0,
    };
    assert_eq!(
        changed(&|r| r
            .defs
            .get_mut(ObjectKind::Barracks)
            .structure
            .as_mut()
            .unwrap()
            .airfield = Some(room)),
        change("Barracks", "airfield")
    );
    assert_eq!(
        changed(&|r| airfield(r).production.as_mut().unwrap().exits = ExitRule::Side(SideLetter::B)),
        change("Airfield", "exits")
    );
    assert_eq!(
        changed(&|r| r.bases.get_mut(UnitBase::HoverCraft).brakes_by_drag = false),
        change("HoverCraft", "brakes_by_drag")
    );
}

/// (M6-SN) The Null Chamber's service (`StructureDef::chamber`, `network::services`): its numbers apply live (the
/// slots, the turn, and the buff's duration, sight and reduction); having one is structure, the Null Chamber's as
/// another type's.
#[test]
fn a_chamber_is_structure_and_its_numbers_live() {
    use crate::defs::{ChamberDef, StructureDef};
    let builtin = Ruleset::builtin();
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    let change = |entry, field| Some(RuleField { entry, field });
    fn structure(rules: &mut Ruleset, kind: ObjectKind) -> &mut StructureDef {
        rules.defs.get_mut(kind).structure.as_mut().unwrap()
    }
    let numbers = ChamberDef {
        slots: 2,
        turn: 3.0,
        duration: 10.0,
        sight: 1.0,
        reduction: 0.5,
    };
    assert_eq!(
        changed(&|r| structure(r, ObjectKind::NullChamber).chamber = Some(numbers)),
        None
    );
    assert_eq!(
        changed(&|r| structure(r, ObjectKind::NullChamber).chamber = None),
        change("NullChamber", "chamber")
    );
    assert_eq!(
        changed(&|r| structure(r, ObjectKind::Hideout).chamber = Some(numbers)),
        change("Hideout", "chamber")
    );
}

/// (M6-CM) The Disciple's Mirage (`crate::mirage`): its numbers apply live (its cost, its count of illusions, their
/// offset and the damage they take; the search for a blocked spot's and the cloud's linger, tunables); having it, a
/// spell of a unit's (`SpellDef::Mirage`), is structure.
#[test]
fn a_mirage_is_structure_and_its_numbers_live() {
    use crate::defs::{MirageDef, SpellDef};
    let builtin = Ruleset::builtin();
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    let change = |entry, field| Some(RuleField { entry, field });
    fn disciple(rules: &mut Ruleset) -> &mut UnitDef {
        rules.defs.get_mut(ObjectKind::Disciple).unit.as_mut().unwrap()
    }
    let mirage = *ObjectKind::Disciple
        .def()
        .unit
        .as_ref()
        .unwrap()
        .mirage()
        .expect("the Disciple casts it");
    let other = MirageDef {
        cost: 60.0,
        illusions: 3,
        offset: 1.5,
        damage_taken: 3.0,
    };
    let with = |spells: &[SpellDef], mirage: Option<MirageDef>| -> &'static [SpellDef] {
        let kept = spells.iter().filter(|spell| spell.mirage().is_none()).copied();
        Vec::leak(kept.chain(mirage.map(SpellDef::Mirage)).collect())
    };
    assert_eq!(
        changed(&|r| disciple(r).spells = with(disciple(r).spells, Some(other))),
        None,
        "its numbers"
    );
    assert_eq!(
        changed(&|r| {
            let spells = &mut r.tuning.spells;
            (
                spells.linger,
                spells.mirage_search_step_su,
                spells.mirage_search_radius_gu,
            ) = (0, 4.0, 5.0);
        }),
        None,
        "the tunables"
    );
    assert_eq!(
        changed(&|r| disciple(r).spells = with(disciple(r).spells, None)),
        change("Disciple", "spells")
    );
    assert_eq!(
        changed(&|r| unit(r).spells = with(&[], Some(mirage))),
        change("Peacekeeper", "spells")
    );
}

/// (M6-SB) The Shadebringer's tanks and the Vault's service (`crate::patches`, `network::services`): their numbers apply
/// live (a tank's capacity, drain, refill, its patches' life and width, its slow's share and time, the Vault's slots,
/// and the trail's spacing, a tunable); having a tank, and so its toggle, what it lays, whether its patches slow and
/// which bases, and having a Vault's service are structure, the Shadebringer's and the Vault's as another type's.
#[test]
fn tanks_and_a_vault_are_structure_and_their_numbers_live() {
    use crate::defs::{PatchKind, SlowDef, StructureDef, TankDef, UnitBase, VaultDef};
    let builtin = Ruleset::builtin();
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    let change = |entry, field| Some(RuleField { entry, field });
    fn shadebringer(rules: &mut Ruleset) -> &mut UnitDef {
        rules.defs.get_mut(ObjectKind::Shadebringer).unit.as_mut().unwrap()
    }
    fn structure(rules: &mut Ruleset, kind: ObjectKind) -> &mut StructureDef {
        rules.defs.get_mut(kind).structure.as_mut().unwrap()
    }
    let sludge = *ObjectKind::Shadebringer
        .def()
        .unit
        .as_ref()
        .unwrap()
        .tank(PatchKind::Sludge)
        .expect("its Sludge's tank");
    let slow = sludge.slow.expect("sludge slows");
    let with = |tanks: Vec<TankDef>| -> &'static [TankDef] { Vec::leak(tanks) };
    let numbers = TankDef {
        capacity: 50.0,
        drain: 4.0,
        refill: 20.0,
        life: 3.0,
        width: 96.0,
        slow: Some(SlowDef {
            by: 0.9,
            after: 0.5,
            ..slow
        }),
        ..sludge
    };
    assert_eq!(
        changed(&|r| shadebringer(r).tanks = with(vec![numbers])),
        None,
        "its numbers"
    );
    assert_eq!(
        changed(&|r| structure(r, ObjectKind::Vault).vault = Some(VaultDef { slots: 5 })),
        None,
        "the Vault's slots"
    );
    assert_eq!(changed(&|r| r.tuning.patches.spacing = 8.0), None, "the tunable");
    assert_eq!(
        changed(&|r| shadebringer(r).tanks = with(vec![])),
        change("Shadebringer", "tanks"),
        "its tank, and its toggle"
    );
    let smoke = TankDef {
        lays: PatchKind::Smoke,
        slow: None,
        ..sludge
    };
    assert_eq!(
        changed(&|r| shadebringer(r).tanks = with(vec![sludge, smoke])),
        change("Shadebringer", "tanks"),
        "a second tank"
    );
    assert_eq!(
        changed(&|r| shadebringer(r).tanks = with(vec![smoke])),
        change("Shadebringer", "lays")
    );
    assert_eq!(
        changed(&|r| shadebringer(r).tanks = with(vec![TankDef { slow: None, ..sludge }])),
        change("Shadebringer", "slow")
    );
    let bases = TankDef {
        slow: Some(SlowDef {
            bases: &[UnitBase::LightInfantry],
            ..slow
        }),
        ..sludge
    };
    assert_eq!(
        changed(&|r| shadebringer(r).tanks = with(vec![bases])),
        change("Shadebringer", "bases")
    );
    assert_eq!(
        changed(&|r| unit(r).tanks = with(vec![sludge])),
        change("Peacekeeper", "tanks")
    );
    assert_eq!(
        changed(&|r| structure(r, ObjectKind::Vault).vault = None),
        change("Vault", "vault")
    );
    assert_eq!(
        changed(&|r| structure(r, ObjectKind::Hideout).vault = Some(VaultDef { slots: 2 })),
        change("Hideout", "vault")
    );
}

/// (M6-ST1) A carrier's hold is structure, what it takes included (`crate::carry`: who boards, and the panel's F,
/// follow it), and its numbers apply live (its capacity, a base's places); a type's own Domain, crushing and
/// yielding over its base's are structure too (`UnitDef::domain`: its collision layer and its grids follow it).
#[test]
fn a_hold_and_a_domain_are_structure_and_the_hold_s_numbers_live() {
    use crate::defs::{CarrierDef, Domain, Places, UnitBase};
    let builtin = Ruleset::builtin();
    let changed = |change: &dyn Fn(&mut Ruleset)| {
        let mut rules = Ruleset::builtin();
        change(&mut rules);
        builtin.structural_diff(&rules)
    };
    let change = |entry, field| Some(RuleField { entry, field });
    fn transport(rules: &mut Ruleset) -> &mut UnitDef {
        rules
            .defs
            .get_mut(ObjectKind::SubterraneanTransport)
            .unit
            .as_mut()
            .unwrap()
    }
    let hold = |capacity, base, places| CarrierDef {
        capacity,
        places: Vec::leak(vec![Places { base, places }]),
    };
    assert_eq!(changed(&|r| transport(r).carries.as_mut().unwrap().capacity = 6), None);
    let heavy = |places| hold(4, UnitBase::HeavyInfantry, places);
    let light = Places {
        base: UnitBase::LightInfantry,
        places: 2,
    };
    let both = |places| CarrierDef {
        places: Vec::leak(vec![light, heavy(places).places[0]]),
        ..heavy(places)
    };
    assert_eq!(
        changed(&|r| transport(r).carries = Some(both(2))),
        None,
        "a base's places"
    );
    assert_eq!(
        changed(&|r| transport(r).carries = None),
        change("SubterraneanTransport", "carries")
    );
    assert_eq!(
        changed(&|r| transport(r).carries = Some(heavy(1))),
        change("SubterraneanTransport", "base"),
        "a base it takes no more"
    );
    assert_eq!(
        changed(&|r| unit(r).carries = Some(hold(4, UnitBase::HeavyInfantry, 1))),
        change("Peacekeeper", "carries")
    );
    assert_eq!(
        changed(&|r| transport(r).domain = None),
        change("SubterraneanTransport", "domain")
    );
    assert_eq!(
        changed(&|r| transport(r).domain = Some(Domain::Air)),
        change("SubterraneanTransport", "domain")
    );
    assert_eq!(
        changed(&|r| transport(r).crushes = Some(false)),
        change("SubterraneanTransport", "crushes")
    );
    assert_eq!(
        changed(&|r| transport(r).yields = None),
        change("SubterraneanTransport", "yields")
    );
    assert_eq!(
        changed(&|r| unit(r).domain = Some(Domain::Ground)),
        change("Peacekeeper", "domain")
    );
}
