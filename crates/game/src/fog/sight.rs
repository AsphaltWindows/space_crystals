//! What the local player is shown (`core/vision.md`): a [`Sight`] through the visions it watches, the best state
//! any of them gives a tile or an object, the records shown on Explored tiles, and the revisions that say when what
//! it shows may have changed.

use bevy::math::Vec2;
use sc_sim::vision::{LastKnown, PlayerVision};
use sc_sim::{Faction, Object, ObjectId, Player, PlayerId, Sim, TilePos, VisibilityState};

/// What one player sees of the sim, as the client shows it (`core/vision.md`): while the fog applies, through the
/// visions they watch ([`Sight::new`]); everything as if Visible while it doesn't.
#[derive(Copy, Clone, Debug)]
pub struct Sight<'a> {
    player: PlayerId,
    /// `None` while the fog doesn't apply.
    fog: Option<Fogged<'a>>,
}

#[derive(Copy, Clone, Debug)]
struct Fogged<'a> {
    /// Every player's vision, in the sim's player order (`Vision::players`).
    visions: &'a [PlayerVision],
    /// The ones the player watches through. None for a player who isn't in the game: they see nothing.
    watched: Watched,
    /// The player plays the GDO (`core/vision.md > Explored Selectability`).
    gdo: bool,
}

/// Which of the sim's visions a [`Sight`] watches through: bit `i` for the vision of the sim's `i`-th player
/// (`Vision::players`), for as many players as a game can have. Part of [`Sight::revision`], so that a view keyed on
/// it redraws when whom the sight watches changes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Watched([u64; 4]);

impl Watched {
    fn contains(self, i: usize) -> bool {
        self.0.get(i / 64).is_some_and(|word| (word >> (i % 64)) & 1 == 1)
    }

    fn insert(&mut self, i: usize) {
        if let Some(word) = self.0.get_mut(i / 64) {
            *word |= 1 << (i % 64);
        }
    }
}

/// A [`Sight`]'s revision ([`Sight::revision`], [`Sight::records_revision`]): whom it watches, and the sum (wrapping)
/// of a counter of each vision it watches. It changes whenever what those visions show may have changed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    pub watched: Watched,
    pub sum: u32,
}

impl<'a> Sight<'a> {
    /// `player`'s sight in `sim`, through the visions they watch if `fog` applies
    /// ([`FogOfWar::applies`](super::FogOfWar::applies)): their own, and while they are defeated their allies' too
    /// (design change 0102: "their fog is the union of their allies' vision", [`Sim::allies`]). A reading: their own
    /// vision stays in the union. Since they own nothing it has no Visible tile, so it adds only what they explored and
    /// remember themselves, and a defeated player without allies (every player until the sim has teams) keeps their map
    /// rather than seeing it go black.
    pub fn new(sim: &'a Sim, player: PlayerId, fog: bool) -> Self {
        if sim.player(player).is_some_and(Player::is_defeated) {
            let mut viewers = vec![player];
            viewers.extend(sim.allies(player));
            Self::watching(sim, player, &viewers, fog)
        } else {
            Self::watching(sim, player, &[player], fog)
        }
    }

    /// `player`'s sight through the visions of `viewers` if `fog` applies: what their union shows (design change
    /// 0102). A tile is in the best state any of them gives it, an object in the best state of its tiles, and an
    /// object out of sight of all of them shows as the latest record any of them keeps of it.
    pub fn watching(sim: &'a Sim, player: PlayerId, viewers: &[PlayerId], fog: bool) -> Self {
        let fog = fog.then(|| {
            let visions = sim.vision().players();
            let mut watched = Watched::default();
            for (i, vision) in visions.iter().enumerate() {
                if viewers.contains(&vision.player()) {
                    watched.insert(i);
                }
            }
            Fogged {
                visions,
                watched,
                gdo: sim.player(player).is_some_and(|p| p.faction == Faction::Gdo),
            }
        });
        Self { player, fog }
    }

    pub fn player(&self) -> PlayerId {
        self.player
    }

    /// The visions it watches through; none while the fog doesn't apply.
    pub(super) fn visions(&self) -> impl Iterator<Item = &'a PlayerVision> + use<'a> {
        self.fog.into_iter().flat_map(|f| {
            f.visions
                .iter()
                .enumerate()
                .filter(move |&(i, _)| f.watched.contains(i))
                .map(|(_, vision)| vision)
        })
    }

    /// Changes whenever the state of a tile it shows may have changed: whom it watches, and the sum of their
    /// `PlayerVision::revision`s (each only grows). `None` while the fog doesn't apply.
    pub fn revision(&self) -> Option<Revision> {
        let f = self.fog?;
        let sum = self.visions().fold(0u32, |sum, v| sum.wrapping_add(v.revision()));
        Some(Revision {
            watched: f.watched,
            sum,
        })
    }

    /// [`Sight::revision`] for the records it shows: whom it watches and the sum of their
    /// `PlayerVision::records_revision`s.
    pub fn records_revision(&self) -> Option<Revision> {
        let f = self.fog?;
        let sum = self
            .visions()
            .fold(0u32, |sum, v| sum.wrapping_add(v.records_revision()));
        Some(Revision {
            watched: f.watched,
            sum,
        })
    }

    /// Whether the fog applies: `false` while everything shows.
    pub fn is_fogged(&self) -> bool {
        self.fog.is_some()
    }

    /// Whether the player plays the GDO; `false` while the fog doesn't apply, which needs no exception.
    pub fn is_gdo(&self) -> bool {
        self.fog.is_some_and(|f| f.gdo)
    }

    /// How the player sees `tile`: its best state among the visions watched.
    pub fn tile(&self, tile: TilePos) -> VisibilityState {
        match self.fog {
            None => VisibilityState::Visible,
            Some(_) => self.best(|v| v.state(tile)),
        }
    }

    /// The best of `state` among the visions watched; Unexplored with none.
    fn best(&self, state: impl Fn(&PlayerVision) -> VisibilityState) -> VisibilityState {
        let mut best = VisibilityState::Unexplored;
        for vision in self.visions() {
            best = best.max(state(vision));
            if best == VisibilityState::Visible {
                break;
            }
        }
        best
    }

    /// How the player sees the tile holding the ground point `ground`.
    pub fn at(&self, ground: Vec2) -> VisibilityState {
        self.tile(TilePos::containing(ground))
    }

    /// The best state among the tiles `object` covers, or Visible if revealed (`PlayerVision::object_state`, 0177).
    pub fn object(&self, object: &Object) -> VisibilityState {
        match self.fog {
            None => VisibilityState::Visible,
            Some(_) => self.best(|v| v.object_state(object)),
        }
    }

    /// Whether the player sees `object` as it is now: its own objects, and the others on a Visible tile. The views
    /// draw only these of the live objects; Explored tiles show [records](Sight::remembered) instead.
    pub fn sees(&self, object: &Object) -> bool {
        object.owner == Some(self.player) || self.object(object) == VisibilityState::Visible
    }

    /// What Explored tiles show (`core/vision.md > Explored`): the records of the structures and resources the
    /// player remembers and doesn't see now, in their last-known state, one per object
    /// ([`Sight::remembered_record`]). None while the fog doesn't apply, when every object shows as it is.
    pub fn remembered(&self) -> impl Iterator<Item = &'a LastKnown> + use<'a> {
        let sight = *self;
        self.visions().flat_map(PlayerVision::remembered).filter(move |record| {
            sight
                .remembered_record(record.id)
                .is_some_and(|shown| std::ptr::eq(shown, *record))
        })
    }

    /// The record shown in place of object `id` ([`Sight::remembered`]), if any: the latest of the records the
    /// visions watched keep of it (the first of them on a tie), shown while its footprint's best tile is Explored:
    /// not while one sees it, nor while none has explored it (Unexplored is "fully black", design change 0159).
    pub fn remembered_record(&self, id: ObjectId) -> Option<&'a LastKnown> {
        let latest = self
            .visions()
            .filter_map(|v| v.record(id))
            .reduce(|latest, record| if record.seen > latest.seen { record } else { latest })?;
        let best = (latest.placement.tiles()).map(|tile| self.tile(tile)).max()?;
        (best == VisibilityState::Explored).then_some(latest)
    }
}
