//! What a player knows of the objects its orders and rally points target, and what a unit does by default against
//! one: `core/vision.md > Explored > Targeting remembered objects` (design change 0139, which answers question Q0017
//! point 2) and `systems/controls/selection_and_hud.md > What a rally point makes a new unit do` (design change
//! 0136).
//!
//! - [`Sim::target_view`]: the one view of a target (`crate::targets`): Visible, Remembered at its last-known position,
//!   or Gone, from the player's sight and records only.
//! - The gate's rule ([`Sim::check_target_known`]): an order or a rally point may name an object the player sees or
//!   remembers. A remembered one is judged by its record, whether or not it still stands, so the gate's verdict tells
//!   the player nothing it doesn't know.
//! - [`Sim::default_order`]: a unit's default command against a target (`crate::orders::resolution`), the
//!   right-click's and the rally point's, on the target as the unit's owner knows it ([`Sim::aimed`]).

use crate::combat::object_domain;
use crate::defs::{Domain, ObjectKind};
use crate::economy::gathering::{gatherable, is_drop_off};
use crate::economy::stations::is_station;
use crate::holding;
use crate::ids::{ObjectId, PlayerId};
use crate::network;
use crate::object::Object;
use crate::orders::resolution::{Abilities, Aim, Aimed, resolve};
use crate::orders::{OrderKind, Target};
use crate::production::Rally;
use crate::sim::{Reject, Sim};
use crate::targets::{self, Live, TargetView};

impl Sim {
    /// What `player` knows of object `id` now, by the vision the last tick left (`crate::targets::view`): Visible
    /// while it sees the object (always its own), Remembered while it only has a record of it out of its sight, and
    /// Gone otherwise. Nothing of an object the player doesn't see is read but whether it exists, and a Remembered
    /// view is the same whether or not it does.
    pub fn target_view(&self, player: PlayerId, id: ObjectId) -> TargetView {
        let live = self.objects.get(&id).map(|o| Live {
            owner: o.owner,
            seen: self.is_visible(player, o),
        });
        targets::view(self.vision.player(player), player, id, live)
    }

    /// Object `id` as `player` knows it, for a unit's default command ([`Aimed`]): the object itself while the player
    /// sees it; its record while the player remembers it (a structure or a resource, on the ground; a Space Crystals
    /// patch is gatherable unless the player knows of a mining structure on it, and no remembered object is the
    /// player's drop-off, since its own objects are never remembered). `None` while the player knows nothing of it.
    pub fn aimed(&self, player: PlayerId, id: ObjectId) -> Option<Aimed> {
        let enemy = |owner: Option<PlayerId>| owner.is_some() && owner != Some(player);
        match self.target_view(player, id) {
            TargetView::Visible => {
                let o = self.objects.get(&id)?;
                Some(Aimed {
                    id,
                    enemy: enemy(o.owner),
                    destructible: o.def().destructible,
                    domain: object_domain(o),
                    gatherable: gatherable(&self.objects, id),
                    station: is_station(o.kind),
                    drop_off: o.owner == Some(player) && is_drop_off(o.kind) && o.is_operational(),
                    // `systems/controls/commands.md > Enter`: an own Tunnel, by its tier (`crate::network`).
                    transit: crate::tunnel::tier_def(o)
                        .filter(|_| o.owner == Some(player) && network::is_exit(o))
                        .map_or(&[], |row| row.transit),
                    // An own building that holds units, in operation (slice M5-F1, `crate::holding`).
                    accepts: holding::holds(o)
                        .filter(|_| o.owner == Some(player) && o.placement.is_some() && o.is_operational())
                        .map_or(&[], |holds| holds.accepts),
                })
            }
            TargetView::Remembered { .. } => {
                let sight = self.vision.player(player)?;
                let record = sight.record(id)?;
                Some(Aimed {
                    id,
                    enemy: enemy(record.owner),
                    destructible: record.kind.def().destructible,
                    domain: Domain::Ground,
                    gatherable: targets::remembered_gatherable(sight, record, self.own_cover(player, id)),
                    station: is_station(record.kind),
                    drop_off: false,
                    transit: &[],
                    accepts: &[],
                })
            }
            TargetView::Gone => None,
        }
    }

    /// Whether one of `player`'s own mining structures stands on patch `id` (it always knows its own).
    fn own_cover(&self, player: PlayerId, id: ObjectId) -> bool {
        (self.objects.values()).any(|o| o.owner == Some(player) && o.extractor.is_some_and(|e| e.patch == id))
    }

    /// The gate's rule for a Gather's patch (`factions/colonists/units/drone.md > Unit Commands`: "right-click
    /// Space Crystal patch"; design change 0139: a patch the player sees or remembers, "Gather" among the targeted
    /// commands): `player` knows of patch `id` ([`Sim::check_target_known`]), and takes it to be a Space Crystals
    /// patch with no mining structure on it (design change 0137: "Drones can't gather a patch covered by a plate"),
    /// as it knows it: a remembered one by its record ([`Reject::NotGatherable`]). Or a Supply Delivery Station
    /// (`factions/syndicate/units/agent.md > Unit Commands`: "Gather — right-click Space Crystals patch or Supply
    /// Delivery Station"; slice M4-I), which only the workers that gather Supplies take (`Sim::gatherers_among`).
    pub(in crate::sim) fn check_gather_target(&self, player: PlayerId, id: ObjectId) -> Result<(), Reject> {
        self.check_target_known(player, id)?;
        if !self
            .aimed(player, id)
            .is_some_and(|aimed| aimed.gatherable || aimed.station)
        {
            return Err(Reject::NotGatherable(id));
        }
        Ok(())
    }

    /// `systems/controls/selection_and_hud.md > RightClickResolution` for one unit (`crate::orders::resolution`): the
    /// order unit `unit` gets by default against `target`, the target as its owner knows it ([`Sim::aimed`]). `None`
    /// if `unit` is no unit that takes orders, or its owner knows nothing of the object. The client resolves a
    /// right-click with the same rule, on what it shows; a rally point with this (design change 0136).
    pub fn default_order(&self, unit: ObjectId, target: Target) -> Option<OrderKind> {
        self.default_order_of(self.objects.get(&unit)?, target)
    }

    /// [`Sim::default_order`] for the unit `o`, on the map or held off it (a unit produced into the Tunnel Network,
    /// `sim/network.rs`).
    pub(in crate::sim) fn default_order_of(&self, o: &Object, target: Target) -> Option<OrderKind> {
        let aim = match target {
            Target::Point(p) => Aim::Ground(p),
            Target::Object(id) => Aim::Object(self.aimed(o.owner?, id)?),
        };
        Some(resolve(Abilities::of(o)?, aim))
    }

    /// Design change 0136 (`systems/controls/selection_and_hud.md > What a rally point makes a new unit do`): the order
    /// unit `unit` gets as it leaves its producer, whose rally point is `rally`: "its **own default right-click
    /// command** resolved against the rally target", as the unit's owner knows the target ([`Sim::default_order`]). A
    /// rally on an object its owner knows nothing of any more (a unit gone out of its sight since the end of the last
    /// tick, before the rally turned into a ground rally, design change 0088) is one on the ground where the rally
    /// shows.
    pub(in crate::sim) fn rally_order(&self, unit: ObjectId, rally: Rally) -> Option<OrderKind> {
        self.rally_order_of(self.objects.get(&unit)?, rally)
    }

    /// [`Sim::rally_order`] for the unit `o`, on the map or held off it.
    pub(in crate::sim) fn rally_order_of(&self, o: &Object, rally: Rally) -> Option<OrderKind> {
        match rally {
            Rally::Point(p) => self.default_order_of(o, Target::Point(p)),
            Rally::Object { id, at } => self
                .default_order_of(o, Target::Object(id))
                .or_else(|| self.default_order_of(o, Target::Point(at))),
        }
    }

    /// The rally point a producer of `player`'s gets on object `id`, which the gate let through
    /// ([`Sim::check_target_known`]): on the object, at its position while the player sees it, at the last-known one
    /// while it remembers it (design change 0139: "The order goes to the object's last-known position").
    pub(in crate::sim) fn rally_on(&self, player: PlayerId, id: ObjectId) -> Option<Rally> {
        let at = match self.target_view(player, id) {
            TargetView::Visible => self.objects.get(&id)?.pos,
            TargetView::Remembered { position, .. } => position,
            TargetView::Gone => return None,
        };
        Some(Rally::Object { id, at })
    }

    /// The gate's rule for an object target (design change 0139: remembered objects "can be targeted by
    /// right-click and by targeted commands (Attack, Gather, Move to object, rally points)"): `player` sees object
    /// `id` or remembers it ([`Sim::target_view`]). An object the player knows nothing of is
    /// [`Reject::UnknownObject`] if there is no such object and [`Reject::NotVisible`] if there is, as before the
    /// answer.
    pub(in crate::sim) fn check_target_known(&self, player: PlayerId, id: ObjectId) -> Result<TargetView, Reject> {
        let view = self.target_view(player, id);
        if view.is_known() {
            return Ok(view);
        }
        self.referenced(id, None)?;
        Err(Reject::NotVisible(id))
    }

    /// The kind of object `id` as `player` knows it, for the gate's checks: a remembered object's by the player's
    /// record, whether or not the object still stands or has changed since; any other's its own
    /// ([`Reject::UnknownObject`] if there is no such object).
    pub(in crate::sim) fn known_kind(&self, player: PlayerId, id: ObjectId) -> Result<ObjectKind, Reject> {
        if let TargetView::Remembered { .. } = self.target_view(player, id)
            && let Some(record) = self.vision.player(player).and_then(|vision| vision.record(id))
        {
            return Ok(record.kind);
        }
        Ok(self.referenced(id, None)?.kind)
    }
}
