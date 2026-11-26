//! Where a unit's amounts come from (slice M5-N1a; design change 0336, `factions/cults/augmentation.md > Research
//! applies to everyone`): "every unit that has that Augmentation gets the new amounts at once, along with units
//! augmented later". A unit holds no level of its own, only which Augmentations it has (`super::Augmentation`): its
//! amounts are read from its owner's current research levels ([`Levels`]) whenever its stats are, through
//! [`Research`], so nothing on the unit goes stale when a level completes. The levels come from the technologies its
//! owner completed at a Temple (slice M5-N3, [`Levels::of`]); a completed level takes the HP of every unit it changes
//! along with its MaxHP (`Sim::complete_research`), and so does a unit's turning neutral (`economy::defeat`), which
//! takes it to no one's levels.

use crate::defs::ResearchLine;
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

    /// `player`'s current levels (slice M5-N3, plan decision T74): each line's highest level among the technologies
    /// the player has completed at a Temple (`Player::techs`), "No research" for a line without one. Here, and only
    /// here, so that every accessor follows (module docs). It allocates nothing, and a player has at most the four
    /// technologies; one without any is the common case, and costs a branch.
    pub fn of(player: &Player) -> Levels {
        if player.techs.is_empty() {
            return Levels::NONE;
        }
        let mut levels = Levels::NONE;
        for tech in &player.techs {
            let def = tech.def();
            let level = levels.line_mut(def.line);
            *level = (*level).max(def.level);
        }
        levels
    }

    /// (M5-N3) The current level of `line`.
    pub fn line(self, line: ResearchLine) -> Level {
        match line {
            ResearchLine::EnhancedConstitution => self.constitution,
            ResearchLine::ImprovedMetabolism => self.metabolism,
        }
    }

    fn line_mut(&mut self, line: ResearchLine) -> &mut Level {
        match line {
            ResearchLine::EnhancedConstitution => &mut self.constitution,
            ResearchLine::ImprovedMetabolism => &mut self.metabolism,
        }
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

    /// Everyone at `levels`, whatever their research: for tests, and for the units of one player whose levels change
    /// (`Sim::complete_research`).
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
