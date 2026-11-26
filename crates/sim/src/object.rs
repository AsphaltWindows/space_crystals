//! Object instances (`core/entities.md > Object Instance`).
//!
//! An `Object` is a plain struct: common fields plus one `Option` per capability.
//! New capabilities (movement, weapon, production, …) are added as new optional
//! fields with their own module, not as separate stores.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::augmentation::Augmentation;
use crate::combat::AttackState;
use crate::construction::Constructor;
use crate::construction::site::Construction;
use crate::conversion::Conversion;
use crate::defs::{Footprint, ObjectDef, ObjectKind};
use crate::economy::{Extractor, Worker};
use crate::ids::{ObjectId, PlayerId};
use crate::mover::Mover;
use crate::orders::Orders;
use crate::popup::PopUp;
use crate::production::Production;
use crate::progress::Progress;
use crate::riot_shield::RiotShield;
use crate::scenario::BuildingOrientation;
use crate::transform::Transform;
use crate::tunnel::TunnelState;
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
    /// Current Shields (`systems/combat.md > Shields`; plan decision T18; `shields`): `Some` for an object whose
    /// kind has MaxShields, the Colonists' units and buildings, from 0 to its maximum now (`Object::max_shields`);
    /// `None` for everything else. Fractional, as HP. Optional in files: an object saved before it existed loads with
    /// full Shields (`sim/load.rs`).
    #[serde(default)]
    pub shields: Option<f32>,
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
    /// Units that carry a Riot Shield: how far up it is, the way it goes being the unit's type
    /// (`factions/gdo/units/peacekeeper.md > Riot Shield`, design change 0173; `riot_shield`).
    pub riot_shield: Option<RiotShield>,
    /// A structure that mines, standing on a Space Crystals patch: the patch and its mining cycle
    /// (`factions/gdo/economy.md > ExtractionPlate`; `economy::mining`). `None` for everything else, a plate on bare
    /// ground included.
    pub extractor: Option<Extractor>,
    /// A structure that produces units: its queue, the progress of the unit in production and its rally point
    /// (`factions/gdo/structures/barracks.md > BarracksInstanceState`; `production`). Optional in files, so that
    /// states saved before it existed still load.
    #[serde(default)]
    pub production: Option<Production>,
    /// A structure that constructs buildings, the Deployment Center and the Extraction Facility: what it is
    /// building, or the building it holds ready to place (`factions/gdo/structures/deployment_center.md >
    /// DeploymentCenterInstanceState`, `factions/gdo/economy.md > ExtractionFacility`; `construction`). `None` for
    /// everything else. Optional in files, so that states saved before it existed still load.
    #[serde(default)]
    pub constructor: Option<Constructor>,
    /// A worker, the Drone: the load it carries, its AssignedDropOff and the patch it gathers
    /// (`factions/colonists/units/drone.md > Gathering`, `factions/colonists/units/drone.md > Assigned Drop-off`;
    /// `economy::gathering`). `None` for everything else. Optional in files, so that states saved before it existed
    /// still load.
    #[serde(default)]
    pub worker: Option<Worker>,
    /// An object transforming into its kind, a building being warped in from a Beacon: what it was, its progress
    /// and what it paid (`factions/colonists/economy.md > Warp In`, plan decision T15; `transform`, which also gives
    /// its interpolated maxima, `Object::max_hp`). `None` for everything else. Optional in files, so that states
    /// saved before it existed still load.
    #[serde(default)]
    pub transform: Option<Transform>,
    /// A Legionnaire whose form isn't its type yet: the form it has, and the frames left of its conversion under way
    /// (`factions/colonists/units/weapon_conversion.md`, plan decision T26, design change 0198; `conversion`).
    /// `None` for everything else. Optional in files, so that states saved before it existed still load, and so do
    /// those saved before design change 0198 (`conversion::saved`).
    #[serde(default, deserialize_with = "crate::conversion::saved")]
    pub conversion: Option<Conversion>,
    /// A Tunnel: its tier (`factions/syndicate/tunnels.md > Tunnel > Tiers`, plan decision T39; `tunnel`), which its
    /// MaxHP follows (`Object::max_hp`). `None` for everything else. Optional in files, and left out of them while
    /// `None`, so that states saved before it existed still load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tunnel: Option<TunnelState>,
    /// A site, a structure under construction on its site: its progress and the unit that builds it
    /// (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`, plan decision T31, slice M4-E;
    /// `construction::site`, which also gives its maximum HP by the ConstructionHP Rule, `Object::max_hp`). `None` for
    /// everything else. Optional in files, and left out of them while `None`, so that states saved before it existed
    /// still load and those without a site are as they were.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction: Option<Construction>,
    /// A neutral expansion's decay (`factions/syndicate/tunnels.md > Losing Tunnels`: "They **slowly lose HP** until
    /// destroyed", slice M4-H, `sim/tunnels.rs`): how much of its MaxHP it has lost to it, counted out exactly per tick
    /// from the tick its Tunnel fell until it is reclaimed or destroyed. `None` for everything else. Optional in files,
    /// and left out of them while `None`, so that states saved before it existed still load and those without one are
    /// as they were.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decay: Option<Progress>,
    /// A turret with a pop-up form: where it is between underground and the surface
    /// (`factions/syndicate/structures/laser_turret.md > Pop-up turret`, plan decision T50, slices M4-S2 and M5-Y;
    /// `popup`), set from its first tick, lowered. `None` for everything else. Optional in files, and left out of them
    /// while `None`, so that states saved before it existed still load and those without a turret are as they were.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub popup: Option<PopUp>,
    /// A Cults unit's lineage (plan decision T56, slice M5-A), one entry per Recruit the unit embodies (the Director's
    /// amendment to T56 for design change 0319, slice M5-C): the Recruitment Hall that produced each
    /// (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`: "Each Recruit tracks which Recruitment
    /// Hall produced it. This lineage persists when a Recruit is trained into another unit. A trained unit's Unit
    /// Control cost equals the number of Recruits consumed to produce it, with each point of cost attributed to the
    /// originating hall of the respective Recruit"). A Recruit has one entry, the Hall that made it
    /// (`Sim::release_unit`, slice M5-C; the starting Hall's for the starting Recruits, slice M5-B); a unit trained
    /// from several Recruits, a Seeker from two, has one per Recruit consumed, the same Hall perhaps twice (slice M5-F2
    /// joins the consumed Recruits' entries, `Sim::train`). Each entry is a point of Unit Control used at its Hall
    /// (`crate::recruitment::Lineages`); an entry whose Hall is gone stays, and is an orphaned point, which counts
    /// toward the global usage only (`factions/cults/structures/recruitment_hall.md > On Destruction`). Empty for
    /// everything else, and for a Cults unit that no Hall produced (a map's or a debug spawn's), whose whole cost is
    /// orphaned (design change 0344: "Recruits no Hall produced [...] are orphans from the start"). A kind change keeps
    /// it (`Sim::change_kind`). Optional in files, and left out of them while empty, so that states saved before it
    /// existed still load and those without one are as they were.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lineage: Vec<ObjectId>,
    /// (M5-F2) A Cults infantry unit's Augmentations (`factions/cults/augmentation.md > AugmentationState (per unit
    /// instance)`, `crate::augmentation`), by which the buildings that hold it choose it (design change 0337): neither
    /// for every other unit. It stays with the unit inside a building and when it is trained into infantry, and goes
    /// when it is trained into a vehicle or an aircraft (`Sim::train`). Optional in files and left out of them while
    /// it has neither, and hashed only then, so that states saved before it existed load and those without one hash as
    /// they did.
    #[serde(default, skip_serializing_if = "Augmentation::is_none")]
    pub augmentation: Augmentation,
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
