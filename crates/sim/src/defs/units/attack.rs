//! A unit type's attack (`systems/combat.md > AttackAttributes`, [`AttackAttrs`]), read by the attack sequence
//! (`crate::combat`), and its Target Lock (`factions/gdo/units/rocketman.md > Target Lock`, [`TargetLockDef`]). A
//! structure that attacks has the same attributes (`StructureDef::attack`). Split off `defs/units.rs` in slice
//! M6-GK; `super` re-exports every item.

use serde::{Deserialize, Serialize};

use crate::units::SU_PER_GU;

/// `systems/combat.md > AttackType`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackType {
    /// `systems/combat.md > FullyConnected`, with its subtype.
    FullyConnected(FullyConnectedSubtype),
    /// `systems/combat.md > HeadDisjointed`.
    HeadDisjointed,
    /// `systems/combat.md > TailDisjointed`.
    TailDisjointed,
    /// `systems/combat.md > DoublyDisjointed`.
    DoublyDisjointed,
}

/// `systems/combat.md > FullyConnected > Subtype`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FullyConnectedSubtype {
    Ranged,
    /// `systems/combat.md > Melee`: "Fixed short range (adjacent contact)": its Range is always [`MELEE_CONTACT_GU`],
    /// and the ElevationModifier never adds to it (`combat::max_range`).
    Melee,
}

/// The Range of every melee attack, grid units: `systems/combat.md > FullyConnected > Melee` ("Short reach: 8 space
/// units for all melee attacks for now") and `systems/combat.md > Range Measurement`, as the Sabre Legionnaire's
/// "Range: melee (adjacent contact)". **Placeholder** (design change 0149: "may become a per-attack value later"): a
/// gap of 8 space units, the one that counts as Proximity (`tuning::orders::Orders::proximity_su`, design change 0032).
/// Exactly 0 would be fragile ("an exact 0 would fail on tiny collision gaps"): two boxes the movement step brings
/// together may end a rounding error apart. A test keeps every melee attack at this Range, with no MinRange
/// (`defs_are_consistent`).
pub const MELEE_CONTACT_GU: f32 = 8.0 / SU_PER_GU;

/// `systems/combat.md > AttackAttributes > TargetDomain`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetDomain {
    Ground,
    Air,
    Universal,
}

/// `systems/combat.md > AttackAttributes > TargetType`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TargetType {
    SingleTarget,
    /// With its `AoERadius`, grid units, and (M5-S8) the domain its blast hits ([`BlastDomain`]).
    AoE {
        radius: f32,
        blast: BlastDomain,
    },
}

/// (M5-S8) Which objects an area attack's blast hits, by their domain (`systems/combat.md > AoEDamage`; `combat::area`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlastDomain {
    /// The attack's TargetDomain: `systems/combat.md > AoEDamage`, "AoE only affects units whose domain matches the
    /// attack's TargetDomain", a Universal blast every domain. Every area attack's but the Rocketman's.
    Attack,
    /// The domain of what it was fired at, fixed as it is fired: `factions/gdo/units/rocketman.md > AttackAttributes
    /// (normal attack)`, "**Domain of the burst** (user, 2026-11-26): it keeps the domain of what it was fired at: fired
    /// at an **air** unit it hits only air units; fired at a **ground** unit or the ground, only ground (and
    /// underground) objects" (design change 0445, answering Q0173). The Rocketman's rule, not AoEDamage's.
    Target,
}

/// `systems/combat.md > AttackAttributes`, read by the attack sequence (`combat`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttackAttrs {
    pub attack_type: AttackType,
    pub target_domain: TargetDomain,
    pub target_type: TargetType,
    pub damage: f32,
    /// Grid units.
    pub range: f32,
    /// Grid units.
    pub min_range: f32,
    /// Space units per tick; only for `HeadDisjointed` and `DoublyDisjointed`.
    pub projectile_speed: Option<f32>,
    /// `systems/combat.md > AttackPhase`: the phases' durations, ticks.
    pub aim_ticks: u32,
    pub firing_ticks: u32,
    pub cooldown_ticks: u32,
    pub reload_ticks: u32,
    /// An attack whose hits spend from its object's energy pool, in surges (`crate::surge`;
    /// `factions/syndicate/units/laser_battery.md > Attack: constant beam with surges`): each hit deals what the charge
    /// it uses comes to, the next surge follows the reload without a new aim, and its unit has Amplify. `None` for
    /// every other attack, whose hits deal its Damage.
    #[serde(default)]
    pub surge: Option<SurgeDef>,
}

/// The surges of an attack ([`AttackAttrs::surge`], `crate::surge`; `factions/syndicate/units/laser_battery.md >
/// Battery`, `factions/syndicate/units/laser_battery.md > Attack: constant beam with surges`: "Damage per surge is
/// proportional to the charge used^(2/3)").
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurgeDef {
    /// Points of energy a normal surge uses if the pool holds them, else whatever is left ("a surge uses the normal
    /// surge charge if the battery holds it, otherwise whatever is left").
    pub draw: f32,
    /// A surge's damage is `constant × used^(2/3)`, `used` in points (`crate::surge::damage`).
    pub constant: f32,
}

/// (M5-K) `factions/gdo/units/rocketman.md > Target Lock (targeted ability)`: the numbers of a unit's Target Lock
/// ([`UnitDef::target_lock`]). Its locked missiles are its weapon's, homing (`combat::target_lock`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TargetLockDef {
    /// LockOnDuration, ticks: "the Rocketman aims at the target continuously for LockOnDuration".
    pub lock_on_ticks: u32,
    /// LockRange, grid units, measured as a Range is (`systems/combat.md > Range Measurement`): "Locking succeeds
    /// only if the target stays within LockRange"; "Locked missiles can be fired from anywhere within LockRange".
    pub lock_range: f32,
}
