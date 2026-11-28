//! (M6-SB) A unit's tanks (plan decision T93, `crate::patches`): what a Shadebringer lays behind it while one of its
//! toggles is on, and how much of it it carries (`factions/syndicate/units/shadebringer.md > Charges and recharging`:
//! "Sludge and Cover each have their **own, separate charge**"; "**Each charge is a tank** that **drains while its
//! toggle is on**").

use serde::{Deserialize, Serialize};

use super::UnitBase;

/// What a patch of the ground is (plan decision T93, `crate::patches`): one of the Shadebringer's two effects, each
/// laid from a tank of its own (`factions/syndicate/units/shadebringer.md`: "Has two **toggleable abilities**, Sludge
/// and Cover"), in their hotkeys' order, R then F (`> Placeholder stats`: "Hotkeys: **R** Sludge, **F** Cover"). Its
/// place in [`PatchKind::ALL`] is its tank's in a unit's `patches::Tanks`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PatchKind {
    /// `> Sludge (toggle)`: "a **trail of sludge** on the ground in its wake", which slows the units that step into
    /// it ([`SlowDef`]).
    Sludge,
    /// `> Cover (toggle)`: "a **stack/wall of smoke** in its wake", which blocks sight: slice M6-SC's.
    Smoke,
}

impl PatchKind {
    /// Every kind, in its tanks' order.
    pub const ALL: [PatchKind; 2] = [PatchKind::Sludge, PatchKind::Smoke];

    /// Its tank's place in a unit's tanks (`patches::Tanks`): its place in [`PatchKind::ALL`].
    pub const fn index(self) -> usize {
        match self {
            PatchKind::Sludge => 0,
            PatchKind::Smoke => 1,
        }
    }
}

/// (M6-SB) One of a unit's tanks (`UnitDef::tanks`, plan decision T93): what it lays while its toggle is on, how much
/// it holds, and the patches it lays (`factions/syndicate/units/shadebringer.md > Placeholder stats`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TankDef {
    /// What it lays: sludge (R), or (slice M6-SC) smoke (F). A unit has one tank of each kind at most.
    pub lays: PatchKind,
    /// How much it holds, full: "**Tanks**: 100 each".
    pub capacity: f32,
    /// How much it drains a second while its toggle is on, moving or not: "drain **10 per second** while on (10 s of
    /// use from full)".
    pub drain: f32,
    /// How much a Vault refills it a second in its owner's Tunnel Network (`network::services`): "refill **5 per
    /// second** inside the network with a Vault".
    pub refill: f32,
    /// How long each patch lasts on the ground, seconds: "each patch lasts **X = 10 s**".
    pub life: f32,
    /// The trail's width, space units: each patch a disc that wide ("trail width 48 su").
    pub width: f32,
    /// What a patch does to a unit whose centre is in it, if it slows it: sludge's ([`SlowDef`]). `None` for a patch
    /// that slows no one: (slice M6-SC) smoke's.
    pub slow: Option<SlowDef>,
}

/// (M6-SB) A patch's slow (`TankDef::slow`, `crate::patches`): `factions/syndicate/units/shadebringer.md > Sludge
/// (toggle)`, "**Infantry and non-hover vehicles** that step into sludge are **slowed by Y%** for **Z seconds**", and
/// `> Placeholder stats`, "units are slowed by **Y = 40%** for **Z = 2 s** after leaving it, and the slow **refreshes
/// while they stay in it**".
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlowDef {
    /// The share of its top speeds a slowed unit loses: "slowed by **Y = 40%**".
    pub by: f32,
    /// How long it stays slowed after it leaves the patch, seconds: "for **Z = 2 s** after leaving it".
    pub after: f32,
    /// The bases it slows, while on the ground: "**Infantry and non-hover vehicles**", "**Mechs** are slowed like
    /// vehicles", "**Drill Units** are slowed while on the surface, and unaffected while underground"; "Not affected:
    /// hover units (HoverVehicle, HoverCraft) and air units".
    #[serde(deserialize_with = "crate::defs::leak::slice")]
    pub bases: &'static [UnitBase],
}
