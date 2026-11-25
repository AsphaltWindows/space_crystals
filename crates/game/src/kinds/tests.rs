use sc_sim::Category;

use super::*;
use crate::command_ui::interface::templates::Planned;

/// Slice M3-J3 (M3-K2's walkthrough): every type's short label is its own, and none is a planned unit's or
/// building's button label ("SC" was both the crystal patch's and the Scout button's); the Peacekeeper's two
/// types have two-letter labels of their own. Slice M4-A: the Agent's AG, the Hideout's HI, and a Tunnel's by
/// tier, T1 to T3, T1 its type's. Slice M4-K: the Optics Lab's OL and the Chemical Plant's CP, their ExpandMenu
/// placeholders' labels before them. Slice M4-S1: the Laser Turret's LT, which its placeholder entry still in the
/// ExpandMenu (until slice M4-S2) shares, being the same type's.
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
        ObjectKind::LaserTurret,
    ];
    assert_eq!(syndicate.map(short_label), ["AG", "T1", "HI", "OL", "CP", "LT"]);
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
