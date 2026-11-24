//! The construction commands in a game (`crate::construction`, slice M2-D1): their rules in the command gate, what
//! applying them does, the refusal a Place reports, and the refunds of the constructors that leave the game.

use glam::Vec2;

use super::Sim;
use super::gate::{Checked, Reject};
use crate::command::{Command, PlayerCommand};
use crate::construction::{self, ConstructionCommand, Constructor};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Placement;
use crate::placement::footprint_size;
use crate::scenario::BuildingOrientation;
use crate::units::TilePos;

/// A construction command that passed the gate, with what its checks resolved.
pub(super) enum Resolved {
    /// Each of `constructors`, in id order, pays for `kind` and starts constructing it.
    Construct {
        kind: ObjectKind,
        constructors: Vec<ObjectId>,
    },
    /// Each of `constructors` cancels its construction or its ready building, with the refund that goes with it.
    Cancel { constructors: Vec<ObjectId> },
    /// `constructor` places its ready `kind` at `placement`, centred on `pos`.
    Place {
        constructor: ObjectId,
        kind: ObjectKind,
        pos: Vec2,
        placement: Placement,
    },
}

impl Sim {
    /// The gate's rules for `Command::Construction` (`crate::construction`), after the player's: the player's own
    /// constructors among `objects`, unique and in id order (ids that no longer exist are dropped, as are the player's
    /// objects that construct nothing; anyone else's object rejects it, [`Reject::NotOwned`]; none left is
    /// [`Reject::NoConstructor`]), then the command's own:
    /// - **Construct**: each constructor, in id order, that has the building on its list, is idle and can pay for it
    ///   from what the bank holds once the constructors before it paid; if none can, the first one's reason
    ///   ([`Reject::CannotConstruct`], [`Reject::ConstructorBusy`], [`Reject::InsufficientCrystals`]);
    /// - **Cancel Construction**: each one constructing, else [`Reject::NotConstructing`];
    /// - **Cancel Ready Building**: each one holding a building ready, else [`Reject::NothingReady`];
    /// - **Place**: one constructor ([`Reject::SeveralConstructors`]) holding a building ready
    ///   ([`Reject::NothingReady`]), which the placement check allows there now ([`Sim::placement_check`]: else
    ///   [`Reject::Unplaceable`] with its first problem).
    pub(super) fn check_construction(
        &self,
        player: PlayerId,
        objects: &[ObjectId],
        command: ConstructionCommand,
    ) -> Result<Checked, Reject> {
        let resolved = match command {
            ConstructionCommand::Construct(kind) => self.check_construct(player, objects, kind)?,
            ConstructionCommand::CancelConstruction => Resolved::Cancel {
                constructors: self.constructors_that(
                    player,
                    objects,
                    |c| c.current().is_some(),
                    Reject::NotConstructing,
                )?,
            },
            ConstructionCommand::CancelReady => Resolved::Cancel {
                constructors: self.constructors_that(player, objects, |c| c.ready().is_some(), Reject::NothingReady)?,
            },
            ConstructionCommand::Place { origin, orientation } => {
                self.check_place(player, objects, origin, orientation)?
            }
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

    /// Construct `kind` (plan decision T7: the bank is checked as the command applies, per constructor in id order).
    fn check_construct(&self, player: PlayerId, objects: &[ObjectId], kind: ObjectKind) -> Result<Resolved, Reject> {
        let mut left = self.player(player).map_or(0, |p| p.crystals);
        let mut constructors = Vec::new();
        let mut refusal = None;
        for id in self.own_constructors(player, objects)? {
            let Some(o) = self.objects.get(&id) else {
                continue;
            };
            match construction::can_start(o, kind, left) {
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
    /// (`factions/gdo/structures.md > DeploymentCenter > AwaitingPlacement`, `factions/gdo/economy.md >
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
                        o.constructor = Some(Constructor::Constructing {
                            kind,
                            since: self.tick,
                            progress: 0,
                        });
                    }
                }
            }
            Resolved::Cancel { constructors } => {
                for id in constructors {
                    let Some(o) = self.objects.get_mut(&id) else {
                        continue;
                    };
                    let refund = construction::refund(o);
                    o.constructor = Some(Constructor::Idle);
                    if let Some(bank) = self.players.iter_mut().find(|p| p.id == player) {
                        bank.credit_crystals(refund);
                    }
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
            }
        }
    }

    /// Plan decision T4: a Place that the placement check refuses as it applies changes nothing (the building stays
    /// ready to place), and the sim reports it, with every problem the check finds there
    /// (`SimEvent::PlacementRefused`), so that the client can tell the player. Every other rejection is silent.
    pub(super) fn report_refusal(&mut self, cmd: &PlayerCommand, reject: Reject) {
        let (
            Reject::Unplaceable(_),
            Command::Construction {
                objects,
                command: ConstructionCommand::Place { origin, orientation },
            },
        ) = (reject, &cmd.command)
        else {
            return;
        };
        // The gate's constructor and building, from the same state.
        let Ok((constructor, kind)) = self.placer(cmd.player, objects) else {
            return;
        };
        let problems = self.placement_check(cmd.player, kind, *origin, *orientation);
        self.events.push(SimEvent::PlacementRefused {
            constructor,
            kind,
            origin: *origin,
            orientation: *orientation,
            problems,
        });
    }

    /// Design change 0084 (`factions/gdo/structures.md > DeploymentCenter > Cancellation`: "Destroyed while
    /// constructing or holding a ready building: refunded as if cancelled"): each constructor among the objects
    /// `gone`, which are leaving the game, gives its owner back what cancelling would ([`construction::refund`]).
    /// For destructions and debug removals alike: the removal path is the one way out of the game.
    pub(super) fn refund_constructors(&mut self, gone: &[ObjectId]) {
        for id in gone {
            let Some((owner, refund)) = self
                .objects
                .get(id)
                .and_then(|o| Some((o.owner?, construction::refund(o))))
            else {
                continue;
            };
            if let Some(bank) = self.players.iter_mut().find(|p| p.id == owner) {
                bank.credit_crystals(refund);
            }
        }
    }
}
