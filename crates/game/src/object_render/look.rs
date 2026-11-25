//! The look of a kind's placeholder, until the art direction (Q0001) settles it: each kind's is in its module
//! (`crate::kinds::ClientKind::look`), and `object_render` draws it, as the SelectionPanel does its portrait.

use sc_sim::ObjectKind;

/// How objects of a kind are drawn.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Look {
    /// A unit: a body of its silhouette's size and of its `shape`, in its owner's colour, with a tick showing its
    /// facing (`super::units`); its label on it if it is `tagged` (`super::UnitTag`).
    Unit { shape: UnitShape, tagged: bool },
    /// A structure: its footprint as placed, in its owner's colour, with its label (`super::structures`).
    Structure,
    /// A Space Crystals Patch's shards (`super::resources`).
    CrystalPatch,
    /// A Supply Delivery Station's shape (`super::resources`).
    SupplyDeliveryStation,
}

/// A unit's body (`super::units`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnitShape {
    /// The one body that turns with its unit's facing (`super::units::body_turns`).
    Round,
    Square,
    /// A square with its corners cut off.
    Octagon,
}

impl Look {
    /// The look of `kind`, from its module.
    pub fn of(kind: ObjectKind) -> Look {
        crate::kinds::of(kind).look
    }

    /// A unit's body; none for other objects.
    pub fn unit_shape(self) -> Option<UnitShape> {
        match self {
            Look::Unit { shape, .. } => Some(shape),
            Look::Structure | Look::CrystalPatch | Look::SupplyDeliveryStation => None,
        }
    }

    /// Whether it is a unit's that carries its label.
    pub fn tagged(self) -> bool {
        matches!(self, Look::Unit { tagged: true, .. })
    }
}
