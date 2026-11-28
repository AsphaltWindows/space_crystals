//! The construction commands in a game (`crate::construction`, slice M2-D1): their rules in the command gate, what
//! applying them does, the refusal a Place reports, and the refunds of the constructors that leave the game. The
//! Warp In and its cancel are construction commands too, whose rules and effects are the transformations'
//! (`sim/transform.rs`, slice M3-E).

use glam::Vec2;

use super::Sim;
use super::gate::{Checked, Reject};
use crate::command::{Command, PlayerCommand};
use crate::construction::{self, ConstructionCommand, Constructor};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement};
use crate::placement::footprint_size;
use crate::scenario::BuildingOrientation;
use crate::units::TilePos;
use crate::{tech, transform};

/// A construction command that passed the gate, with what its checks resolved.
pub(super) enum Resolved {
    /// Each of `constructors`, in id order, pays for `kind` and starts constructing it.
    Construct {
        kind: ObjectKind,
        constructors: Vec<ObjectId>,
    },
    /// Each of `constructors` cancels its construction or its ready building, with the refund that goes with it and
    /// its `SimEvent::ConstructionCancelled`; then each of `sites` is cancelled (`sim/sites.rs`).
    Cancel {
        constructors: Vec<ObjectId>,
        sites: Vec<ObjectId>,
    },
    /// `constructor` places its ready `kind` at `placement`, centred on `pos`.
    Place {
        constructor: ObjectId,
        kind: ObjectKind,
        pos: Vec2,
        placement: Placement,
    },
    /// `beacon` pays for `kind` and starts warping it in at `placement` (`sim/transform.rs`).
    WarpIn {
        beacon: ObjectId,
        kind: ObjectKind,
        placement: Placement,
    },
    /// Each of `objects`, in id order, cancels its transformation, with the refund that goes with it
    /// (`sim/transform.rs`).
    CancelTransform { objects: Vec<ObjectId> },
}

impl Sim {
    /// The gate's rules for `Command::Construction` (`crate::construction`), after the player's: the player's own
    /// constructors among `objects`, unique and in id order (ids that no longer exist are dropped, as are the player's
    /// objects that construct nothing; anyone else's object rejects it, [`Reject::NotOwned`]; none left is
    /// [`Reject::NoConstructor`]), then the command's own:
    /// - **Construct**: each constructor, in id order, that has the building on its list, whose player owns what its
    ///   entry and the building require (slice M4-K, `crate::tech`: `core/tech_and_research.md > Tech Prerequisites >
    ///   Check Timing`, "checked only at the moment construction is started"), is idle and can pay for it from what
    ///   the bank holds once the constructors before it paid; if none can, the first one's reason
    ///   ([`Reject::CannotConstruct`], [`Reject::Requires`], [`Reject::ConstructorBusy`],
    ///   [`Reject::InsufficientCrystals`]);
    /// - **Cancel Construction**: each one constructing, and each of the player's sites among `objects` (`sim/sites.rs`,
    ///   slice M4-E; design change 0234: a site's "only command is X Cancel Construction"); with no site, as without
    ///   them: [`Reject::NotConstructing`] if no constructor is constructing;
    /// - **Cancel Ready Building**: each one holding a building ready, else [`Reject::NothingReady`];
    /// - **Place**: one constructor ([`Reject::SeveralConstructors`]) holding a building ready
    ///   ([`Reject::NothingReady`]), which the placement check allows there now ([`Sim::placement_check`]: else
    ///   [`Reject::Unplaceable`] with its first problem);
    /// - **Warp In** and **Cancel Warp In**: the Beacon's and the transforming objects' rules (`Sim::check_warp_in`,
    ///   `Sim::check_cancel_transform`).
    pub(super) fn check_construction(
        &self,
        player: PlayerId,
        objects: &[ObjectId],
        command: ConstructionCommand,
    ) -> Result<Checked, Reject> {
        let resolved = match command {
            ConstructionCommand::Construct(kind) => self.check_construct(player, objects, kind)?,
            ConstructionCommand::CancelConstruction => self.check_cancel_construction(player, objects)?,
            ConstructionCommand::CancelReady => Resolved::Cancel {
                constructors: self.constructors_that(player, objects, |c| c.ready().is_some(), Reject::NothingReady)?,
                sites: Vec::new(),
            },
            ConstructionCommand::Place { origin, orientation } => {
                self.check_place(player, objects, origin, orientation)?
            }
            ConstructionCommand::WarpIn { kind, orientation } => {
                self.check_warp_in(player, objects, kind, orientation)?
            }
            ConstructionCommand::CancelTransform => self.check_cancel_transform(player, objects)?,
        };
        Ok(Checked::Construction { player, resolved })
    }

    /// The player's constructors among `objects`, unique and in id order: at least one, else
    /// [`Reject::NoConstructor`].
    fn own_constructors(&self, player: PlayerId, objects: &[ObjectId]) -> Result<Vec<ObjectId>, Reject> {
        let constructors = self.own_units(player, objects, |o| o.constructor.is_some())?;
        if constructors.is_empty() {
            return Err(Reject::NoConstructor);
        }
        Ok(constructors)
    }

    /// The player's constructors among `objects` whose state `keep` keeps: at least one, else `none`.
    fn constructors_that(
        &self,
        player: PlayerId,
        objects: &[ObjectId],
        keep: impl Fn(&Constructor) -> bool,
        none: Reject,
    ) -> Result<Vec<ObjectId>, Reject> {
        let kept: Vec<ObjectId> = self
            .own_constructors(player, objects)?
            .into_iter()
            .filter(|id| {
                self.objects
                    .get(id)
                    .and_then(|o| o.constructor.as_ref())
                    .is_some_and(&keep)
            })
            .collect();
        if kept.is_empty() {
            return Err(none);
        }
        Ok(kept)
    }

    /// Cancel Construction: the constructors constructing among `objects`, and the player's sites among them.
    fn check_cancel_construction(&self, player: PlayerId, objects: &[ObjectId]) -> Result<Resolved, Reject> {
        let sites = self.own_units(player, objects, Object::is_site)?;
        let constructing = self.constructors_that(player, objects, |c| c.current().is_some(), Reject::NotConstructing);
        match constructing {
            Ok(constructors) => Ok(Resolved::Cancel { constructors, sites }),
            Err(_) if !sites.is_empty() => Ok(Resolved::Cancel {
                constructors: Vec::new(),
                sites,
            }),
            Err(reject) => Err(reject),
        }
    }

    /// Construct `kind` (plan decision T7: the bank is checked as the command applies, per constructor in id order).
    fn check_construct(&self, player: PlayerId, objects: &[ObjectId], kind: ObjectKind) -> Result<Resolved, Reject> {
        let mut left = self.player(player).map_or(0, |p| p.crystals);
        let mut constructors = Vec::new();
        let mut refusal = None;
        for id in self.own_constructors(player, objects)? {
            let Some(o) = self.objects.get(&id) else {
                continue;
            };
            // What its entry and the building require (slice M4-K, `crate::tech`), after whether it builds it at all.
            let requires = construction::recipe(o.kind, kind)
                .into_iter()
                .flat_map(tech::recipe_requires);
            let started = self.check_requires(player, requires);
            match started.and_then(|()| construction::can_start(o, kind, left)) {
                Ok(recipe) => {
                    left -= recipe.cost;
                    constructors.push(id);
                }
                Err(reject) => {
                    refusal.get_or_insert(reject);
                }
            }
        }
        if constructors.is_empty() {
            return Err(refusal.unwrap_or(Reject::NoConstructor));
        }
        Ok(Resolved::Construct { kind, constructors })
    }

    /// The constructor of a Place among `objects`, and the building it holds ready: exactly one constructor
    /// ([`Reject::SeveralConstructors`]; design change 0071: placement never moves on to another builder), which holds
    /// a building ready ([`Reject::NothingReady`]).
    fn placer(&self, player: PlayerId, objects: &[ObjectId]) -> Result<(ObjectId, ObjectKind), Reject> {
        let constructors = self.own_constructors(player, objects)?;
        let [id] = constructors[..] else {
            return Err(Reject::SeveralConstructors);
        };
        let ready = self.objects.get(&id).and_then(|o| o.constructor?.ready());
        Ok((id, ready.ok_or(Reject::NothingReady)?))
    }

    /// Place the ready building with the top-left tile of its footprint at `origin`, turned by `orientation`
    /// (`factions/gdo/structures/deployment_center.md > AwaitingPlacement`, `factions/gdo/economy.md >
    /// ExtractionFacility > AwaitingPlacement`): where the placement check in Command mode finds no problem now, as
    /// the command applies (plan decision T4), with the state the earlier commands of the tick left.
    fn check_place(
        &self,
        player: PlayerId,
        objects: &[ObjectId],
        origin: TilePos,
        orientation: BuildingOrientation,
    ) -> Result<Resolved, Reject> {
        let (constructor, kind) = self.placer(player, objects)?;
        if self.next_id == u32::MAX {
            return Err(Reject::OutOfIds);
        }
        if let Some(problem) = self.placement_check(player, kind, origin, orientation).first_problem() {
            return Err(Reject::Unplaceable(problem));
        }
        // A valid footprint is a structure's, on the map.
        let (w, h) = footprint_size(kind, orientation).ok_or(Reject::CannotConstruct(kind))?;
        let placement = Placement {
            origin,
            w,
            h,
            rotation: orientation.rotation,
            flip_h: orientation.flip_h,
            flip_v: orientation.flip_v,
        };
        Ok(Resolved::Place {
            constructor,
            kind,
            pos: placement.center(),
            placement,
        })
    }

    /// Carries out a construction command the gate resolved for `player`, whose constructors they are.
    pub(super) fn apply_construction(&mut self, player: PlayerId, resolved: Resolved) {
        match resolved {
            Resolved::Construct { kind, constructors } => {
                for id in constructors {
                    let (Some(o), Some(bank)) = (
                        self.objects.get_mut(&id),
                        self.players.iter_mut().find(|p| p.id == player),
                    ) else {
                        continue;
                    };
                    let Some(recipe) = construction::recipe(o.kind, kind) else {
                        continue;
                    };
                    // The gate made sure that the bank pays for every constructor of the command.
                    if bank.spend_crystals(recipe.cost) {
                        o.constructor = Some(Constructor::Constructing { kind, progress: 0 });
                    }
                }
            }
            Resolved::Cancel { constructors, sites } => {
                for id in constructors {
                    self.cancel_construction(id);
                }
                for site in sites {
                    self.cancel_site(site);
                }
            }
            Resolved::Place {
                constructor,
                kind,
                pos,
                placement,
            } => {
                if let Some(state) = self.objects.get_mut(&constructor).and_then(|o| o.constructor.as_mut()) {
                    *state = Constructor::Idle;
                }
                let structure = self.insert_object(kind, Some(player), pos, Some(placement), None);
                self.events.push(SimEvent::StructurePlaced { constructor, structure });
                // A Supply Tower comes with its chopper (`sim/supply_run.rs`).
                self.spawn_attached_chopper(structure);
            }
            Resolved::WarpIn {
                beacon,
                kind,
                placement,
            } => self.start_warp_in(player, beacon, kind, placement),
            Resolved::CancelTransform { objects } => {
                for id in objects {
                    self.cancel_transform(id);
                }
            }
        }
    }

    /// Plan decision T4: a Place that the placement check refuses as it applies changes nothing (the building stays
    /// ready to place), and the sim reports it, with every problem the check finds there
    /// (`SimEvent::PlacementRefused`), so that the client can tell the player. A Warp In refused so (slice M3-E)
    /// is reported the same way, with its Beacon as the constructor ([`Sim::warp_in_check`]). Every other
    /// rejection is silent.
    pub(super) fn report_refusal(&mut self, cmd: &PlayerCommand, reject: Reject) {
        // A Tunnel's Expand (slice M4-F, `sim/expansions.rs`).
        if let Command::Tunnel { tunnel, command } = &cmd.command {
            return self.report_tunnel_refusal(*tunnel, *command, reject);
        }
        let (Reject::Unplaceable(_), Command::Construction { objects, command }) = (reject, &cmd.command) else {
            return;
        };
        // The gate's constructor and building, and its check, from the same state.
        let refused = match *command {
            ConstructionCommand::Place { origin, orientation } => {
                self.placer(cmd.player, objects).ok().map(|(constructor, kind)| {
                    let problems = self.placement_check(cmd.player, kind, origin, orientation);
                    (constructor, kind, origin, orientation, problems)
                })
            }
            ConstructionCommand::WarpIn { kind, orientation } => {
                self.warper(cmd.player, objects).ok().and_then(|beacon| {
                    let problems = self.warp_in_check(beacon, kind, orientation)?;
                    let footprint = transform::warp_in_footprint(&self.object(beacon)?.placement?, kind, orientation)?;
                    Some((beacon, kind, footprint.origin, orientation, problems))
                })
            }
            _ => None,
        };
        if let Some((constructor, kind, origin, orientation, problems)) = refused {
            self.events.push(SimEvent::PlacementRefused {
                constructor,
                kind,
                origin,
                orientation,
                problems,
            });
        }
    }

    /// Design change 0084 (`factions/gdo/structures/deployment_center.md > Cancellation`: "Destroyed while
    /// constructing or holding a ready building: refunded as if cancelled"): each constructor among the objects
    /// `gone`, which are leaving the game, gives its owner back what cancelling would ([`construction::refund`]).
    /// For destructions and debug removals alike: the removal path is the one way out of the game.
    pub(super) fn refund_constructors(&mut self, gone: &[ObjectId]) {
        for &id in gone {
            self.cancel_construction(id);
        }
    }

    /// Constructor `id` gives up its construction or its ready building, and its owner gets back what cancelling
    /// gives ([`construction::refund`]), with a `SimEvent::ConstructionCancelled` that says how much. Nothing for an
    /// object that isn't an owned constructor with something to cancel.
    fn cancel_construction(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let (Some(owner), Some(kind)) = (o.owner, o.constructor.and_then(|c| c.current().or(c.ready()))) else {
            return;
        };
        let refund = construction::refund(o);
        o.constructor = Some(Constructor::Idle);
        if let Some(bank) = self.players.iter_mut().find(|p| p.id == owner) {
            bank.credit_crystals(refund);
        }
        self.events.push(SimEvent::ConstructionCancelled {
            constructor: id,
            kind,
            refund,
        });
    }
}
