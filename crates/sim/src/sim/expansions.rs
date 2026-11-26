//! A Tunnel's expansions in a game (slice M4-F; `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState >
//! ExpandMenu`, `factions/syndicate/tunnel_menus.md > AwaitingPlacement (Expansion)`,
//! `factions/syndicate/tunnel_menus.md > Expansions under construction`): a Tunnel builds an underground expansion in
//! its Tunnel Area as its one operation (`tunnel::Operation`), on a site that is the expansion from its first frame
//! (plan decision T31, `construction::site`).
//!
//! - **The gate** ([`Sim::check_expand`], `TunnelCommand::Expand`): the Tunnel is its player's and in operation
//!   (`Sim::check_tunnel`), performs no operation ("A Tunnel can only perform one operation at a time",
//!   `factions/syndicate/tunnels.md > Construction and Upgrade Rules`), and builds the kind (`StructureDef::expansion`)
//!   at its tier ("its Tier Requirement is at or below **this** Tunnel's tier", `factions/syndicate/tunnel_menus.md >
//!   ExpandMenu`); its TechPrerequisite holds ("satisfied anywhere in the Tunnel Network", `Sim::check_requires`, slice
//!   M4-K: [`Reject::Requires`]), and the bank pays its cost. These are [`Sim::can_expand`], which the ExpandMenu greys
//!   its entries by. Then the placement, by the truth (`Sim::expansion_truth`, design change 0240: "judged by the
//!   truth; the placement is refused"), reported to the player as a Place's is (`SimEvent::PlacementRefused`, plan
//!   decision T4), the Tunnel its constructor.
//! - **The start** ([`Sim::start_expansion`]): "**Paid when the ghost is placed** ... **Tunnel Space is used once
//!   construction has started.** Today placing the ghost starts construction at once" (design change 0254): the site
//!   comes into the game as the command applies, its owner's, which Tunnel Space counts from then on
//!   (`economy::army`, an underground structure in the Tunnel Area), at 10% of its MaxHP ("the ConstructionHP Rule
//!   applies to every expansion (starts at 10%, grows with progress)"), its functions held back as a transforming
//!   building's are (`sim/transform.rs`: a Hideout trains nothing yet). Hidden from the other players as any owned
//!   underground structure (`vision::hidden_from`).
//! - **The frames**: the sites' step (`sim/sites.rs`): one a tick from the next tick on, the HP following the maximum
//!   ("damage taken stays").
//! - **Completion** ([`Sim::complete_expansion`]): its functions start, as a new object of its kind has them (a Hideout
//!   trains Agents into the Tunnel Network), and the Tunnel's operation is over.
//! - **Cancel** ([`Sim::cancel_expansion`]): "while building, the expansion offers only **X Cancel Construction**,
//!   refunding **75%** of its cost (rounded down). The Tunnel's X cancels whatever operation it is running"
//!   (`TunnelCommand::Cancel`, [`Sim::cancel_operation`]; `economy::Refund::Expansion`).
//! - **Its site gone** any other way (destroyed, a debug removal): the Tunnel's operation ends with it (the removal
//!   path's reference cleanup, `sim/removal.rs`), with no refund. **Its Tunnel destroyed mid-build**: "the
//!   unfinished expansion is destroyed, no refund", in the same removal (`sim/tunnels.rs`, slice M4-H).
//! - **Eject** goes on meanwhile: "Eject still works (it isn't an operation)".
//! - **Regeneration** ([`Sim::regenerate_expansions`], slice M5-N8): "**Not repaired; they regenerate** ... a completed
//!   expansion **regenerates HP gradually** while it is part of a Tunnel system (its Tunnel is alive). Placeholder rate
//!   (designer): **1% of MaxHP per second**, always on, **including during combat**"
//!   (`factions/syndicate/structures/README.md > Tunnel Expansions`, design changes 0329 and 0330).

use super::Sim;
use super::gate::{Checked, Reject};
use super::transform::hold_back;
use crate::augmentation::Research;
use crate::command::TunnelCommand;
use crate::construction::site::{Construction, expansion_def};
use crate::defs::ObjectKind;
use crate::economy::Refund;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::network;
use crate::object::{Object, Placement};
use crate::orders::build::footprint;
use crate::popup::{self, PopUp};
use crate::progress::Progress;
use crate::regeneration;
use crate::scenario::BuildingOrientation;
use crate::tech;
use crate::tuning::tunnel::EXPANSION_REGEN_PER_SECOND;
use crate::tunnel::{self, Operation};
use crate::units::TilePos;

impl Sim {
    /// `factions/syndicate/tunnel_menus.md > ExpandMenu`: whether `player`'s Tunnel `tunnel` may build an expansion of
    /// `kind` now, wherever it goes: the gate's answer to an Expand short of its placement (module docs), by which the
    /// ExpandMenu greys an entry ("unavailable entries are greyed", design change 0257) and says why. The Tunnel is the
    /// player's ([`Reject::NotOwned`]) and in operation ([`Reject::NotATunnel`]), as for every Tunnel command.
    pub fn can_expand(&self, player: PlayerId, tunnel: ObjectId, kind: ObjectKind) -> Result<(), Reject> {
        let target = self.referenced(tunnel, Some(player))?;
        if !network::is_exit(target) {
            return Err(Reject::NotATunnel(tunnel));
        }
        self.expansion_available(player, target, kind)
    }

    /// Tunnel `tunnel`'s operation, if it performs one, with how far it is: an expansion's its site's progress
    /// (`Construction::progress`), an upgrade's its own (slice M4-G). For the Tunnel's InfoPanel: "**The Tunnel
    /// meanwhile** shows the progress" (`factions/syndicate/tunnel_menus.md > Expansions under construction`).
    pub fn operation_progress(&self, tunnel: ObjectId) -> Option<(Operation, Progress)> {
        let operation = self.object(tunnel)?.tunnel.as_ref()?.operation?;
        let progress = match operation {
            Operation::Building(site) => self.object(site)?.construction?.progress,
            Operation::Upgrading { progress, .. } => progress,
        };
        Some((operation, progress))
    }

    /// The gate for `TunnelCommand::Expand` on `tunnel`, the player's Tunnel in operation (`Sim::check_tunnel`): the
    /// Tunnel may build the `kind` now ([`Sim::can_expand`]), an id is left for it, and the footprint at `origin`,
    /// turned by `orientation`, is valid by the truth ([`Reject::Unplaceable`], the check's first problem).
    pub(super) fn check_expand(
        &self,
        player: PlayerId,
        tunnel: &Object,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    ) -> Result<Checked, Reject> {
        self.expansion_available(player, tunnel, kind)?;
        if self.next_id == u32::MAX {
            return Err(Reject::OutOfIds);
        }
        let check = (self.expansion_truth(tunnel, kind, origin, orientation)).ok_or(Reject::NotATunnel(tunnel.id))?;
        if let Some(problem) = check.first_problem() {
            return Err(Reject::Unplaceable(problem));
        }
        let placement = footprint(kind, origin, orientation).ok_or(Reject::CannotConstruct(kind))?;
        Ok(Checked::Expand {
            tunnel: tunnel.id,
            kind,
            placement,
        })
    }

    /// The gate for `TunnelCommand::Cancel` on `tunnel`, the player's Tunnel in operation: it performs an operation
    /// ([`Reject::NoOperation`]).
    pub(super) fn check_cancel_operation(&self, tunnel: &Object) -> Result<Checked, Reject> {
        match tunnel.tunnel.as_ref().and_then(|state| state.operation) {
            Some(_) => Ok(Checked::CancelOperation { tunnel: tunnel.id }),
            None => Err(Reject::NoOperation(tunnel.id)),
        }
    }

    /// [`Sim::can_expand`] once the Tunnel is known to be `player`'s in operation: no operation under way, a kind it
    /// builds at its tier, the prerequisites, and the Space Crystals.
    fn expansion_available(&self, player: PlayerId, tunnel: &Object, kind: ObjectKind) -> Result<(), Reject> {
        let state = tunnel.tunnel.as_ref().ok_or(Reject::NotATunnel(tunnel.id))?;
        if state.operation.is_some() {
            return Err(Reject::OperationUnderWay(tunnel.id));
        }
        let def = expansion_def(kind).ok_or(Reject::CannotConstruct(kind))?;
        if state.tier < def.tier {
            return Err(Reject::TierRequirement { kind, tier: def.tier });
        }
        // `factions/syndicate/structures/README.md > Tunnel Expansions > Expansion Requirements`: its TechPrerequisite,
        // "checked across the whole Tunnel Network" (slice M4-K, `crate::tech`).
        self.check_requires(player, tech::structure_requires(kind).iter().copied())?;
        if self.player(player).map_or(0, |p| p.crystals) < def.cost {
            return Err(Reject::InsufficientCrystals);
        }
        Ok(())
    }

    /// Tunnel `tunnel` starts building `player`'s expansion of `kind` on `placement`, as the gate resolved it (module
    /// docs): the bank pays, the site comes into the game with its HP at the ConstructionHP maximum and its functions
    /// held back, and it is the Tunnel's operation; a `SimEvent::ExpansionStarted`.
    pub(super) fn start_expansion(
        &mut self,
        player: PlayerId,
        tunnel: ObjectId,
        kind: ObjectKind,
        placement: Placement,
    ) {
        let Some(def) = expansion_def(kind) else {
            return;
        };
        // The gate made sure that the bank pays.
        let bank = self.players.iter_mut().find(|p| p.id == player);
        if !bank.is_some_and(|bank| bank.spend_crystals(def.cost)) {
            return;
        }
        let site = self.insert_object(kind, Some(player), placement.center(), Some(placement), None);
        if let Some(o) = self.objects.get_mut(&site) {
            o.construction = Some(Construction::expansion(def, tunnel));
            o.hp = o.max_hp();
            hold_back(o);
            // A turret's site is lowered from its first frame, as every turret is (`Sim::new_object`): "The **site is
            // hidden** like an expansion under construction" (`factions/syndicate/structures/laser_turret.md >
            // Construction`), and it completes lowered (`popup`).
            debug_assert!(popup::def_of(kind).is_none() || o.popup == Some(PopUp::LOWERED));
        }
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut()) {
            state.operation = Some(Operation::Building(site));
        }
        self.events.push(SimEvent::ExpansionStarted { tunnel, site });
    }

    /// Expansion `site`, which Tunnel `tunnel` built, is complete (`Sim::complete_site` took its construction off):
    /// its functions start as a new object of its kind has them (the transformations' rule, `Sim::start_functions`),
    /// and the Tunnel's operation is over; a `SimEvent::ExpansionCompleted`.
    pub(super) fn complete_expansion(&mut self, site: ObjectId, tunnel: ObjectId) {
        self.start_functions(site);
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut())
            && state.operation == Some(Operation::Building(site))
        {
            state.operation = None;
        }
        self.events.push(SimEvent::ExpansionCompleted { tunnel, site });
    }

    /// `TunnelCommand::Cancel` on `tunnel` (`factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`: "**X**
    /// Cancel (upgrade, construction or the current operation)"): its operation is cancelled, an expansion as by its
    /// own Cancel Construction ([`Sim::cancel_expansion`]), an upgrade refunded at the current top price
    /// (`Sim::cancel_upgrade`, slice M4-G).
    pub(super) fn cancel_operation(&mut self, tunnel: ObjectId) {
        let operation = (self.objects.get(&tunnel)).and_then(|o| o.tunnel.as_ref()?.operation);
        match operation {
            Some(Operation::Building(site)) => self.cancel_expansion(site),
            Some(Operation::Upgrading { .. }) => self.cancel_upgrade(tunnel),
            None => {}
        }
    }

    /// Cancel Construction on the expansion `site` (`ConstructionCommand::CancelConstruction` on it, through
    /// `Sim::cancel_site`, or its Tunnel's X): it leaves the game, which ends its Tunnel's operation (the removal path,
    /// `sim/removal.rs`), and its owner gets back "**75%** of its cost (rounded down)" (design change 0254,
    /// `economy::Refund::Expansion`); a `SimEvent::ExpansionCancelled`. Nothing for an object that isn't a Tunnel's
    /// site.
    pub(super) fn cancel_expansion(&mut self, site: ObjectId) {
        let Some((owner, kind, tunnel)) =
            (self.objects.get(&site)).and_then(|o| Some((o.owner?, o.kind, o.construction?.builder.tunnel()?)))
        else {
            return;
        };
        self.remove_objects(&[site]);
        let refund = expansion_def(kind).map_or(0, |def| Refund::Expansion.of(def.cost));
        if let Some(bank) = self.players.iter_mut().find(|p| p.id == owner) {
            bank.credit_crystals(refund);
        }
        self.events.push(SimEvent::ExpansionCancelled {
            tunnel,
            site,
            kind,
            refund,
        });
    }

    /// Plan decision T4 for an Expand (`Sim::report_refusal`): one that the truth refuses as it applies changes
    /// nothing, and the sim reports it with every problem the check finds there (`SimEvent::PlacementRefused`, the
    /// Tunnel as its constructor), so that the client can tell the player: "judged by the truth; the placement is
    /// refused" (design change 0240), where the ghost judged by what the player knows. Every other refusal of a Tunnel
    /// command is silent.
    pub(super) fn report_tunnel_refusal(&mut self, tunnel: ObjectId, command: TunnelCommand, reject: Reject) {
        let (
            Reject::Unplaceable(_),
            TunnelCommand::Expand {
                kind,
                origin,
                orientation,
            },
        ) = (reject, command)
        else {
            return;
        };
        let problems = (self.objects.get(&tunnel)).and_then(|o| self.expansion_truth(o, kind, origin, orientation));
        if let Some(problems) = problems {
            self.events.push(SimEvent::PlacementRefused {
                constructor: tunnel,
                kind,
                origin,
                orientation,
                problems,
            });
        }
    }

    /// (M5-N8) The expansions' regeneration (module docs), in the economy phase beside the decay: each complete
    /// underground expansion (`StructureDef::underground`; a Laser Turret, which Agents repair, is none) with a parent
    /// Tunnel (`tunnel::area::parent`: its owner's Tunnel in operation whose area contains it) regains
    /// `tuning::tunnel::EXPANSION_REGEN_PER_SECOND` of its MaxHP a second, up to it (`crate::regeneration`), in id
    /// order. A neutral one, its Tunnel lost, decays instead (`sim/tunnels.rs`).
    pub(super) fn regenerate_expansions(&mut self) {
        let research = Research::new(&self.players);
        let damaged = |o: &Object| {
            let underground = o.def().structure.as_ref().is_some_and(|s| s.underground);
            underground
                && o.owner.is_some()
                && o.is_operational()
                && o.hp.zip(o.max_hp_with(research)).is_some_and(|(hp, max)| hp < max)
        };
        if !self.objects.values().any(damaged) {
            return;
        }
        let tunnels: Vec<&Object> = self.objects.values().filter(|o| o.tunnel.is_some()).collect();
        let regenerating: Vec<ObjectId> = (self.objects.values())
            .filter(|o| damaged(o) && tunnel::area::parent(o, tunnels.iter().copied()).is_some())
            .map(|o| o.id)
            .collect();
        for id in regenerating {
            if let Some(o) = self.objects.get_mut(&id) {
                regeneration::regenerate(o, EXPANSION_REGEN_PER_SECOND, research);
            }
        }
    }
}

#[cfg(test)]
mod tests;
