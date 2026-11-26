//! The placement preview of a Recruitment Hall (slice M5-J2; `factions/cults/structures/recruitment_hall.md >
//! Recruitment Area display`, design changes 0350, 0351 and 0352, answering Q0107): "**Placing a Hall** shows its 10x10
//! area: the tiles it would claim, the tiles already claimed by your own Halls, the non-Recruitable tiles, and its
//! Effectiveness in %". A pure function of the Halls it is given ([`Preview::of`]), by the game's rule of claiming
//! (`super::winners`), so that the two can't drift apart. Display only: a Hall claims nothing until it is complete
//! (design change 0351: "a Hall still claims nothing until it is complete (0341)"), and the game's claims are
//! [`super::Claims`]'s.
//!
//! - **The claimants** ([`Claimant`]), in the order they claim ([`Rank`]): the completed Halls first, as they claim
//!   now (`factions/cults/structures/recruitment_hall.md > TileClaiming`: completed first, a tie to the nearer centre,
//!   then the lower id), then the player's own Halls not completed yet, their sites and their placed ghosts, in their
//!   order of placement (design change 0351: "The preview treats own pending Halls as claimants, in their order of
//!   placement"), then the Hall being placed. Who they are is the caller's: what the player knows (the client's
//!   `placement::recruitment_areas`): their own Halls, and the other players' Halls they see or remember, as last seen
//!   (design changes 0350 and 0352: "The preview reads the player's memory of enemy Halls, not the true state").
//! - **Its tiles** ([`PreviewTile`]): each tile of the area claimable, taken by another Hall (and whose:
//!   [`Claimer`]), or not Recruitable, as far as the player knows the ground (`known`; a tile off the map is none, and
//!   counts among the area's tiles all the same, as in the game's claims).
//! - **The Effectiveness** ([`Preview::effectiveness_percent`]): the claimable tiles over the area's, rounded down
//!   (`super::effectiveness_percent`, as the Hall's own InfoPanel shows it).

use super::{Contender, area, area_square, def_of, effectiveness_percent, recruitable, winners};
use crate::defs::ObjectKind;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::object::Placement;
use crate::units::{Tick, TilePos};

/// Whose a Hall of the preview is, to the player it previews for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Claimer {
    /// One of their completed Halls: "the tiles already claimed by your own Halls".
    Own,
    /// One of their Halls not completed yet, a site or a placed ghost (design change 0351: "marked as taken by them").
    OwnPending,
    /// Another player's Hall, as far as they know it (design changes 0350 and 0352: "shown as unclaimable").
    Other,
}

/// When a Hall of the preview claims, the least first ([`Claimant::rank`]); the Hall being placed comes after them all.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    /// A completed Hall, completed in that tick: it claims as it does in the game ("Priority goes to the Hall completed
    /// first", ties to the nearer centre, then the lower id).
    Completed(Tick),
    /// A Hall of the player's not completed yet, the n-th in their order of placement (design change 0351).
    Pending(u32),
}

/// A Hall the preview counts ([`Preview::of`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Claimant {
    /// Its object, if it is one (a ghost isn't): a tie between completed Halls goes to the lower id.
    pub id: Option<ObjectId>,
    /// A Hall's kind: one without a Recruitment Area claims nothing.
    pub kind: ObjectKind,
    /// Its footprint.
    pub placement: Placement,
    pub claimer: Claimer,
    pub rank: Rank,
}

/// What the preview finds on a tile of the area of the Hall placed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PreviewTile {
    /// The Hall would claim it: "the tiles it would claim".
    Claimable,
    /// A Hall that claims before it does has it.
    Taken {
        /// Whose it is.
        claimer: Claimer,
        /// Which of the claimants given it is, by its index among them.
        by: usize,
    },
    /// No Hall claims it: "the non-Recruitable tiles" (Water), and those off the map.
    NotRecruitable,
}

/// The preview of a Hall being placed: each tile of its area as the Halls its player knows of leave it (module docs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview {
    /// The tiles of its Recruitment Area row by row from the north-west (`super::area`), those off the map included.
    pub tiles: Vec<(TilePos, PreviewTile)>,
}

impl Preview {
    /// The preview of a Hall of `kind` at `placement` on `map`, among `claimants` (module docs): `known` says where the
    /// player knows the ground. A tile whose ground they don't know (one they never explored) is taken to be
    /// Recruitable, as the placement checks let such ground pass (`crate::placement::Mode::Known`; design change 0120:
    /// "Never-explored tiles pass"). `None` if `kind` has no Recruitment Area.
    pub fn of(
        kind: ObjectKind,
        placement: Placement,
        claimants: &[Claimant],
        map: &Map,
        known: impl Fn(TilePos) -> bool,
    ) -> Option<Preview> {
        let def = def_of(kind)?;
        let (origin, side) = area_square(&placement, def);
        // A Hall claims only in its own area: the others can't touch this one's tiles.
        let near = |c: &Claimant| {
            let Some(other) = def_of(c.kind) else {
                return false;
            };
            let (o, s) = area_square(&c.placement, other);
            o.x < origin.x + side && origin.x < o.x + s && o.y < origin.y + side && origin.y < o.y + s
        };
        let others = (claimants.iter().enumerate())
            .filter(|(_, c)| near(c))
            .filter_map(|(i, c)| {
                Some(Contender {
                    rank: Order::Known(c.rank),
                    id: (c.id, i),
                    placement: c.placement,
                    def: def_of(c.kind)?,
                })
            });
        let placed = Contender {
            rank: Order::Placed,
            id: (None, usize::MAX),
            placement,
            def,
        };
        let ground = |tile: TilePos| map.in_bounds(tile) && (!known(tile) || recruitable(map, tile));
        let won: std::collections::BTreeMap<TilePos, usize> = (winners(others.chain([placed]), ground))
            .map(|(tile, (_, i))| (tile, i))
            .collect();
        let tiles = area(&placement, def)
            .map(|tile| {
                let found = match won.get(&tile) {
                    None => PreviewTile::NotRecruitable,
                    Some(&by) => (claimants.get(by)).map_or(PreviewTile::Claimable, |c| PreviewTile::Taken {
                        claimer: c.claimer,
                        by,
                    }),
                };
                (tile, found)
            })
            .collect();
        Some(Preview { tiles })
    }

    /// The tiles the Hall would claim.
    pub fn claimed(&self) -> u32 {
        let claimable = self.tiles.iter().filter(|(_, found)| *found == PreviewTile::Claimable);
        u32::try_from(claimable.count()).unwrap_or(u32::MAX)
    }

    /// Its Effectiveness in whole percent, rounded down (`factions/cults/structures/recruitment_hall.md >
    /// Effectiveness`): the tiles it would claim over its area's.
    pub fn effectiveness_percent(&self) -> u32 {
        effectiveness_percent(self.claimed(), u32::try_from(self.tiles.len()).unwrap_or(u32::MAX))
    }
}

/// The order the preview's Halls claim in: those it is given by their ranks, then the one placed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Order {
    Known(Rank),
    Placed,
}

#[cfg(test)]
mod tests;
