//! Starting Conditions (slice M2-F1, plan decision T11): what each faction starts a game with
//! (`factions/gdo/economy.md > Starting Conditions`, `factions/colonists/economy.md > Starting Conditions`,
//! `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`), and what that means for a map's Starting Layouts
//! (`core/map_editor.md > Starting Layouts`).
//!
//! - **The table** ([`StartingConditions::of`]) lives in the sim, one for every faction (plan decision T63): the
//!   GDO's, the Colonists' (slice M3-C), the Syndicate's (slice M4-B) and the Cults' (slice M5-B, which ended T11's
//!   staging: no faction is "not implemented yet" any more).
//! - **Game start**: a player's banks start at their faction's Starting Conditions (`crate::Sim::try_new`), and their
//!   objects are those of their faction's Starting Layout at their slot (`Scenario::start_objects`), the Cults'
//!   starting units with their layout's Recruitment Hall as their lineage (`factions/cults/economy.md > Starting
//!   Conditions`).
//! - **Skirmish Default restriction** (`core/map_editor.md > Starting Layouts > Skirmish Default restriction`, design
//!   changes 0009 and 0012; checked by `MapDef::validate`): on a Skirmish Default map a layout holds only its
//!   faction's Starting Conditions objects, at most as many of each kind as they do
//!   ([`ScenarioProblem::LayoutExtra`]), and every faction's layout is complete at every start
//!   ([`ScenarioProblem::LayoutMissing`]).
//! - **Outdated layouts** (`core/map_editor.md > Outdated layouts`) are derived, never stored, so the map format stays
//!   at version 3 (T11): a layout is outdated when it holds objects but not exactly its faction's current Starting
//!   Conditions ([`StartingConditions::is_outdated`], [`MapDef::outdated_layouts`]).
//! - **The camera's start** (`core/camera.md > Starting Position`): the faction's primary starting structure
//!   ([`StartingConditions::primary`]), where the player's layout puts it ([`Scenario::primary_structure`]).

use std::collections::BTreeMap;

use glam::Vec2;

use super::mapdef::{LayoutObject, MapDef};
use super::validate::locate_oriented;
use super::{Scenario, ScenarioProblem};
use crate::defs::ObjectKind;
use crate::ids::{PlayerId, SlotId};
use crate::player::Faction;

/// A faction's Starting Conditions: the objects every Starting Layout of it holds, the banks a player of it starts
/// with, and the structure the camera starts on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartingConditions {
    /// The starting objects: each kind once, with how many, in the design's order.
    pub objects: &'static [(ObjectKind, u32)],
    /// Starting Space Crystals (`factions/gdo/overview.md > GDOResources > Space Crystals`).
    pub crystals: u32,
    /// Starting Supplies (`factions/gdo/overview.md > GDOResources > Supplies`).
    pub supplies: u32,
    /// `core/camera.md > Starting Position`: the primary starting structure, which the camera starts on when the map
    /// sets no camera start for the player's slot.
    pub primary: ObjectKind,
}

/// `factions/gdo/economy.md > Starting Conditions`: "One pre-built Deployment Center", "One pre-built Extraction
/// Facility", "Two Extraction Plates already placed on Space Crystals patches"; "Starting resources: 100 Space
/// Crystals", "Starting Supplies: 1". The starting power, +9, follows from the objects (`crate::economy::power`).
static GDO: StartingConditions = StartingConditions {
    objects: &[
        (ObjectKind::DeploymentCenter, 1),
        (ObjectKind::ExtractionFacility, 1),
        (ObjectKind::ExtractionPlate, 2),
    ],
    crystals: 100,
    supplies: 1,
    // `core/camera.md > Starting Position`: "GDO: Deployment Center".
    primary: ObjectKind::DeploymentCenter,
};

/// `factions/colonists/economy.md > Starting Conditions`: "one pre-built Egress Node, one pre-built Robo Portal [...]
/// and **four Drones**"; "Starting resources: **100 Space Crystals**". The Colonists have no Supplies, so 0; their
/// "Starting Alloys, Essence and Flux: **0**" are every player's (`crate::Player::starting`, plan decision T21). The
/// two structures "are already-Warped-In": each provides its 5 Beacon Capacity from the start (`crate::economy::
/// army`), 10 in all, of which the Drones use 4.
static COLONISTS: StartingConditions = StartingConditions {
    objects: &[
        (ObjectKind::EgressNode, 1),
        (ObjectKind::RoboPortal, 1),
        (ObjectKind::Drone, 4),
    ],
    crystals: 100,
    supplies: 0,
    // `core/camera.md > Starting Position`: "Colonists: starting Robo Portal".
    primary: ObjectKind::RoboPortal,
};

/// `factions/syndicate/tunnels.md > Tunnel > Starting Conditions` (slice M4-B): "One pre-built **Tier 1 Tunnel**
/// containing one pre-built **Hideout**", "One **Agent**", "**1 Supply** (enough for the 2nd Tunnel, which costs 1
/// Supply)", "Starting Space Crystals: **100** (same as GDO and the Colonists)". The Tunnel is at Tier 1 as every new
/// one is (`crate::tunnel::BASE_TIER`); the layout puts the Hideout inside its Tunnel Area (`crate::placement::
/// check_layout`). Tunnel Space follows from the objects (`crate::economy::army`): 48 tunnel-region cells less the
/// Hideout's 4, so 11, of which the Agent uses 1.
static SYNDICATE: StartingConditions = StartingConditions {
    objects: &[
        (ObjectKind::Tunnel, 1),
        (ObjectKind::Hideout, 1),
        (ObjectKind::Agent, 1),
    ],
    crystals: 100,
    supplies: 1,
    // `core/camera.md > Starting Position`: "Syndicate: starting Tunnel".
    primary: ObjectKind::Tunnel,
};

/// `factions/cults/economy.md > Starting Conditions` (slice M5-B): "One pre-built **Recruitment Hall**", "One
/// pre-built **Storage**", "Six **Recruits**, all belonging to the starting Recruitment Hall (they count toward its
/// LocalUnitControlUsage — see RecruitmentHall > Unit Control Tracking)", "**0 Space Crystals**". The Cults have no
/// Supplies, so 0. The Recruits' lineage is the Hall of their layout (`Scenario::start_objects`), so a Hall whose
/// area is wholly Recruitable starts at 6 of its 20 and queues its seventh Recruit at once (`crate::recruitment`).
static CULTS: StartingConditions = StartingConditions {
    objects: &[
        (ObjectKind::RecruitmentHall, 1),
        (ObjectKind::Storage, 1),
        (ObjectKind::Recruit, 6),
    ],
    crystals: 0,
    supplies: 0,
    // `core/camera.md > Starting Position`: "Cults: starting Recruitment Hall".
    primary: ObjectKind::RecruitmentHall,
};

impl StartingConditions {
    /// `faction`'s Starting Conditions: every faction has them (plan decision T63, slice M5-B).
    pub fn of(faction: Faction) -> &'static StartingConditions {
        match faction {
            Faction::Gdo => &GDO,
            Faction::Colonists => &COLONISTS,
            Faction::Syndicate => &SYNDICATE,
            Faction::Cults => &CULTS,
        }
    }

    /// How many objects of `kind` they hold: 0 for a kind they don't.
    pub fn count(&self, kind: ObjectKind) -> u32 {
        self.objects.iter().filter(|(k, _)| *k == kind).map(|&(_, n)| n).sum()
    }

    /// The kinds of which `layout` holds fewer objects than they do, each with how many it lacks, in their order:
    /// what the editor lists as missing (`core/map_editor.md > Starting Layouts > Skirmish Default restriction`).
    pub fn missing(&self, layout: &[LayoutObject]) -> Vec<(ObjectKind, u32)> {
        self.objects
            .iter()
            .filter_map(|&(kind, n)| {
                let held = layout.iter().filter(|o| o.kind == kind).count();
                let held = u32::try_from(held).unwrap_or(u32::MAX);
                (held < n).then(|| (kind, n - held))
            })
            .collect()
    }

    /// The indices of the objects of `layout` beyond them, in order: those of a kind they don't hold, and those that
    /// come after as many of their kind as they hold (so of three plates, the third is the extra one).
    pub fn extra(&self, layout: &[LayoutObject]) -> Vec<usize> {
        let mut seen: BTreeMap<ObjectKind, u32> = BTreeMap::new();
        layout
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                let n = seen.entry(o.kind).or_default();
                *n = n.saturating_add(1);
                (*n > self.count(o.kind)).then_some(i)
            })
            .collect()
    }

    /// Whether `layout` holds exactly their objects: none missing and none extra.
    pub fn matches(&self, layout: &[LayoutObject]) -> bool {
        self.missing(layout).is_empty() && self.extra(layout).is_empty()
    }

    /// `core/map_editor.md > Starting Layouts > Outdated layouts`, derived (plan decision T11): whether `layout` holds
    /// objects but doesn't [`match`](Self::matches) them. An empty layout is not outdated: it was never made for any
    /// Starting Conditions ("Starting Layouts start empty", design change 0009). Being derived, the flag can't tell
    /// a layout made for older Starting Conditions from one still being placed, nor, on a Custom map (which may hold
    /// any objects), from one made different on purpose: T11's alternative, a stored fingerprint, would.
    pub fn is_outdated(&self, layout: &[LayoutObject]) -> bool {
        !layout.is_empty() && !self.matches(layout)
    }
}

impl MapDef {
    /// `core/map_editor.md > Starting Layouts > Outdated layouts`: the layouts flagged as outdated
    /// ([`StartingConditions::is_outdated`]), slot by slot, each slot's in [`Faction::ALL`] order. A flag only:
    /// whether the map can be played is [`MapDef::validate`]'s.
    pub fn outdated_layouts(&self) -> Vec<(SlotId, Faction)> {
        self.slot_ids()
            .zip(&self.slots)
            .flat_map(|(slot, s)| {
                s.layouts
                    .iter()
                    .filter(|&(faction, objects)| StartingConditions::of(faction).is_outdated(objects))
                    .map(move |(faction, _)| (slot, faction))
            })
            .collect()
    }
}

/// The Skirmish Default restriction of one layout, `objects` of `faction` at `slot` (`core/map_editor.md > Starting
/// Layouts > Skirmish Default restriction`, design changes 0009 and 0012): each object beyond the faction's Starting
/// Conditions, with its index ([`ScenarioProblem::LayoutExtra`]), and each kind that it lacks
/// ([`ScenarioProblem::LayoutMissing`]).
pub(super) fn restriction(
    slot: SlotId,
    faction: Faction,
    objects: &[LayoutObject],
) -> (Vec<(usize, ScenarioProblem)>, Vec<ScenarioProblem>) {
    let start = StartingConditions::of(faction);
    let extra = start
        .extra(objects)
        .into_iter()
        .filter_map(|index| {
            let kind = objects.get(index)?.kind;
            let allowed = start.count(kind);
            Some((
                index,
                ScenarioProblem::LayoutExtra {
                    slot,
                    faction,
                    index,
                    kind,
                    allowed,
                },
            ))
        })
        .collect();
    let missing = start
        .missing(objects)
        .into_iter()
        .map(|(kind, missing)| ScenarioProblem::LayoutMissing {
            slot,
            faction,
            kind,
            missing,
        })
        .collect();
    (extra, missing)
}

impl Scenario {
    /// `core/camera.md > Starting Position`: the centre of `player`'s primary starting structure (the GDO's
    /// Deployment Center, the Syndicate's Tunnel, the Cults' Recruitment Hall, the Colonists' Robo Portal), where the
    /// game will create it: the first object of that kind in the Starting Layout of the player's faction at their
    /// slot. `None` when the player isn't in the setup, their slot isn't on the map, or the layout holds no such object
    /// that fits the map.
    pub fn primary_structure(&self, player: PlayerId) -> Option<Vec2> {
        let p = self.setup.player(player)?;
        let slot = self.map.slot(p.slot)?;
        let primary = StartingConditions::of(p.faction).primary;
        let o = slot.layouts.get(p.faction).iter().find(|o| o.kind == primary)?;
        let (center, _) = locate_oriented(&self.map.terrain, primary.def().footprint, o.at, o.orientation).ok()?;
        Some(center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::SpawnAt;
    use crate::units::TilePos;

    fn laid(kind: ObjectKind) -> LayoutObject {
        LayoutObject {
            kind,
            at: SpawnAt::Tile(TilePos::new(0, 0)),
            orientation: Default::default(),
        }
    }

    /// `factions/gdo/economy.md > Starting Conditions`: a Deployment Center, an Extraction Facility, two plates and
    /// 100 Space Crystals (Supplies 0, design change 0077); `factions/colonists/economy.md > Starting Conditions`: an
    /// Egress Node, a Robo Portal, four Drones and 100 Space Crystals, the camera on the Robo Portal
    /// (`core/camera.md > Starting Position`); `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`: a
    /// Tunnel, a Hideout, an Agent, 100 Space Crystals and 1 Supply, the camera on the Tunnel;
    /// `factions/cults/economy.md > Starting Conditions` (slice M5-B, plan decision T63): a Recruitment Hall, a
    /// Storage, six Recruits and 0 Space Crystals, the camera on the Hall.
    #[test]
    fn every_faction_has_starting_conditions() {
        use ObjectKind::{DeploymentCenter, Drone, EgressNode, ExtractionFacility, ExtractionPlate, RoboPortal};
        let gdo = StartingConditions::of(Faction::Gdo);
        assert_eq!(
            gdo.objects,
            [(DeploymentCenter, 1), (ExtractionFacility, 1), (ExtractionPlate, 2)]
        );
        assert_eq!((gdo.crystals, gdo.supplies, gdo.primary), (100, 1, DeploymentCenter));
        assert_eq!(
            [ExtractionPlate, ObjectKind::Barracks, ObjectKind::Peacekeeper].map(|k| gdo.count(k)),
            [2, 0, 0]
        );
        let colonists = StartingConditions::of(Faction::Colonists);
        assert_eq!(colonists.objects, [(EgressNode, 1), (RoboPortal, 1), (Drone, 4)]);
        assert_eq!(
            (colonists.crystals, colonists.supplies, colonists.primary),
            (100, 0, RoboPortal)
        );
        assert_eq!([Drone, ObjectKind::Beacon].map(|k| colonists.count(k)), [4, 0]);
        use ObjectKind::{Agent, Hideout, Tunnel};
        let syndicate = StartingConditions::of(Faction::Syndicate);
        assert_eq!(syndicate.objects, [(Tunnel, 1), (Hideout, 1), (Agent, 1)]);
        assert_eq!(
            (syndicate.crystals, syndicate.supplies, syndicate.primary),
            (100, 1, Tunnel)
        );
        use ObjectKind::{Recruit, RecruitmentHall, Storage};
        let cults = StartingConditions::of(Faction::Cults);
        assert_eq!(cults.objects, [(RecruitmentHall, 1), (Storage, 1), (Recruit, 6)]);
        assert_eq!((cults.crystals, cults.supplies, cults.primary), (0, 0, RecruitmentHall));
        assert_eq!([Recruit, ObjectKind::Armory].map(|k| cults.count(k)), [6, 0]);
    }

    /// The missing kinds, in the table's order, and the extra objects, the later ones of a kind.
    #[test]
    fn a_layout_is_compared_kind_by_kind() {
        use ObjectKind::{Barracks, DeploymentCenter, ExtractionFacility, ExtractionPlate};
        let gdo = StartingConditions::of(Faction::Gdo);
        assert_eq!(
            gdo.missing(&[]),
            [(DeploymentCenter, 1), (ExtractionFacility, 1), (ExtractionPlate, 2)]
        );
        let layout = [
            laid(ExtractionPlate),
            laid(Barracks),
            laid(ExtractionPlate),
            laid(DeploymentCenter),
            laid(ExtractionPlate),
        ];
        assert_eq!(gdo.missing(&layout), [(ExtractionFacility, 1)]);
        assert_eq!(gdo.extra(&layout), [1, 4]);
        assert!(!gdo.matches(&layout));
        let full = [
            laid(DeploymentCenter),
            laid(ExtractionPlate),
            laid(ExtractionFacility),
            laid(ExtractionPlate),
        ];
        assert!(gdo.matches(&full), "in any order");
    }

    /// Design change 0012's "if Starting Conditions change, affected layouts are flagged as outdated": a layout made
    /// for today's GDO conditions is outdated under conditions with one more starting unit, and an empty one never
    /// is.
    #[test]
    fn a_layout_made_for_other_conditions_is_outdated() {
        use ObjectKind::{DeploymentCenter, ExtractionFacility, ExtractionPlate, Peacekeeper};
        let gdo = StartingConditions::of(Faction::Gdo);
        let made = [
            laid(DeploymentCenter),
            laid(ExtractionFacility),
            laid(ExtractionPlate),
            laid(ExtractionPlate),
        ];
        assert!(!gdo.is_outdated(&made));
        static CHANGED: [(ObjectKind, u32); 4] = [
            (DeploymentCenter, 1),
            (ExtractionFacility, 1),
            (ExtractionPlate, 2),
            (Peacekeeper, 1),
        ];
        let changed = StartingConditions {
            objects: &CHANGED,
            ..gdo.clone()
        };
        assert!(changed.is_outdated(&made));
        assert!(gdo.is_outdated(&made[..3]), "a plate short");
        assert!(!changed.is_outdated(&[]) && !gdo.is_outdated(&[]), "an empty layout");
    }
}
