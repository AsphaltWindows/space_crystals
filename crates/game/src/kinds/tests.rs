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
/// Field Engineer's Construct menu's Bunker, BKR (BR is the Barracks'), and First Aid Tent, FAT. Slice M6-SK: the six
/// expansions with their ExpandMenu placeholders' labels (VT, VA, FE, NC, AA, OC), and the units by the same rules,
/// AB, LB and LBD, with three letters where two are taken: the Shadebringer's SBR (SR is the Shrine's), the
/// Subterranean Transport's STR and STB (ST is the Scout's), the Phantom's PHM (PM is the Prayer-mobile's), the
/// Specter's SPR and the Prism's PRM. Slice M6-GK: the GDO's other kinds, a one-word name's first and last letters, a
/// longer name's initials, and where those are taken, a letter more: the Factory's FY, the Airfield's AD, Advanced
/// Mechanics' AM, Propulsion Labs' PL, the Industrial Powerplant's IP, the Radar Station's RS, the Defense System's
/// DS, the Missile Turret's MTT (MT is the Militant's), the Outpost's OT, the Bunker's BKR and the First Aid Tent's FAT
/// (their entries' labels), the Supply Tower's SPT (ST is the Scout's, STR the Subterranean Transport's); the Raptor's
/// RR, the Twister's TR, the Prospector's PR, the Artillery Rig's ARG and ARD (AR is the Altar's), the Hornet's HT,
/// the Dragonfly's DY, the Drone Operator's DO, its drone's DRN (DR is the Colonists' Drone's), the Supply Chopper's
/// SCR (SC is the patch's) and the mine's ME; the Deployment Center's eight entries with their types' labels. Slice
/// M6-GF: the Twister's mines' placeholders, Lay and Buy (not the mine's ME).
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
    let gdo = &ObjectKind::ALL[ObjectKind::Factory as usize..=ObjectKind::Mine as usize];
    assert_eq!(
        gdo.iter().map(|&kind| short_label(kind)).collect::<Vec<_>>(),
        [
            "FY", "AD", "AM", "PL", "IP", "RS", "DS", "MTT", "OT", "BKR", "FAT", "SPT", "RR", "TR", "PR", "ARG", "ARD",
            "HT", "DY", "DO", "DRN", "SCR", "ME"
        ],
        "slice M6-GK's kinds"
    );
    let expansions = [
        ObjectKind::Vault,
        ObjectKind::VehicleAssembly,
        ObjectKind::Forge,
        ObjectKind::NullChamber,
        ObjectKind::AerospaceAssembly,
        ObjectKind::OrbitalCommand,
    ];
    assert_eq!(
        expansions.map(short_label),
        ["VT", "VA", "FE", "NC", "AA", "OC"],
        "slice M6-SK's expansions, each with its ExpandMenu placeholder's label"
    );
    let syndicate_units = [
        ObjectKind::AssaultBike,
        ObjectKind::Shadebringer,
        ObjectKind::LaserBattery,
        ObjectKind::LaserBatteryDeployed,
        ObjectKind::SubterraneanTransport,
        ObjectKind::SubterraneanTransportBurrowed,
        ObjectKind::Phantom,
        ObjectKind::Specter,
        ObjectKind::Prism,
    ];
    assert_eq!(
        syndicate_units.map(short_label),
        ["AB", "SBR", "LB", "LBD", "STR", "STB", "PHM", "SPR", "PRM"]
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

/// A kind's module agrees with its def: a unit's look is a unit's, or (slice M6-GK) a mine's, "closer to a permanently
/// deployed unit" (design change 0426), a structure's a structure's, a turret's a structure's that attacks (plan
/// decision T49: `StructureDef::attack`), a runway's a structure's whose aircraft take off along it (slice M5-N7:
/// `ExitRule::Runway`), (M6-GH2) the pads' a structure's its aircraft land on (`StructureDef::airfield`), a map
/// resource's its own; and a kind has a shot style exactly when it has a weapon, a unit's
/// or a structure's.
#[test]
fn every_kind_module_fits_its_def() {
    for &kind in ObjectKind::ALL {
        let (def, client) = (kind.def(), of(kind));
        let attacks = |structure: &sc_sim::StructureDef| structure.attack.is_some();
        let runway = |structure: &sc_sim::StructureDef| {
            let exits = structure.production.as_ref().map(|production| production.exits);
            matches!(exits, Some(sc_sim::defs::ExitRule::Runway { .. }))
        };
        let pads = |structure: &sc_sim::StructureDef| structure.airfield.is_some();
        let look_fits = match client.look {
            Look::Unit { .. } => def.unit.is_some() || def.category == Category::Mine,
            Look::Structure => def
                .structure
                .as_ref()
                .is_some_and(|s| !attacks(s) && !runway(s) && !pads(s)),
            Look::Turret => def.structure.as_ref().is_some_and(attacks),
            Look::Runway => def.structure.as_ref().is_some_and(|s| !attacks(s) && runway(s)),
            Look::Pads => def.structure.as_ref().is_some_and(|s| !attacks(s) && pads(s)),
            Look::CrystalPatch | Look::SupplyDeliveryStation => def.category == Category::Resource,
        };
        assert!(look_fits, "{kind:?}'s look is {:?}", client.look);
        let armed =
            def.unit.as_ref().is_some_and(|unit| unit.attack.is_some()) || def.structure.as_ref().is_some_and(attacks);
        assert_eq!(client.shot.is_some(), armed, "{kind:?}");
    }
}
