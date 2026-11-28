use super::*;
use crate::command::Ability;
use crate::defs::UnitBase;

fn sludge() -> &'static TankDef {
    def_of(PatchKind::Sludge).expect("the Shadebringer's Sludge")
}

fn patch(kind: PatchKind, centre: Vec2, until: Tick) -> Patch {
    Patch {
        kind,
        by: ObjectId(1),
        centre,
        radius: 24.0,
        until,
    }
}

/// The Shadebringer's tank in fixed point (module docs): 100 is 6 553 600, drained 40 960 a tick, so that a full tank
/// lasts 160 ticks ("10 s of use from full"), and filled 20 480 a tick, 320 ticks from empty ("refill **5 per
/// second**"); a patch lasts 160 ticks, and the slow 32 after the last tick in sludge, 33 in all.
#[test]
fn the_tank_s_numbers_in_fixed_point_follow_the_design() {
    let def = sludge();
    assert_eq!((capacity(def), drain(def), refill(def)), (6_553_600, 40_960, 20_480));
    assert_eq!((capacity(def) / drain(def), capacity(def) / refill(def)), (160, 320));
    assert_eq!(capacity(def) % drain(def) + capacity(def) % refill(def), 0, "exactly");
    assert_eq!(life(def), 160);
    let slow = def.slow.expect("sludge slows");
    assert_eq!((slow_after(&slow), slow_duration()), (32, 33));
    assert_eq!(BuffKind::Sludge.duration(), 33);
    // A tank never takes a step of nothing, however small its numbers.
    let tiny = TankDef {
        drain: 1e-9,
        refill: 1e-9,
        life: 1e-9,
        ..*def
    };
    assert_eq!((drain(&tiny), refill(&tiny), life(&tiny)), (1, 1, 1));
}

/// The Shadebringer is the one type with tanks (`def_of` reads its), Sludge's alone until slice M6-SC: a new one has
/// them full and off, and no other object has any.
#[test]
fn the_shadebringer_is_the_one_type_with_tanks() {
    let with: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|kind| kind.def().unit.as_ref().is_some_and(|unit| !unit.tanks.is_empty()))
        .collect();
    assert_eq!(with, [ObjectKind::Shadebringer]);
    assert_eq!(fresh(ObjectKind::Shadebringer), Some(Tanks::default()));
    assert_eq!(
        (fresh(ObjectKind::AssaultBike), fresh(ObjectKind::Tunnel)),
        (None, None)
    );
    assert_eq!(
        def_of(PatchKind::Sludge),
        tank_def(ObjectKind::Shadebringer, PatchKind::Sludge)
    );
    assert_eq!(def_of(PatchKind::Smoke), None, "slice M6-SC's");
    let full = Tanks::default();
    assert!(!full.any_on() && !full.any_used());
    assert_eq!(
        (full.sludge.fraction(sludge()), full.sludge.is_empty(sludge())),
        (1.0, false)
    );
}

/// "**Infantry and non-hover vehicles** that step into sludge are **slowed**", Mechs and Drill Units on the surface
/// too; "Not affected: hover units (HoverVehicle, HoverCraft) and air units": by base and domain, of the kinds there
/// are. A Drill Unit's base is underground until slice M6-ST1 surfaces it ("unaffected while underground").
#[test]
fn sludge_slows_the_ground_units_but_hover_units_and_aircraft() {
    let slowed = |base: UnitBase| {
        let kinds: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
            .filter(|kind| kind.def().unit.as_ref().is_some_and(|unit| unit.base == base))
            .collect();
        let each: Vec<bool> = kinds.iter().map(|&kind| slows(kind)).collect();
        assert!(each.windows(2).all(|pair| pair[0] == pair[1]), "{base:?}: {kinds:?}");
        each.first().copied()
    };
    let rows: Vec<(UnitBase, Option<bool>)> = UnitBase::ALL.into_iter().map(|base| (base, slowed(base))).collect();
    assert_eq!(
        rows,
        [
            (UnitBase::LightInfantry, Some(true)),
            (UnitBase::HeavyInfantry, Some(true)),
            (UnitBase::WheeledVehicle, Some(true)),
            (UnitBase::TrackedVehicle, Some(true)),
            (UnitBase::DrillUnit, Some(false)),
            (UnitBase::HoverVehicle, Some(false)),
            (UnitBase::Mech, None),
            (UnitBase::HoverCraft, Some(false)),
            (UnitBase::Glider, Some(false)),
        ],
        "no type is a Mech yet"
    );
    assert!(!slows(ObjectKind::Tunnel), "a building");
    assert!(slows(ObjectKind::Shadebringer), "not immune");
}

/// A unit is in a patch when its centre is in the disc, or on its edge (module docs): at 24 su from a 48 su patch's
/// centre, not a 64th more.
#[test]
fn a_patch_covers_its_disc_and_its_edge() {
    let p = patch(PatchKind::Sludge, Vec2::new(100.0, 100.0), 10);
    assert!(p.covers(Vec2::new(100.0, 100.0)));
    assert!(p.covers(Vec2::new(124.0, 100.0)));
    assert!(p.covers(Vec2::new(100.0, 76.0)));
    assert!(!p.covers(Vec2::new(124.0 + 1.0 / 64.0, 100.0)));
    assert!(!p.covers(Vec2::new(117.0, 117.0)), "past the edge on the diagonal");
}

/// The store finds a patch from any point it covers, those in the cells round its centre's included, and only of its
/// kind; it counts its patches by kind.
#[test]
fn the_store_finds_a_patch_from_the_cells_round_it() {
    let mut patches = Patches::default();
    assert!(patches.is_empty() && !patches.covers(PatchKind::Sludge, Vec2::ZERO));
    // Just inside the corner of cell (1, 1), so that its disc reaches into the three cells round that corner.
    let centre = Vec2::new(CELL + 4.0, CELL + 4.0);
    patches.add(patch(PatchKind::Sludge, centre, 10));
    assert_eq!(
        (
            patches.len(),
            patches.count(PatchKind::Sludge),
            patches.count(PatchKind::Smoke)
        ),
        (1, 1, 0)
    );
    for point in [
        centre,
        Vec2::new(CELL - 10.0, CELL + 4.0),
        Vec2::new(CELL + 4.0, CELL - 10.0),
        Vec2::new(CELL - 8.0, CELL - 8.0),
    ] {
        assert!(patches.covers(PatchKind::Sludge, point), "{point}");
        assert!(!patches.covers(PatchKind::Smoke, point), "{point}");
    }
    assert!(!patches.covers(PatchKind::Sludge, Vec2::new(CELL - 21.0, CELL + 4.0)));
    // A second patch far off, and one wider than the map's cells: each found where it lies.
    patches.add(patch(PatchKind::Smoke, Vec2::new(5000.0, 300.0), 10));
    patches.add(Patch {
        radius: 100_000.0,
        ..patch(PatchKind::Sludge, Vec2::new(-3000.0, 9000.0), 10)
    });
    assert_eq!((patches.len(), patches.count(PatchKind::Sludge)), (3, 2));
    assert!(patches.covers(PatchKind::Smoke, Vec2::new(5010.0, 290.0)));
    assert!(
        patches.covers(PatchKind::Sludge, Vec2::new(40_000.0, 40_000.0)),
        "the wide one"
    );
}

/// A patch is on the ground at the end of every tick before its `until`, and its step's expiry in that tick removes it;
/// laid again, it lasts from the tick it is laid again (module docs). Only its layer's patch at its centre is.
#[test]
fn patches_expire_in_their_tick_and_last_on_when_laid_again() {
    let mut patches = Patches::default();
    let at = Vec2::new(300.0, 300.0);
    patches.add(patch(PatchKind::Sludge, at, 160));
    patches.add(patch(PatchKind::Sludge, Vec2::new(324.0, 300.0), 170));
    patches.expire(159);
    assert_eq!(patches.len(), 2);
    assert!(
        !patches.renew(ObjectId(2), PatchKind::Sludge, at, 200),
        "another layer's"
    );
    assert!(!patches.renew(ObjectId(1), PatchKind::Smoke, at, 200), "another kind");
    assert!(
        !patches.renew(ObjectId(1), PatchKind::Sludge, at + Vec2::X, 200),
        "another place"
    );
    patches.expire(160);
    assert_eq!(patches.len(), 1, "the first patch's tick");
    assert!(!patches.covers(PatchKind::Sludge, at - Vec2::X * 10.0));
    let last = Vec2::new(324.0, 300.0);
    assert!(patches.renew(ObjectId(1), PatchKind::Sludge, last, 400));
    patches.expire(399);
    assert_eq!(patches.iter().map(|p| p.until).collect::<Vec<_>>(), [400]);
    patches.expire(400);
    assert!(patches.is_empty());
    assert_eq!((patches.count(PatchKind::Sludge), patches.reach), (0, 0.0));
}

/// The store saves as one list, cell by cell, each cell's in the order laid, and loads as it was: the same patches, in
/// the same order, the same hash.
#[test]
fn the_store_saves_as_one_list_and_loads_alike() {
    let mut patches = Patches::default();
    for (i, x) in [900.0, 10.0, 940.0, 20.0].into_iter().enumerate() {
        patches.add(patch(PatchKind::Sludge, Vec2::new(x, 50.0), 100 + i as Tick));
    }
    let order: Vec<Tick> = patches.iter().map(|p| p.until).collect();
    assert_eq!(order, [101, 103, 100, 102], "cell by cell, each in the order laid");
    let json = serde_json::to_string(&patches).unwrap();
    let back: Patches = serde_json::from_str(&json).unwrap();
    assert_eq!(back, patches);
    let hash = |patches: &Patches| {
        let mut h = StateHasher::new();
        patches.hash(&mut h);
        h.finish()
    };
    assert_eq!(hash(&back), hash(&patches));
    assert!(json.starts_with('['), "{json}");
}

/// A tank full and off leaves its unit's hash and file alike (module docs): a unit's tanks hash only what isn't so,
/// each under its sub-tag, so that a Cover (slice M6-SC) never used changes nothing.
#[test]
fn a_tank_full_and_off_is_left_out() {
    let hash = |tanks: &Tanks| {
        let mut h = StateHasher::new();
        tanks.hash(&mut h);
        h.finish()
    };
    let full = Tanks::default();
    assert_eq!(hash(&full), StateHasher::new().finish());
    assert_eq!(serde_json::to_string(&full).unwrap(), "{}");
    let mut used = full;
    used.sludge.used = 40_960;
    assert_ne!(hash(&used), hash(&full));
    let mut smoke = full;
    smoke.smoke.used = 40_960;
    assert_ne!(hash(&smoke), hash(&used), "each under its sub-tag");
    assert_eq!(serde_json::to_string(&used).unwrap(), r#"{"sludge":{"used":40960}}"#);
    let back: Tanks = serde_json::from_str(r#"{"sludge":{"used":40960}}"#).unwrap();
    assert_eq!(back, used);
}

/// A toggle's press says which tank it switches, and which way ([`Ability::toggle`], [`Ability::switching`]).
#[test]
fn each_toggle_switches_its_own_tank() {
    for kind in PatchKind::ALL {
        for on in [false, true] {
            assert_eq!(Ability::switching(kind, on).toggle(), Some((kind, on)));
        }
    }
    assert_eq!(
        Ability::switching(PatchKind::Sludge, true),
        Ability::Sludge { on: true }
    );
    assert_eq!(
        Ability::switching(PatchKind::Smoke, false),
        Ability::Cover { on: false }
    );
    assert_eq!(Ability::Mirage.toggle(), None);
}

/// "Entering the Tunnel Network switches both toggles off: it comes out with both off" (design change 0425,
/// `Sim::take_off_map`): both go off and their trails end, each tank keeping its charge.
#[test]
fn entering_the_network_switches_both_toggles_off() {
    let on = |used| Tank {
        used,
        on: true,
        laid: Some(Vec2::new(10.0, 20.0)),
    };
    let mut tanks = Tanks {
        sludge: on(40_960),
        smoke: on(81_920),
    };
    assert!(tanks.any_on());
    tanks.switch_off();
    let off = |used| Tank {
        used,
        on: false,
        laid: None,
    };
    assert_eq!(
        tanks,
        Tanks {
            sludge: off(40_960),
            smoke: off(81_920)
        }
    );
}
