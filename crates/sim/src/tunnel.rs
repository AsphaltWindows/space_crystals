//! The Tunnel's tier — plan decision T39 (`factions/syndicate/tunnels.md > Tunnel > Tiers`): one Tunnel type, whose
//! objects carry their tier ([`TunnelState`], `Object::tunnel`), rather than a type per tier, so that counts,
//! selections and records see the same object through its upgrades. The tier table is the type's
//! (`StructureDef::tunnel`, a `TunnelDef`).
//!
//! - **State**: every object of a type with a tier table has a [`TunnelState`], and no other object; a new one is at
//!   Tier 1 ([`fresh`]: `factions/syndicate/tunnels.md > Starting Conditions`, "One pre-built **Tier 1 Tunnel**", and a
//!   Tunnel an Agent builds starts at the base tier too). Hashed only while set, so that the states of games without a
//!   Tunnel hash as they did before the field existed, and serialized.
//! - **MaxHP** is the tier's (`Object::max_hp`, [`tier_def`]): 600, 800 and 1000.
//! - **A change of tier** ([`set_tier`]) is the one place a tier changes: so far only the debug command
//!   (`DebugCommand::SetTier`); the upgrades of slice M4-G will end there too. The HP follows the MaxHP (design change
//!   0231 point 4: "on completion, current HP rises with max HP (damage stays)").
//!
//! The Tunnel Area's radius and the transit by tier are in the table for the slices that use them (M4-B, M4-C, M4-F).

pub mod area;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, TierDef, TunnelDef};
/// A Tunnel's hatch squares, read off its sides as placed (slice M4-D, `economy::drop_off`): Space Crystals at Side B,
/// Supplies at Side D (design changes 0248 and 0255 point 6).
pub use crate::economy::drop_off::{Resource, hatch_square};
use crate::hash::{StateHash, StateHasher};
use crate::network::Eject;
use crate::object::Object;
use crate::transform::follow_max;

/// The tier a Tunnel starts at: Tier 1, the base (`factions/syndicate/tunnels.md > Tunnel > Tiers`).
pub const BASE_TIER: u8 = 1;

/// A Tunnel's state (plan decision T39): its tier, from 1 to the number of its table's tiers (3), and its eject queue
/// (slice M4-C). The upgrade under way joins it in slice M4-G.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TunnelState {
    pub tier: u8,
    /// `factions/syndicate/tunnels.md > EjectMenu` (design changes 0250, 0251 and 0253): the units waiting to come out
    /// of this Tunnel's exit path, first come, first served: those a player's Eject reserved and those a surface rally
    /// point sends out by themselves, each held in its owner's network (`crate::network`). It goes with the Tunnel:
    /// "if the Tunnel is destroyed with a queue, the queued units stay in the network". Left out of files and of the
    /// hash while empty, so that the states without a queue are as they were.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub eject: Vec<Eject>,
}

impl TunnelState {
    /// A Tunnel at `tier` with nothing queued to come out.
    pub fn at(tier: u8) -> Self {
        Self {
            tier,
            eject: Vec::new(),
        }
    }
}

/// The tier table of objects of `kind`: the Tunnel's; `None` for every other type.
pub fn def_of(kind: ObjectKind) -> Option<&'static TunnelDef> {
    kind.def().structure.as_ref()?.tunnel.as_ref()
}

/// What a new object of `kind` starts with: a Tunnel at [`BASE_TIER`], every other type nothing.
pub(crate) fn fresh(kind: ObjectKind) -> Option<TunnelState> {
    def_of(kind).map(|_| TunnelState::at(BASE_TIER))
}

/// The row of `o`'s tier in its table: its MaxHP, Tunnel Area radius and transit now. `None` for an object that isn't
/// a Tunnel.
pub fn tier_def(o: &Object) -> Option<&'static TierDef> {
    def_of(o.kind)?.tier(o.tunnel.as_ref()?.tier)
}

/// Object `o`, a Tunnel, goes to tier `tier`: its MaxHP becomes that tier's, and its HP rises or falls by as much,
/// so that the damage it has taken stays (design change 0231 point 4, by the rule of a changing maximum:
/// `transform::follow_max`, which never kills a Tunnel lowered by the debug command). Returns whether it did: not for
/// an object that isn't a Tunnel, nor for a tier its table hasn't.
pub(crate) fn set_tier(o: &mut Object, tier: u8) -> bool {
    let Some(def) = def_of(o.kind) else {
        return false;
    };
    let (Some(state), Some(to)) = (o.tunnel.as_mut(), def.tier(tier)) else {
        return false;
    };
    let from = def.tier(state.tier).map_or(to.max_hp, |row| row.max_hp);
    state.tier = tier;
    if let Some(hp) = &mut o.hp {
        *hp = follow_max(*hp, from, to.max_hp);
    }
    true
}

impl StateHash for TunnelState {
    fn hash(&self, h: &mut StateHasher) {
        let TunnelState { tier, eject } = self;
        tier.hash(h);
        // Only while something is queued, so that a Tunnel without a queue hashes as before the queue existed.
        if !eject.is_empty() {
            h.tag(1);
            eject.hash(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{ObjectId, PlayerId};
    use crate::object::Placement;
    use crate::units::TilePos;

    /// A Tunnel of player 1's at Tier 1, as `Sim::insert_object` makes one, with `hp`.
    fn tunnel(hp: f32) -> Object {
        let kind = ObjectKind::Tunnel;
        let placement = Placement::new(TilePos::new(2, 2), kind.def().footprint);
        Object {
            id: ObjectId(1),
            kind,
            owner: Some(PlayerId(1)),
            pos: placement.center(),
            prev_pos: placement.center(),
            facing: 0.0,
            prev_facing: 0.0,
            hp: Some(hp),
            shields: None,
            placement: Some(placement),
            resource: None,
            orders: None,
            mover: None,
            attack: None,
            riot_shield: None,
            extractor: None,
            production: None,
            constructor: None,
            worker: None,
            transform: None,
            conversion: None,
            tunnel: fresh(kind),
        }
    }

    /// Plan decision T39: only the Tunnel carries a tier, from Tier 1; its MaxHP is its tier's.
    #[test]
    fn a_tunnel_starts_at_tier_1_with_its_max_hp() {
        for &kind in ObjectKind::ALL {
            assert_eq!(fresh(kind).is_some(), kind == ObjectKind::Tunnel, "{kind:?}");
        }
        let t = tunnel(600.0);
        assert_eq!(t.tunnel, Some(TunnelState::at(1)));
        assert_eq!(
            (t.max_hp(), tier_def(&t).map(|row| row.area_radius)),
            (Some(600.0), Some(2))
        );
    }

    /// `factions/syndicate/tunnels.md > Tiers` (HP 600 / 800 / 1000) and design change 0231 point 4: "A Tunnel at
    /// 500/600 HP upgraded to Tier 2 ends at 700/800".
    #[test]
    fn a_change_of_tier_keeps_the_damage() {
        let mut t = tunnel(500.0);
        assert!(set_tier(&mut t, 2));
        assert_eq!((t.hp, t.max_hp()), (Some(700.0), Some(800.0)));
        assert!(set_tier(&mut t, 3));
        assert_eq!((t.hp, t.max_hp()), (Some(900.0), Some(1000.0)));
        // Lowered (by the debug command only): the damage stays, and a Tunnel that would reach 0 keeps its HP.
        assert!(set_tier(&mut t, 1));
        assert_eq!((t.hp, t.max_hp()), (Some(500.0), Some(600.0)));
        let mut low = tunnel(1000.0);
        assert!(set_tier(&mut low, 3));
        low.hp = Some(300.0);
        assert!(set_tier(&mut low, 1));
        assert_eq!(low.hp, Some(300.0));
        // No tier 0 or 4, and nothing but a Tunnel has a tier.
        assert!(!set_tier(&mut t, 0) && !set_tier(&mut t, 4));
        assert_eq!(t.tunnel, Some(TunnelState::at(1)));
        let mut other = tunnel(1.0);
        other.kind = ObjectKind::Hideout;
        assert!(!set_tier(&mut other, 2));
    }

    /// Saves: a Tunnel's object file holds its tier; any other object's has no `tunnel` key, so its file is as it
    /// was before tiers existed, and such a file loads with none.
    #[test]
    fn only_a_tunnel_saves_a_tier() {
        let mut t = tunnel(600.0);
        assert!(set_tier(&mut t, 2));
        let json = serde_json::to_value(&t).unwrap();
        assert_eq!(json["tunnel"], serde_json::json!({ "tier": 2 }));
        assert_eq!(serde_json::from_value::<Object>(json).unwrap(), t);
        let mut other = tunnel(400.0);
        (other.kind, other.tunnel) = (ObjectKind::Hideout, None);
        let json = serde_json::to_value(&other).unwrap();
        assert!(json.get("tunnel").is_none(), "{json}");
        assert_eq!(serde_json::from_value::<Object>(json).unwrap(), other);
    }

    /// The tier hashes as a value of its own; an object without one hashes nothing for it (`Object`'s hash).
    #[test]
    fn the_tier_is_hashed() {
        let hash = |state: TunnelState| {
            let mut h = StateHasher::new();
            state.hash(&mut h);
            h.finish()
        };
        assert_ne!(hash(TunnelState::at(1)), hash(TunnelState::at(2)));
        // The eject queue is hashed while it holds something (slice M4-C).
        let queued = TunnelState {
            eject: vec![Eject {
                unit: ObjectId(7),
                order: None,
            }],
            ..TunnelState::at(1)
        };
        assert_ne!(hash(queued), hash(TunnelState::at(1)));
    }
}
