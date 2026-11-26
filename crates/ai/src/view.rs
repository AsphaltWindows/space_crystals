//! What a computer player knows of the game, and the only way it reads it (`core/game_rules.md > Computer players
//! (playtest tool)`: "**Fair knowledge**: a computer player sees only what its own objects see and remembers what it
//! has seen, as the fog of war does for a human player (core/vision.md). It also knows the map and every player's
//! starting location. It never sees the whole map"; design change 0402: "Its knowledge must go through the same vision
//! and fog layer a player's client uses").
//!
//! A [`View`] holds the game privately: the rest of the crate reads it only through the View's methods, which show
//! - **its own** objects, the units held off the map included, its banks, Power, army cap and build area, all as they
//!   are;
//! - **other players'** objects only while it sees them (`PlayerVision::sees`, as the client draws them) or, for a
//!   structure, as it remembers them out of its sight (`vision::LastKnown`), never a unit out of its sight
//!   ([`View::known`]);
//! - **the map**: its terrain (presets, passability, heights), the neutral objects as the map placed them (known from
//!   the start, `systems/controls/behaviors.md > PathKnowledge`: "Map-placed objects (crystal patches, Supply Delivery
//!   Stations) are known from the start"), a patch's amount as last seen, and every player's start location
//!   ([`Start`]);
//! - **the gate's checks** a player's client asks: placements (`Sim::placement_check` and the others), casts, and the
//!   gate itself for the player's own commands ([`View::validate`]). They answer as the client's ghosts do: a tile the
//!   player doesn't see reports only that ("NotVisible"), and a worker's or a Tunnel's checks go by what its owner
//!   knows. A computer player validates only commands it built from what the View shows.

#[cfg(test)]
mod tests;

use std::cell::OnceCell;

use sc_sim::economy::{ArmyCap, PowerSummary};
use sc_sim::network::Held;
use sc_sim::vision::{LastKnown, PlayerVision, VisibilityState};
use sc_sim::{
    AreaAbility, BuildArea, BuildingOrientation, Category, Command, Map, Object, ObjectId, ObjectKind, Placement,
    PlacementCheck, Player, PlayerCommand, PlayerId, Reject, ResourceState, Scenario, Sim, Tick, TilePos, Vec2,
};

/// A player's start location (`core/game_rules.md > Computer players (playtest tool)`: a computer player "knows the
/// map and every player's starting location"): the start of the slot the player takes (`core/map_editor.md > Map
/// Creation`). The sim doesn't keep the slots; the scenario does ([`Start::all`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Start {
    pub player: PlayerId,
    pub tile: TilePos,
}

impl Start {
    /// The centre of its tile, space units.
    pub fn point(&self) -> Vec2 {
        self.tile.center()
    }

    /// Every player's start location in `scenario`, in player order: player `n` is its setup's entry `n - 1`
    /// (`GameSetup::players`), at the start of the slot it takes. A player whose slot the map lacks has none.
    pub fn all(scenario: &Scenario) -> Vec<Start> {
        (scenario.setup.players.iter().enumerate())
            .filter_map(|(i, setup)| {
                let player = PlayerId::from_index(i)?;
                let slot = scenario.map.slot(setup.slot)?;
                Some(Start {
                    player,
                    tile: slot.start,
                })
            })
            .collect()
    }
}

/// What a player knows of an object it doesn't own ([`View::known`]): the object as it is now while the player sees
/// it, else, for a structure or a resource, as the player last saw it (`core/vision.md > Explored`: "Structures and
/// resources are shown in their last-known state (may have been destroyed or changed since)").
#[derive(Clone, Debug, PartialEq)]
pub struct Known {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    /// Where it stands (or stood), space units: a structure's footprint's centre.
    pub pos: Vec2,
    /// Its footprint; `None` for a unit.
    pub placement: Option<Placement>,
    /// Its HP, `None` for an indestructible type.
    pub hp: Option<f32>,
    /// A map resource's state: a patch's RemainingAmount as last seen.
    pub resource: Option<ResourceState>,
    /// The tick at whose end the player last saw it: the latest tick while it is in sight.
    pub seen: Tick,
    /// Whether the player sees it now.
    pub in_sight: bool,
}

impl Known {
    /// `o` as a player that sees it at the end of tick `now` knows it.
    fn seen(o: &Object, now: Tick) -> Self {
        Self {
            id: o.id,
            kind: o.kind,
            owner: o.owner,
            pos: o.pos,
            placement: o.placement,
            hp: o.hp,
            resource: o.resource.clone(),
            seen: now,
            in_sight: true,
        }
    }

    /// The player's record of an object out of its sight.
    fn remembered(r: &LastKnown) -> Self {
        Self {
            id: r.id,
            kind: r.kind,
            owner: r.owner,
            pos: r.pos(),
            placement: Some(r.placement),
            hp: r.hp,
            resource: r.resource.clone(),
            seen: r.seen,
            in_sight: false,
        }
    }

    /// Whether it is a structure (not a unit nor a map resource).
    pub fn is_structure(&self) -> bool {
        self.kind.def().category == Category::Structure
    }

    /// A patch's RemainingAmount as last seen; `None` for anything but a Space Crystals patch.
    pub fn crystals_left(&self) -> Option<u32> {
        match self.resource {
            Some(ResourceState::CrystalPatch { remaining }) => Some(remaining),
            _ => None,
        }
    }
}

/// What one player knows of the game (module docs), for one tick's thinking.
pub struct View<'a> {
    sim: &'a Sim,
    player: PlayerId,
    starts: &'a [Start],
    /// [`View::known`], worked out once.
    known: OnceCell<Vec<Known>>,
}

impl<'a> View<'a> {
    /// `player`'s view of `sim`, with every player's start location (`starts`, [`Start::all`]).
    pub fn new(sim: &'a Sim, player: PlayerId, starts: &'a [Start]) -> Self {
        Self {
            sim,
            player,
            starts,
            known: OnceCell::new(),
        }
    }

    /// The player it is the view of.
    pub fn player(&self) -> PlayerId {
        self.player
    }

    /// The latest tick: the game as its end left it.
    pub fn tick(&self) -> Tick {
        self.sim.tick()
    }

    /// The player: their faction, their banks and whether they are defeated. `None` for a player not in the game.
    pub fn me(&self) -> Option<&'a Player> {
        self.sim.player(self.player)
    }

    /// Whether the player may still give commands: they are in the game and not defeated, and the game isn't over
    /// (`core/game_rules.md > End of Game`; the gate refuses everything else).
    pub fn in_game(&self) -> bool {
        self.sim.outcome().is_none() && self.me().is_some_and(|p| !p.is_defeated())
    }

    /// Every player, defeated or not, in id order: who plays, their names and factions (shown to every player at
    /// game start), and who has been defeated (announced to every player). Their banks are their own business: read
    /// [`View::me`] for the player's.
    pub fn players(&self) -> impl Iterator<Item = (PlayerId, bool)> + 'a {
        self.sim.players().iter().map(|p| (p.id, p.is_defeated()))
    }

    /// Every player's start location, in player order.
    pub fn starts(&self) -> &'a [Start] {
        self.starts
    }

    /// The player's own start location, if the map gives one.
    pub fn my_start(&self) -> Option<Start> {
        self.starts.iter().copied().find(|s| s.player == self.player)
    }

    /// The start locations of the other players still in the game, in player order.
    pub fn enemy_starts(&self) -> impl Iterator<Item = Start> + '_ {
        let standing = |id| self.sim.player(id).is_some_and(|p| !p.is_defeated());
        (self.starts.iter().copied()).filter(move |s| s.player != self.player && standing(s.player))
    }

    /// The map's terrain: its size, tile presets (Buildable, Traversible …) and corner heights.
    pub fn map(&self) -> &'a Map {
        self.sim.map()
    }

    /// The player's objects on the map, in id order.
    pub fn own(&self) -> impl Iterator<Item = &'a Object> + 'a {
        let player = self.player;
        self.sim.objects().filter(move |o| o.owner == Some(player))
    }

    /// The player's units held off the map (in a Tunnel Network or a building that holds units), by id.
    pub fn own_held(&self) -> impl Iterator<Item = &'a Held> + 'a {
        let player = self.player;
        self.sim.held().filter(move |h| h.object.owner == Some(player))
    }

    /// One of the player's own objects on the map; `None` for another player's, or one gone.
    pub fn own_object(&self, id: ObjectId) -> Option<&'a Object> {
        self.sim.object(id).filter(|o| o.owner == Some(self.player))
    }

    /// The player's Power at the end of the latest tick (`factions/gdo/overview.md > Power`).
    pub fn power(&self) -> PowerSummary {
        self.sim.power(self.player)
    }

    /// The player's army cap (Unit Control, Beacon Capacity, Tunnel Space: `Sim::army_cap`).
    pub fn army_cap(&self) -> Option<ArmyCap> {
        self.sim.army_cap(self.player)
    }

    /// The player's GDO build area (`factions/gdo/structures/README.md > GDOBuildArea`), from their own buildings.
    pub fn build_area(&self) -> BuildArea {
        self.sim.build_area(self.player)
    }

    /// How the player sees `tile` now (`core/vision.md > VisibilityStateEnum`).
    pub fn visibility(&self, tile: TilePos) -> VisibilityState {
        self.sim.visibility(self.player, tile)
    }

    /// What the player knows of the objects it doesn't own, in id order (module docs): every object it sees now, as it
    /// is, and every structure and resource it remembers out of its sight, as it last saw it. Nothing else: no unit out
    /// of its sight, no structure it never saw (or that hides from it underground), but the neutral objects the map
    /// placed, which every player remembers from the start.
    pub fn known(&self) -> &[Known] {
        self.known.get_or_init(|| self.work_out_known())
    }

    /// [`View::known`].
    fn work_out_known(&self) -> Vec<Known> {
        let Some(vision) = self.vision() else {
            return Vec::new();
        };
        let now = self.sim.tick();
        let mut known: Vec<Known> = (self.sim.objects())
            .filter(|o| o.owner != Some(self.player) && vision.sees(o))
            .map(|o| Known::seen(o, now))
            .collect();
        let remembered = vision
            .remembered()
            .filter(|r| known.binary_search_by_key(&r.id, |k| k.id).is_err());
        let remembered: Vec<Known> = remembered.map(Known::remembered).collect();
        known.extend(remembered);
        known.sort_by_key(|k| k.id);
        known
    }

    /// The structures of other players that the player knows ([`View::known`]), in id order.
    pub fn enemy_structures(&self) -> impl Iterator<Item = &Known> {
        let player = self.player;
        (self.known().iter()).filter(move |k| k.owner.is_some_and(|p| p != player) && k.is_structure())
    }

    /// The Space Crystals patches the player knows, as the map placed them and as it last saw them, in id order.
    pub fn patches(&self) -> impl Iterator<Item = &Known> {
        self.known().iter().filter(|k| k.crystals_left().is_some())
    }

    /// `core/entities.md > Placement Validation > Direct Placement`: what keeps the player from placing a `kind` with
    /// its footprint's top-left tile at `origin` now (`Sim::placement_check`, the client's ghost).
    pub fn placement_check(
        &self,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    ) -> PlacementCheck {
        self.sim.placement_check(self.player, kind, origin, orientation)
    }

    /// `core/entities.md > Placement Validation > Warp In`, for one of the player's Beacons (`Sim::warp_in_check`).
    pub fn warp_in_check(
        &self,
        beacon: ObjectId,
        kind: ObjectKind,
        orientation: BuildingOrientation,
    ) -> Option<PlacementCheck> {
        self.own_object(beacon)?;
        self.sim.warp_in_check(beacon, kind, orientation)
    }

    /// A Deploy's check, for one of the player's Drones (`Sim::deploy_check`).
    pub fn deploy_check(&self, drone: ObjectId, origin: TilePos, queued: bool) -> Option<PlacementCheck> {
        self.own_object(drone)?;
        self.sim.deploy_check(drone, origin, queued)
    }

    /// A worker Build's check, for one of the player's workers (`Sim::build_check`).
    pub fn build_check(
        &self,
        worker: ObjectId,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
        queued: bool,
    ) -> Option<PlacementCheck> {
        self.own_object(worker)?;
        self.sim.build_check(worker, kind, origin, orientation, queued)
    }

    /// An expansion's check, for one of the player's Tunnels (`Sim::expansion_check`).
    pub fn expansion_check(
        &self,
        tunnel: ObjectId,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    ) -> Option<PlacementCheck> {
        self.own_object(tunnel)?;
        self.sim.expansion_check(tunnel, kind, origin, orientation)
    }

    /// Which of the player's `casters` would cast `ability` now (`Sim::can_cast`).
    pub fn can_cast(&self, casters: &[ObjectId], ability: AreaAbility) -> Result<ObjectId, Reject> {
        self.sim.can_cast(self.player, casters, ability)
    }

    /// Whether one of the player's Tunnels may build an expansion of `kind` now (`Sim::can_expand`).
    pub fn can_expand(&self, tunnel: ObjectId, kind: ObjectKind) -> Result<(), Reject> {
        self.sim.can_expand(self.player, tunnel, kind)
    }

    /// Whether one of the player's Tunnels may be upgraded now (`Sim::can_upgrade`).
    pub fn can_upgrade(&self, tunnel: ObjectId) -> Result<(), Reject> {
        self.sim.can_upgrade(self.player, tunnel)
    }

    /// The gate's verdict on the player's `command` as the latest tick left the game (`Sim::validate`): what the
    /// client asks before it sends a command.
    pub fn validate(&self, command: &Command) -> Result<(), Reject> {
        let command = PlayerCommand {
            player: self.player,
            command: command.clone(),
        };
        self.sim.validate(&command)
    }

    /// The player's vision: what it sees and remembers.
    fn vision(&self) -> Option<&'a PlayerVision> {
        self.sim.vision().player(self.player)
    }
}
