//! Things that happened during a tick, for the client (effects, sounds, UI
//! notifications). Events are informational output only: the sim never reads them.

use serde::{Deserialize, Serialize};

use crate::ids::ObjectId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SimEvent {
    Spawned(ObjectId),
    Removed(ObjectId),
    /// A Supply Delivery Station received a delivery.
    SuppliesDelivered {
        station: ObjectId,
        amount: u32,
    },
}
