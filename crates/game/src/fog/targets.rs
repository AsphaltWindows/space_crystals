//! What the local player knows of an object its orders and rally points target, as the client shows it:
//! `core/vision.md > Explored > Targeting remembered objects` (design change 0139): "The order goes to the object's
//! **last-known position**. [...] Nothing about the real state leaks: the cursor, the command and any rally marker
//! treat the remembered object as present until the player sees otherwise." The sim's [`TargetView`]
//! (`sc_sim::targets`), through a [`Sight`], so that it agrees with what the world view draws: the object itself
//! while the sight shows it, the record drawn in its place while the player remembers it (slice M3-K2a).

use sc_sim::{ObjectId, Sim, TargetView};

use super::Sight;

impl Sight<'_> {
    /// What the player knows of object `id` (`sc_sim::targets::view`, design change 0139), as this sight shows it:
    /// - **Visible** while it shows the object as it is ([`Sight::sees`]: the player's own always, and every object
    ///   while the fog doesn't apply);
    /// - else **Remembered**, at the last-known position and footprint of the record shown in its place
    ///   ([`Sight::remembered_record`]), whether the object still stands there, stands elsewhere (a Beacon whose
    ///   deployment was cancelled walks away as a Drone, `sc_sim::deploy`) or is gone;
    /// - else **Gone**: a unit out of sight (units aren't remembered), or an object the player never saw.
    ///
    /// Nothing is read of an object the sight doesn't show but whether it exists. With the fog on, a player's sight
    /// is their own vision and this is `Sim::target_view`, what their orders go by.
    pub fn target_view(&self, sim: &Sim, id: ObjectId) -> TargetView {
        if sim.object(id).is_some_and(|object| self.sees(object)) {
            return TargetView::Visible;
        }
        match self.remembered_record(id) {
            Some(record) => TargetView::Remembered {
                position: record.pos(),
                footprint: record.placement,
            },
            None => TargetView::Gone,
        }
    }
}
