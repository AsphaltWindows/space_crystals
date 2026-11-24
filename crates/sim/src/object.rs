//! Object instances (`core/entities.md > Object Instance`).
//!
//! An `Object` is a plain struct: common fields plus one `Option` per capability.
//! New capabilities (movement, weapon, production, …) are added as new optional
//! fields with their own module, not as separate stores.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::combat::AttackState;
use crate::defs::{Footprint, ObjectDef, ObjectKind};
use crate::economy::Extractor;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::Mover;
use crate::orders::Orders;
use crate::production::Production;
use crate::riot_shield::RiotShield;
use crate::scenario::BuildingOrientation;
use crate::units::{SU_PER_GU, TilePos};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    /// Center position, space units.
    pub pos: Vec2,
    /// `pos` at the start of the most recent tick (for client-side interpolation).
    pub prev_pos: Vec2,
    /// Radians. 0 = east (+x); positive = clockwise on screen (+y is south).
    pub facing: f32,
    pub prev_facing: f32,
    /// Current HP of destructible objects. Fractional (`systems/combat.md > MinimumDamage`).
    pub hp: Option<f32>,
    /// Footprint of tile-based objects (structures, resources).
    pub placement: Option<Placement>,
    /// Resource node state (`core/map.md > Resource`).
    pub resource: Option<ResourceState>,
    /// Units: the command queue and the current order (`systems/controls/commands.md > Unit Command`).
    pub orders: Option<Orders>,
    /// Units: movement state (`docs/movement.md > 3`).
    pub mover: Option<Mover>,
    /// Units that can attack: the attack sequence (`systems/combat.md`; `combat`).
    pub attack: Option<AttackState>,
    /// Units that carry a Riot Shield: its mode (`factions/gdo/units.md > Peacekeeper > Riot Shield`;
    /// `riot_shield`).
    pub riot_shield: Option<RiotShield>,
    /// A structure that mines, standing on a Space Crystals patch: the patch and its mining cycle
    /// (`factions/gdo/economy.md > ExtractionPlate`; `economy::mining`). `None` for everything else, a plate on bare
    /// ground included.
    pub extractor: Option<Extractor>,
    /// A structure that produces units: its queue, the progress of the unit in production and its rally point
    /// (`factions/gdo/structures.md > Barracks > BarracksInstanceState`; `production`). Optional in files, so that
    /// states saved before it existed still load.
    #[serde(default)]
    pub production: Option<Production>,
}

impl Object {
    pub fn def(&self) -> &'static ObjectDef {
        self.kind.def()
    }
}

/// `core/entities.md > Structure Instance`: rotation in 90° steps, plus flips.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

/// Where a tile-based object sits.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placement {
    /// Top-left tile of the footprint.
    pub origin: TilePos,
    /// Footprint size in tiles **after** rotation (90°/270° swap w and h).
    pub w: u32,
    pub h: u32,
    pub rotation: Rotation,
    pub flip_h: bool,
    pub flip_v: bool,
}

impl Placement {
    /// Unrotated placement of `footprint` with its top-left at `origin`.
    /// Panics for unit silhouettes (units have no tile placement).
    pub fn new(origin: TilePos, footprint: Footprint) -> Self {
        let Footprint::Tiles { w, h } = footprint else {
            panic!("only tile footprints can be placed");
        };
        Self {
            origin,
            w,
            h,
            rotation: Rotation::R0,
            flip_h: false,
            flip_v: false,
        }
    }

    /// Center of the footprint, space units.
    pub fn center(&self) -> Vec2 {
        self.origin.corner() + Vec2::new(self.w as f32, self.h as f32) * (SU_PER_GU / 2.0)
    }

    /// All tiles covered, row by row.
    pub fn tiles(&self) -> impl Iterator<Item = TilePos> + '_ {
        let o = self.origin;
        (0..self.h as i32).flat_map(move |dy| (0..self.w as i32).map(move |dx| TilePos::new(o.x + dx, o.y + dy)))
    }

    /// Whether it covers a tile that `other` covers.
    pub fn overlaps(&self, other: &Placement) -> bool {
        // 64-bit ends: no overflow, wherever a footprint lies.
        let span = |start: i32, len: u32| (i64::from(start), i64::from(start) + i64::from(len));
        let apart = |(a0, a1): (i64, i64), (b0, b1): (i64, i64)| a1 <= b0 || b1 <= a0;
        !(apart(span(self.origin.x, self.w), span(other.origin.x, other.w))
            || apart(span(self.origin.y, self.h), span(other.origin.y, other.h)))
    }

    /// How it is turned (`core/entities.md > Structure Type`).
    pub fn orientation(&self) -> BuildingOrientation {
        BuildingOrientation {
            rotation: self.rotation,
            flip_h: self.flip_h,
            flip_v: self.flip_v,
        }
    }
}

/// State of map resources (`core/map.md`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResourceState {
    /// `core/map.md > SpaceCrystalsPatch > RemainingAmount`.
    CrystalPatch { remaining: u32 },
    /// `core/map.md > SupplyDeliveryStation`. Delivery countdown only runs while
    /// `current_supplies == 0`.
    SupplyStation {
        delivery_size: u32,
        /// Ticks between deliveries (`DeliveryInterval`).
        delivery_interval: u32,
        current_supplies: u32,
        /// Ticks until the next delivery (`NextDeliveryTime`); `None` while supplies remain.
        next_delivery_in: Option<u32>,
    },
}
