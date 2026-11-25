use sc_sim::Category;

use super::*;
use crate::command_ui::interface::templates::Planned;

/// Slice M3-J3 (M3-K2's walkthrough): every type's short label is its own, and none is a planned unit's or
/// building's button label ("SC" was both the crystal patch's and the Scout button's); the Peacekeeper's two
/// types have two-letter labels of their own. Slice M4-A: the Agent's AG, the Hideout's HI, and a Tunnel's by
/// tier, T1 to T3, T1 its type's. Slice M4-J2: a planned entry of a type in the game, the Hideout's Guard, shows
/// that type's own label.
#[test]
fn every_label_is_its_own() {
    let planned = Planned::ALL.into_iter().filter(|planned| planned.kind().is_none());
    let labels: Vec<&str> = (ObjectKind::ALL.iter().map(|&kind| short_label(kind)))
        .chain([2, 3].map(tier_label))
        .chain(planned.map(Planned::label))
        .collect();
    for planned in Planned::ALL {
        if let Some(kind) = planned.kind() {
            assert_eq!(planned.label(), short_label(kind), "{planned:?}");
        }
    }
    let syndicate = [ObjectKind::Agent, ObjectKind::Tunnel, ObjectKind::Hideout].map(short_label);
    assert_eq!(syndicate, ["AG", "T1", "HI"]);
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

/// A kind's module agrees with its def: a unit's look is a unit's, a structure's a structure's, a map resource's
/// its own; and a kind has a shot style exactly when it has a weapon.
#[test]
fn every_kind_module_fits_its_def() {
    for &kind in ObjectKind::ALL {
        let (def, client) = (kind.def(), of(kind));
        let look_fits = match client.look {
            Look::Unit { .. } => def.unit.is_some(),
            Look::Structure => def.structure.is_some(),
            Look::CrystalPatch | Look::SupplyDeliveryStation => def.category == Category::Resource,
        };
        assert!(look_fits, "{kind:?}'s look is {:?}", client.look);
        let armed = def.unit.as_ref().is_some_and(|unit| unit.attack.is_some());
        assert_eq!(client.shot.is_some(), armed, "{kind:?}");
    }
}
