//! (M6-GD) Unit stocks — plan decision T89, `factions/gdo/units/drone_operator.md > Drone Stock`: "Starts with **1**
//! drone when trained; holds a **maximum of 1**"; "**Build Drone** is one of the Drone Operator's abilities: once its
//! drone has been used, it purchases/builds a replacement". A unit whose type has a stock (`UnitDef::stock`) holds a
//! [`Stock`]: how many items it has, and the purchase under way.
//!
//! - **The start**: its type's `start` items as it comes into the game ([`fresh`], `Sim::new_object`).
//! - **Buying** (`Ability::Buy`, the operator's F; `sim/drones.rs`): its type's price in Space Crystals is paid at the
//!   press, through the bank's checked spend (plan decision T7), and the purchase takes its type's frames, one in each
//!   economy phase from that of the press on, at full speed whatever the Power: units draw none (T89; [`step`]). Then
//!   the stock holds one more. It keeps the unit's orders, as a Riot Shield or a Convert does. Refused while the stock
//!   is full, while a purchase runs, and (design change 0421, answering Q0155) while the unit's drone is out: "Build
//!   Drone can start only once the operator's drone has **ended** (detonated, fallen or lost with the leash,
//!   battery)" ([`refusal`]). So the stock never passes its type's max.
//! - **Spending** is an order (T89): the operator's Launch (`OrderKind::Launch`, `crate::tether`), and slice M6-GM's
//!   Lay Mine.
//!
//! No type of the design gains or loses a stock by a change of kind: `Sim::change_kind` keeps it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, StockDef};
use crate::economy::RATIO_ONE;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::progress::Progress;

/// A unit's stock (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock {
    /// The items it holds, from none to its type's max.
    pub count: u32,
    /// The purchase under way, its frames done of its type's; none while it buys nothing. Left out of files while
    /// none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buying: Option<Progress>,
}

/// Why a unit can't start a purchase now ([`refusal`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Unbuyable {
    /// Its stock holds its type's max.
    Full,
    /// It buys one already.
    Buying,
    /// Its drone is out (`Object::drone`).
    DroneOut,
}

/// The stock of `kind`'s type, if it has one.
pub fn def_of(kind: ObjectKind) -> Option<StockDef> {
    kind.def().unit.as_ref()?.stock
}

/// What a unit of `kind` starts with (module docs): its type's start, at most its max, nothing being bought; none for
/// a type without a stock.
pub fn fresh(kind: ObjectKind) -> Option<Stock> {
    def_of(kind).map(|def| Stock {
        count: def.start.min(def.max),
        buying: None,
    })
}

/// Why `o`, a unit with a stock, can't start a purchase now, if it can't (module docs), in this order: its drone is
/// out, it buys one already, its stock is full. Whether its owner can pay is the gate's to say.
pub fn refusal(o: &Object, stock: &Stock) -> Option<Unbuyable> {
    let max = def_of(o.kind).map_or(0, |def| def.max);
    if o.drone.is_some() {
        Some(Unbuyable::DroneOut)
    } else if stock.buying.is_some() {
        Some(Unbuyable::Buying)
    } else if stock.count >= max {
        Some(Unbuyable::Full)
    } else {
        None
    }
}

/// The purchase of one more item starts in `o`'s stock (module docs), its type's frames long: the gate checked it may
/// ([`refusal`]), and its owner has paid. Returns whether it did.
pub fn start_buying(o: &mut Object) -> bool {
    let Some(def) = def_of(o.kind) else {
        return false;
    };
    let Some(stock) = &mut o.stock else {
        return false;
    };
    stock.buying = Some(Progress::new(def.frames));
    true
}

/// The stocks' step, in the economy phase: every purchase under way of the units before `first_new` takes one frame at
/// full speed (module docs: units draw no Power), in id order, and one whose frames are done adds its item to the
/// stock, never past its type's max. Returns the units whose purchase completed.
pub fn step(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId) -> Vec<ObjectId> {
    let mut bought = Vec::new();
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(stock) = &mut o.stock else {
            continue;
        };
        let Some(progress) = &mut stock.buying else {
            continue;
        };
        if !progress.step(RATIO_ONE) {
            continue;
        }
        if def_of(o.kind).is_some_and(|def| stock.count < def.max) {
            stock.count += 1;
        }
        stock.buying = None;
        bought.push(o.id);
    }
    bought
}

impl StateHash for Stock {
    fn hash(&self, h: &mut StateHasher) {
        let Stock { count, buying } = self;
        count.hash(h);
        buying.hash(h);
    }
}

#[cfg(test)]
mod tests;
