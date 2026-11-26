//! Where a unit's amounts come from (slice M5-N1a; design change 0336, `factions/cults/augmentation.md > Research
//! applies to everyone`): "every unit that has that Augmentation gets the new amounts at once, along with units
//! augmented later". A unit holds no level of its own, only which Augmentations it has (`super::Augmentation`): its
//! amounts are read from its owner's current research levels ([`Levels`]) whenever its stats are, through
//! [`Research`], so nothing on the unit goes stale when a level completes.

use crate::ids::PlayerId;
use crate::player::Player;

/// A research level of one Augmentation's line (`factions/cults/structures/temple.md > Research (first set)`: "Each
/// Augmentation has its own two-level improvement line"): a column of the effects table
/// (`factions/cults/augmentation.md > Augmentation Effects`, `defs::AugmentationTable`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// "No research".
    #[default]
    NoResearch,
    /// "Level 1 researched".
    L1,
    /// "Level 2 researched".
    L2,
}

impl Level {
    /// The three, in the table's order.
    pub const ALL: [Level; 3] = [Level::NoResearch, Level::L1, Level::L2];

    /// Its column of the effects table, from 0.
    pub fn index(self) -> usize {
        self as usize
    }
}

/// A player's current levels of the two lines (`factions/cults/structures/temple.md > Research (first set)`): Enhanced
/// Constitution's and Improved Metabolism's, which the amounts of the Augmentation of each name follow.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Levels {
    pub constitution: Level,
    pub metabolism: Level,
}

impl Levels {
    /// Neither line researched: the table's "No research" column for both.
    pub const NONE: Levels = Levels {
        constitution: Level::NoResearch,
        metabolism: Level::NoResearch,
    };

    /// `player`'s current levels. No player researches yet: the Temple does from slice M5-N3, whose techs (plan
    /// decision T74, per player) give them here, and only here, so that every accessor follows.
    pub fn of(_player: &Player) -> Levels {
        Levels::NONE
    }
}

/// Every player's research, as the stat accessors read it (module docs): `Object::max_hp_with`, `Object::movement`,
/// `Object::max_speed` and `combat::weapon` take one, and look up the unit's owner's [`Levels`] only for a unit that
/// has an Augmentation. Copy and small, so that a system holding the objects mutably takes it beside them (a borrow
/// of the players alone); the rest ask `Sim::research`.
#[derive(Copy, Clone, Debug)]
pub struct Research<'a> {
    source: Source<'a>,
}

#[derive(Copy, Clone, Debug)]
enum Source<'a> {
    /// The game's players, each at their own levels ([`Levels::of`]).
    Players(&'a [Player]),
    /// Everyone at these.
    Everyone(Levels),
}

impl Research<'static> {
    /// No one's research: every amount at "No research". For tests, and for an object outside any game.
    pub const NONE: Research<'static> = Research::at(Levels::NONE);

    /// Everyone at `levels`: the amounts at levels that no research gives yet, for tests (the Temple researches from
    /// slice M5-N3).
    pub const fn at(levels: Levels) -> Self {
        Research {
            source: Source::Everyone(levels),
        }
    }
}

impl<'a> Research<'a> {
    /// The research of `players`, the game's.
    pub fn new(players: &'a [Player]) -> Self {
        Self {
            source: Source::Players(players),
        }
    }

    /// The current levels of `owner`'s research ([`Levels::of`]); none for an object without an owner, or one whose
    /// player isn't in the game.
    pub fn levels(self, owner: Option<PlayerId>) -> Levels {
        match self.source {
            Source::Players(players) => {
                let player = owner.and_then(|id| players.iter().find(|player| player.id == id));
                player.map_or(Levels::NONE, Levels::of)
            }
            Source::Everyone(levels) => levels,
        }
    }
}
