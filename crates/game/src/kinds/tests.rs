use sc_sim::Category;

use super::*;
use crate::command_ui::interface::templates::Planned;

/// Slice M3-J3 (M3-K2's walkthrough): every type's short label is its own, and none is a planned unit's or
/// building's button label ("SC" was both the crystal patch's and the Scout button's); the Peacekeeper's two
/// types have two-letter labels of their own. Slice M4-A: the Agent's AG, the Hideout's HI, and a Tunnel's by
/// tier, T1 to T3, T1 its type's. Slice M4-K: the Optics Lab's OL and the Chemical Plant's CP, their ExpandMenu
/// placeholders' labels before them; slice M4-U the Grenadier's GR, its Hideout placeholder's. Slice M4-S1: the Laser
/// Turret's LT, which its placeholder entry still in the ExpandMenu (until slice M4-S2) shares, being the same type's.
/// Slice M5-A: the Cults' RT, MT, GU, RH, SE and AY, by `Planned::label`'s rules, but for the Gunner's GU (its first
/// and last letters are the Grenadier's GR). Slice M5-G2: the Cults' buildings with their Construct menus' entries'
/// labels (`kinds::cults`), and the Disciple's DE, the Seeker's SKR, the Sparrow's SW, the Hand of Heaven's HOH and the
/// Prayer-mobile's PM. Slice M5-N9: the Career Campus's CC and the Field Engineer's FEN (FE is the Forge's), and the
/// Field Engineer's Construct menu's Bunker, BKR (BR is the Barracks'), and First Aid Tent, FAT.
#[test]
fn every_label_is_its_own() {
    let in_game = |planned: &Planned| ObjectKind::ALL.iter().any(|kind| kind.def().name == planned.name());
    for planned in Planned::ALL.iter().filter(|planned| in_game(planned)) {
        let kind = ObjectKind::ALL.iter().find(|kind| kind.def().name == planned.name());
        assert_eq!(
            kind.map(|&kind| short_label(kind)),
            Some(planned.label()),
            "{planned:?}"
        );
    }
    let labels: Vec<&str> = (ObjectKind::ALL.iter().map(|&kind| short_label(kind)))
        .chain([2, 3].map(tier_label))
        .chain(
            Planned::ALL
                .iter()
                .filter(|planned| !in_game(planned))
                .map(|planned| planned.label()),
        )
        .collect();
    let syndicate = [
        ObjectKind::Agent,
        ObjectKind::Tunnel,
        ObjectKind::Hideout,
        ObjectKind::OpticsLab,
        ObjectKind::ChemicalPlant,
        ObjectKind::Grenadier,
        ObjectKind::LaserTurret,
    ];
    assert_eq!(syndicate.map(short_label), ["AG", "T1", "HI", "OL", "CP", "GR", "LT"]);
    let cults = [
        ObjectKind::Recruit,
        ObjectKind::Militant,
        ObjectKind::Gunner,
        ObjectKind::RecruitmentHall,
        ObjectKind::Storage,
        ObjectKind::Armory,
    ];
    assert_eq!(cults.map(short_label), ["RT", "MT", "GU", "RH", "SE", "AY"]);
    let buildings = [
        ObjectKind::Gunworks,
        ObjectKind::Garrison,
        ObjectKind::ChopShop,
        ObjectKind::MachineShop,
        ObjectKind::Airstrip,
        ObjectKind::ScrapHangar,
        ObjectKind::Altar,
        ObjectKind::Temple,
        ObjectKind::Cathedral,
        ObjectKind::Shrine,
    ];
    assert_eq!(
        buildings.map(short_label),
        ["GS", "GA", "CS", "MS", "AP", "SH", "AR", "TE", "CL", "SR"],
        "slice M5-G2's buildings, each with its Construct menus' entry's label"
    );
    let units = [
        ObjectKind::Disciple,
        ObjectKind::Seeker,
        ObjectKind::Sparrow,
        ObjectKind::HandOfHeaven,
        ObjectKind::PrayerMobile,
    ];
    assert_eq!(units.map(short_label), ["DE", "SKR", "SW", "HOH", "PM"]);
    assert_eq!(
        [ObjectKind::CareerCampus, ObjectKind::FieldEngineer].map(short_label),
        ["CC", "FEN"]
    );
    assert_eq!(
        [Planned::Bunker, Planned::FirstAidTent].map(|p| p.label()),
        ["BKR", "FAT"]
    );
    assert_eq!([0, 1, 2, 3, 4].map(tier_label), ["T1", "T1", "T2", "T3", "T1"]);
    let tier = |tier| object_label(ObjectKind::Tunnel, Some(TunnelState::at(tier)));
    assert_eq!((tier(3), object_label(ObjectKind::Agent, None)), ("T3", "AG"));
    let distinct: std::collections::BTreeSet<&str> = labels.iter().copied().collect();
    assert_eq!(distinct.len(), labels.len(), "{labels:?}");
    assert_eq!(
        [ObjectKind::Peacekeeper, ObjectKind::PeacekeeperShielded].map(short_label),
        ["PK", "PS"]
    );
    assert_eq!(
        (short_label(ObjectKind::CrystalPatch), Planned::Scout.label()),
        ("SC", "ST")
    );
}

/// A kind's module agrees with its def: a unit's look is a unit's, a structure's a structure's, a turret's a
/// structure's that attacks (plan decision T49: `StructureDef::attack`), a map resource's its own; and a kind has a
/// shot style exactly when it has a weapon, a unit's or a structure's.
#[test]
fn every_kind_module_fits_its_def() {
    for &kind in ObjectKind::ALL {
        let (def, client) = (kind.def(), of(kind));
        let attacks = |structure: &sc_sim::StructureDef| structure.attack.is_some();
        let look_fits = match client.look {
            Look::Unit { .. } => def.unit.is_some(),
            Look::Structure => def.structure.as_ref().is_some_and(|s| !attacks(s)),
            Look::Turret => def.structure.as_ref().is_some_and(attacks),
            Look::CrystalPatch | Look::SupplyDeliveryStation => def.category == Category::Resource,
        };
        assert!(look_fits, "{kind:?}'s look is {:?}", client.look);
        let armed =
            def.unit.as_ref().is_some_and(|unit| unit.attack.is_some()) || def.structure.as_ref().is_some_and(attacks);
        assert_eq!(client.shot.is_some(), armed, "{kind:?}");
    }
}
